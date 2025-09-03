use crate::types::{Choice, Usage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

impl ChatCompletionResponse {
    /// Get the first textual content from the first choice.
    /// Returns an error if choices are empty or content is non-text.
    pub fn first_text(&self) -> Result<&str, crate::types::AiLibError> {
        let choice = self
            .choices
            .get(0)
            .ok_or_else(|| crate::types::AiLibError::InvalidModelResponse("empty choices".into()))?;
        match &choice.message.content {
            crate::types::common::Content::Text(t) => Ok(t.as_str()),
            other => Err(crate::types::AiLibError::InvalidModelResponse(format!(
                "expected text content, got {:?}", other
            ))),
        }
    }
}
