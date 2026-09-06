use crate::anthropic::tools::{Tool, ToolResult, ToolSpec, ToolUse};
use anyhow::bail;
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use std::pin::Pin;

pub fn calculator_tool() -> impl Tool {
    Calculator {}
}

pub struct Calculator {}

impl Tool for Calculator {
    fn name(&self) -> &'static str {
        "calculator"
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "calculator",
            description: Some("Perform basic arithmetic operations."),
            input_schema: schema_for!(CalculatorInput).to_value(),
        }
    }

    fn run(
        &self,
        tool_use: ToolUse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ToolResult>> + Send + '_>> {
        Box::pin(async move {
            let c = serde_json::from_value::<CalculatorInput>(tool_use.input)?;
            c.calculate()
                .map(|r| ToolResult::new(tool_use.id, format!("{}", r)))
        })
    }
}

#[derive(JsonSchema, Deserialize)]
struct CalculatorInput {
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

impl CalculatorInput {
    pub fn calculate(&self) -> anyhow::Result<f64> {
        match self.operator {
            CalculatorOperator::Add => Ok(self.first_number + self.second_number),
            CalculatorOperator::Subtract => Ok(self.first_number - self.second_number),
            CalculatorOperator::Multiply => Ok(self.first_number * self.second_number),
            CalculatorOperator::Divide => {
                if self.second_number != 0.0 {
                    Ok(self.first_number / self.second_number)
                } else {
                    bail!("tool Calculator - divide by zero")
                }
            }
        }
    }
}
