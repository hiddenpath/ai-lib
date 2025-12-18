use async_trait::async_trait;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::interceptors::{Interceptor, RequestContext};
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Circuit is open, requests fail fast
    HalfOpen, // Testing if service is back
}

/// Circuit breaker interceptor
pub struct CircuitBreakerInterceptor {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Arc<AtomicU32>, // 0=Closed, 1=Open, 2=HalfOpen
    failure_count: Arc<AtomicU32>,
    last_failure_time: Arc<AtomicU64>, // Unix timestamp in seconds
}

impl CircuitBreakerInterceptor {
    /// Create a new circuit breaker interceptor
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            state: Arc::new(AtomicU32::new(0)), // Closed
            failure_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl Default for CircuitBreakerInterceptor {
    /// Create with default settings (5 failures, 60s recovery)
    fn default() -> Self {
        Self::new(5, Duration::from_secs(60))
    }
}

impl CircuitBreakerInterceptor {
    fn get_state(&self) -> CircuitState {
        match self.state.load(Ordering::Relaxed) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }

    fn set_state(&self, state: CircuitState) {
        let value = match state {
            CircuitState::Closed => 0,
            CircuitState::Open => 1,
            CircuitState::HalfOpen => 2,
        };
        self.state.store(value, Ordering::Relaxed);
    }

    fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.set_state(CircuitState::Closed);
    }

    fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.last_failure_time.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Ordering::Relaxed,
        );

        if count >= self.failure_threshold {
            self.set_state(CircuitState::Open);
        }
    }

    fn should_allow_request(&self) -> bool {
        match self.get_state() {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let last_failure = self.last_failure_time.load(Ordering::Relaxed);

                if now - last_failure >= self.recovery_timeout.as_secs() {
                    self.set_state(CircuitState::HalfOpen);
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[async_trait]
impl Interceptor for CircuitBreakerInterceptor {
    async fn on_request(&self, ctx: &RequestContext, _req: &ChatCompletionRequest) {
        if !self.should_allow_request() {
            // Circuit breaker opened: In production, this would be logged to metrics
            let _ = ctx;
        }
    }

    async fn on_response(
        &self,
        _ctx: &RequestContext,
        _req: &ChatCompletionRequest,
        resp: &ChatCompletionResponse,
    ) {
        if resp.choices.is_empty() {
            self.record_failure();
        } else {
            self.record_success();
        }
    }

    async fn on_error(
        &self,
        _ctx: &RequestContext,
        _req: &ChatCompletionRequest,
        err: &AiLibError,
    ) {
        // Only count certain errors as failures
        match err {
            AiLibError::NetworkError(_) | AiLibError::TimeoutError(_) => {
                self.record_failure();
            }
            _ => {
                // Don't count auth errors, validation errors, etc. as circuit breaker failures
            }
        }
    }
}

/// Wrapper that implements circuit breaker logic around a function call
pub struct CircuitBreakerWrapper {
    interceptor: CircuitBreakerInterceptor,
}

impl CircuitBreakerWrapper {
    pub fn new(interceptor: CircuitBreakerInterceptor) -> Self {
        Self { interceptor }
    }
}

impl Default for CircuitBreakerWrapper {
    fn default() -> Self {
        Self::new(CircuitBreakerInterceptor::default())
    }
}

impl CircuitBreakerWrapper {
    /// Execute a function with circuit breaker protection
    pub async fn execute<F, Fut>(
        &self,
        ctx: &RequestContext,
        _req: &ChatCompletionRequest,
        f: F,
    ) -> Result<ChatCompletionResponse, AiLibError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<ChatCompletionResponse, AiLibError>>,
    {
        if !self.interceptor.should_allow_request() {
            return Err(AiLibError::ProviderError(format!(
                "Circuit breaker is OPEN for {}:{}",
                ctx.provider, ctx.model
            )));
        }

        match f().await {
            Ok(response) => {
                self.interceptor.record_success();
                Ok(response)
            }
            Err(err) => {
                self.interceptor.record_failure();
                Err(err)
            }
        }
    }
}
