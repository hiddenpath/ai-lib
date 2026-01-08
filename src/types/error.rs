//! 错误处理模块，提供统一的错误类型和处理机制
//!
//! Error handling module providing unified error types and handling mechanisms.
//!
//! This module defines `AiLibError` as the primary error type throughout ai-lib,
//! with proper error classification for retry logic and observability.

use crate::transport::TransportError;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Transient errors - safe to retry
    Transient,
    /// Client-side issues - fix configuration/request
    Client,
    /// Provider/server-side issues
    Server,
    /// Fatal issues - do not retry automatically
    Fatal,
}

#[derive(Error, Debug, Clone, PartialEq)]
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

    #[error("Parse error: {0}")]
    ParseError(String),

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
    /// Classify the error by severity for observability/failover logic
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AiLibError::NetworkError(_)
            | AiLibError::TimeoutError(_)
            | AiLibError::RateLimitExceeded(_) => ErrorSeverity::Transient,
            AiLibError::InvalidRequest(_)
            | AiLibError::ConfigurationError(_)
            | AiLibError::ContextLengthExceeded(_)
            | AiLibError::SerializationError(_)
            | AiLibError::DeserializationError(_)
            | AiLibError::FileError(_)
            | AiLibError::ModelNotFound(_) => ErrorSeverity::Client,
            AiLibError::AuthenticationError(_)
            | AiLibError::UnsupportedFeature(_)
            | AiLibError::RetryExhausted(_) => ErrorSeverity::Fatal,
            AiLibError::ProviderError(_) | AiLibError::InvalidModelResponse(_) => {
                ErrorSeverity::Server
            }
            AiLibError::TransportError(err) => match err {
                TransportError::Timeout(_) | TransportError::RateLimitExceeded => {
                    ErrorSeverity::Transient
                }
                TransportError::InvalidUrl(_)
                | TransportError::JsonError(_)
                | TransportError::ClientError { .. } => ErrorSeverity::Client,
                TransportError::AuthenticationError(_) => ErrorSeverity::Fatal,
                TransportError::ServerError { .. } | TransportError::HttpError(_) => {
                    ErrorSeverity::Server
                }
            },
            AiLibError::ParseError(_) => ErrorSeverity::Client,
        }
    }

    /// Structured error code for logs/metrics
    pub fn error_code(&self) -> &'static str {
        match self {
            AiLibError::ProviderError(_) => "PROVIDER_ERROR",
            AiLibError::TransportError(_) => "TRANSPORT_ERROR",
            AiLibError::InvalidRequest(_) => "INVALID_REQUEST",
            AiLibError::RateLimitExceeded(_) => "RATE_LIMIT",
            AiLibError::AuthenticationError(_) => "AUTH_FAILED",
            AiLibError::ConfigurationError(_) => "CONFIG_ERROR",
            AiLibError::NetworkError(_) => "NETWORK_ERROR",
            AiLibError::TimeoutError(_) => "TIMEOUT",
            AiLibError::RetryExhausted(_) => "RETRY_EXHAUSTED",
            AiLibError::SerializationError(_) => "SERIALIZATION_ERROR",
            AiLibError::DeserializationError(_) => "DESERIALIZATION_ERROR",
            AiLibError::FileError(_) => "FILE_ERROR",
            AiLibError::UnsupportedFeature(_) => "UNSUPPORTED_FEATURE",
            AiLibError::ModelNotFound(_) => "MODEL_NOT_FOUND",
            AiLibError::InvalidModelResponse(_) => "INVALID_RESPONSE",
            AiLibError::ContextLengthExceeded(_) => "CONTEXT_TOO_LONG",
            AiLibError::ParseError(_) => "PARSE_ERROR",
        }
    }

    /// Returns an uppercase code that embeds severity, e.g. `TRANSIENT_TIMEOUT`
    pub fn error_code_with_severity(&self) -> String {
        format!("{:?}_{}", self.severity(), self.error_code()).to_uppercase()
    }

    /// Attach contextual information to the error message.
    pub fn with_context(self, context: impl Into<String>) -> Self {
        let ctx = context.into();
        match self {
            AiLibError::ProviderError(msg) => AiLibError::ProviderError(prepend_context(&ctx, msg)),
            AiLibError::InvalidRequest(msg) => {
                AiLibError::InvalidRequest(prepend_context(&ctx, msg))
            }
            AiLibError::RateLimitExceeded(msg) => {
                AiLibError::RateLimitExceeded(prepend_context(&ctx, msg))
            }
            AiLibError::AuthenticationError(msg) => {
                AiLibError::AuthenticationError(prepend_context(&ctx, msg))
            }
            AiLibError::ConfigurationError(msg) => {
                AiLibError::ConfigurationError(prepend_context(&ctx, msg))
            }
            AiLibError::NetworkError(msg) => AiLibError::NetworkError(prepend_context(&ctx, msg)),
            AiLibError::TimeoutError(msg) => AiLibError::TimeoutError(prepend_context(&ctx, msg)),
            AiLibError::RetryExhausted(msg) => {
                AiLibError::RetryExhausted(prepend_context(&ctx, msg))
            }
            AiLibError::SerializationError(msg) => {
                AiLibError::SerializationError(prepend_context(&ctx, msg))
            }
            AiLibError::DeserializationError(msg) => {
                AiLibError::DeserializationError(prepend_context(&ctx, msg))
            }
            AiLibError::FileError(msg) => AiLibError::FileError(prepend_context(&ctx, msg)),
            AiLibError::UnsupportedFeature(msg) => {
                AiLibError::UnsupportedFeature(prepend_context(&ctx, msg))
            }
            AiLibError::ModelNotFound(msg) => AiLibError::ModelNotFound(prepend_context(&ctx, msg)),
            AiLibError::InvalidModelResponse(msg) => {
                AiLibError::InvalidModelResponse(prepend_context(&ctx, msg))
            }
            AiLibError::ContextLengthExceeded(msg) => {
                AiLibError::ContextLengthExceeded(prepend_context(&ctx, msg))
            }
            AiLibError::TransportError(err) => {
                AiLibError::TransportError(transport_with_context(err, &ctx))
            }
            AiLibError::ParseError(msg) => AiLibError::ParseError(prepend_context(&ctx, msg)),
        }
    }

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
            AiLibError::ParseError(_) => "Data parsing failed",
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

fn prepend_context(context: &str, message: String) -> String {
    if message.is_empty() {
        context.to_string()
    } else {
        format!("{context}: {message}")
    }
}

fn transport_with_context(err: TransportError, context: &str) -> TransportError {
    match err {
        TransportError::HttpError(msg) => TransportError::HttpError(prepend_context(context, msg)),
        TransportError::JsonError(msg) => TransportError::JsonError(prepend_context(context, msg)),
        TransportError::InvalidUrl(msg) => {
            TransportError::InvalidUrl(prepend_context(context, msg))
        }
        TransportError::AuthenticationError(msg) => {
            TransportError::AuthenticationError(prepend_context(context, msg))
        }
        TransportError::RateLimitExceeded => TransportError::RateLimitExceeded,
        TransportError::ServerError { status, message } => TransportError::ServerError {
            status,
            message: prepend_context(context, message),
        },
        TransportError::ClientError { status, message } => TransportError::ClientError {
            status,
            message: prepend_context(context, message),
        },
        TransportError::Timeout(msg) => TransportError::Timeout(prepend_context(context, msg)),
    }
}
