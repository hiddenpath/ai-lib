use thiserror::Error;
use crate::transport::TransportError;

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
                transport_err.to_string().contains("timeout") ||
                transport_err.to_string().contains("connection")
            },
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
}
