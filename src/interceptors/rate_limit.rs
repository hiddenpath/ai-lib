use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::interceptors::{Interceptor, RequestContext};
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};

/// Rate limiter using token bucket algorithm
pub struct RateLimitInterceptor {
    requests_per_minute: u32,
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
}

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    capacity: f64,
    refill_rate: f64, // tokens per second
}

impl TokenBucket {
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            capacity,
            refill_rate,
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * self.refill_rate;
        
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }

    fn time_until_next_token(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::from_secs(0)
        } else {
            let tokens_needed = 1.0 - self.tokens;
            Duration::from_secs_f64(tokens_needed / self.refill_rate)
        }
    }
}

impl RateLimitInterceptor {
    /// Create a new rate limit interceptor
    pub fn new(requests_per_minute: u32) -> Self {
        let capacity = requests_per_minute as f64;
        let _refill_rate = capacity / 60.0; // tokens per second
        
        Self {
            requests_per_minute,
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create with default rate limit (60 requests per minute)
    pub fn default() -> Self {
        Self::new(60)
    }
}

impl Default for RateLimitInterceptor {
    fn default() -> Self {
        Self::new(60)
    }
}

impl RateLimitInterceptor {
    fn get_bucket_key(&self, ctx: &RequestContext) -> String {
        format!("{}:{}", ctx.provider, ctx.model)
    }

    fn try_acquire_token(&self, ctx: &RequestContext) -> Result<(), Duration> {
        let key = self.get_bucket_key(ctx);
        let mut buckets = self.buckets.lock().unwrap();
        
        let bucket = buckets.entry(key).or_insert_with(|| {
            let capacity = self.requests_per_minute as f64;
            let refill_rate = capacity / 60.0;
            TokenBucket::new(capacity, refill_rate)
        });

        if bucket.try_consume(1.0) {
            Ok(())
        } else {
            Err(bucket.time_until_next_token())
        }
    }
}

#[async_trait]
impl Interceptor for RateLimitInterceptor {
    async fn on_request(&self, ctx: &RequestContext, _req: &ChatCompletionRequest) {
        match self.try_acquire_token(ctx) {
            Ok(_) => {
                // Token acquired successfully
            }
            Err(wait_time) => {
                // Rate limit exceeded: In production, this would be logged to metrics
                let _ = (ctx, wait_time);
            }
        }
    }
}

/// Wrapper that implements rate limiting around a function call
pub struct RateLimitWrapper {
    interceptor: RateLimitInterceptor,
}

impl RateLimitWrapper {
    pub fn new(interceptor: RateLimitInterceptor) -> Self {
        Self { interceptor }
    }

    pub fn default() -> Self {
        Self::new(RateLimitInterceptor::default())
    }
}

impl RateLimitWrapper {
    /// Execute a function with rate limiting
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
        match self.interceptor.try_acquire_token(ctx) {
            Ok(_) => f().await,
            Err(wait_time) => Err(AiLibError::RateLimitExceeded(format!(
                "Rate limit exceeded for {}:{}, try again in {:?}",
                ctx.provider, ctx.model, wait_time
            ))),
        }
    }
}
