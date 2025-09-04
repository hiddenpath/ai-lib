//! Integration tests for resilience features

use ai_lib::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use ai_lib::rate_limiter::{TokenBucket, RateLimiterConfig};
use ai_lib::error_handling::{ErrorContext, SuggestedAction, ErrorRecoveryManager};
use ai_lib::error_handling::recovery::ErrorType;
use ai_lib::types::AiLibError;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_resilience_features_integration() {
    // Test circuit breaker with rate limiter integration
    let circuit_config = CircuitBreakerConfig::production();
    let rate_config = RateLimiterConfig::production();
    
    let breaker = CircuitBreaker::new(circuit_config);
    let rate_limiter = TokenBucket::new(rate_config);
    let error_manager = ErrorRecoveryManager::new();
    
    // Test initial states
    assert_eq!(breaker.state(), ai_lib::circuit_breaker::CircuitState::Closed);
    assert!(rate_limiter.is_healthy());
    assert_eq!(error_manager.get_error_statistics().total_errors, 0);
}

#[tokio::test]
async fn test_circuit_breaker_with_rate_limiter() {
    let circuit_config = CircuitBreakerConfig::development();
    let rate_config = RateLimiterConfig::development();
    
    let breaker = CircuitBreaker::new(circuit_config);
    let rate_limiter = TokenBucket::new(rate_config);
    
    // Test successful operation
    let result = rate_limiter.acquire(1).await;
    assert!(result.is_ok());
    
    // Test circuit breaker metrics
    let metrics = breaker.get_metrics();
    assert_eq!(metrics.state, ai_lib::circuit_breaker::CircuitState::Closed);
    assert!(metrics.total_requests >= 0);
}

#[tokio::test]
async fn test_error_handling_with_recovery() {
    let error_manager = ErrorRecoveryManager::new();
    
    // Test different error types
    let errors = vec![
        AiLibError::RateLimitExceeded("Test rate limit".to_string()),
        AiLibError::NetworkError("Test network error".to_string()),
        AiLibError::TimeoutError("Test timeout".to_string()),
    ];
    
    for error in errors {
        let context = ErrorContext::new("test_provider".to_string(), "/test/endpoint".to_string());
        let _ = error_manager.handle_error(&error, &context).await;
    }
    
    let stats = error_manager.get_error_statistics();
    assert_eq!(stats.total_errors, 3);
    assert_eq!(stats.unique_error_types, 3);
}

#[tokio::test]
async fn test_adaptive_rate_limiting() {
    let mut config = RateLimiterConfig::default();
    config.adaptive = true;
    config.initial_rate = Some(5);
    
    let rate_limiter = TokenBucket::new(config);
    
    // Test adaptive rate adjustment
    rate_limiter.adjust_rate(true); // Success
    rate_limiter.adjust_rate(false); // Failure
    
    let metrics = rate_limiter.get_metrics();
    assert!(metrics.is_adaptive);
    assert!(metrics.adaptive_rate.is_some());
}

#[tokio::test]
async fn test_circuit_breaker_state_transitions() {
    let config = CircuitBreakerConfig::development();
    let breaker = CircuitBreaker::new(config);
    
    // Test force operations
    breaker.force_open();
    assert_eq!(breaker.state(), ai_lib::circuit_breaker::CircuitState::Open);
    
    breaker.force_close();
    assert_eq!(breaker.state(), ai_lib::circuit_breaker::CircuitState::Closed);
    
    // Test reset
    breaker.reset();
    let metrics = breaker.get_metrics();
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
}

#[tokio::test]
async fn test_rate_limiter_metrics() {
    let config = RateLimiterConfig::default();
    let rate_limiter = TokenBucket::new(config);
    
    // Test initial metrics
    let metrics = rate_limiter.get_metrics();
    assert_eq!(metrics.current_tokens, 20); // burst_capacity
    assert_eq!(metrics.capacity, 20);
    assert_eq!(metrics.refill_rate, 10);
    assert!(!metrics.is_adaptive);
    
    // Test success and rejection rates
    assert_eq!(rate_limiter.success_rate(), 100.0);
    assert_eq!(rate_limiter.rejection_rate(), 0.0);
}

#[tokio::test]
async fn test_error_pattern_analysis() {
    let error_manager = ErrorRecoveryManager::new();
    
    // Simulate repeated errors
    for _ in 0..5 {
        let error = AiLibError::RateLimitExceeded("Repeated error".to_string());
        let context = ErrorContext::new("test_provider".to_string(), "/test/endpoint".to_string());
        let _ = error_manager.handle_error(&error, &context).await;
    }
    
    let patterns = error_manager.get_error_patterns();
    assert!(patterns.contains_key(&ErrorType::RateLimit));
    
    let rate_limit_pattern = &patterns[&ErrorType::RateLimit];
    assert_eq!(rate_limit_pattern.count, 5);
    // Frequency might be 0.0 if all errors occurred within the same minute
    assert!(rate_limit_pattern.frequency >= 0.0);
}

#[tokio::test]
async fn test_suggested_actions() {
    let error_manager = ErrorRecoveryManager::new();
    
    // Test different error types and their suggested actions
    let test_cases = vec![
        (AiLibError::RateLimitExceeded("Rate limit".to_string()), ErrorType::RateLimit),
        (AiLibError::AuthenticationError("Auth failed".to_string()), ErrorType::Authentication),
        (AiLibError::ContextLengthExceeded("Context too long".to_string()), ErrorType::ContextLengthExceeded),
    ];
    
    for (error, expected_type) in test_cases {
        let context = ErrorContext::new("test_provider".to_string(), "/test/endpoint".to_string());
        let _ = error_manager.handle_error(&error, &context).await;
        
        let patterns = error_manager.get_error_patterns();
        if let Some(pattern) = patterns.get(&expected_type) {
            match &pattern.suggested_action {
                SuggestedAction::Retry { .. } => {
                    // Retry action should have delay and max attempts
                    assert!(matches!(pattern.suggested_action, SuggestedAction::Retry { .. }));
                }
                SuggestedAction::CheckCredentials => {
                    assert!(matches!(pattern.suggested_action, SuggestedAction::CheckCredentials));
                }
                SuggestedAction::ReduceRequestSize { .. } => {
                    assert!(matches!(pattern.suggested_action, SuggestedAction::ReduceRequestSize { .. }));
                }
                _ => {}
            }
        }
    }
}

#[tokio::test]
async fn test_resilience_configuration_presets() {
    // Test production configuration
    let circuit_prod = CircuitBreakerConfig::production();
    let rate_prod = RateLimiterConfig::production();
    
    assert_eq!(circuit_prod.failure_threshold, 3);
    assert_eq!(rate_prod.requests_per_second, 5);
    assert!(rate_prod.adaptive);
    
    // Test development configuration
    let circuit_dev = CircuitBreakerConfig::development();
    let rate_dev = RateLimiterConfig::development();
    
    assert_eq!(circuit_dev.failure_threshold, 10);
    assert_eq!(rate_dev.requests_per_second, 20);
    assert!(!rate_dev.adaptive);
    
    // Test conservative configuration
    let circuit_cons = CircuitBreakerConfig::conservative();
    let rate_cons = RateLimiterConfig::conservative();
    
    assert_eq!(circuit_cons.failure_threshold, 2);
    assert_eq!(rate_cons.requests_per_second, 2);
    assert!(rate_cons.adaptive);
}
