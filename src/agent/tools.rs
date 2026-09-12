pub mod calculator;
pub mod search_web;

pub use crate::agent::tools::calculator::calculator_tool;
pub use crate::agent::tools::search_web::search_web_tool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolUse {
    id: String,
    name: String,
    input: Value,
}

impl ToolUse {
    pub fn find_tool<'a>(&self, tools: &'a [Box<dyn Tool>]) -> Option<&'a dyn Tool> {
        tools.iter()
            .find(|&t| t.name() == self.name)
            .map(|t| t.as_ref())
    }

    pub fn error_result(&self, message: &str) -> ToolResult {
        ToolResult::new(self.id.clone(), message.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolResult {
    tool_use_id: String,
    content: String,
}

impl ToolResult {
    pub fn new(tool_use_id: String, content: String) -> Self {
        ToolResult {
            tool_use_id,
            content,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct ToolSpec {
    name: &'static str,
    description: Option<&'static str>,
    input_schema: Value,
}

pub trait Tool {
    fn name(&self) -> &'static str;
    fn spec(&self) -> ToolSpec;
    fn run<'a>(
        &'a self,
        tool_use: ToolUse,
    ) -> Pin<Box<dyn Future<Output = Result<ToolResult, ToolError>> + Send + 'a>>;
}

#[derive(Debug, Error)]
#[error("id: {tool_use_id}, cause: {cause}")]
pub struct ToolError { tool_use_id: String, cause: anyhow::Error }

impl ToolError {
    pub fn tool_result(&self) -> ToolResult {
        ToolResult { tool_use_id: self.tool_use_id.clone(), content: self.cause.to_string() }
    }
}
