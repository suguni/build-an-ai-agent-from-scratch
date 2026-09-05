use crate::anthropic::common::{ContentBlockParam, MessageParam, Role};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::anthropic::tools::ToolUse;

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    id: String,
    container: Option<Value>,
    content: Vec<ContentBlockParam>,
    model: String,
    role: Role,
    stop_details: Option<Value>,
    stop_reason: Option<StopReason>,
    stop_sequence: Option<String>,
    #[serde(rename = "type")]
    kind: ResponseType,
    usage: ResponseUsage,
}

impl Message {
    pub fn message_param(&self) -> MessageParam {
        MessageParam::new(self.role, self.content.clone())
    }

    pub fn next_tool_use(&self) -> bool {
        if let Some(r) = &self.stop_reason {
            match r {
                StopReason::ToolUse => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn tool_calls(&self) -> Vec<ToolUse> {
        self.content
            .iter()
            .filter_map(|c| match c {
                ContentBlockParam::ToolUse(t) => Some(t.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    pub fn text(&self) -> String {
        self.content
            .iter()
            .map(|c| match c {
                ContentBlockParam::Text { text } => text.clone(),
                _ => format!("{:?}", c),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn contents(&self) -> Vec<&ContentBlockParam> {
        self.content.iter().collect::<Vec<_>>()
    }

    pub fn usage_io_tokens(&self) -> (u32, u32) {
        (self.usage.input_tokens, self.usage.output_tokens)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseUsage {
    input_tokens: u32,
    cache_creation_input_tokens: u32,
    cache_read_input_tokens: u32,
    cache_creation: Option<ResponseUsageCacheCreation>,
    output_tokens: u32,
    service_tier: String,
    inference_geo: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseUsageCacheCreation {
    ephemeral_5m_input_tokens: u32,
    ephemeral_1h_input_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Message,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    PauseTurn,
    Refusal,
    ModelContextWindowExceeded,
}

#[cfg(test)]
mod tests {
    use super::*;

    const RESPONSE: &str = r#"{"model":"claude-haiku-4-5-20251001","id":"msg_011CeX1SMgRB2y3ZwvJXGzVW","type":"message","role":"assistant","content":[{"type":"text","text":"안녕하세요! 👋 \n\n무엇을 도와드릴까요?"}],"stop_reason":"end_turn","stop_sequence":null,"stop_details":null,"usage":{"input_tokens":11,"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"cache_creation":{"ephemeral_5m_input_tokens":0,"ephemeral_1h_input_tokens":0},"output_tokens":29,"service_tier":"standard","inference_geo":"not_available"}}"#;

    #[test]
    fn test_deserialize_response() {
        let response: Message = serde_json::from_str(RESPONSE).expect("invalid config");

        assert_eq!(response.role, Role::Assistant);
        assert_eq!(response.kind, ResponseType::Message);
    }
}
