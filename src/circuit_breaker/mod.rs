//! 熔断器模块，提供AI API调用的弹性保护机制
//!
//! Circuit breaker module providing resilient protection for AI API calls.
//!
//! This module implements circuit breaker patterns to prevent cascading failures
//! and improve system resilience when calling AI providers.
//!
//! The circuit breaker monitors failure rates and temporarily stops making requests
//! to failing providers, allowing them time to recover.

pub mod breaker;
pub mod config;
pub mod state;

pub use breaker::CircuitBreaker;
pub use config::CircuitBreakerConfig;
pub use state::CircuitState;
