pub mod calculator;

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::anthropic::tools::calculator::Calculator;

pub use crate::anthropic::tools::calculator::calculator_tool;

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
    pub fn run(self) -> Option<ToolResult> {
        if self.name == "calculator".to_string() {
            if let Ok(c) = serde_json::from_value::<Calculator>(self.input) {
                Some(ToolResult {
                    tool_use_id: self.id.clone(),
                    content: format!("{}", c.calculate()),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Tool {
    name: &'static str,
    description: Option<&'static str>,
    input_schema: Value,
}
