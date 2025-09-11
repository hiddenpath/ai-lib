use async_trait::async_trait;

use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};

pub mod breaker;
pub mod default;
pub mod rate_limit;
pub mod retry;
pub mod timeout;

pub use breaker::CircuitBreakerInterceptor;
pub use default::{create_default_interceptors, DefaultInterceptorsBuilder};
pub use rate_limit::RateLimitInterceptor;
pub use retry::RetryInterceptor;
pub use timeout::TimeoutInterceptor;

/// Request context passed to interceptors. Keep minimal to avoid API churn.
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub provider: String,
    pub model: String,
}

/// Response context passed to interceptors.
#[derive(Debug, Clone)]
pub struct ResponseContext {
    pub success: bool,
}

/// Interceptor trait for cross-cutting concerns (retry, rate limit, circuit breaker, audit, etc.)
#[async_trait]
pub trait Interceptor: Send + Sync {
    /// Called before the request is executed.
    async fn on_request(&self, _ctx: &RequestContext, _req: &ChatCompletionRequest) {}

    /// Called after the response is received (or after an error occurs if `on_error` is also implemented).
    async fn on_response(
        &self,
        _ctx: &RequestContext,
        _req: &ChatCompletionRequest,
        _resp: &ChatCompletionResponse,
    ) {
    }

    /// Called when an error happens during request execution.
    async fn on_error(&self, _ctx: &RequestContext, _req: &ChatCompletionRequest, _err: &AiLibError) {}
}

/// A simple interceptor pipeline that runs hooks in order.
pub struct InterceptorPipeline {
    pub(crate) interceptors: Vec<Box<dyn Interceptor>>,
}

impl InterceptorPipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self { interceptors: Vec::new() }
    }

    /// Add an interceptor to the pipeline
    pub fn with<I: Interceptor + 'static>(mut self, interceptor: I) -> Self {
        self.interceptors.push(Box::new(interceptor));
        self
    }

    /// Run the pipeline around a provided async function that performs the actual call.
    /// This avoids tying the pipeline directly to a specific adapter type.
    pub async fn execute<F, Fut>(
        &self,
        ctx: &RequestContext,
        req: &ChatCompletionRequest,
        f: F,
    ) -> Result<ChatCompletionResponse, AiLibError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<ChatCompletionResponse, AiLibError>>,
    {
        // on_request hooks
        for ic in &self.interceptors {
            ic.on_request(ctx, req).await;
        }

        // execute core
        match f().await {
            Ok(resp) => {
                for ic in &self.interceptors {
                    ic.on_response(ctx, req, &resp).await;
                }
                Ok(resp)
            }
            Err(err) => {
                for ic in &self.interceptors {
                    ic.on_error(ctx, req, &err).await;
                }
                Err(err)
            }
        }
    }
}

impl Default for InterceptorPipeline {
    fn default() -> Self { Self::new() }
}


