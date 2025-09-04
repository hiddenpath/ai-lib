//! Error recovery strategies and management

use crate::types::AiLibError;
use crate::error_handling::{ErrorContext, SuggestedAction};
use crate::metrics::Metrics;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

/// Error recovery manager
pub struct ErrorRecoveryManager {
    error_history: Arc<Mutex<VecDeque<ErrorRecord>>>,
    recovery_strategies: HashMap<ErrorType, Box<dyn RecoveryStrategy>>,
    // Metrics and monitoring
    metrics: Option<Arc<dyn Metrics>>,
    start_time: Instant,
    // Error pattern analysis
    error_patterns: Arc<Mutex<HashMap<ErrorType, ErrorPattern>>>,
}

/// Error pattern analysis for intelligent recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub error_type: ErrorType,
    pub count: u32,
    pub first_occurrence: chrono::DateTime<chrono::Utc>,
    pub last_occurrence: chrono::DateTime<chrono::Utc>,
    pub frequency: f64, // errors per minute
    pub suggested_action: SuggestedAction,
    pub recovery_attempts: u32,
    pub successful_recoveries: u32,
}

/// Record of an error for tracking patterns
#[derive(Debug, Clone)]
pub struct ErrorRecord {
    pub error_type: ErrorType,
    pub context: ErrorContext,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of errors for categorization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    RateLimit,
    Network,
    Authentication,
    Provider,
    Timeout,
    Configuration,
    Validation,
    Serialization,
    Deserialization,
    FileOperation,
    ModelNotFound,
    ContextLengthExceeded,
    UnsupportedFeature,
    Unknown,
}

/// Trait for implementing recovery strategies
#[async_trait]
pub trait RecoveryStrategy: Send + Sync {
    /// Check if this strategy can recover from the given error
    async fn can_recover(&self, error: &AiLibError) -> bool;
    
    /// Attempt to recover from the error
    async fn recover(&self, error: &AiLibError, context: &ErrorContext) -> Result<(), AiLibError>;
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager
    pub fn new() -> Self {
        Self {
            error_history: Arc::new(Mutex::new(VecDeque::new())),
            recovery_strategies: HashMap::new(),
            metrics: None,
            start_time: Instant::now(),
            error_patterns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new error recovery manager with metrics
    pub fn with_metrics(metrics: Arc<dyn Metrics>) -> Self {
        Self {
            error_history: Arc::new(Mutex::new(VecDeque::new())),
            recovery_strategies: HashMap::new(),
            metrics: Some(metrics),
            start_time: Instant::now(),
            error_patterns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a recovery strategy for a specific error type
    pub fn register_strategy(&mut self, error_type: ErrorType, strategy: Box<dyn RecoveryStrategy>) {
        self.recovery_strategies.insert(error_type, strategy);
    }

    /// Handle an error and attempt recovery
    pub async fn handle_error(&self, error: &AiLibError, context: &ErrorContext) -> Result<(), AiLibError> {
        let error_type = self.classify_error(error);
        
        // Record the error
        self.record_error(error_type.clone(), context.clone()).await;
        
        // Try to find a recovery strategy
        if let Some(strategy) = self.recovery_strategies.get(&error_type) {
            if strategy.can_recover(error).await {
                return strategy.recover(error, context).await;
            }
        }
        
        Err((*error).clone())
    }

    /// Classify an error into a specific type
    fn classify_error(&self, error: &AiLibError) -> ErrorType {
        match error {
            AiLibError::RateLimitExceeded(_) => ErrorType::RateLimit,
            AiLibError::NetworkError(_) => ErrorType::Network,
            AiLibError::AuthenticationError(_) => ErrorType::Authentication,
            AiLibError::ProviderError(_) => ErrorType::Provider,
            AiLibError::TimeoutError(_) => ErrorType::Timeout,
            AiLibError::ConfigurationError(_) => ErrorType::Configuration,
            AiLibError::InvalidRequest(_) => ErrorType::Validation,
            AiLibError::SerializationError(_) => ErrorType::Serialization,
            AiLibError::DeserializationError(_) => ErrorType::Deserialization,
            AiLibError::FileError(_) => ErrorType::FileOperation,
            AiLibError::ModelNotFound(_) => ErrorType::ModelNotFound,
            AiLibError::ContextLengthExceeded(_) => ErrorType::ContextLengthExceeded,
            AiLibError::UnsupportedFeature(_) => ErrorType::UnsupportedFeature,
            _ => ErrorType::Unknown,
        }
    }

    /// Generate intelligent suggested action based on error pattern
    fn generate_suggested_action(&self, error_type: &ErrorType, pattern: &ErrorPattern) -> SuggestedAction {
        match error_type {
            ErrorType::RateLimit => {
                if pattern.frequency > 10.0 {
                    SuggestedAction::SwitchProvider {
                        alternative_providers: vec!["groq".to_string(), "anthropic".to_string()],
                    }
                } else {
                    SuggestedAction::Retry {
                        delay_ms: 60000,
                        max_attempts: 3,
                    }
                }
            }
            ErrorType::Network => {
                SuggestedAction::Retry {
                    delay_ms: 2000,
                    max_attempts: 5,
                }
            }
            ErrorType::Authentication => {
                SuggestedAction::CheckCredentials
            }
            ErrorType::Provider => {
                SuggestedAction::SwitchProvider {
                    alternative_providers: vec!["openai".to_string(), "groq".to_string()],
                }
            }
            ErrorType::Timeout => {
                SuggestedAction::Retry {
                    delay_ms: 5000,
                    max_attempts: 3,
                }
            }
            ErrorType::ContextLengthExceeded => {
                SuggestedAction::ReduceRequestSize {
                    max_tokens: Some(1000),
                }
            }
            ErrorType::ModelNotFound => {
                SuggestedAction::ContactSupport {
                    reason: "Model not found - please verify model name".to_string(),
                }
            }
            _ => SuggestedAction::NoAction,
        }
    }

    /// Record an error in the history
    async fn record_error(&self, error_type: ErrorType, mut context: ErrorContext) {
        let now = chrono::Utc::now();
        let record = ErrorRecord {
            error_type: error_type.clone(),
            context: context.clone(),
            timestamp: now,
        };
        
        // Update error patterns
        self.update_error_pattern(&error_type, now).await;
        
        // Generate suggested action based on pattern
        let suggested_action = self.get_suggested_action_for_error(&error_type).await;
        context.suggested_action = suggested_action;
        
        let mut history = self.error_history.lock().unwrap();
        history.push_back(record);
        
        // Keep only the last 1000 records
        if history.len() > 1000 {
            history.pop_front();
        }
        
        // Record metrics
        if let Some(metrics) = &self.metrics {
            metrics.incr_counter(&format!("errors.{}", self.error_type_name(&error_type)), 1).await;
        }
    }

    /// Update error pattern analysis
    async fn update_error_pattern(&self, error_type: &ErrorType, timestamp: chrono::DateTime<chrono::Utc>) {
        let mut patterns = self.error_patterns.lock().unwrap();
        
        let pattern = patterns.entry(error_type.clone()).or_insert_with(|| ErrorPattern {
            error_type: error_type.clone(),
            count: 0,
            first_occurrence: timestamp,
            last_occurrence: timestamp,
            frequency: 0.0,
            suggested_action: SuggestedAction::NoAction,
            recovery_attempts: 0,
            successful_recoveries: 0,
        });
        
        pattern.count += 1;
        pattern.last_occurrence = timestamp;
        
        // Calculate frequency (errors per minute)
        let duration = pattern.last_occurrence.signed_duration_since(pattern.first_occurrence);
        if duration.num_minutes() > 0 {
            pattern.frequency = pattern.count as f64 / duration.num_minutes() as f64;
        }
        
        // Generate suggested action
        pattern.suggested_action = self.generate_suggested_action(error_type, pattern);
    }

    /// Get suggested action for a specific error type
    async fn get_suggested_action_for_error(&self, error_type: &ErrorType) -> SuggestedAction {
        let patterns = self.error_patterns.lock().unwrap();
        if let Some(pattern) = patterns.get(error_type) {
            pattern.suggested_action.clone()
        } else {
            SuggestedAction::NoAction
        }
    }

    /// Get error type name for metrics
    fn error_type_name(&self, error_type: &ErrorType) -> String {
        match error_type {
            ErrorType::RateLimit => "rate_limit".to_string(),
            ErrorType::Network => "network".to_string(),
            ErrorType::Authentication => "authentication".to_string(),
            ErrorType::Provider => "provider".to_string(),
            ErrorType::Timeout => "timeout".to_string(),
            ErrorType::Configuration => "configuration".to_string(),
            ErrorType::Validation => "validation".to_string(),
            ErrorType::Serialization => "serialization".to_string(),
            ErrorType::Deserialization => "deserialization".to_string(),
            ErrorType::FileOperation => "file_operation".to_string(),
            ErrorType::ModelNotFound => "model_not_found".to_string(),
            ErrorType::ContextLengthExceeded => "context_length_exceeded".to_string(),
            ErrorType::UnsupportedFeature => "unsupported_feature".to_string(),
            ErrorType::Unknown => "unknown".to_string(),
        }
    }

    /// Get error patterns for analysis
    pub fn get_error_patterns(&self) -> HashMap<ErrorType, ErrorPattern> {
        self.error_patterns.lock().unwrap().clone()
    }

    /// Get error statistics
    pub fn get_error_statistics(&self) -> ErrorStatistics {
        let patterns = self.error_patterns.lock().unwrap();
        let total_errors: u32 = patterns.values().map(|p| p.count).sum();
        let most_common_error = patterns
            .values()
            .max_by_key(|p| p.count)
            .map(|p| p.error_type.clone());
        
        ErrorStatistics {
            total_errors,
            unique_error_types: patterns.len(),
            most_common_error,
            patterns: patterns.clone(),
        }
    }

    /// Reset all error tracking
    pub fn reset(&self) {
        let mut history = self.error_history.lock().unwrap();
        history.clear();
        
        let mut patterns = self.error_patterns.lock().unwrap();
        patterns.clear();
    }
}

/// Error statistics for monitoring and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    pub total_errors: u32,
    pub unique_error_types: usize,
    pub most_common_error: Option<ErrorType>,
    pub patterns: HashMap<ErrorType, ErrorPattern>,
}
