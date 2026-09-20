use anyhow::Context;
use crate::agent::tools::tavily_client::TavilyClient;
use crate::{ollama, rag};
use crate::rag::vector::fixed_length_chunking;

pub async fn web_vector_search() -> anyhow::Result<()> {
    let api_key = std::env::var("TAVILY_API_KEY").context("TAVILY_API_KEY 가져오기 실패")?;
    let tavily_client = TavilyClient::new(&api_key)?;

    let request = tavily_client.new_request("2025 Nobel Prize winners")
        .max_results(10)
        .include_raw_content(true);

    let mut chunks = vec![];

    let response = tavily_client.search(&request).await?;

    for result in response.iter() {
        println!("query result -> {:?}", result);
        if let Some(ref raw_content) = result.raw_content {
            let text = format!("Title: {}\n{}", result.title, raw_content);
            for chunk in fixed_length_chunking(&text, 500, 50) {
                chunks.push(WebSearchChunk {
                    text: chunk.into(),
                    title: &result.title,
                    url: &result.url,
                });
            }
        }
    }

    let chunk_texts = chunks.iter().map(|c| &c.text).collect::<Vec<_>>();
    println!("chunk count -> {}", chunk_texts.len());

    let chunk_embeddings = ollama::get_embeddings(&chunk_texts).await?;

    let query = "quantum computing";
    let results = rag::vector::vector_search(query, &chunk_embeddings, 3).await?;

    println!("query: {}", query);
    println!("======================================================");
    for (i, r) in results {
        println!("[{}] Similarity: {}", i, r);
        println!("{:?}", chunks[i]);
    }

    Ok(())
}

#[derive(Debug)]
struct WebSearchChunk<'a> {
    text: String,
    title: &'a str,
    url: &'a str,
}

pub async fn simple_vector_search() -> anyhow::Result<()> {
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
