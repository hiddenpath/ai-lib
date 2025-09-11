//! Circuit breaker implementation

use crate::circuit_breaker::{CircuitBreakerConfig, CircuitState};
use crate::metrics::Metrics;
use crate::types::AiLibError;
use futures::Future;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Circuit breaker error types
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open: {0}")]
    CircuitOpen(String),
    #[error("Request timeout: {0}")]
    RequestTimeout(String),
    #[error("Underlying error: {0}")]
    Underlying(#[from] AiLibError),
    #[error("Circuit breaker is disabled")]
    Disabled,
}

/// Circuit breaker metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerMetrics {
    pub state: CircuitState,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub timeout_requests: u64,
    pub circuit_open_count: u64,
    pub circuit_close_count: u64,
    pub current_failure_count: u32,
    pub current_success_count: u32,
    #[serde(skip)]
    pub last_failure_time: Option<Instant>,
    #[serde(skip)]
    pub uptime: Duration,
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    config: CircuitBreakerConfig,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    // Metrics
    total_requests: Arc<AtomicU64>,
    successful_requests: Arc<AtomicU64>,
    failed_requests: Arc<AtomicU64>,
    timeout_requests: Arc<AtomicU64>,
    circuit_open_count: Arc<AtomicU64>,
    circuit_close_count: Arc<AtomicU64>,
    start_time: Instant,
    // Optional metrics collector
    metrics: Option<Arc<dyn Metrics>>,
    // Circuit breaker enabled flag
    enabled: bool,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            config,
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            timeout_requests: Arc::new(AtomicU64::new(0)),
            circuit_open_count: Arc::new(AtomicU64::new(0)),
            circuit_close_count: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            metrics: None,
            enabled: true,
        }
    }

    /// Create a new circuit breaker with metrics collection
    pub fn with_metrics(config: CircuitBreakerConfig, metrics: Arc<dyn Metrics>) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            config,
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            timeout_requests: Arc::new(AtomicU64::new(0)),
            circuit_open_count: Arc::new(AtomicU64::new(0)),
            circuit_close_count: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            metrics: Some(metrics),
            enabled: true,
        }
    }

    /// Enable or disable the circuit breaker
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Execute a function with circuit breaker protection
    pub async fn call<F, T>(&self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: Future<Output = Result<T, AiLibError>>,
    {
        // Check if circuit breaker is enabled
        if !self.enabled {
            return f.await.map_err(CircuitBreakerError::Underlying);
        }

        // Increment total requests counter
        self.total_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Check if we should allow the request
        if !self.should_allow_request().await {
            return Err(CircuitBreakerError::CircuitOpen(
                "Circuit breaker is open".to_string(),
            ));
        }

        // Execute the request with timeout
        let result = timeout(self.config.request_timeout, f).await;

        match result {
            Ok(Ok(response)) => {
                self.on_success().await;
                Ok(response)
            }
            Ok(Err(error)) => {
                self.on_failure().await;
                Err(CircuitBreakerError::Underlying(error))
            }
            Err(_) => {
                self.on_timeout().await;
                Err(CircuitBreakerError::RequestTimeout(
                    "Request timed out".to_string(),
                ))
            }
        }
    }

    /// Check if the circuit breaker should allow a request
    async fn should_allow_request(&self) -> bool {
        let state = *self.state.lock().unwrap();

        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if enough time has passed to try half-open
                let allow_half_open = {
                    let last = self.last_failure_time.lock().unwrap();
                    last.and_then(|t| Some(t.elapsed() >= self.config.recovery_timeout))
                        .unwrap_or(false)
                };
                if allow_half_open {
                    self.transition_to_half_open().await;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Handle successful request
    async fn on_success(&self) {
        self.successful_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut record_closed_metric = false;
        {
            let mut state = self.state.lock().unwrap();
            match *state {
                CircuitState::Closed => {
                    // Reset failure count on success
                    self.failure_count
                        .store(0, std::sync::atomic::Ordering::Relaxed);
                }
                CircuitState::HalfOpen => {
                    let success_count = self
                        .success_count
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                        + 1;
                    if success_count >= self.config.success_threshold {
                        *state = CircuitState::Closed;
                        self.success_count
                            .store(0, std::sync::atomic::Ordering::Relaxed);
                        self.circuit_close_count
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        record_closed_metric = true;
                    }
                }
                CircuitState::Open => {
                    // This shouldn't happen, but handle gracefully
                }
            }
        }
        if record_closed_metric {
            if let Some(metrics) = &self.metrics {
                metrics.incr_counter("circuit_breaker.closed", 1).await;
            }
        }
    }

    /// Handle failed request
    async fn on_failure(&self) {
        self.failed_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let failure_count = self
            .failure_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;

        // Record failure time
        *self.last_failure_time.lock().unwrap() = Some(Instant::now());

        // Check if we should open the circuit
        if failure_count >= self.config.failure_threshold {
            {
                let mut state = self.state.lock().unwrap();
                *state = CircuitState::Open;
            }
            self.circuit_open_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            // Record metrics
            if let Some(metrics) = &self.metrics {
                let m = metrics.clone();
                m.incr_counter("circuit_breaker.opened", 1).await;
            }
        }
    }

    /// Handle timeout request
    async fn on_timeout(&self) {
        self.timeout_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.failed_requests
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let failure_count = self
            .failure_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;

        // Record failure time
        *self.last_failure_time.lock().unwrap() = Some(Instant::now());

        // Check if we should open the circuit
        if failure_count >= self.config.failure_threshold {
            {
                let mut state = self.state.lock().unwrap();
                *state = CircuitState::Open;
            }
            self.circuit_open_count
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            // Record metrics
            if let Some(metrics) = &self.metrics {
                let m = metrics.clone();
                m.incr_counter("circuit_breaker.opened", 1).await;
            }
        }
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::HalfOpen;
        self.success_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }

    /// Get current failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get current success count
    pub fn success_count(&self) -> u32 {
        self.success_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get comprehensive metrics
    pub fn get_metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            state: self.state(),
            total_requests: self
                .total_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            successful_requests: self
                .successful_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            failed_requests: self
                .failed_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            timeout_requests: self
                .timeout_requests
                .load(std::sync::atomic::Ordering::Relaxed),
            circuit_open_count: self
                .circuit_open_count
                .load(std::sync::atomic::Ordering::Relaxed),
            circuit_close_count: self
                .circuit_close_count
                .load(std::sync::atomic::Ordering::Relaxed),
            current_failure_count: self.failure_count(),
            current_success_count: self.success_count(),
            last_failure_time: *self.last_failure_time.lock().unwrap(),
            uptime: self.start_time.elapsed(),
        }
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total = self
            .total_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 {
            return 100.0;
        }
        let successful = self
            .successful_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        (successful as f64 / total as f64) * 100.0
    }

    /// Get failure rate as a percentage
    pub fn failure_rate(&self) -> f64 {
        let total = self
            .total_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let failed = self
            .failed_requests
            .load(std::sync::atomic::Ordering::Relaxed);
        (failed as f64 / total as f64) * 100.0
    }

    /// Check if circuit breaker is healthy
    pub fn is_healthy(&self) -> bool {
        self.state() == CircuitState::Closed && self.failure_rate() < 50.0
    }

    /// Reset all counters and state
    pub fn reset(&self) {
        self.failure_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.success_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.total_requests
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.successful_requests
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.failed_requests
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.timeout_requests
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.circuit_open_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.circuit_close_count
            .store(0, std::sync::atomic::Ordering::Relaxed);

        let mut state = self.state.lock().unwrap();
        *state = CircuitState::Closed;

        let mut last_failure = self.last_failure_time.lock().unwrap();
        *last_failure = None;
    }

    /// Force circuit breaker to open state
    pub fn force_open(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::Open;
        self.circuit_open_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Force circuit breaker to closed state
    pub fn force_close(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::Closed;
        self.failure_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.success_count
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.circuit_close_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
