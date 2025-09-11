//! Rate limiting and backpressure control for AI API calls
//!
//! This module provides rate limiting functionality to prevent overwhelming
//! AI providers and implement backpressure mechanisms.

pub mod backpressure;
pub mod config;
pub mod token_bucket;

pub use backpressure::{BackpressureController, BackpressurePermit};
pub use config::RateLimiterConfig;
pub use token_bucket::TokenBucket;
