use std::time::Duration;

use crate::interceptors::{
    InterceptorPipeline,
    CircuitBreakerInterceptor, RateLimitInterceptor, RetryInterceptor, TimeoutInterceptor,
};

/// Default interceptors bundle configuration
#[derive(Debug, Clone)]
pub struct DefaultInterceptorsConfig {
    pub retry_max_attempts: u32,
    pub retry_base_delay: Duration,
    pub retry_max_delay: Duration,
    pub timeout_duration: Duration,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_recovery: Duration,
    pub rate_limit_per_minute: u32,
}

impl Default for DefaultInterceptorsConfig {
    fn default() -> Self {
        Self {
            retry_max_attempts: 3,
            retry_base_delay: Duration::from_secs(1),
            retry_max_delay: Duration::from_secs(10),
            timeout_duration: Duration::from_secs(30),
            circuit_breaker_threshold: 5,
            circuit_breaker_recovery: Duration::from_secs(60),
            rate_limit_per_minute: 60,
        }
    }
}

/// Builder for creating default interceptors bundle
pub struct DefaultInterceptorsBuilder {
    config: DefaultInterceptorsConfig,
    enable_retry: bool,
    enable_timeout: bool,
    enable_circuit_breaker: bool,
    enable_rate_limit: bool,
}

impl DefaultInterceptorsBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: DefaultInterceptorsConfig::default(),
            enable_retry: true,
            enable_timeout: true,
            enable_circuit_breaker: true,
            enable_rate_limit: true,
        }
    }

    /// Configure retry settings
    pub fn with_retry(mut self, max_attempts: u32, base_delay: Duration, max_delay: Duration) -> Self {
        self.config.retry_max_attempts = max_attempts;
        self.config.retry_base_delay = base_delay;
        self.config.retry_max_delay = max_delay;
        self
    }

    /// Configure timeout settings
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.config.timeout_duration = duration;
        self
    }

    /// Configure circuit breaker settings
    pub fn with_circuit_breaker(mut self, threshold: u32, recovery: Duration) -> Self {
        self.config.circuit_breaker_threshold = threshold;
        self.config.circuit_breaker_recovery = recovery;
        self
    }

    /// Configure rate limit settings
    pub fn with_rate_limit(mut self, requests_per_minute: u32) -> Self {
        self.config.rate_limit_per_minute = requests_per_minute;
        self
    }

    /// Enable or disable retry interceptor
    pub fn enable_retry(mut self, enable: bool) -> Self {
        self.enable_retry = enable;
        self
    }

    /// Enable or disable timeout interceptor
    pub fn enable_timeout(mut self, enable: bool) -> Self {
        self.enable_timeout = enable;
        self
    }

    /// Enable or disable circuit breaker interceptor
    pub fn enable_circuit_breaker(mut self, enable: bool) -> Self {
        self.enable_circuit_breaker = enable;
        self
    }

    /// Enable or disable rate limit interceptor
    pub fn enable_rate_limit(mut self, enable: bool) -> Self {
        self.enable_rate_limit = enable;
        self
    }

    /// Build the interceptors pipeline
    pub fn build(self) -> InterceptorPipeline {
        let mut pipeline = InterceptorPipeline::new();

        if self.enable_retry {
            let retry = RetryInterceptor::new(
                self.config.retry_max_attempts,
                self.config.retry_base_delay,
                self.config.retry_max_delay,
            );
            pipeline = pipeline.with(retry);
        }

        if self.enable_timeout {
            let timeout = TimeoutInterceptor::new(self.config.timeout_duration);
            pipeline = pipeline.with(timeout);
        }

        if self.enable_circuit_breaker {
            let breaker = CircuitBreakerInterceptor::new(
                self.config.circuit_breaker_threshold,
                self.config.circuit_breaker_recovery,
            );
            pipeline = pipeline.with(breaker);
        }

        if self.enable_rate_limit {
            let rate_limit = RateLimitInterceptor::new(self.config.rate_limit_per_minute);
            pipeline = pipeline.with(rate_limit);
        }

        pipeline
    }
}

impl Default for DefaultInterceptorsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default interceptors pipeline with sensible defaults
pub fn create_default_interceptors() -> InterceptorPipeline {
    DefaultInterceptorsBuilder::new().build()
}

/// Create a minimal interceptors pipeline (retry + timeout only)
pub fn create_minimal_interceptors() -> InterceptorPipeline {
    DefaultInterceptorsBuilder::new()
        .enable_circuit_breaker(false)
        .enable_rate_limit(false)
        .build()
}

/// Create a production-ready interceptors pipeline
pub fn create_production_interceptors() -> InterceptorPipeline {
    DefaultInterceptorsBuilder::new()
        .with_retry(5, Duration::from_secs(2), Duration::from_secs(30))
        .with_timeout(Duration::from_secs(60))
        .with_circuit_breaker(10, Duration::from_secs(120))
        .with_rate_limit(120)
        .build()
}
