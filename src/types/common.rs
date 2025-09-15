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

    /// Create image content from a file path - the file will be automatically processed
    /// (uploaded or inlined as data URL) by the AI client when used in a request
    pub fn from_image_file<P: AsRef<std::path::Path>>(path: P) -> Self {
        let path = path.as_ref();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());
        let mime = path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "png" => Some("image/png"),
                "jpg" | "jpeg" => Some("image/jpeg"),
                "gif" => Some("image/gif"),
                "webp" => Some("image/webp"),
                "svg" => Some("image/svg+xml"),
                _ => None,
            })
            .map(|s| s.to_string());
        
        Content::Image {
            url: None, // Will be filled by the client
            mime,
            name,
        }
    }

    /// Create audio content from a file path - the file will be automatically processed
    /// (uploaded or inlined as data URL) by the AI client when used in a request
    pub fn from_audio_file<P: AsRef<std::path::Path>>(path: P) -> Self {
        let path = path.as_ref();
        let mime = path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "mp3" => Some("audio/mpeg"),
                "wav" => Some("audio/wav"),
                "ogg" => Some("audio/ogg"),
                "m4a" => Some("audio/mp4"),
                "flac" => Some("audio/flac"),
                _ => None,
            })
            .map(|s| s.to_string());
        
        Content::Audio {
            url: None, // Will be filled by the client
            mime,
        }
    }

    /// Create image content from a data URL (base64 encoded)
    pub fn from_data_url(data_url: String, mime: Option<String>, name: Option<String>) -> Self {
        Content::Image {
            url: Some(data_url),
            mime,
            name,
        }
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

// Usage and UsageStatus have moved to `types::response` because they are
// semantically part of the chat response metadata. We keep these
// re-exports here for backward compatibility but mark them deprecated to
// guide users to the new location.
#[deprecated(note = "Usage has been moved to `ai_lib::types::response::Usage`. Please import from there or use `ai_lib::Usage`. This alias will be removed before 1.0.")]
pub use crate::types::response::Usage;

#[deprecated(note = "UsageStatus has been moved to `ai_lib::types::response::UsageStatus`. Please import from there or use `ai_lib::UsageStatus`. This alias will be removed before 1.0.")]
pub use crate::types::response::UsageStatus;
