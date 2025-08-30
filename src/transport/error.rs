use thiserror::Error;

/// 传输层错误类型，统一封装HTTP和JSON错误
///
/// Transport layer error types with unified encapsulation of HTTP and JSON errors
///
/// Unified encapsulation of all HTTP-level errors and JSON parsing errors
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON serialization/deserialization failed: {0}")]
    JsonError(#[from] serde_json::Error),

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
