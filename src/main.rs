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
use crate::anthropic::tools::{calculator_tool, tavily_tool};

mod anthropic;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = get_api_key()?;
    let system_prompt = "You are a helpful assistant. Use the search tool when you need current information.";
    let mut agent = Agent::new(&api_key, vec![calculator_tool(), tavily_tool()], system_prompt);
    let response = agent.chat("Who won the 2025 Nobel Prize in Physics?").await?;
    println!("{response}");
    Ok(())
}

fn get_api_key() -> anyhow::Result<String> {
    dotenvy::dotenv()?;
    std::env::var("ANTHROPIC_API_KEY").context("ANTHROPIC_API_KEY 가져오기 실패")
}
