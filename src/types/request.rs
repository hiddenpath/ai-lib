use crate::types::Message;
use crate::types::{FunctionCallPolicy, Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    /// Optional function/tool definitions for Function Calling
    pub functions: Option<Vec<Tool>>,
    /// Function call policy: "auto"/"none"/specific name
    pub function_call: Option<FunctionCallPolicy>,
}

impl ChatCompletionRequest {
    pub fn new(model: String, messages: Vec<Message>) -> Self {
        Self {
            model,
            messages,
            temperature: None,
            max_tokens: None,
            stream: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            functions: None,
            function_call: None,
        }
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Drop previous conversational messages while keeping system messages and the last non-system message.
    /// Useful to reset context while preserving system instructions.
    pub fn ignore_previous(mut self) -> Self {
        use crate::types::Role;
        let mut new_msgs: Vec<Message> = self
            .messages
            .iter()
            .filter(|m| matches!(m.role, Role::System))
            .cloned()
            .collect();
        if let Some(last) = self
            .messages
            .iter()
            .rev()
            .find(|m| !matches!(m.role, Role::System))
        {
            new_msgs.push(last.clone());
        }
        self.messages = new_msgs;
        self
    }
}
