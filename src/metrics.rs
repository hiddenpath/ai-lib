//! 指标收集模块，提供可插拔的性能监控和统计功能
//!
//! Metrics collection module providing pluggable performance monitoring and statistics.
//!
//! This module defines the `Metrics` trait for collecting performance data,
//! usage statistics, and error rates from AI provider interactions.
//!
//! Simple, injectable metrics trait used by adapters/clients.
//! Keep the surface minimal: counters, gauges and a timer RAII helper.

use async_trait::async_trait;
#[async_trait]
pub trait Metrics: Send + Sync + 'static {
    /// Increment a named counter by `value`.
    async fn incr_counter(&self, name: &str, value: u64);

    /// Record a gauge metric.
    async fn record_gauge(&self, name: &str, value: f64);

    /// Start a timer for a named operation. Returns a boxed Timer which should be stopped
    /// when the operation completes. Implementations may return None if timers aren't supported.
    async fn start_timer(&self, name: &str) -> Option<Box<dyn Timer + Send>>;

    /// Record a histogram value for a named metric.
    async fn record_histogram(&self, name: &str, value: f64);

    /// Record a histogram value with tags/labels.
    async fn record_histogram_with_tags(&self, name: &str, value: f64, tags: &[(&str, &str)]);

    /// Increment a counter with tags/labels.
    async fn incr_counter_with_tags(&self, name: &str, value: u64, tags: &[(&str, &str)]);

    /// Record a gauge with tags/labels.
    async fn record_gauge_with_tags(&self, name: &str, value: f64, tags: &[(&str, &str)]);

    /// Record an error occurrence.
    async fn record_error(&self, name: &str, error_type: &str);

    /// Record a success/failure boolean metric.
    async fn record_success(&self, name: &str, success: bool);
}

/// Timer interface returned by Metrics::start_timer.
pub trait Timer: Send {
    /// Stop the timer and record the duration.
    fn stop(self: Box<Self>);
}

/// No-op metrics implementation suitable as a default.
pub struct NoopMetrics;

#[async_trait]
impl Metrics for NoopMetrics {
    async fn incr_counter(&self, _name: &str, _value: u64) {}
    async fn record_gauge(&self, _name: &str, _value: f64) {}
    async fn start_timer(&self, _name: &str) -> Option<Box<dyn Timer + Send>> {
        None
    }
    async fn record_histogram(&self, _name: &str, _value: f64) {}
    async fn record_histogram_with_tags(&self, _name: &str, _value: f64, _tags: &[(&str, &str)]) {}
    async fn incr_counter_with_tags(&self, _name: &str, _value: u64, _tags: &[(&str, &str)]) {}
    async fn record_gauge_with_tags(&self, _name: &str, _value: f64, _tags: &[(&str, &str)]) {}
    async fn record_error(&self, _name: &str, _error_type: &str) {}
    async fn record_success(&self, _name: &str, _success: bool) {}
}

/// A no-op timer (returned when StartTimer implementations want to return a concrete value).
pub struct NoopTimer;
impl Timer for NoopTimer {
    fn stop(self: Box<Self>) {}
}

impl NoopMetrics {
    pub fn new() -> Self {
        NoopMetrics
    }
}

impl Default for NoopMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience methods for common metric patterns
#[allow(async_fn_in_trait)]
pub trait MetricsExt: Metrics {
    /// Record a request with timing and success/failure
    async fn record_request(
        &self,
        name: &str,
        timer: Option<Box<dyn Timer + Send>>,
        success: bool,
    ) {
        if let Some(t) = timer {
            t.stop();
        }
        self.record_success(name, success).await;
    }

    /// Record a request with timing, success/failure, and tags
    async fn record_request_with_tags(
        &self,
        name: &str,
        timer: Option<Box<dyn Timer + Send>>,
        success: bool,
        tags: &[(&str, &str)],
    ) {
        if let Some(t) = timer {
            t.stop();
        }
        self.record_success(name, success).await;
        // Record additional metrics with tags
        self.incr_counter_with_tags(&format!("{}.total", name), 1, tags)
            .await;
        if success {
            self.incr_counter_with_tags(&format!("{}.success", name), 1, tags)
                .await;
        } else {
            self.incr_counter_with_tags(&format!("{}.failure", name), 1, tags)
                .await;
        }
    }

    /// Record an error with context
    async fn record_error_with_context(&self, name: &str, error_type: &str, context: &str) {
        self.record_error(name, error_type).await;
        self.incr_counter_with_tags(name, 1, &[("error_type", error_type), ("context", context)])
            .await;
    }

    /// Record a complete request with timing, status, and success metrics
    async fn record_complete_request(
        &self,
        name: &str,
        timer: Option<Box<dyn Timer + Send>>,
        status_code: u16,
        success: bool,
        tags: &[(&str, &str)],
    ) {
        if let Some(t) = timer {
            t.stop();
        }

        // Record basic metrics
        self.record_success(name, success).await;
        self.record_gauge_with_tags(&format!("{}.status_code", name), status_code as f64, tags)
            .await;

        // Record counters
        self.incr_counter_with_tags(&format!("{}.total", name), 1, tags)
            .await;
        if success {
            self.incr_counter_with_tags(&format!("{}.success", name), 1, tags)
                .await;
        } else {
            self.incr_counter_with_tags(&format!("{}.failure", name), 1, tags)
                .await;
        }
    }

    /// Record latency percentiles for a batch of measurements
    async fn record_batch_latency_percentiles(
        &self,
        name: &str,
        measurements: &[f64],
        tags: &[(&str, &str)],
    ) {
        if measurements.is_empty() {
            return;
        }

        let mut sorted = measurements.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = sorted.len();
        let p50 = sorted[(len * 50 / 100).min(len - 1)];
        let p95 = sorted[(len * 95 / 100).min(len - 1)];
        let p99 = sorted[(len * 99 / 100).min(len - 1)];

        self.record_gauge_with_tags(&format!("{}.latency.p50", name), p50, tags)
            .await;
        self.record_gauge_with_tags(&format!("{}.latency.p95", name), p95, tags)
            .await;
        self.record_gauge_with_tags(&format!("{}.latency.p99", name), p99, tags)
            .await;
    }
}

impl<T: Metrics> MetricsExt for T {}

/// Centralized metric key helpers to standardize naming across adapters
pub mod keys {
    /// Request counter key
    pub fn requests(provider: &str) -> String {
        format!("{}.requests", provider)
    }
    /// Request duration timer key (ms)
    pub fn request_duration_ms(provider: &str) -> String {
        format!("{}.request_duration_ms", provider)
    }
    /// Success ratio gauge/histogram can be derived; optional explicit keys
    pub fn success(provider: &str) -> String {
        format!("{}.success", provider)
    }
    pub fn failure(provider: &str) -> String {
        format!("{}.failure", provider)
    }

    /// Latency percentile keys
    pub fn latency_p50(provider: &str) -> String {
        format!("{}.latency_p50", provider)
    }
    pub fn latency_p95(provider: &str) -> String {
        format!("{}.latency_p95", provider)
    }
    pub fn latency_p99(provider: &str) -> String {
        format!("{}.latency_p99", provider)
    }

    /// Status code distribution key
    pub fn status_codes(provider: &str) -> String {
        format!("{}.status_codes", provider)
    }

    /// Error rate key
    pub fn error_rate(provider: &str) -> String {
        format!("{}.error_rate", provider)
    }

    /// Throughput key (requests per second)
    pub fn throughput(provider: &str) -> String {
        format!("{}.throughput", provider)
    }

    /// Cost metrics keys
    pub fn cost_usd(provider: &str) -> String {
        format!("{}.cost_usd", provider)
    }
    pub fn cost_per_request(provider: &str) -> String {
        format!("{}.cost_per_request", provider)
    }
    pub fn tokens_input(provider: &str) -> String {
        format!("{}.tokens_input", provider)
    }
    pub fn tokens_output(provider: &str) -> String {
        format!("{}.tokens_output", provider)
    }

    /// Routing metrics keys
    pub fn routing_requests(route: &str) -> String {
        format!("routing.{}.requests", route)
    }
    pub fn routing_selected(route: &str) -> String {
        format!("routing.{}.selected", route)
    }
    pub fn routing_health_fail(route: &str) -> String {
        format!("routing.{}.health_fail", route)
    }
}

/// Minimal cost accounting helper (feature-gated)
#[cfg(feature = "cost_metrics")]
pub mod cost {
    use crate::metrics::Metrics;

    /// Compute cost using env vars like COST_INPUT_PER_1K and COST_OUTPUT_PER_1K (USD)
    pub fn estimate_usd(input_tokens: u32, output_tokens: u32) -> f64 {
        let in_rate = std::env::var("COST_INPUT_PER_1K")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        let out_rate = std::env::var("COST_OUTPUT_PER_1K")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        (input_tokens as f64 / 1000.0) * in_rate + (output_tokens as f64 / 1000.0) * out_rate
    }

    /// Report cost via Metrics as histogram (usd) and counters by provider/model
    pub async fn record_cost<M: Metrics + ?Sized>(m: &M, provider: &str, model: &str, usd: f64) {
        m.record_histogram_with_tags("cost.usd", usd, &[("provider", provider), ("model", model)])
            .await;
    }
}

// Environment variables for optional features
//
// cost_metrics (if enabled):
// - COST_INPUT_PER_1K: USD per 1000 input tokens
// - COST_OUTPUT_PER_1K: USD per 1000 output tokens
//
// Note: In enterprise deployments (ai-lib PRO), these can be centrally managed
// and hot-reloaded via external configuration providers.
