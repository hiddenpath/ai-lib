use reqwest::{Client, Proxy};

/// Shared reqwest client builder to unify proxy/timeout/pool settings across adapters.
/// Keep conservative defaults; adapters can still add per-request headers.
pub fn build_shared_client() -> Result<Client, String> {
	let mut builder = Client::builder();

	// Timeout: default 30s; allow override via AI_HTTP_TIMEOUT_SECS
	let timeout = std::env::var("AI_HTTP_TIMEOUT_SECS")
		.ok()
		.and_then(|s| s.parse::<u64>().ok())
		.unwrap_or(30);
	builder = builder.timeout(std::time::Duration::from_secs(timeout));

	// Proxy: AI_PROXY_URL if set
	if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
		if let Ok(proxy) = Proxy::all(&proxy_url) {
			builder = builder.proxy(proxy);
		}
	}

	// Connection pool tuning (optional):
	if let Some(max_idle) = std::env::var("AI_HTTP_POOL_MAX_IDLE_PER_HOST").ok().and_then(|v| v.parse::<usize>().ok()) {
		builder = builder.pool_max_idle_per_host(max_idle);
	}
	if let Some(idle_ms) = std::env::var("AI_HTTP_POOL_IDLE_TIMEOUT_MS").ok().and_then(|v| v.parse::<u64>().ok()) {
		builder = builder.pool_idle_timeout(std::time::Duration::from_millis(idle_ms));
	}

	builder.build().map_err(|e| format!("Failed to build reqwest client: {}", e))
}


