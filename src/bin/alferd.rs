//! Alfred daemon - keeps LLM model loaded for fast inference

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

// Import from alfred crate
use alfred::config;
use alfred::llm;

#[derive(Deserialize)]
struct Request {
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Serialize)]
struct Response {
    result: Option<String>,
    error: Option<String>,
    id: u64,
}

fn handle_request(request: &Request) -> Response {
    let result = match request.method.as_str() {
        "ping" => Ok("pong".to_string()),
        "shutdown" => Ok("shutting_down".to_string()),
        "generate" => {
            let prompt = request.params.get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let max_tokens = request.params.get("max_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(256) as u32;
            llm::generate_local(prompt, max_tokens)
        }
        "generate_commit_message" => {
            let diff = request.params.get("diff")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            llm::generate_commit_message(diff)
        }
        "suggest_branch_name" => {
            let description = request.params.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            llm::suggest_branch_name(description)
        }
        "suggest_conflict_resolution" => {
            let file = request.params.get("file")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let ours = request.params.get("ours")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let theirs = request.params.get("theirs")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let base = request.params.get("base")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            llm::suggest_conflict_resolution(file, ours, theirs, base)
        }
        "suggest_rebase_strategy" => {
            let commits: Vec<String> = request.params.get("commits")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let onto = request.params.get("onto")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            llm::suggest_rebase_strategy(&commits, onto)
        }
        _ => Err(anyhow!("Unknown method: {}", request.method)),
    };

    match result {
        Ok(r) => Response {
            result: Some(r),
            error: None,
            id: request.id,
        },
        Err(e) => Response {
            result: None,
            error: Some(e.to_string()),
            id: request.id,
        },
    }
}

fn handle_client(
    mut stream: TcpStream,
    last_activity: &AtomicU64,
    shutdown_flag: &AtomicBool,
) -> Result<bool> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    match reader.read_line(&mut line) {
        Ok(0) => return Ok(false), // Connection closed
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
        Err(e) => return Err(e.into()),
    }

    // Update last activity timestamp
    last_activity.store(
        Instant::now().elapsed().as_secs(),
        Ordering::Relaxed,
    );

    let request: Request = serde_json::from_str(&line)
        .with_context(|| "Failed to parse request")?;

    let should_shutdown = request.method == "shutdown";
    let response = handle_request(&request);

    let mut response_str = serde_json::to_string(&response)?;
    response_str.push('\n');
    stream.write_all(response_str.as_bytes())?;
    stream.flush()?;

    if should_shutdown {
        shutdown_flag.store(true, Ordering::Relaxed);
    }

    Ok(should_shutdown)
}

fn write_pid_file() -> Result<()> {
    let pid = std::process::id();
    let pid_path = config::pid_file();
    fs::create_dir_all(config::alfred_dir())?;
    fs::write(&pid_path, pid.to_string())?;
    Ok(())
}

fn remove_pid_file() {
    let _ = fs::remove_file(config::pid_file());
}

fn main() -> Result<()> {
    println!("{}", "Alfred Daemon starting...".cyan());

    // Load configuration
    let daemon_config = config::get_daemon_config();
    let port = daemon_config.port;
    let idle_timeout = Duration::from_secs((daemon_config.idle_timeout_minutes * 60) as u64);
    let has_timeout = daemon_config.idle_timeout_minutes > 0;

    // Write PID file
    write_pid_file()?;

    // Load model
    println!("{}", "Loading LLM model...".cyan());
    if let Err(e) = llm::load_model() {
        eprintln!("{} {}", "Error loading model:".red(), e);
        remove_pid_file();
        return Err(e);
    }
    println!("{}", "Model loaded successfully!".green());

    // Bind to port
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .with_context(|| format!("Failed to bind to {}", addr))?;

    listener.set_nonblocking(true)?;

    println!("{} {}", "Listening on".green(), addr.cyan());
    if has_timeout {
        println!(
            "{} {} {}",
            "Idle timeout:".dimmed(),
            daemon_config.idle_timeout_minutes,
            "minutes".dimmed()
        );
    } else {
        println!("{}", "Idle timeout: disabled".dimmed());
    }

    // Shutdown flag
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_ctrlc = shutdown_flag.clone();

    // Handle Ctrl+C
    ctrlc::set_handler(move || {
        shutdown_flag_ctrlc.store(true, Ordering::Relaxed);
    })?;

    // Track last activity for idle timeout
    let start_time = Instant::now();
    let last_activity = Arc::new(AtomicU64::new(0));

    // Main loop
    while !shutdown_flag.load(Ordering::Relaxed) {
        // Accept connections
        match listener.accept() {
            Ok((stream, _addr)) => {
                if let Err(e) = handle_client(stream, &last_activity, &shutdown_flag) {
                    eprintln!("{} {}", "Client error:".red(), e);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No connection waiting, sleep briefly
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                eprintln!("{} {}", "Accept error:".red(), e);
            }
        }

        // Check idle timeout
        if has_timeout {
            let last = last_activity.load(Ordering::Relaxed);
            let idle_duration = if last == 0 {
                start_time.elapsed()
            } else {
                Duration::from_secs(start_time.elapsed().as_secs() - last)
            };

            if idle_duration > idle_timeout {
                println!("{}", "Idle timeout reached, shutting down...".yellow());
                break;
            }
        }
    }

    // Cleanup
    println!("{}", "Shutting down...".cyan());
    remove_pid_file();
    println!("{}", "Daemon stopped.".green());

    Ok(())
}
