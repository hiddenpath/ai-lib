use async_trait::async_trait;

/// Simple, injectable metrics trait used by adapters/clients.
/// Keep the surface minimal: counters, gauges and a timer RAII helper.
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
    pub fn success(provider: &str) -> String { format!("{}.success", provider) }
    pub fn failure(provider: &str) -> String { format!("{}.failure", provider) }
}

/// Minimal cost accounting helper (feature-gated)
#[cfg(feature = "cost_metrics")]
pub mod cost {
    use crate::metrics::Metrics;

    /// Compute cost using env vars like COST_INPUT_PER_1K and COST_OUTPUT_PER_1K (USD)
    pub fn estimate_usd(input_tokens: u32, output_tokens: u32) -> f64 {
        let in_rate = std::env::var("COST_INPUT_PER_1K").ok().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        let out_rate = std::env::var("COST_OUTPUT_PER_1K").ok().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        (input_tokens as f64 / 1000.0) * in_rate + (output_tokens as f64 / 1000.0) * out_rate
    }

    /// Report cost via Metrics as histogram (usd) and counters by provider/model
    pub async fn record_cost<M: Metrics + ?Sized>(m: &M, provider: &str, model: &str, usd: f64) {
        m.record_histogram_with_tags("cost.usd", usd, &[("provider", provider), ("model", model)]).await;
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
