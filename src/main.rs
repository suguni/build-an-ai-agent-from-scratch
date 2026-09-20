#![allow(unused)]

use agent::tools::{calculator_tool, search_web_tool};
use anyhow::Context;
use std::io::Write;

pub mod agent;
pub mod anthropic;
pub mod ollama;
pub mod rag;
pub mod vector_search;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;
    chat().await?;
    Ok(())
}

async fn chat() -> anyhow::Result<()> {
    let api_key = get_api_key()?;
    let system_prompt = "You are a helpful assistant. Use the search tool when you need current information. Answer with Korean.";
    let mut agent = agent::Agent::new(
        &api_key,
        vec![Box::new(calculator_tool()), Box::new(search_web_tool()?)],
        system_prompt,
    );
    // what is 1234 times 5678 ?
    // Who won the 2025 Nobel Prize in Physics?
    let response = agent.chat("If marathon runner Eliud Kipchoge could maintain his world record pace indefinitely, how long would it take him to reach the Moon?").await?;
    println!("{response}");
    Ok(())
}

fn get_api_key() -> anyhow::Result<String> {
    dotenvy::dotenv()?;
    std::env::var("ANTHROPIC_API_KEY").context("ANTHROPIC_API_KEY 가져오기 실패")
}
