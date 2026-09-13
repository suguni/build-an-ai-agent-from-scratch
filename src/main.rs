#![allow(unused)]

use agent::tools::{calculator_tool, search_web_tool};
use anyhow::Context;
use std::io::Write;

pub mod anthropic;
pub mod agent;
pub mod ollama;
pub mod rag;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vector_search().await?;
    // chat().await
    Ok(())
}

async fn vector_search() -> anyhow::Result<()> {
    let documents = [
        "Python is a programming language",
        "Machine learning uses Python extensively",
        "Cats are popular pets",
        "Deep learning is a subset of machine learning"
    ];

    let doc_embeddings = ollama::get_embeddings(&documents).await?;

    let results = rag::vector::vector_search(
        "Artificial Intelligence", &doc_embeddings, 4).await?;

    for (i, similarity) in results {
        println!("{}: {}", documents[i], similarity);
    }

    Ok(())
}

async fn similarity() -> anyhow::Result<()> {
    let sentences = [
        "The cat is sleeping on the couch",
        "A kitten is playing with a toy",
        "The dog is running in the park"
    ];

    let embeddings = ollama::get_embeddings(&sentences).await?;
    let cat_kitten = innr::cosine(&embeddings[0], &embeddings[1]);
    let cat_dog = innr::cosine(&embeddings[0], &embeddings[2]);

    println!("Cat vs Kitten: {cat_kitten}");
    println!("Cat vs Dog: {cat_dog}");

    Ok(())
}

async fn chat() -> anyhow::Result<()> {
    let api_key = get_api_key()?;
    let system_prompt =
        "You are a helpful assistant. Use the search tool when you need current information. Answer with Korean.";
    let mut agent = agent::Agent::new(
        &api_key,
        vec![Box::new(calculator_tool()), Box::new(search_web_tool())],
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
