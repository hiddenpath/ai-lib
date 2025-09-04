use thiserror::Error;

/// Transport layer error types, unified encapsulation of HTTP and JSON errors
///
/// Transport layer error types with unified encapsulation of HTTP and JSON errors
///
/// Unified encapsulation of all HTTP-level errors and JSON parsing errors
#[derive(Error, Debug, Clone)]
pub enum TransportError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("JSON serialization/deserialization failed: {0}")]
    JsonError(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Server error: {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("Client error: {status} - {message}")]
    ClientError { status: u16, message: String },

    #[error("Timeout error: {0}")]
    Timeout(String),
}

impl TransportError {
    /// Create error from HTTP status code
    pub fn from_status(status: u16, message: String) -> Self {
        match status {
            400..=499 => Self::ClientError { status, message },
            500..=599 => Self::ServerError { status, message },
            _ => Self::InvalidUrl(format!("Unexpected status code: {}", status)),
        }
    }
}

impl From<reqwest::Error> for TransportError {
    fn from(err: reqwest::Error) -> Self {
        Self::HttpError(err.to_string())
    }
}

impl From<serde_json::Error> for TransportError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonError(err.to_string())
    }
}
