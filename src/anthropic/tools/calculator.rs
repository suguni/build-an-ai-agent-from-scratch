use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use crate::anthropic::tools::Tool;

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
    pub fn calculate(&self) -> f64 {
        match self.operator {
            CalculatorOperator::Add => self.first_number + self.second_number,
            CalculatorOperator::Subtract => self.first_number - self.second_number,
            CalculatorOperator::Multiply => self.first_number * self.second_number,
            CalculatorOperator::Divide => self.first_number / self.second_number,
        }
    }
}
