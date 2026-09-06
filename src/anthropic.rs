use crate::anthropic::common::{ContentBlockParam, MessageParam, Role};
use crate::anthropic::request::Request;
use crate::anthropic::response::Message;
use crate::anthropic::tools::{Tool, ToolSpec};
use anyhow::Context;
use serde_json::{Value, from_str};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

pub mod common;
pub mod request;
pub mod response;
pub mod tools;

pub struct Agent {
    api_key: String,
    turn: u16,
    run: String,
    log_filename: String,
    messages: Vec<MessageParam>,
    tools: Vec<Box<dyn Tool>>,
    system_prompt: String,
}

pub const DEFAULT_MODEL: &str = "claude-haiku-4-5";
pub const DEFAULT_MAX_TOKEN: u32 = 1024;

const LOG_FILE_NAME: &str = "ring0.jsonl";
const MESSAGE_URI: &str = "https://api.anthropic.com/v1/messages";

impl Agent
{
    pub fn new(api_key: &str, tools: Vec<Box<dyn Tool>>, system_prompt: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            turn: 1,
            run: get_run().unwrap(),
            log_filename: LOG_FILE_NAME.to_string(),
            messages: Vec::new(),
            tools,
            system_prompt: system_prompt.to_string(),
        }
    }

    pub async fn chat(&mut self, message: &str) -> anyhow::Result<String> {
        let user_chat = MessageParam::user(message);
        self.messages.push(user_chat);

        let message = loop {
            let request = Request::create(
                DEFAULT_MODEL,
                DEFAULT_MAX_TOKEN,
                self.messages.clone(),
                vec![ContentBlockParam::text(&self.system_prompt)],
                None,
                self.tools.iter().map(|t| t.spec()).collect::<Vec<_>>(),
            );

            let response = self.call(&request).await?;
            if !response.is_success {
                anyhow::bail!(
                    "응답 오류 status: {}, response: {}",
                    response.status,
                    response.raw_body
                );
            }

            let message =
                from_str::<Message>(&response.raw_body).context("응답 deserialize 실패")?;
            self.messages.push(message.message_param());

            if message.next_tool_use() {
                let tool_result = self.use_tool(&message).await;
                self.messages.push(tool_result);
            } else {
                break message;
            }
        };

        Ok(message.text())
    }

    async fn use_tool(&self, message: &Message) -> MessageParam {
        let mut result = vec![];
        for tool_use in message.tool_calls() {
            if let Some(tool) = tool_use.find_tool(&self.tools) {
                match tool.run(tool_use).await {
                    Ok(t) => result.push(ContentBlockParam::ToolResult(t)),
                    Err(e) => eprintln!("{:?}", e),
                }
            } else {
                eprintln!("cannot find tool {:?}", tool_use);
            }
        }
        MessageParam::new(Role::User, result)
    }

    pub async fn call(&self, req: &Request) -> anyhow::Result<CallResponse> {
        let client = reqwest::Client::new();
        self.log_request(&req)?;

        let response = client
            .post(MESSAGE_URI)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&req)
            .send()
            .await
            .context("호출 오류");

        match response {
            Ok(response) => match self.handle_response(response).await {
                Ok(call_response) => {
                    self.log_response(&call_response)?;
                    Ok(call_response)
                }
                Err(e) => {
                    self.log_error(&e.to_string())?;
                    Err(e)
                }
            },
            Err(e) => {
                self.log_error(&e.to_string())?;
                Err(e)
            }
        }
    }

    async fn handle_response(&self, response: reqwest::Response) -> anyhow::Result<CallResponse> {
        let headers = extract_headers(&response, HEADERS);
        let status = response.status().as_u16();
        let is_success = response.status().is_success();
        let raw_body = response.text().await.context("응답 읽기 실패")?;
        Ok(CallResponse {
            status,
            is_success,
            headers,
            raw_body,
        })
    }

    fn log_request(&self, request: &Request) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc();
        let ts = now.format(&Rfc3339)?;
        let json = serde_json::json!({
            "v": 1,
            "run": self.run,
            "turn": self.turn,
            "ts": ts,
            "dir": "req",
            "body": request,
        });
        let log = serde_json::to_string(&json)?;
        write_log(&self.log_filename, &log)?;
        Ok(())
    }

    fn log_error(&self, msg: &str) -> anyhow::Result<()> {
        let now = OffsetDateTime::now_utc();
        let ts = now.format(&Rfc3339)?;

        let json = serde_json::json!({
            "v": 1,
            "run": self.run,
            "turn": self.turn,
            "ts": ts,
            "dir": "err",
            "body": msg
        });
        let log = serde_json::to_string(&json)?;
        write_log(&self.log_filename, &log)?;
        Ok(())
    }

    fn log_response(&self, call_response: &CallResponse) -> anyhow::Result<()> {
        let (body, body_raw) = match from_str::<Value>(&call_response.raw_body) {
            Ok(v) => (Some(v), None),
            Err(_) => (None, Some(call_response.raw_body.clone())),
        };

        let now = OffsetDateTime::now_utc();
        let ts = now.format(&Rfc3339)?;

        let json = serde_json::json!({
            "v": 1,
            "run": self.run,
            "turn": self.turn,
            "ts": ts,
            "dir": "res",
            "status": call_response.status,
            "headers": call_response.headers,
            "body": body,
            "body_raw": body_raw,
        });

        let log = serde_json::to_string(&json)?;
        write_log(&self.log_filename, &log)?;
        Ok(())
    }
}

const HEADERS: &'static [&'static str] = &[
    "request-id",
    "anthropic-ratelimit-input-tokens-limit",
    "anthropic-ratelimit-input-tokens-remaining",
    "anthropic-ratelimit-input-tokens-reset",
    "anthropic-ratelimit-output-tokens-limit",
    "anthropic-ratelimit-output-tokens-remaining",
    "anthropic-ratelimit-output-tokens-reset",
    "anthropic-ratelimit-requests-limit",
    "anthropic-ratelimit-requests-remaining",
    "anthropic-ratelimit-requests-reset",
    "anthropic-ratelimit-tokens-limit",
    "anthropic-ratelimit-tokens-remaining",
    "anthropic-ratelimit-tokens-reset",
];

fn write_log(filename: &str, log: &str) -> anyhow::Result<()> {
    let mut file = File::options()
        .create(true)
        .append(true)
        .open(filename)
        .context("파일 열기 실패")?;
    writeln!(file, "{}", log)?;
    file.flush()?;
    Ok(())
}

fn extract_headers(response: &reqwest::Response, headers: &[&str]) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|&key| {
            response
                .headers()
                .get(key)
                .map(|v| v.to_str().ok())
                .flatten()
                .map(|v| (key.to_string(), v.to_string()))
        })
        .collect::<HashMap<_, _>>()
}

fn get_run() -> anyhow::Result<String> {
    let now = OffsetDateTime::now_utc();
    let format = format_description!("[year][month][day]-[hour][minute][second]");
    now.format(&format).context("run 생성 포맷 오류")
}

pub struct CallResponse {
    pub status: u16,
    pub is_success: bool,
    pub headers: HashMap<String, String>,
    pub raw_body: String,
}
