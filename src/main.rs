use anyhow::Context;
use serde_json::{Value, from_str};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use crate::anthropic::common::MessageParam;
use crate::anthropic::request::Request;
use crate::anthropic::response::Message;

mod anthropic;
mod common;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = get_api_key()?;

    structured_response(&api_key).await?;
    // multi_call(&api_key).await?;

    Ok(())
}

async fn structured_response(api_key: &str) -> anyhow::Result<()> {
    let message_param = MessageParam::user("my name is Stuart 이고 이메일은 stuart@example.com, 전화번호는 010-1234-4432 이다.");
    let config = Request::config(serde_json::json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "email": {"type": "string"},
            "phone": {"type": "string"},
        },
        "required": ["name", "email"],
        "additionalProperties": false
    }));
    let request = Request::message(message_param, config);
    let _message = call(&api_key, &request).await?;
    Ok(())
}

async fn multi_call(api_key: &str) -> anyhow::Result<()> {
    let mut message_params = vec![];

    message_params.push(MessageParam::user("my name is Stuart"));
    let request = Request::messages(&message_params);
    let message = call(&api_key, &request).await?;

    message_params.push(message.message_param());
    message_params.push(MessageParam::user("what is my name?"));
    let request = Request::messages(&message_params);
    let _contents = call(&api_key, &request).await?;

    Ok(())
}

async fn call(api_key: &str, request: &Request) -> anyhow::Result<Message> {
    let run = get_run()?;

    let response = call_message(api_key, &run, 1, request).await?;
    if !response.is_success {
        anyhow::bail!("응답 오류 status: {}, response: {}", response.status, response.raw_body);
    }

    let message = from_str::<Message>(&response.raw_body).context("응답 deserialize 실패")?;
    let (input_tokens, output_tokens) = message.usage_io_tokens();
    println!("{}", message.text());
    println!("input tokens: {input_tokens}, output tokens: {output_tokens}");

    Ok(message)
}

fn get_api_key() -> anyhow::Result<String> {
    dotenvy::dotenv()?;
    std::env::var("ANTHROPIC_API_KEY").context("ANTHROPIC_API_KEY 가져오기 실패")
}

fn get_run() -> anyhow::Result<String> {
    let now = OffsetDateTime::now_utc();
    let format = format_description!("[year][month][day]-[hour][minute][second]");
    now.format(&format).context("run 생성 포맷 오류")
}

const MESSAGE_URI: &str = "https://api.anthropic.com/v1/messages";

async fn call_message(
    api_key: &str,
    run: &str,
    turn: u16,
    req: &Request,
) -> anyhow::Result<CallResponse> {
    let client = reqwest::Client::new();

    log_request(run, turn, &req)?;

    let response = client
        .post(MESSAGE_URI)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&req)
        .send()
        .await
        .context("호출 오류");

    match response {
        Ok(response) => match handle_response(response).await {
            Ok(call_response) => {
                log_response(run, turn, &call_response)?;
                Ok(call_response)
            }
            Err(e) => {
                log_error(run, turn, &e.to_string())?;
                Err(e)
            }
        },
        Err(e) => {
            log_error(run, turn, &e.to_string())?;
            Err(e)
        }
    }
}

async fn handle_response(response: reqwest::Response) -> anyhow::Result<CallResponse> {
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

fn log_request(run: &str, turn: u16, request: &Request) -> anyhow::Result<()> {
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
    write_log(&serde_json::to_string(&json)?)?;
    Ok(())
}

fn log_error(run: &str, turn: u16, msg: &str) -> anyhow::Result<()> {
    let now = OffsetDateTime::now_utc();
    let ts = now.format(&Rfc3339)?;

    let json = serde_json::json!({
        "v": 1,
        "run": run,
        "turn": turn,
        "ts": ts,
        "dir": "err",
        "body": msg
    });
    write_log(&serde_json::to_string(&json)?)?;
    Ok(())
}

fn log_response(run: &str, turn: u16, call_response: &CallResponse) -> anyhow::Result<()> {
    let (body, body_raw) = match from_str::<Value>(&call_response.raw_body) {
        Ok(v) => (Some(v), None),
        Err(_) => (None, Some(call_response.raw_body.clone()))
    };

    let now = OffsetDateTime::now_utc();
    let ts = now.format(&Rfc3339)?;

    let json = serde_json::json!({
        "v": 1,
        "run": run,
        "turn": turn,
        "ts": ts,
        "dir": "res",
        "status": call_response.status,
        "headers": call_response.headers,
        "body": body,
        "body_raw": body_raw,
    });

    write_log(&serde_json::to_string(&json)?)?;
    Ok(())
}

const LOG_FILE_NAME: &str = "ring0.jsonl";

fn write_log(log: &str) -> anyhow::Result<()> {
    let mut file = File::options().create(true).append(true).open(LOG_FILE_NAME).context("파일 열기 실패")?;
    writeln!(file, "{}", log)?;
    file.flush()?;
    Ok(())
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

struct CallResponse {
    status: u16,
    is_success: bool,
    headers: HashMap<String, String>,
    raw_body: String,
}
