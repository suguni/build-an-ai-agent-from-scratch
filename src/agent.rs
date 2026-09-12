use crate::agent::event::Event;
use crate::agent::execution_context::ExecutionContext;
use crate::anthropic::http_client;
use crate::anthropic::common::{ContentBlockParam, MessageParam, Role};
use crate::anthropic::request::Request;
use crate::anthropic::response::Message;
use anyhow::Context;
use serde_json::from_str;
use crate::agent::tools::{Tool, ToolUse};

pub mod event;
pub mod execution_context;
pub mod tools;

pub struct Agent {
    tools: Vec<Box<dyn Tool>>,
    system_prompt: String,
    client: http_client::HttpClient,
    max_turn: u32,
}

pub const DEFAULT_MODEL: &str = "claude-haiku-4-5";
pub const DEFAULT_MAX_TOKEN: u32 = 1024;

impl Agent {
    pub fn new(api_key: &str, tools: Vec<Box<dyn Tool>>, system_prompt: &str) -> Self {
        Self {
            tools,
            system_prompt: system_prompt.to_string(),
            client: http_client::HttpClient::new(api_key),
            max_turn: 100,
        }
    }

    pub async fn chat(&mut self, message: &str) -> anyhow::Result<String> {
        let mut context = ExecutionContext::new(self.max_turn);

        let user_chat = MessageParam::user(message);
        context.add_event(Event::User(user_chat));

        while context.keep_going() {
            self.step(&mut context).await?;
        }

        context.final_message().context("???")
    }

    async fn step(&mut self, context: &mut ExecutionContext) -> anyhow::Result<()> {
        let request = self.prepare_llm_request(context);

        let response = self.think(&request, &context).await?;
        let message = from_str::<Message>(&response.raw_body).context("응답 deserialize 실패")?;
        context.add_event(Event::Llm(message));

        if let Some(tool_calls) = context.last_tool_calls() {
            if let Some(tool_result) = self.act(tool_calls, context).await {
                context.add_event(Event::User(tool_result));
            }
        }

        context.inc_step();
        Ok(())
    }

    fn prepare_llm_request(&self, context: &ExecutionContext) -> Request {
        Request::create(
            DEFAULT_MODEL,
            DEFAULT_MAX_TOKEN,
            context.messages(),
            vec![ContentBlockParam::text(&self.system_prompt)],
            None,
            self.tools.iter().map(|t| t.spec()).collect::<Vec<_>>(),
        )
    }

    async fn think(
        &self,
        request: &Request,
        context: &ExecutionContext,
    ) -> anyhow::Result<http_client::Response> {
        let response = self
            .client
            .call(&serde_json::to_value(request)?, &context)
            .await?;

        if !response.is_success {
            anyhow::bail!(
                "응답 오류 status: {}, response: {}",
                response.status,
                response.raw_body
            );
        }

        Ok(response)
    }

    async fn act(&self, tool_calls: Vec<ToolUse>, context: &ExecutionContext) -> Option<MessageParam> {
        let mut result = vec![];
        for tool_use in tool_calls {
            let tool_result = if let Some(tool) = tool_use.find_tool(&self.tools) {
                tool.run(tool_use).await.unwrap_or_else(|e| e.tool_result())
            } else {
                tool_use.error_result(&format!("cannot find tool {:?}", tool_use))
            };
            result.push(ContentBlockParam::ToolResult(tool_result));
        }
        if result.is_empty() {
            None
        } else {
            Some(MessageParam::new(Role::User, result))
        }

    }
}
