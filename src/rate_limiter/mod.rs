//! 速率限制和背压控制模块，防止AI API调用过载
//!
//! Rate limiting and backpressure control module for AI API calls.
//!
//! This module implements token bucket algorithms and backpressure mechanisms
//! to prevent overwhelming AI providers with too many concurrent requests.
//!
//! Key components:
//! - `TokenBucket`: Token bucket rate limiter implementation
//! - `BackpressureController`: Manages concurrent request limits
//! - `RateLimiterConfig`: Configuration for rate limiting parameters

pub mod backpressure;
pub mod config;
pub mod token_bucket;

pub use backpressure::{BackpressureController, BackpressurePermit};
pub use config::RateLimiterConfig;
pub use token_bucket::TokenBucket;
