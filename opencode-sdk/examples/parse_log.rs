//! Parse opencode-sdk log file and extract event payloads.
//!
//! Run with: cargo run --example parse_log [LOG_PATH]
//!
//! Default log path: ~/.local/share/opencode-sdk/opencode-sdk.log
//! Output: one JSON object per parsed event (timestamp, type, payload).

use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Default log path under user's local share.
fn default_log_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencode-sdk")
        .join("opencode-sdk.log")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_log_path);

    if !log_path.exists() {
        eprintln!("Log file not found: {}", log_path.display());
        std::process::exit(1);
    }

    let file = BufReader::new(File::open(&log_path)?);
    let mut lines = file.lines();

    // Log line: "2026-02-01T03:25:09.387422Z  INFO opencode_sdk::event: event full payload:"
    // Next lines until a line that looks like a new log entry (timestamp at start) are the payload.
    let log_prefix = regex::Regex::new(r"^(\d{4}-\d{2}-\d{2}T[^\s]+)\s+(\w+)\s+([^:]+):\s*(.*)$")?;

    let mut pending_line: Option<String> = None;

    loop {
        let line = if let Some(pl) = pending_line.take() {
            pl
        } else if let Some(Ok(l)) = lines.next() {
            l
        } else {
            break;
        };

        let trimmed = line.trim();
        if let Some(cap) = log_prefix.captures(trimmed) {
            let ts = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let msg = cap.get(4).map(|m| m.as_str()).unwrap_or("").trim();

            if msg == "event full payload:" || msg.starts_with("event full payload:") {
                let mut json_lines = Vec::new();
                if msg.len() > "event full payload:".len() {
                    let rest = msg["event full payload:".len()..].trim();
                    if !rest.is_empty() {
                        json_lines.push(rest.to_string());
                    }
                }
                while let Some(Ok(next_line)) = lines.next() {
                    let t = next_line.trim();
                    if t.is_empty() {
                        continue;
                    }
                    if log_prefix.is_match(t) {
                        pending_line = Some(next_line);
                        break;
                    }
                    json_lines.push(next_line);
                }
                let joined = json_lines.join("\n");
                let json_str = joined.trim();
                if json_str.is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<Value>(json_str) {
                    let event_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("?");
                    let out = serde_json::json!({
                        "timestamp": ts,
                        "type": event_type,
                        "payload": v
                    });
                    println!("{}", serde_json::to_string(&out)?);
                }
            }
        }
    }

    Ok(())
}
