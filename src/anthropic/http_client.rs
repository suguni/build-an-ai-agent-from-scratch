use anyhow::Context;
use serde_json::{Value, from_str};
use std::borrow::Cow;
use std::collections::HashMap;

use crate::agent::execution_context::ExecutionContext;
use crate::anthropic::call_logger::HttpCallLogger;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const MESSAGE_URI: &'static str = "https://api.anthropic.com/v1/messages";

pub struct HttpClient {
    api_key: Cow<'static, str>,
}

impl HttpClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: Cow::from(api_key.to_owned()),
        }
    }

    pub async fn call<'a>(
        &self,
        body: &Value,
        context: &ExecutionContext,
    ) -> anyhow::Result<Response> {
        let client = reqwest::Client::new();

        context.log_request(&body)?;

        let response = client
            .post(MESSAGE_URI)
            .header("x-api-key", self.api_key.as_ref())
            .header("anthropic-version", "2023-06-01")
            .json(body)
            .send()
            .await
            .context("호출 오류");

        match response {
            Ok(response) => match self.handle_response(response).await {
                Ok(call_response) => {
                    context.log_response(&call_response)?;
                    Ok(call_response)
                }
                Err(e) => {
                    context.log_error(&e.to_string())?;
                    Err(e)
                }
            },
            Err(e) => {
                context.log_error(&e.to_string())?;
                Err(e)
            }
        }
    }

    async fn handle_response(&self, response: reqwest::Response) -> anyhow::Result<Response> {
        let headers = extract_headers(&response, HEADERS);
        let status = response.status().as_u16();
        let is_success = response.status().is_success();
        let raw_body = response.text().await.context("응답 읽기 실패")?;
        Ok(Response {
            status,
            is_success,
            headers,
            raw_body,
        })
    }
}

pub const HEADERS: &'static [&'static str] = &[
    "request-id",
    "anthropic-ratelimit-input-tokens-limit",
    "anthropic-ratelimit-input-tokens-remaining",
    "anthropic-ratelimit-input-tokens-reset",
    "anthropic-ratelimit-output-tokens-limit",
    "anthropic-ratelimit-output-tokens-remaining",
    "anthropic-ratelimit-output-tokens-reset",
    "anthropic-ratelimit-requests-limit",
    "anthropic-ratelimit-requests-remaining",
    "anthropic-ratelimit-requests-reset",
    "anthropic-ratelimit-tokens-limit",
    "anthropic-ratelimit-tokens-remaining",
    "anthropic-ratelimit-tokens-reset",
];

pub fn extract_headers(response: &reqwest::Response, headers: &[&str]) -> HashMap<String, String> {
    headers
        .iter()
        .filter_map(|&key| {
            response
                .headers()
                .get(key)
                .map(|v| v.to_str().ok())
                .flatten()
                .map(|v| (key.to_string(), v.to_string()))
        })
        .collect::<HashMap<_, _>>()
}
pub struct Response {
    pub status: u16,
    pub is_success: bool,
    pub headers: HashMap<String, String>,
    pub raw_body: String,
}
