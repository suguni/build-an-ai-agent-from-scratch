use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::anthropic::tools::{ToolResult, ToolUse};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageParam {
    role: Role,
    content: Vec<ContentBlockParam>,
}

impl MessageParam {
    pub fn new(role: Role, content: Vec<ContentBlockParam>) -> Self {
        Self { role, content }
    }

    pub fn user(message: &str) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlockParam::Text {
                text: message.to_string(),
            }],
        }
    }
}


// https://platform.claude.com/docs/ko/api/messages/create#message_param.content[1]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockParam {
    Text { text: String },
    ToolUse(ToolUse),
    ToolResult(ToolResult),
    #[serde(other)]
    Unknown,
}

impl ContentBlockParam {
    pub fn text(text: &str) -> Self {
        ContentBlockParam::Text {
            text: text.to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

#[cfg(test)]
mod tests {
    use super::*;

    const RESPONSE_CONTENT: &str =
        r#"{"type":"text","text":"안녕하세요! 👋 \n\n무엇을 도와드릴까요?"}"#;

    #[test]
    fn test_deserialize_response_content() {
        let response: ContentBlockParam =
            serde_json::from_str(RESPONSE_CONTENT).expect("invalid config");

        let text = match response {
            ContentBlockParam::Text { text } => text,
            _ => unreachable!("invalid type"),
        };

        assert_eq!(text, "안녕하세요! 👋 \n\n무엇을 도와드릴까요?".to_string())
    }

    #[test]
    fn test_deserialize_unknown_response_content() {
        let response: ContentBlockParam =
            serde_json::from_str(r#"{"type":"__hello__"}"#).expect("invalid config");

        match response {
            ContentBlockParam::Unknown => {}
            _ => unreachable!("invalid type"),
        }
    }
}
