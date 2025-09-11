//! Circuit breaker implementation for resilient AI API calls
//!
//! This module provides circuit breaker functionality to prevent cascading failures
//! and improve system resilience when calling AI providers.

pub mod breaker;
pub mod config;
pub mod state;

pub use breaker::CircuitBreaker;
pub use config::CircuitBreakerConfig;
pub use state::CircuitState;
