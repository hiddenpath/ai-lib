use crate::types::Choice;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Usage token counts returned by providers as part of chat response.
///
/// Semantically this is response metadata (per-request usage info) and therefore
/// lives in the `types::response` module. For convenience this type is also
/// re-exported at the crate root as `ai_lib::Usage`.
#[doc(alias = "token_usage")]
#[doc(alias = "usage_info")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Indicates the reliability and source of usage data
#[doc(alias = "usage_status")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageStatus {
    /// Usage data is accurate and finalized from the provider
    #[serde(rename = "finalized")]
    Finalized,
    /// Usage data is estimated (e.g., using tokenizer approximation)
    #[serde(rename = "estimated")]
    Estimated,
    /// Usage data is not yet available (e.g., streaming in progress)
    #[serde(rename = "pending")]
    Pending,
    /// Provider doesn't support usage tracking
    #[serde(rename = "unsupported")]
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    /// Indicates the reliability and source of usage data
    pub usage_status: UsageStatus,
}

impl ChatCompletionResponse {
    /// Get the first textual content from the first choice.
    /// Returns an error if choices are empty or content is non-text.
    pub fn first_text(&self) -> Result<&str, crate::types::AiLibError> {
        let choice = self.choices.first().ok_or_else(|| {
            crate::types::AiLibError::InvalidModelResponse("empty choices".into())
        })?;
        match &choice.message.content {
            crate::types::common::Content::Text(t) => Ok(t.as_str()),
            other => Err(crate::types::AiLibError::InvalidModelResponse(format!(
                "expected text content, got {:?}",
                other
            ))),
        }
    }
}
