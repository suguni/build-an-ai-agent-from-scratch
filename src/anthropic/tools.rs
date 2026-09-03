use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Debug, Serialize)]
pub struct Tool {
    name: &'static str,
    description: Option<&'static str>,
    input_schema: Value,
}

pub fn calculator_tool() -> Tool {
    Tool {
        name: "calculator",
        description: Some("Perform basic arithmetic operations."),
        input_schema: schema_for!(Calculator).to_value(),
    }
}

#[derive(JsonSchema, Deserialize)]
pub struct Calculator {
    operator: CalculatorOperator,
    first_number: f64,
    second_number: f64,
}

#[derive(JsonSchema, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalculatorOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Calculator {
    fn calculate(&self) -> f64 {
        match self.operator {
            CalculatorOperator::Add => self.first_number + self.second_number,
            CalculatorOperator::Subtract => self.first_number - self.second_number,
            CalculatorOperator::Multiply => self.first_number * self.second_number,
            CalculatorOperator::Divide => self.first_number / self.second_number,
        }
    }
}
