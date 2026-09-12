use crate::agent::event::Event;
use crate::agent::tools::ToolUse;
use crate::anthropic::http_client::Response;
use crate::anthropic::call_logger::HttpCallLogger;
use crate::anthropic::common::MessageParam;
use anyhow::Context;
use serde_json::Value;
use time::OffsetDateTime;
use time::macros::format_description;

pub struct ExecutionContext {
    turn: u32,
    max_turn: u32,
    run: String,
    events: Vec<Event>,
    logger: HttpCallLogger,
}

impl ExecutionContext {
    pub fn new(max_turn: u32) -> Self {
        ExecutionContext {
            turn: 1,
            max_turn,
            run: get_run().unwrap(),
            events: vec![],
            logger: HttpCallLogger::new(),
        }
    }

    pub fn keep_going(&self) -> bool {
        self.turn < self.max_turn && self.keep_going_event()
    }

    pub fn last_tool_calls(&self) -> Option<Vec<ToolUse>> {
        if let Event::Llm(message) = self.events.last()? {
            let calls = message.tool_calls();
            if calls.is_empty() {
                None
            } else {
                Some(calls)
            }
        } else {
            None
        }
    }

    pub fn inc_step(&mut self) {
        self.turn += 1;
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn messages(&self) -> Vec<MessageParam> {
        self.events.iter().map(|event| {
            match event {
                Event::User(m) => m.clone(),
                Event::Llm(m) => m.message_param(),
            }
        }).collect::<Vec<_>>()
    }

    pub fn final_message(&self) -> Option<String> {
        if let Some(e) = self.events.last() {
            match e {
                Event::User(_) => None,
                Event::Llm(message) => Some(message.text())
            }
        } else {
            None
        }
    }

    fn keep_going_event(&self) -> bool {
        if let Some(e) = self.events.last() {
            dbg!(e);
            match e {
                Event::User(_) => true,
                Event::Llm(message) => message.next_tool_use()
            }
        } else {
            true
        }
    }

    pub fn log_request(&self, request: &Value) -> anyhow::Result<()> {
        self.logger.request(request, &self.run, self.turn)
    }

    pub fn log_response(&self, response: &Response) -> anyhow::Result<()> {
        self.logger.response(response, &self.run, self.turn)
    }

    pub fn log_error(&self, error_message: &str) -> anyhow::Result<()> {
        self.logger.error(error_message, &self.run, self.turn)
    }
}

fn get_run() -> anyhow::Result<String> {
    let now = OffsetDateTime::now_utc();
    let format = format_description!("[year][month][day]-[hour][minute][second]");
    now.format(&format).context("run 생성 포맷 오류")
}
