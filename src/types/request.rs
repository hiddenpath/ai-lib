use serde::{Deserialize, Serialize};
use crate::types::Message;

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
}
