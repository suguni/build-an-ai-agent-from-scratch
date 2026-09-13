use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tavily::{SearchRequest, Tavily};

pub struct TavilyClient {
    client: Tavily,
    api_key: String,
}

impl TavilyClient {
    pub fn new(api_key: &str) -> tavily::Result<Self> {
        let client = Tavily::builder(api_key)
            .timeout(Duration::from_secs(60))
            .max_retries(5)
            .build()?;

        Ok(Self {
            client,
            api_key: api_key.to_string(),
        })
    }

    pub fn new_request(&self, query: &str) -> SearchRequest {
        SearchRequest::new(&self.api_key, query)
    }

    pub async fn search(&self, request: &SearchRequest) -> anyhow::Result<Vec<TavilySearchResult>> {
        self.client
            .call(&request)
            .await
            .map(|it| {
                it.results
                    .into_iter()
                    .map(|s| TavilySearchResult {
                        title: s.title,
                        url: s.url,
                        content: s.content,
                        raw_content: s.raw_content,
                        score: s.score,
                    })
                    .collect::<Vec<_>>()
            })
            .context("Tavily 검색 오류")
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TavilySearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub raw_content: Option<String>,
    pub score: f32,
}
