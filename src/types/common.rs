use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
	pub role: Role,
	pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
	#[serde(rename = "system")]
	System,
	#[serde(rename = "user")]
	User,
	#[serde(rename = "assistant")]
	Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
	pub index: u32,
	pub message: Message,
	pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
	pub prompt_tokens: u32,
	pub completion_tokens: u32,
	pub total_tokens: u32,
}
