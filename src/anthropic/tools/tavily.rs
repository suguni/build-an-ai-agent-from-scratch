use crate::anthropic::tools::Tool;
use anyhow::Context;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tavily::{SearchRequest, SearchResult, Tavily};

pub fn tavily_tool() -> Tool {
    Tool {
        name: "search_web",
        description: Some("Search the web for the given query."),
        input_schema: schema_for!(SearchWeb).to_value(),
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchWeb {
    query: String,
    max_result: i32,
    topic: Topic,
    time_range: Option<String>,
    country: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Topic {
    General,
    News,
    Finance
}

impl Topic {
    fn as_str(&self) -> &'static str {
        match self {
            Topic::General => "general",
            Topic::News => "news",
            Topic::Finance => "finance",
        }
    }
}

impl SearchWeb {
    pub async fn search(&self) -> anyhow::Result<String> {
        dotenvy::dotenv()?;

        let api_key = std::env::var("TAVILY_API_KEY").context("TAVILY_API_KEY 가져오기 실패")?;

        let client = Tavily::builder(&api_key)
            .timeout(Duration::from_secs(60))
            .max_retries(5)
            .build()?;

        let mut req = SearchRequest::new(api_key, &self.query)
            .max_results(self.max_result)
            .topic(self.topic.as_str());

        let result = client.call(&req).await.map(|it| {
            it.results
                .into_iter()
                .map(|s| MySearchResult {
                    title: s.title,
                    url: s.url,
                    content: s.content,
                    raw_content: s.raw_content,
                    score: s.score,
                })
                .collect::<Vec<_>>()
        })?;

        serde_json::to_string(&result).context("검색 결과 serialize 실패")
    }
}

#[derive(Debug, Serialize)]
struct MySearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
    pub raw_content: Option<String>,
    pub score: f32,
}
