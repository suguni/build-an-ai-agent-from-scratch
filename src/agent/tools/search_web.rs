use crate::agent::tools::{Tool, ToolError, ToolResult, ToolSpec, ToolUse};
use anyhow::Context;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;
use std::time::Duration;
use tavily::{SearchRequest, SearchResult, Tavily};

pub struct SearchWeb {}

pub fn search_web_tool() -> impl Tool {
    SearchWeb {}
}

impl Tool for SearchWeb {
    fn name(&self) -> &'static str {
        "search_web"
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "search_web",
            description: Some("Search the web for the given query."),
            input_schema: schema_for!(SearchWebInput).to_value(),
        }
    }

    fn run(
        &self,
        tool_use: ToolUse,
    ) -> Pin<Box<dyn Future<Output = Result<ToolResult, ToolError>> + Send + '_>> {
        Box::pin(async move {
            let search = serde_json::from_value::<SearchWebInput>(tool_use.input)
                .map_err(|e| ToolError { tool_use_id: tool_use.id.clone(), cause: anyhow::Error::new(e) })?;

            search
                .search()
                .await
                .map_err(|e| ToolError { tool_use_id: tool_use.id.clone(), cause: e })
                .map(|r| ToolResult::new(tool_use.id, format!("{}", r)))
        })
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchWebInput {
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
    Finance,
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

impl SearchWebInput {
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
