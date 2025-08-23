use serde::{Deserialize, Serialize};
use crate::types::{Choice, Usage};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
	pub id: String,
	pub object: String,
	pub created: u64,
	pub model: String,
	pub choices: Vec<Choice>,
	pub usage: Usage,
}
