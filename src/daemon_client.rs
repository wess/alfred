use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::config;

#[derive(Serialize)]
struct Request {
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Deserialize)]
struct Response {
    result: Option<String>,
    error: Option<String>,
    #[allow(dead_code)]
    id: u64,
}

pub struct DaemonClient {
    stream: TcpStream,
    request_id: u64,
}

impl DaemonClient {
    fn send_request(&mut self, method: &str, params: serde_json::Value) -> Result<String> {
        self.request_id += 1;

        let request = Request {
            method: method.to_string(),
            params,
            id: self.request_id,
        };

        let mut request_str = serde_json::to_string(&request)?;
        request_str.push('\n');

        self.stream
            .write_all(request_str.as_bytes())
            .with_context(|| "Failed to send request to daemon")?;

        self.stream.flush()?;

        let mut reader = BufReader::new(&self.stream);
        let mut response_str = String::new();
        reader
            .read_line(&mut response_str)
            .with_context(|| "Failed to read response from daemon")?;

        let response: Response = serde_json::from_str(&response_str)
            .with_context(|| "Failed to parse daemon response")?;

        if let Some(error) = response.error {
            return Err(anyhow!("Daemon error: {}", error));
        }

        response.result.ok_or_else(|| anyhow!("Empty response from daemon"))
    }

    pub fn ping(&mut self) -> Result<String> {
        self.send_request("ping", serde_json::json!({}))
    }

    pub fn shutdown(&mut self) -> Result<String> {
        self.send_request("shutdown", serde_json::json!({}))
    }

    pub fn generate(&mut self, prompt: &str, max_tokens: u32) -> Result<String> {
        self.send_request(
            "generate",
            serde_json::json!({
                "prompt": prompt,
                "max_tokens": max_tokens
            }),
        )
    }

    pub fn generate_commit_message(&mut self, diff: &str) -> Result<String> {
        self.send_request(
            "generate_commit_message",
            serde_json::json!({
                "diff": diff
            }),
        )
    }

    pub fn suggest_branch_name(&mut self, description: &str) -> Result<String> {
        self.send_request(
            "suggest_branch_name",
            serde_json::json!({
                "description": description
            }),
        )
    }

    pub fn suggest_conflict_resolution(
        &mut self,
        file: &str,
        ours: &str,
        theirs: &str,
        base: &str,
    ) -> Result<String> {
        self.send_request(
            "suggest_conflict_resolution",
            serde_json::json!({
                "file": file,
                "ours": ours,
                "theirs": theirs,
                "base": base
            }),
        )
    }

    pub fn suggest_rebase_strategy(&mut self, commits: &[String], onto: &str) -> Result<String> {
        self.send_request(
            "suggest_rebase_strategy",
            serde_json::json!({
                "commits": commits,
                "onto": onto
            }),
        )
    }
}

pub fn is_daemon_running() -> bool {
    connect().is_ok()
}

pub fn connect() -> Result<DaemonClient> {
    let daemon_config = config::get_daemon_config();
    let addr = format!("127.0.0.1:{}", daemon_config.port);

    let stream = TcpStream::connect_timeout(
        &addr.parse().unwrap(),
        Duration::from_millis(100),
    )
    .with_context(|| "Daemon not running")?;

    stream.set_read_timeout(Some(Duration::from_secs(120)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let mut client = DaemonClient {
        stream,
        request_id: 0,
    };

    // Verify connection with ping
    client.ping()?;

    Ok(client)
}
