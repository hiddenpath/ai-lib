//! Error context and suggested actions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Context information for error tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// AI provider that generated the error
    pub provider: String,
    /// API endpoint that was called
    pub endpoint: String,
    /// Request ID for tracking
    pub request_id: Option<String>,
    /// Timestamp when the error occurred
    pub timestamp: DateTime<Utc>,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Suggested action to take
    pub suggested_action: SuggestedAction,
}

/// Suggested actions for error recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestedAction {
    /// Retry the request with specified delay and max attempts
    Retry { delay_ms: u64, max_attempts: u32 },
    /// Switch to alternative providers
    SwitchProvider { alternative_providers: Vec<String> },
    /// Reduce request size or complexity
    ReduceRequestSize { max_tokens: Option<u32> },
    /// Check API credentials
    CheckCredentials,
    /// Contact support with specific reason
    ContactSupport { reason: String },
    /// No specific action recommended
    NoAction,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(provider: String, endpoint: String) -> Self {
        Self {
            provider,
            endpoint,
            request_id: None,
            timestamp: Utc::now(),
            retry_count: 0,
            suggested_action: SuggestedAction::NoAction,
        }
    }

    /// Create error context with retry information
    pub fn with_retry(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }

    /// Create error context with request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}
