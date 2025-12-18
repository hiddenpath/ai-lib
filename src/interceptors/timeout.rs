use async_trait::async_trait;
use std::time::Duration;
use tokio::time::timeout;

use crate::interceptors::{Interceptor, RequestContext};
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};

/// Timeout interceptor that wraps requests with a timeout
pub struct TimeoutInterceptor {
    timeout_duration: Duration,
}

impl TimeoutInterceptor {
    /// Create a new timeout interceptor
    pub fn new(timeout_duration: Duration) -> Self {
        Self { timeout_duration }
    }
}

impl Default for TimeoutInterceptor {
    /// Create with default timeout (30 seconds)
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

#[async_trait]
impl Interceptor for TimeoutInterceptor {
    async fn on_request(&self, ctx: &RequestContext, _req: &ChatCompletionRequest) {
        // Log timeout setting (in a real implementation, you'd use proper logging)
        // Timeout setting: In production, this would be logged to metrics
        let _ = (self.timeout_duration, ctx);
    }
}

/// Wrapper that implements timeout logic around a function call
pub struct TimeoutWrapper {
    interceptor: TimeoutInterceptor,
}

impl TimeoutWrapper {
    pub fn new(interceptor: TimeoutInterceptor) -> Self {
        Self { interceptor }
    }
}

impl Default for TimeoutWrapper {
    fn default() -> Self {
        Self::new(TimeoutInterceptor::default())
    }
}

impl TimeoutWrapper {
    /// Execute a function with timeout
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
        match timeout(self.interceptor.timeout_duration, f()).await {
            Ok(result) => result,
            Err(_) => Err(AiLibError::TimeoutError(format!(
                "Request timeout after {:?} for {}:{}",
                self.interceptor.timeout_duration, ctx.provider, ctx.model
            ))),
        }
    }
}
