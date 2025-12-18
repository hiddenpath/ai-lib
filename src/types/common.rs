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
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());
        let mime = path
            .extension()
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
        let mime = path
            .extension()
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

impl Message {
    /// Create a new message with the specified role and content.
    pub fn new(role: Role, content: impl Into<Content>) -> Self {
        Self {
            role,
            content: content.into(),
            function_call: None,
        }
    }

    /// Create a user message with text content.
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::prelude::*;
    ///
    /// let msg = Message::user("Hello, how are you?");
    /// assert!(matches!(msg.role, Role::User));
    /// ```
    pub fn user<S: Into<String>>(text: S) -> Self {
        Self {
            role: Role::User,
            content: Content::Text(text.into()),
            function_call: None,
        }
    }

    /// Create an assistant message with text content.
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::prelude::*;
    ///
    /// let msg = Message::assistant("I'm doing well, thank you!");
    /// assert!(matches!(msg.role, Role::Assistant));
    /// ```
    pub fn assistant<S: Into<String>>(text: S) -> Self {
        Self {
            role: Role::Assistant,
            content: Content::Text(text.into()),
            function_call: None,
        }
    }

    /// Create a system message with text content.
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::prelude::*;
    ///
    /// let msg = Message::system("You are a helpful assistant.");
    /// assert!(matches!(msg.role, Role::System));
    /// ```
    pub fn system<S: Into<String>>(text: S) -> Self {
        Self {
            role: Role::System,
            content: Content::Text(text.into()),
            function_call: None,
        }
    }

    /// Create a user message with custom content (text, image, audio, etc.).
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::prelude::*;
    ///
    /// let msg = Message::user_with_content(Content::from_image_file("photo.jpg"));
    /// ```
    pub fn user_with_content(content: Content) -> Self {
        Self {
            role: Role::User,
            content,
            function_call: None,
        }
    }

    /// Create an assistant message with a function call.
    pub fn assistant_with_function_call(
        text: impl Into<String>,
        function_call: crate::types::function_call::FunctionCall,
    ) -> Self {
        Self {
            role: Role::Assistant,
            content: Content::Text(text.into()),
            function_call: Some(function_call),
        }
    }
}

/// Allow `Into<Content>` for `String` to enable ergonomic `Message::new(Role::User, "hello")`
impl From<String> for Content {
    fn from(s: String) -> Self {
        Content::Text(s)
    }
}

/// Allow `Into<Content>` for `&str` to enable ergonomic `Message::new(Role::User, "hello")`
impl From<&str> for Content {
    fn from(s: &str) -> Self {
        Content::Text(s.to_string())
    }
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

// Note: Usage and UsageStatus have been moved to `types::response` as of 1.0.
// Import from `ai_lib::Usage` / `ai_lib::UsageStatus` or `ai_lib::types::response::{Usage, UsageStatus}`.
