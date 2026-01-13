//! Daemon management commands

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::fs;
use std::process::Command;

use crate::config;
use crate::daemon_client;
use crate::ui;

pub fn show_help() {
    ui::heading("Daemon Commands");
    println!();
    println!("  {} {}", "alfred daemon start".cyan(), "Start the daemon".dimmed());
    println!("  {} {}", "alfred daemon stop".cyan(), "Stop the daemon".dimmed());
    println!("  {} {}", "alfred daemon status".cyan(), "Check daemon status".dimmed());
    println!("  {} {}", "alfred daemon install".cyan(), "Install as system service".dimmed());
    println!("  {} {}", "alfred daemon uninstall".cyan(), "Uninstall system service".dimmed());
    println!();
    ui::dim("The daemon keeps the LLM model loaded in memory for faster inference.");
}

pub fn start() -> Result<()> {
    // Check if already running
    if daemon_client::is_daemon_running() {
        ui::success("Daemon is already running");
        return Ok(());
    }

    ui::info("Starting alfred daemon...");

    // Find alferd binary - check same directory as alfred first
    let alferd_path = find_alferd_binary()?;

    // Start daemon in background
    let child = Command::new(&alferd_path)
        .spawn()
        .with_context(|| format!("Failed to start daemon from {}", alferd_path.display()))?;

    ui::success(format!("Daemon started (PID: {})", child.id()));
    ui::dim("Model will be loaded on first request");

    Ok(())
}

pub fn stop() -> Result<()> {
    if !daemon_client::is_daemon_running() {
        ui::warn("Daemon is not running");
        return Ok(());
    }

    ui::info("Stopping daemon...");

    match daemon_client::connect() {
        Ok(mut client) => {
            client.shutdown()?;
            ui::success("Daemon stopped");
        }
        Err(_) => {
            // Try to kill by PID file
            kill_by_pid_file()?;
        }
    }

    Ok(())
}

pub fn status() -> Result<()> {
    let daemon_config = config::get_daemon_config();

    println!("{}", "Daemon Status".cyan().bold());
    println!();

    if daemon_client::is_daemon_running() {
        println!("  {} {}", "Status:".dimmed(), "Running".green());

        // Try to get PID from file
        if let Ok(pid) = read_pid_file() {
            println!("  {} {}", "PID:".dimmed(), pid);
        }
    } else {
        println!("  {} {}", "Status:".dimmed(), "Stopped".red());
    }

    println!("  {} {}", "Port:".dimmed(), daemon_config.port);
    println!(
        "  {} {}",
        "Idle timeout:".dimmed(),
        if daemon_config.idle_timeout_minutes > 0 {
            format!("{} minutes", daemon_config.idle_timeout_minutes)
        } else {
            "disabled".to_string()
        }
    );

    // Check if service is installed
    let service_installed = is_service_installed();
    println!(
        "  {} {}",
        "Service:".dimmed(),
        if service_installed {
            "Installed".green().to_string()
        } else {
            "Not installed".dimmed().to_string()
        }
    );

    Ok(())
}

pub fn install() -> Result<()> {
    let alferd_path = find_alferd_binary()?;

    #[cfg(target_os = "macos")]
    {
        install_launchd(&alferd_path)?;
    }

    #[cfg(target_os = "linux")]
    {
        install_systemd(&alferd_path)?;
    }

    #[cfg(target_os = "windows")]
    {
        return Err(anyhow!(
            "Windows service installation not yet implemented.\n\
             Run 'alfred daemon start' manually for now."
        ));
    }

    Ok(())
}

pub fn uninstall() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        uninstall_launchd()?;
    }

    #[cfg(target_os = "linux")]
    {
        uninstall_systemd()?;
    }

    #[cfg(target_os = "windows")]
    {
        return Err(anyhow!("Windows service uninstallation not yet implemented."));
    }

    Ok(())
}

// --- Helper functions ---

fn find_alferd_binary() -> Result<std::path::PathBuf> {
    // Check same directory as current executable
    if let Ok(current_exe) = std::env::current_exe() {
        let dir = current_exe.parent().unwrap();
        let alferd = dir.join("alferd");
        if alferd.exists() {
            return Ok(alferd);
        }
    }

    // Check PATH
    if let Ok(output) = Command::new("which").arg("alferd").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(std::path::PathBuf::from(path));
            }
        }
    }

    // Check common locations
    let home = dirs::home_dir().unwrap_or_default();
    let common_paths: Vec<std::path::PathBuf> = vec![
        std::path::PathBuf::from("/usr/local/bin/alferd"),
        std::path::PathBuf::from("/usr/bin/alferd"),
        home.join(".cargo/bin/alferd"),
    ];

    for path in common_paths {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(anyhow!(
        "Could not find alferd binary.\n\
         Make sure it's installed in the same directory as alfred or in your PATH."
    ))
}

fn read_pid_file() -> Result<u32> {
    let pid_path = config::pid_file();
    let content = fs::read_to_string(&pid_path)
        .with_context(|| "No PID file found")?;
    content.trim().parse().with_context(|| "Invalid PID")
}

fn kill_by_pid_file() -> Result<()> {
    let pid = read_pid_file()?;

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let _ = Command::new("kill").arg(pid.to_string()).exec();
    }

    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()?;
    }

    // Remove PID file
    let _ = fs::remove_file(config::pid_file());

    ui::success("Daemon stopped");
    Ok(())
}

fn is_service_installed() -> bool {
    #[cfg(target_os = "macos")]
    {
        launchd_plist_path().exists()
    }

    #[cfg(target_os = "linux")]
    {
        systemd_service_path().exists()
    }

    #[cfg(target_os = "windows")]
    {
        false // TODO: Check Windows service
    }
}

// --- macOS launchd ---

#[cfg(target_os = "macos")]
fn launchd_plist_path() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join("Library/LaunchAgents/com.alfred.daemon.plist")
}

#[cfg(target_os = "macos")]
fn install_launchd(alferd_path: &std::path::Path) -> Result<()> {
    let plist_path = launchd_plist_path();

    // Create LaunchAgents directory if needed
    if let Some(parent) = plist_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.alfred.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>StandardOutPath</key>
    <string>{}/alferd.log</string>
    <key>StandardErrorPath</key>
    <string>{}/alferd.error.log</string>
</dict>
</plist>
"#,
        alferd_path.display(),
        config::alfred_dir().display(),
        config::alfred_dir().display()
    );

    fs::write(&plist_path, plist_content)?;

    // Load the service
    let output = Command::new("launchctl")
        .args(["load", "-w"])
        .arg(&plist_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to load launchd service: {}", stderr));
    }

    ui::success("Daemon installed as launchd service");
    ui::dim(format!("Plist: {}", plist_path.display()));
    ui::dim("The daemon will start automatically at login");

    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_launchd() -> Result<()> {
    let plist_path = launchd_plist_path();

    if !plist_path.exists() {
        ui::warn("Service is not installed");
        return Ok(());
    }

    // Unload the service
    let _ = Command::new("launchctl")
        .args(["unload", "-w"])
        .arg(&plist_path)
        .output();

    // Remove plist file
    fs::remove_file(&plist_path)?;

    ui::success("Daemon service uninstalled");

    Ok(())
}

// --- Linux systemd ---

#[cfg(target_os = "linux")]
fn systemd_service_path() -> std::path::PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".config/systemd/user/alfred.service")
}

#[cfg(target_os = "linux")]
fn install_systemd(alferd_path: &std::path::Path) -> Result<()> {
    let service_path = systemd_service_path();

    // Create systemd user directory if needed
    if let Some(parent) = service_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let service_content = format!(
        r#"[Unit]
Description=Alfred AI Daemon
After=network.target

[Service]
Type=simple
ExecStart={}
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
"#,
        alferd_path.display()
    );

    fs::write(&service_path, service_content)?;

    // Reload systemd and enable service
    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output()?;

    let output = Command::new("systemctl")
        .args(["--user", "enable", "--now", "alfred"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Failed to enable systemd service: {}", stderr));
    }

    ui::success("Daemon installed as systemd user service");
    ui::dim(format!("Service file: {}", service_path.display()));
    ui::dim("The daemon will start automatically at login");

    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_systemd() -> Result<()> {
    let service_path = systemd_service_path();

    if !service_path.exists() {
        ui::warn("Service is not installed");
        return Ok(());
    }

    // Disable and stop service
    let _ = Command::new("systemctl")
        .args(["--user", "disable", "--now", "alfred"])
        .output();

    // Remove service file
    fs::remove_file(&service_path)?;

    // Reload systemd
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();

    ui::success("Daemon service uninstalled");

    Ok(())
}
