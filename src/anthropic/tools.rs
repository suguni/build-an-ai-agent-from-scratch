pub mod calculator;
pub mod search_web;

use crate::anthropic::tools::calculator::Calculator;
use anyhow::bail;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;

pub use crate::anthropic::tools::calculator::calculator_tool;
use crate::anthropic::tools::search_web::SearchWebInput;
pub use crate::anthropic::tools::search_web::search_web_tool;

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
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ToolResult>> + Send + 'a>>;
}
