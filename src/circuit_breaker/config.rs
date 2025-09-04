//! Circuit breaker configuration

use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Configuration for circuit breaker behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit
    pub failure_threshold: u32,
    /// Time to wait before attempting to close the circuit
    pub recovery_timeout: Duration,
    /// Number of successful requests needed to close the circuit from half-open state
    pub success_threshold: u32,
    /// Timeout for individual requests
    pub request_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
            request_timeout: Duration::from_secs(10),
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a production-ready configuration
    pub fn production() -> Self {
        Self {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 2,
            request_timeout: Duration::from_secs(15),
        }
    }

    /// Create a development configuration with more lenient settings
    pub fn development() -> Self {
        Self {
            failure_threshold: 10,
            recovery_timeout: Duration::from_secs(15),
            success_threshold: 5,
            request_timeout: Duration::from_secs(5),
        }
    }

    /// Create a conservative configuration for critical systems
    pub fn conservative() -> Self {
        Self {
            failure_threshold: 2,
            recovery_timeout: Duration::from_secs(120),
            success_threshold: 5,
            request_timeout: Duration::from_secs(30),
        }
    }
}
