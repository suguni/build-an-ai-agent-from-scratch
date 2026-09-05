#![allow(unused)]

use anyhow::Context;
use serde_json::{Value, from_str};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use tavily::Tavily;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use crate::anthropic::Agent;
use crate::anthropic::common::{ContentBlockParam, MessageParam, Role};
use crate::anthropic::request::Request;
use crate::anthropic::response::Message;
use crate::anthropic::tools::calculator_tool;

mod anthropic;
mod common;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = get_api_key()?;
    let tavily = tavily_client()?;

    // let response = tavily.search("Kipchoge's marathon world record").await?;
    // println!("{:?}", response);

    tool_call(&api_key).await?;
    // concurrent_call(&api_key).await?;
    // structured_response(&api_key).await?;
    // multi_call(&api_key).await?;
    Ok(())
}

async fn tool_call(api_key: &str) -> anyhow::Result<()> {
    // question & tool
    let user_message_param = MessageParam::user("What is 1234 x 5678 ?");
    let request = Request::message_with_tool(&[user_message_param.clone()], calculator_tool());
    let response_message = call(&api_key, &request).await?;

    let bot_message_param = response_message.message_param();
    // dbg!(&bot_message_param);

    // tool call
    let tool_results = response_message.tool_calls()
        .into_iter()
        .filter_map(|tool_use| { if let Some(t) = tool_use.run() { Some(ContentBlockParam::ToolResult(t)) } else { None } })
        .collect::<Vec<_>>();

    let tool_result_message_param = MessageParam::new(Role::User, tool_results);
    // dbg!(&tool_result_message_param);

    let request = Request::message_with_tool(&[
        user_message_param.clone(),
        bot_message_param,
        tool_result_message_param
    ], calculator_tool());
    // dbg!(&request);

    // send tool result
    let response_message = call(&api_key, &request).await?;

    dbg!(&response_message.text());
    Ok(())
}

async fn send(api_key: &str, msg: &str) -> anyhow::Result<Message> {
    let message_param = MessageParam::user(msg);
    let request = Request::message(message_param);
    call(&api_key, &request).await
}

async fn concurrent_call(api_key: &str) -> anyhow::Result<()> {
    let messages = [
        "What is 2 + 2?",
        "What is the capital of Japan?",
        "Who wrote Romeo and Juliet?"
    ];

    let mut handles = vec![];

    for msg in messages {
        let api_key = api_key.to_string();
        let handle = tokio::spawn(async move {
            let message_param = MessageParam::user(msg);
            let request = Request::message(message_param);
            call(&api_key, &request).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await?;
    }

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
    let request = Request::message_with_config(message_param, Some(config));
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

    let agent = Agent::new(api_key);

    let response = agent.call_message(request).await?;
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

fn tavily_client() -> anyhow::Result<Tavily> {
    dotenvy::dotenv()?;
    let key = std::env::var("TAVILY_API_KEY").context("TAVILY_API_KEY 가져오기 실패")?;
    Tavily::builder(&key)
        .timeout(Duration::from_secs(60))
        .max_retries(5)
        .build()
        .context("Tavily creation failed")
}

