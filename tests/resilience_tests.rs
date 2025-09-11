//! Tests for resilience features

use ai_lib::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use ai_lib::error_handling::recovery::ErrorType;
use ai_lib::error_handling::{ErrorContext, ErrorRecoveryManager, SuggestedAction};
use ai_lib::rate_limiter::{RateLimiterConfig, TokenBucket};
use ai_lib::types::AiLibError;
use std::time::Duration;

#[tokio::test]
async fn test_circuit_breaker_basic_functionality() {
    let config = CircuitBreakerConfig::default();
    let breaker = CircuitBreaker::new(config);

    // Test initial state
    assert_eq!(
        breaker.state(),
        ai_lib::circuit_breaker::CircuitState::Closed
    );
    assert_eq!(breaker.failure_count(), 0);
    assert_eq!(breaker.success_count(), 0);
    assert!(breaker.is_healthy());
}

#[tokio::test]
async fn test_circuit_breaker_metrics() {
    let config = CircuitBreakerConfig::default();
    let breaker = CircuitBreaker::new(config);

    // Test initial metrics
    let metrics = breaker.get_metrics();
    assert_eq!(metrics.state, ai_lib::circuit_breaker::CircuitState::Closed);
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
    assert_eq!(breaker.success_rate(), 100.0);
    assert_eq!(breaker.failure_rate(), 0.0);
}

#[tokio::test]
async fn test_circuit_breaker_force_operations() {
    let config = CircuitBreakerConfig::default();
    let breaker = CircuitBreaker::new(config);

    // Test force open
    breaker.force_open();
    assert_eq!(breaker.state(), ai_lib::circuit_breaker::CircuitState::Open);

    // Test force close
    breaker.force_close();
    assert_eq!(
        breaker.state(),
        ai_lib::circuit_breaker::CircuitState::Closed
    );

    // Test reset
    breaker.reset();
    let metrics = breaker.get_metrics();
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
}

#[tokio::test]
async fn test_circuit_breaker_config_presets() {
    // Test production config
    let prod_config = CircuitBreakerConfig::production();
    assert_eq!(prod_config.failure_threshold, 3);
    assert_eq!(prod_config.recovery_timeout, Duration::from_secs(60));

    // Test development config
    let dev_config = CircuitBreakerConfig::development();
    assert_eq!(dev_config.failure_threshold, 10);
    assert_eq!(dev_config.recovery_timeout, Duration::from_secs(15));

    // Test conservative config
    let conservative_config = CircuitBreakerConfig::conservative();
    assert_eq!(conservative_config.failure_threshold, 2);
    assert_eq!(
        conservative_config.recovery_timeout,
        Duration::from_secs(120)
    );
}

#[tokio::test]
async fn test_token_bucket_basic_functionality() {
    let config = RateLimiterConfig::default();
    let bucket = TokenBucket::new(config);

    // Test initial state
    assert_eq!(bucket.tokens(), 20); // burst_capacity
    assert!(bucket.is_healthy());
    assert_eq!(bucket.success_rate(), 100.0);
    assert_eq!(bucket.rejection_rate(), 0.0);
}

#[tokio::test]
async fn test_token_bucket_metrics() {
    let config = RateLimiterConfig::default();
    let bucket = TokenBucket::new(config);

    // Test initial metrics
    let metrics = bucket.get_metrics();
    assert_eq!(metrics.current_tokens, 20);
    assert_eq!(metrics.capacity, 20);
    assert_eq!(metrics.refill_rate, 10);
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.rejected_requests, 0);
    assert!(!metrics.is_adaptive);
}

#[tokio::test]
async fn test_token_bucket_adaptive_rate() {
    let mut config = RateLimiterConfig::default();
    config.adaptive = true;
    config.initial_rate = Some(5);
    let bucket = TokenBucket::new(config);

    // Test adaptive rate adjustment
    bucket.adjust_rate(true); // Success - should increase rate
    bucket.adjust_rate(false); // Failure - should decrease rate

    // Test manual rate setting
    bucket.set_adaptive_rate(15);
    let metrics = bucket.get_metrics();
    assert_eq!(metrics.adaptive_rate, Some(15));

    // Test reset
    bucket.reset_adaptive_rate();
    let metrics = bucket.get_metrics();
    assert_eq!(metrics.adaptive_rate, Some(10)); // Should reset to refill_rate
}

#[tokio::test]
async fn test_token_bucket_enable_disable() {
    let config = RateLimiterConfig::default();
    let bucket = TokenBucket::new(config);

    // Test disable
    bucket.set_enabled(false);
    let result = bucket.acquire(1).await;
    assert!(result.is_ok()); // Should succeed when disabled

    // Test enable
    bucket.set_enabled(true);
    bucket.reset();
    let result = bucket.acquire(1).await;
    assert!(result.is_ok()); // Should succeed when enabled
}

#[tokio::test]
async fn test_token_bucket_config_presets() {
    // Test production config
    let prod_config = RateLimiterConfig::production();
    assert_eq!(prod_config.requests_per_second, 5);
    assert_eq!(prod_config.burst_capacity, 10);
    assert!(prod_config.adaptive);

    // Test development config
    let dev_config = RateLimiterConfig::development();
    assert_eq!(dev_config.requests_per_second, 20);
    assert_eq!(dev_config.burst_capacity, 50);
    assert!(!dev_config.adaptive);

    // Test conservative config
    let conservative_config = RateLimiterConfig::conservative();
    assert_eq!(conservative_config.requests_per_second, 2);
    assert_eq!(conservative_config.burst_capacity, 5);
    assert!(conservative_config.adaptive);
}

#[tokio::test]
async fn test_error_context_creation() {
    let context = ErrorContext::new("groq".to_string(), "/chat/completions".to_string());

    assert_eq!(context.provider, "groq");
    assert_eq!(context.endpoint, "/chat/completions");
    assert_eq!(context.retry_count, 0);
    assert!(matches!(
        context.suggested_action,
        SuggestedAction::NoAction
    ));
}

#[tokio::test]
async fn test_error_context_with_retry() {
    let context = ErrorContext::new("openai".to_string(), "/v1/chat/completions".to_string())
        .with_retry(3)
        .with_request_id("req-123".to_string());

    assert_eq!(context.provider, "openai");
    assert_eq!(context.endpoint, "/v1/chat/completions");
    assert_eq!(context.retry_count, 3);
    assert_eq!(context.request_id, Some("req-123".to_string()));
}

#[tokio::test]
async fn test_suggested_actions() {
    // Test retry action
    let retry_action = SuggestedAction::Retry {
        delay_ms: 5000,
        max_attempts: 3,
    };
    assert!(matches!(retry_action, SuggestedAction::Retry { .. }));

    // Test switch provider action
    let switch_action = SuggestedAction::SwitchProvider {
        alternative_providers: vec!["groq".to_string(), "anthropic".to_string()],
    };
    assert!(matches!(
        switch_action,
        SuggestedAction::SwitchProvider { .. }
    ));

    // Test no action
    let no_action = SuggestedAction::NoAction;
    assert!(matches!(no_action, SuggestedAction::NoAction));
}

#[tokio::test]
async fn test_error_recovery_manager_basic_functionality() {
    let manager = ErrorRecoveryManager::new();

    // Test initial state
    let stats = manager.get_error_statistics();
    assert_eq!(stats.total_errors, 0);
    assert_eq!(stats.unique_error_types, 0);
    assert!(stats.most_common_error.is_none());
}

#[tokio::test]
async fn test_error_recovery_manager_error_handling() {
    let manager = ErrorRecoveryManager::new();

    // Create test error and context
    let error = AiLibError::RateLimitExceeded("Test rate limit".to_string());
    let context = ErrorContext::new("test_provider".to_string(), "/test/endpoint".to_string());

    // Handle error
    let result = manager.handle_error(&error, &context).await;
    assert!(result.is_err()); // Should fail since no recovery strategy is registered

    // Check error patterns
    let patterns = manager.get_error_patterns();
    assert!(patterns.contains_key(&ErrorType::RateLimit));

    let stats = manager.get_error_statistics();
    assert_eq!(stats.total_errors, 1);
    assert_eq!(stats.unique_error_types, 1);
    assert_eq!(stats.most_common_error, Some(ErrorType::RateLimit));
}

#[tokio::test]
async fn test_error_recovery_manager_multiple_errors() {
    let manager = ErrorRecoveryManager::new();

    // Record multiple errors of different types
    let errors = vec![
        (
            AiLibError::RateLimitExceeded("Rate limit 1".to_string()),
            ErrorType::RateLimit,
        ),
        (
            AiLibError::RateLimitExceeded("Rate limit 2".to_string()),
            ErrorType::RateLimit,
        ),
        (
            AiLibError::NetworkError("Network error".to_string()),
            ErrorType::Network,
        ),
        (
            AiLibError::TimeoutError("Timeout".to_string()),
            ErrorType::Timeout,
        ),
    ];

    for (error, expected_type) in errors {
        let context = ErrorContext::new("test_provider".to_string(), "/test/endpoint".to_string());
        let _ = manager.handle_error(&error, &context).await;
    }

    let stats = manager.get_error_statistics();
    assert_eq!(stats.total_errors, 4);
    assert_eq!(stats.unique_error_types, 3);
    assert_eq!(stats.most_common_error, Some(ErrorType::RateLimit)); // Most frequent
}

#[tokio::test]
async fn test_error_recovery_manager_reset() {
    let manager = ErrorRecoveryManager::new();

    // Record some errors
    let error = AiLibError::RateLimitExceeded("Test".to_string());
    let context = ErrorContext::new("test_provider".to_string(), "/test/endpoint".to_string());
    let _ = manager.handle_error(&error, &context).await;

    // Verify errors are recorded
    let stats = manager.get_error_statistics();
    assert_eq!(stats.total_errors, 1);

    // Reset
    manager.reset();

    // Verify reset
    let stats = manager.get_error_statistics();
    assert_eq!(stats.total_errors, 0);
    assert_eq!(stats.unique_error_types, 0);
}
