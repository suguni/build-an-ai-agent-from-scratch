use crate::agent::tools::{Tool, ToolError, ToolResult, ToolSpec, ToolUse};
use anyhow::Context;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;
use std::time::Duration;
use tavily::{SearchRequest, SearchResult, Tavily};
use crate::agent::tools::tavily_client::TavilyClient;

pub struct SearchWeb {
    tavily_client: TavilyClient,
}

pub fn search_web_tool() -> anyhow::Result<impl Tool> {
    dotenvy::dotenv()?;
    let api_key = std::env::var("TAVILY_API_KEY").context("TAVILY_API_KEY 가져오기 실패")?;
    let tavily_client = TavilyClient::new(&api_key)?;
    Ok(SearchWeb { tavily_client })
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
                .search(&self.tavily_client)
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
    pub async fn search(&self, tavily_client: &TavilyClient) -> anyhow::Result<String> {
        let req = tavily_client
            .new_request(&self.query)
            .max_results(self.max_result)
            .topic(self.topic.as_str());
        let result = tavily_client.search(&req).await?;
        serde_json::to_string(&result).context("검색 결과 serialize 실패")
    }
}
