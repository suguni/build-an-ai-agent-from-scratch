pub mod calculator;
pub mod tavily;

use crate::anthropic::tools::calculator::Calculator;
use anyhow::bail;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::anthropic::tools::calculator::calculator_tool;
use crate::anthropic::tools::tavily::SearchWeb;
pub use crate::anthropic::tools::tavily::tavily_tool;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolUse {
    id: String,
    name: String,
    input: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolResult {
    tool_use_id: String,
    content: String,
}

impl ToolUse {
    pub async fn run(self) -> anyhow::Result<ToolResult> {
        let content = if self.name == "calculator".to_string() {
            let c = serde_json::from_value::<Calculator>(self.input)?;
            format!("{}", c.calculate()?)
        } else if self.name == "search_web".to_string() {
            let search = serde_json::from_value::<SearchWeb>(self.input)?;
            format!("{}", search.search().await?)
        } else {
            eprintln!("unknown tool {}", self.name);
            bail!("unknown tool {}", self.name);
        };

        Ok(ToolResult {
            tool_use_id: self.id.clone(),
            content,
        })
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Tool {
    name: &'static str,
    description: Option<&'static str>,
    input_schema: Value,
}
