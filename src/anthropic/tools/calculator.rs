use crate::anthropic::tools::Tool;
use anyhow::bail;
use schemars::{JsonSchema, schema_for};
use serde::Deserialize;

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
