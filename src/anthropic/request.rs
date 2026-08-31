use crate::anthropic::common::{ContentBlockParam, MessageParam, Role};
use crate::common;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct Request {
    // https://platform.claude.com/docs/ko/api/messages/create#create.model
    model: String,
    max_tokens: u32,
    messages: Vec<MessageParam>,
    system: Vec<ContentBlockParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<OutputConfig>
}

impl Request {
    pub fn new(model: &str, max_tokens: u32, message: &str) -> Self {
        Self {
            model: model.to_string(),
            max_tokens,
            messages: vec![MessageParam::new(
                Role::User,
                vec![ContentBlockParam::text(message)],
            )],
            system: vec![],
            output_config: None,
        }
    }

    pub fn message(message: MessageParam, config: OutputConfig) -> Self {
        Self {
            model: common::DEFAULT_MODEL.to_string(),
            max_tokens: common::DEFAULT_MAX_TOKEN,
            messages: vec![message],
            system: vec![],
            output_config: Some(config),
        }
    }

    pub fn messages(message_params: &[MessageParam]) -> Self {
        Self {
            model: common::DEFAULT_MODEL.to_string(),
            max_tokens: common::DEFAULT_MAX_TOKEN,
            messages: message_params.to_vec(),
            system: vec![],
            output_config: None,
        }
    }

    pub fn config(schema: Value) -> OutputConfig {
        OutputConfig {
            effort: None,
            format: Some(JSONOutputFormat::JsonSchema { schema })
        }
    }

}


#[derive(Debug,  Serialize)]
pub struct OutputConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    effort: Option<EffortLevel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<JSONOutputFormat>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JSONOutputFormat {
    JsonSchema { schema: Value }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_request() {
        let request = Request::new("claude-haiku-4-5", 1024, "안녕");
        let body = serde_json::to_value(&request).expect("serialize 실패");
        assert_eq!(
            body,
            serde_json::json!({
                "model": "claude-haiku-4-5",
                "max_tokens": 1024,
                "messages": [{"role": "user", "content": [ {"type": "text", "text": "안녕"}]}]
            })
        );
    }
}
