//! Token bucket rate limiter implementation

use crate::metrics::Metrics;
use crate::rate_limiter::RateLimiterConfig;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Rate limiter metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterMetrics {
    pub current_tokens: u64,
    pub capacity: u64,
    pub refill_rate: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub rejected_requests: u64,
    pub adaptive_rate: Option<u64>,
    pub is_adaptive: bool,
    pub uptime: Duration,
}

/// Token bucket rate limiter
pub struct TokenBucket {
    capacity: u64,
    tokens: AtomicU64,
    refill_rate: u64,
    last_refill: AtomicU64,
    // Adaptive rate limiting
    adaptive: bool,
    adaptive_rate: AtomicU64,
    min_rate: u64,
    max_rate: u64,
    // Metrics
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    rejected_requests: AtomicU64,
    start_time: Instant,
    // Optional metrics collector
    metrics: Option<Arc<dyn Metrics>>,
    // Rate limiter enabled flag
    enabled: AtomicBool,
}

impl TokenBucket {
    /// Create a new token bucket with the given configuration
    pub fn new(config: RateLimiterConfig) -> Self {
        let capacity = config.burst_capacity;
        let refill_rate = config.requests_per_second;
        let adaptive = config.adaptive;
        let initial_rate = config.initial_rate.unwrap_or(refill_rate);

        Self {
            capacity,
            tokens: AtomicU64::new(capacity),
            refill_rate,
            last_refill: AtomicU64::new(Instant::now().elapsed().as_millis() as u64),
            adaptive,
            adaptive_rate: AtomicU64::new(initial_rate),
            min_rate: (refill_rate / 4).max(1), // Minimum 25% of original rate
            max_rate: refill_rate * 2,          // Maximum 200% of original rate
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            rejected_requests: AtomicU64::new(0),
            start_time: Instant::now(),
            metrics: None,
            enabled: AtomicBool::new(true),
        }
    }

    /// Create a new token bucket with metrics collection
    pub fn with_metrics(config: RateLimiterConfig, metrics: Arc<dyn Metrics>) -> Self {
        let mut bucket = Self::new(config);
        bucket.metrics = Some(metrics);
        bucket
    }

    /// Enable or disable the rate limiter
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
    }

    /// Acquire the specified number of tokens
    pub async fn acquire(&self, tokens: u64) -> Result<(), RateLimitError> {
        // Check if rate limiter is enabled
        if !self.enabled.load(Ordering::Acquire) {
            return Ok(());
        }

        // Increment total requests counter
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        if tokens > self.capacity {
            self.rejected_requests.fetch_add(1, Ordering::Relaxed);
            return Err(RateLimitError::RequestTooLarge {
                requested: tokens,
                max_allowed: self.capacity,
            });
        }

        loop {
            self.refill_tokens();

            let current = self.tokens.load(Ordering::Acquire);
            if current >= tokens {
                if self
                    .tokens
                    .compare_exchange_weak(
                        current,
                        current - tokens,
                        Ordering::Release,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    self.successful_requests.fetch_add(1, Ordering::Relaxed);

                    // Record metrics
                    if let Some(metrics) = &self.metrics {
                        metrics
                            .incr_counter("rate_limiter.requests_successful", 1)
                            .await;
                    }

                    return Ok(());
                }
            } else {
                // Calculate wait time based on current rate
                let current_rate = if self.adaptive {
                    self.adaptive_rate.load(Ordering::Acquire)
                } else {
                    self.refill_rate
                };

                let wait_time = (tokens - current) * 1000 / current_rate;
                if wait_time > 0 {
                    sleep(Duration::from_millis(wait_time)).await;
                }
            }
        }
    }

    /// Refill tokens based on elapsed time
    fn refill_tokens(&self) {
        let now = Instant::now().elapsed().as_millis() as u64;
        let last_refill = self.last_refill.load(Ordering::Acquire);
        let elapsed = now - last_refill;

        if elapsed > 0 {
            let current_rate = if self.adaptive {
                self.adaptive_rate.load(Ordering::Acquire)
            } else {
                self.refill_rate
            };

            let tokens_to_add = (elapsed * current_rate) / 1000;
            if tokens_to_add > 0 {
                self.last_refill.store(now, Ordering::Release);

                let current = self.tokens.load(Ordering::Acquire);
                let new_tokens = (current + tokens_to_add).min(self.capacity);
                self.tokens.store(new_tokens, Ordering::Release);
            }
        }
    }

    /// Get current token count
    pub fn tokens(&self) -> u64 {
        self.tokens.load(Ordering::Acquire)
    }

    /// Get comprehensive metrics
    pub fn get_metrics(&self) -> RateLimiterMetrics {
        RateLimiterMetrics {
            current_tokens: self.tokens(),
            capacity: self.capacity,
            refill_rate: self.refill_rate,
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            rejected_requests: self.rejected_requests.load(Ordering::Relaxed),
            adaptive_rate: if self.adaptive {
                Some(self.adaptive_rate.load(Ordering::Relaxed))
            } else {
                None
            },
            is_adaptive: self.adaptive,
            uptime: self.start_time.elapsed(),
        }
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 100.0;
        }
        let successful = self.successful_requests.load(Ordering::Relaxed);
        (successful as f64 / total as f64) * 100.0
    }

    /// Get rejection rate as a percentage
    pub fn rejection_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let rejected = self.rejected_requests.load(Ordering::Relaxed);
        (rejected as f64 / total as f64) * 100.0
    }

    /// Adjust adaptive rate based on success/failure patterns
    pub fn adjust_rate(&self, success: bool) {
        if !self.adaptive {
            return;
        }

        let current_rate = self.adaptive_rate.load(Ordering::Acquire);
        let new_rate = if success {
            // Increase rate gradually on success
            (current_rate * 11) / 10 // 10% increase
        } else {
            // Decrease rate more aggressively on failure
            (current_rate * 9) / 10 // 10% decrease
        };

        let clamped_rate = new_rate.clamp(self.min_rate, self.max_rate);
        self.adaptive_rate.store(clamped_rate, Ordering::Release);

        // Record metrics
        if let Some(metrics) = &self.metrics {
            tokio::spawn({
                let metrics = metrics.clone();
                async move {
                    metrics
                        .record_gauge("rate_limiter.adaptive_rate", clamped_rate as f64)
                        .await;
                }
            });
        }
    }

    /// Reset adaptive rate to initial value
    pub fn reset_adaptive_rate(&self) {
        if self.adaptive {
            self.adaptive_rate
                .store(self.refill_rate, Ordering::Release);
        }
    }

    /// Set adaptive rate manually
    pub fn set_adaptive_rate(&self, rate: u64) {
        if self.adaptive {
            let clamped_rate = rate.clamp(self.min_rate, self.max_rate);
            self.adaptive_rate.store(clamped_rate, Ordering::Release);
        }
    }

    /// Check if rate limiter is healthy
    pub fn is_healthy(&self) -> bool {
        self.success_rate() > 80.0 && self.rejection_rate() < 20.0
    }

    /// Reset all counters
    pub fn reset(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.successful_requests.store(0, Ordering::Relaxed);
        self.rejected_requests.store(0, Ordering::Relaxed);
        self.tokens.store(self.capacity, Ordering::Relaxed);
        self.reset_adaptive_rate();
    }
}

/// Rate limiter error types
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Request size {requested} exceeds maximum allowed {max_allowed}")]
    RequestTooLarge { requested: u64, max_allowed: u64 },
    #[error("Rate limiter is disabled")]
    Disabled,
}
