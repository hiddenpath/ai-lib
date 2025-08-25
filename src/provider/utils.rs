use crate::types::AiLibError;
use reqwest::Client;
use std::time::Duration;

/// Simple health check helper for provider base URLs.
/// Tries to GET {base_url}/models (common OpenAI-compatible endpoint) or the base URL
/// and returns Ok(()) if reachable and returns an AiLibError otherwise.
pub async fn health_check(base_url: &str) -> Result<(), AiLibError> {
    let client_builder = Client::builder().timeout(Duration::from_secs(5));

    // honor AI_PROXY_URL if set
    let client_builder = if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            client_builder.proxy(proxy)
        } else {
            client_builder
        }
    } else {
        client_builder
    };

    let client = client_builder.build().map_err(|e| AiLibError::NetworkError(format!("Failed to build HTTP client: {}", e)))?;

    // Try models endpoint first
    let models_url = if base_url.ends_with('/') {
        format!("{}models", base_url)
    } else {
        format!("{}/models", base_url)
    };

    let resp = client.get(&models_url).send().await;
    match resp {
        Ok(r) if r.status().is_success() => return Ok(()),
        _ => {
            // fallback to base url
            let resp2 = client.get(base_url).send().await;
            match resp2 {
                Ok(r2) if r2.status().is_success() => Ok(()),
                Ok(r2) => Err(AiLibError::NetworkError(format!("Health check returned status {}", r2.status()))),
                Err(e) => Err(AiLibError::NetworkError(format!("Health check request failed: {}", e))),
            }
        }
    }
}
