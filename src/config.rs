use std::time::Duration;
use crate::circuit_breaker::CircuitBreakerConfig;
use crate::rate_limiter::RateLimiterConfig;
use crate::error_handling::ErrorThresholds;

/// Minimal explicit connection/configuration options.
///
/// Library users can pass an instance of this struct to `AiClient::with_options` to
/// explicitly control base URL, proxy, API key and timeout without relying exclusively
/// on environment variables. Any field left as `None` will fall back to existing
/// environment variable behavior or library defaults.
#[derive(Clone, Debug)]
pub struct ConnectionOptions {
    pub base_url: Option<String>,
    pub proxy: Option<String>,
    pub api_key: Option<String>,
    pub timeout: Option<Duration>,
    pub disable_proxy: bool,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            base_url: None,
            proxy: None,
            api_key: None,
            timeout: None,
            disable_proxy: false,
        }
    }
}

impl ConnectionOptions {
    /// Hydrate unset fields from environment variables (lightweight fallback logic).
    ///
    /// `provider_env_prefix` may be something like `OPENAI`, `GROQ`, etc., used to look up
    /// a provider specific API key prior to the generic fallback `AI_API_KEY`.
    pub fn hydrate_with_env(mut self, provider_env_prefix: &str) -> Self {
        // API key precedence: explicit > <PROVIDER>_API_KEY > AI_API_KEY
        if self.api_key.is_none() {
            let specific = format!("{}_API_KEY", provider_env_prefix);
            self.api_key = std::env::var(&specific)
                .ok()
                .or_else(|| std::env::var("AI_API_KEY").ok());
        }
        // Base URL precedence: explicit > AI_BASE_URL (generic) > leave None (caller/adapter handles default)
        if self.base_url.is_none() {
            if let Ok(v) = std::env::var("AI_BASE_URL") {
                self.base_url = Some(v);
            }
        }
        // Proxy precedence: explicit > AI_PROXY_URL
        if self.proxy.is_none() && !self.disable_proxy {
            self.proxy = std::env::var("AI_PROXY_URL").ok();
        }
        // Timeout precedence: explicit > AI_TIMEOUT_SECS > default handled by caller
        if self.timeout.is_none() {
            if let Ok(v) = std::env::var("AI_TIMEOUT_SECS") {
                if let Ok(secs) = v.parse::<u64>() {
                    self.timeout = Some(Duration::from_secs(secs));
                }
            }
        }
        self
    }
}

/// Resilience configuration for advanced error handling and rate limiting
#[derive(Debug, Clone)]
pub struct ResilienceConfig {
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    pub rate_limiter: Option<RateLimiterConfig>,
    pub backpressure: Option<BackpressureConfig>,
    pub error_handling: Option<ErrorHandlingConfig>,
}

/// Backpressure configuration
#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    pub max_concurrent_requests: usize,
}

/// Error handling configuration
#[derive(Debug, Clone)]
pub struct ErrorHandlingConfig {
    pub enable_recovery: bool,
    pub enable_monitoring: bool,
    pub error_thresholds: ErrorThresholds,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            circuit_breaker: None,
            rate_limiter: None,
            backpressure: None,
            error_handling: None,
        }
    }
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
        }
    }
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            enable_recovery: true,
            enable_monitoring: true,
            error_thresholds: ErrorThresholds::default(),
        }
    }
}

impl ResilienceConfig {
    /// Create smart defaults for production use
    pub fn smart_defaults() -> Self {
        Self {
            circuit_breaker: Some(CircuitBreakerConfig::default()),
            rate_limiter: Some(RateLimiterConfig::default()),
            backpressure: Some(BackpressureConfig::default()),
            error_handling: Some(ErrorHandlingConfig::default()),
        }
    }

    /// Create production-ready configuration
    pub fn production() -> Self {
        Self {
            circuit_breaker: Some(CircuitBreakerConfig::production()),
            rate_limiter: Some(RateLimiterConfig::production()),
            backpressure: Some(BackpressureConfig {
                max_concurrent_requests: 50,
            }),
            error_handling: Some(ErrorHandlingConfig {
                enable_recovery: true,
                enable_monitoring: true,
                error_thresholds: ErrorThresholds {
                    error_rate_threshold: 0.05, // 5% error rate
                    consecutive_errors: 3,
                    time_window: Duration::from_secs(30),
                },
            }),
        }
    }

    /// Create development configuration
    pub fn development() -> Self {
        Self {
            circuit_breaker: Some(CircuitBreakerConfig::development()),
            rate_limiter: Some(RateLimiterConfig::development()),
            backpressure: Some(BackpressureConfig {
                max_concurrent_requests: 200,
            }),
            error_handling: Some(ErrorHandlingConfig {
                enable_recovery: false,
                enable_monitoring: false,
                error_thresholds: ErrorThresholds::default(),
            }),
        }
    }
}
