use const_format::concatcp;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct HttpClient {}

const URL_BASE: &'static str = "http://localhost:11434";
const API_EMBEDDING: &'static str = concatcp!(URL_BASE, "/api/embed");

const EMBEDDING_MODEL: &'static str = "qwen3-embedding";
// const EMBEDDING_MODEL: &'static str = "nomic-embed-text";

impl HttpClient {
    pub fn new() -> Self {
        HttpClient {}
    }

    pub async fn embedding<R: AsRef<str> + Serialize>(&self, input: &[R]) -> reqwest::Result<OllamaResponse> {
        let client = reqwest::Client::new();
        let input = input.as_ref();
        let response = client
            .post(API_EMBEDDING)
            .json(&json!({
                "model": EMBEDDING_MODEL,
                "input": input 
            }))
            .send()
            .await?;

        response.json::<OllamaResponse>().await
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub embeddings: Vec<Vec<f32>>,
    pub total_duration: u64,
    pub load_duration: u64,
    pub prompt_eval_count: u32,
}
