//! Circuit breaker implementation for resilient AI API calls
//! 
//! This module provides circuit breaker functionality to prevent cascading failures
//! and improve system resilience when calling AI providers.

pub mod state;
pub mod config;
pub mod breaker;

pub use state::CircuitState;
pub use config::CircuitBreakerConfig;
pub use breaker::CircuitBreaker;
