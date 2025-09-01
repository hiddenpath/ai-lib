use std::time::Duration;

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
