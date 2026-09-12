use crate::anthropic::common::MessageParam;
use crate::anthropic::response::Message;

#[derive(Debug)]
pub enum Event {
    User(MessageParam),
    Llm(Message)
}
