use anyhow::Context;
use serde_json::{Value, from_str};
use std::fs::File;
use std::io::Write;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::anthropic::http_client::Response;

const LOG_FILE_NAME: &'static str = "ring0.jsonl";

pub struct HttpCallLogger {
    log_filename: String,
}

impl HttpCallLogger {
    
    pub fn new() -> Self {
        Self { log_filename: LOG_FILE_NAME.to_string() }
    }
    
    pub fn request(&self, request: &Value, run: &str, turn: u32) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc();
        let ts = now.format(&Rfc3339)?;
        let json = serde_json::json!({
            "v": 1,
            "run": run,
            "turn": turn,
            "ts": ts,
            "dir": "req",
            "body": request,
        });
        let log = serde_json::to_string(&json)?;
        write_log(&self.log_filename, &log)?;
        Ok(())
    }

    pub fn error(&self, error_message: &str, run: &str, turn: u32) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc();
        let ts = now.format(&Rfc3339)?;
        let json = serde_json::json!({
            "v": 1,
            "run": run,
            "turn": turn,
            "ts": ts,
            "dir": "err",
            "body": error_message
        });
        let log = serde_json::to_string(&json)?;
        write_log(&self.log_filename, &log)?;
        Ok(())
    }

    pub fn response(&self, response: &Response, run: &str, turn: u32) -> anyhow::Result<()> {
        let (body, body_raw) = match from_str::<Value>(&response.raw_body) {
            Ok(v) => (Some(v), None),
            Err(_) => (None, Some(response.raw_body.clone())),
        };

        let now = OffsetDateTime::now_utc();
        let ts = now.format(&Rfc3339)?;

        let json = serde_json::json!({
            "v": 1,
            "run": run,
            "turn": turn,
            "ts": ts,
            "dir": "res",
            "status": response.status,
            "headers": response.headers,
            "body": body,
            "body_raw": body_raw,
        });

        let log = serde_json::to_string(&json)?;
        write_log(&self.log_filename, &log)?;
        Ok(())
    }
}

pub fn write_log(filename: &str, log: &str) -> anyhow::Result<()> {
    let mut file = File::options()
        .create(true)
        .append(true)
        .open(filename)
        .context("파일 열기 실패")?;
    writeln!(file, "{}", log)?;
    file.flush()?;
    Ok(())
}
