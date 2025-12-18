use async_trait::async_trait;
use std::time::Duration;
use tokio::time::sleep;

use crate::interceptors::{Interceptor, RequestContext};
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};

/// Retry interceptor with exponential backoff
pub struct RetryInterceptor {
    max_retries: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl RetryInterceptor {
    /// Create a new retry interceptor
    pub fn new(max_retries: u32, base_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
        }
    }
}

impl Default for RetryInterceptor {
    /// Create with default settings (3 retries, 1s base delay, 10s max delay)
    fn default() -> Self {
        Self::new(3, Duration::from_secs(1), Duration::from_secs(10))
    }
}

impl RetryInterceptor {
    /// Check if an error is retryable
    fn is_retryable_error(&self, error: &AiLibError) -> bool {
        matches!(
            error,
            AiLibError::NetworkError(_)
                | AiLibError::TimeoutError(_)
                | AiLibError::RateLimitExceeded(_)
                | AiLibError::ProviderError(_)
        )
    }

    /// Calculate delay with exponential backoff
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_ms = self.base_delay.as_millis() as u64 * 2_u64.pow(attempt);
        let capped_delay = delay_ms.min(self.max_delay.as_millis() as u64);
        Duration::from_millis(capped_delay)
    }
}

#[async_trait]
impl Interceptor for RetryInterceptor {
    async fn on_error(&self, ctx: &RequestContext, _req: &ChatCompletionRequest, err: &AiLibError) {
        if self.is_retryable_error(err) {
            // Log retry attempt (in a real implementation, you'd use proper logging)
            // Retry attempt: In production, this would be logged to metrics
            let _ = (ctx, err);
        }
    }
}

/// Wrapper that implements retry logic around a function call
pub struct RetryWrapper {
    interceptor: RetryInterceptor,
}

impl RetryWrapper {
    pub fn new(interceptor: RetryInterceptor) -> Self {
        Self { interceptor }
    }
}

impl Default for RetryWrapper {
    fn default() -> Self {
        Self::new(RetryInterceptor::default())
    }
}

impl RetryWrapper {
    /// Execute a function with retry logic
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
        let mut last_error = None;

        for attempt in 0..=self.interceptor.max_retries {
            match f().await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err.clone());

                    if attempt < self.interceptor.max_retries
                        && self.interceptor.is_retryable_error(&err)
                    {
                        let delay = self.interceptor.calculate_delay(attempt);
                        // Retry attempt: In production, this would be logged to metrics
                        let _ = (attempt, ctx, delay);
                        sleep(delay).await;
                    } else {
                        break;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| AiLibError::ProviderError("Max retries exceeded".to_string())))
    }
}
