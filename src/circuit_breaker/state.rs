//! Circuit breaker state management

use serde::{Deserialize, Serialize};
use std::fmt;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Normal state - requests are allowed through
    Closed,
    /// Circuit is open - all requests are rejected
    Open,
    /// Half-open state - limited requests allowed for testing
    HalfOpen,
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}
