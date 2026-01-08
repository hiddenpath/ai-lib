use crate::types::Message;
use crate::types::{FunctionCallPolicy, Tool};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub top_k: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
    pub logprobs: Option<bool>,
    pub top_logprobs: Option<u32>,
    pub seed: Option<u64>,
    /// Structured output / JSON mode selection
    pub response_format_mode: Option<String>,
    /// Optional function/tool definitions for Function Calling
    pub functions: Option<Vec<Tool>>,
    /// Function call policy: "auto"/"none"/specific name
    pub function_call: Option<FunctionCallPolicy>,
    /// Provider-specific extension properties (serialized verbatim)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Map<String, Value>>,
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
            top_k: None,
            stop_sequences: None,
            logprobs: None,
            top_logprobs: None,
            seed: None,
            response_format_mode: None,
            functions: None,
            function_call: None,
            extensions: None,
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

    pub fn with_functions(mut self, functions: Vec<Tool>) -> Self {
        self.functions = Some(functions);
        self
    }

    pub fn with_function_call(mut self, function_call: FunctionCallPolicy) -> Self {
        self.function_call = Some(function_call);
        self
    }

    /// Attach provider-specific extensions to the request (serialized verbatim).
    pub fn with_extension(mut self, key: &str, value: serde_json::Value) -> Self {
        let map = self.extensions.get_or_insert_with(Map::new);
        map.insert(key.to_string(), value);
        self
    }

    pub fn apply_extensions(&self, target: &mut serde_json::Value) {
        if let (Some(ext), Some(obj)) = (&self.extensions, target.as_object_mut()) {
            for (k, v) in ext {
                obj.insert(k.clone(), v.clone());
            }
        }
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
