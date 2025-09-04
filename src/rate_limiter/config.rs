//! Rate limiter configuration

use serde::{Deserialize, Serialize};

/// Configuration for rate limiting behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    /// Maximum requests per second
    pub requests_per_second: u64,
    /// Burst capacity for handling traffic spikes
    pub burst_capacity: u64,
    /// Whether to use adaptive rate limiting
    pub adaptive: bool,
    /// Initial rate for adaptive mode
    pub initial_rate: Option<u64>,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_capacity: 20,
            adaptive: false,
            initial_rate: None,
        }
    }
}

impl RateLimiterConfig {
    /// Create a production-ready configuration
    pub fn production() -> Self {
        Self {
            requests_per_second: 5,
            burst_capacity: 10,
            adaptive: true,
            initial_rate: Some(5),
        }
    }

    /// Create a development configuration with higher limits
    pub fn development() -> Self {
        Self {
            requests_per_second: 20,
            burst_capacity: 50,
            adaptive: false,
            initial_rate: None,
        }
    }

    /// Create a conservative configuration for rate-limited providers
    pub fn conservative() -> Self {
        Self {
            requests_per_second: 2,
            burst_capacity: 5,
            adaptive: true,
            initial_rate: Some(1),
        }
    }
}
