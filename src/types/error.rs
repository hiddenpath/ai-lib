use crate::transport::TransportError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiLibError {
    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Transport error: {0}")]
    TransportError(#[from] TransportError),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Retry exhausted: {0}")]
    RetryExhausted(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("File operation error: {0}")]
    FileError(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Invalid model response: {0}")]
    InvalidModelResponse(String),

    #[error("Context length exceeded: {0}")]
    ContextLengthExceeded(String),
}

impl AiLibError {
    /// Determine if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            AiLibError::NetworkError(_) => true,
            AiLibError::TimeoutError(_) => true,
            AiLibError::RateLimitExceeded(_) => true,
            AiLibError::TransportError(transport_err) => {
                // Check if it's a temporary network error
                transport_err.to_string().contains("timeout")
                    || transport_err.to_string().contains("connection")
                    || transport_err.to_string().contains("temporary")
            }
            _ => false,
        }
    }

    /// Get suggested retry delay (milliseconds)
    pub fn retry_delay_ms(&self) -> u64 {
        match self {
            AiLibError::RateLimitExceeded(_) => 60000, // 1 minute
            AiLibError::NetworkError(_) => 1000,       // 1 second
            AiLibError::TimeoutError(_) => 2000,       // 2 seconds
            _ => 1000,
        }
    }

    /// Get error context for debugging
    pub fn context(&self) -> &str {
        match self {
            AiLibError::ProviderError(_) => "Provider API call failed",
            AiLibError::TransportError(_) => "Network transport layer error",
            AiLibError::InvalidRequest(_) => "Invalid request parameters",
            AiLibError::RateLimitExceeded(_) => "API rate limit exceeded",
            AiLibError::AuthenticationError(_) => "Authentication failed",
            AiLibError::ConfigurationError(_) => "Configuration validation failed",
            AiLibError::NetworkError(_) => "Network connectivity issue",
            AiLibError::TimeoutError(_) => "Request timed out",
            AiLibError::RetryExhausted(_) => "All retry attempts failed",
            AiLibError::SerializationError(_) => "Request serialization failed",
            AiLibError::DeserializationError(_) => "Response deserialization failed",
            AiLibError::FileError(_) => "File operation failed",
            AiLibError::UnsupportedFeature(_) => "Feature not supported by provider",
            AiLibError::ModelNotFound(_) => "Specified model not found",
            AiLibError::InvalidModelResponse(_) => "Invalid response from model",
            AiLibError::ContextLengthExceeded(_) => "Context length limit exceeded",
        }
    }

    /// Check if error is related to authentication
    pub fn is_auth_error(&self) -> bool {
        match self {
            AiLibError::AuthenticationError(_) => true,
            AiLibError::TransportError(TransportError::AuthenticationError(_)) => true,
            AiLibError::TransportError(TransportError::ClientError { status, .. }) => {
                *status == 401 || *status == 403
            }
            _ => false,
        }
    }

    /// Check if error is related to configuration
    pub fn is_config_error(&self) -> bool {
        matches!(self, AiLibError::ConfigurationError(_))
    }

    /// Check if error is related to request validation
    pub fn is_request_error(&self) -> bool {
        matches!(
            self,
            AiLibError::InvalidRequest(_)
                | AiLibError::ContextLengthExceeded(_)
                | AiLibError::UnsupportedFeature(_)
        )
    }
}
