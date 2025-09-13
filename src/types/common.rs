use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Message content — moved to an enum to support multimodal and structured content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Content {
    #[serde(rename = "text")]
    Text(String),
    /// Generic JSON content for structured payloads (e.g. function call args)
    #[serde(rename = "json")]
    Json(JsonValue),
    /// Reference to an image (url) or metadata; adapters may upload or inline as needed
    #[serde(rename = "image")]
    Image {
        url: Option<String>,
        mime: Option<String>,
        name: Option<String>,
    },
    /// Reference to audio content
    #[serde(rename = "audio")]
    Audio {
        url: Option<String>,
        mime: Option<String>,
    },
}

impl Content {
    /// Return a best-effort textual representation for legacy code paths
    pub fn as_text(&self) -> String {
        match self {
            Content::Text(s) => s.clone(),
            Content::Json(v) => v.to_string(),
            Content::Image { url, .. } => url.clone().unwrap_or_default(),
            Content::Audio { url, .. } => url.clone().unwrap_or_default(),
        }
    }

    /// Convenience constructor for text content
    pub fn new_text<S: Into<String>>(s: S) -> Self {
        Content::Text(s.into())
    }

    /// Convenience constructor for JSON content
    pub fn new_json(v: JsonValue) -> Self {
        Content::Json(v)
    }

    /// Convenience constructor for image content
    pub fn new_image(url: Option<String>, mime: Option<String>, name: Option<String>) -> Self {
        Content::Image { url, mime, name }
    }

    /// Convenience constructor for audio content
    pub fn new_audio(url: Option<String>, mime: Option<String>) -> Self {
        Content::Audio { url, mime }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Content,
    /// Optional function call payload when assistant invokes a tool
    pub function_call: Option<crate::types::function_call::FunctionCall>,
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

/// Indicates the reliability and source of usage data
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
