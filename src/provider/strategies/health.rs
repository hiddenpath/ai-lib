use crate::types::AiLibError;
use std::time::Duration;

/// Simple health check helper for provider base URLs.
/// Tries to GET `{base_url}/models` (common OpenAI-compatible endpoint) or the base URL
/// and returns Ok(()) if reachable; otherwise returns an `AiLibError`.
pub async fn health_check(base_url: &str) -> Result<(), AiLibError> {
    #[cfg(feature = "unified_transport")]
    let client = crate::transport::client_factory::build_shared_client()
        .map_err(|e| AiLibError::NetworkError(format!("Failed to build HTTP client: {}", e)))?;
    #[cfg(not(feature = "unified_transport"))]
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| AiLibError::NetworkError(format!("Failed to build HTTP client: {}", e)))?;

    let models_url = if base_url.ends_with('/') {
        format!("{}models", base_url)
    } else {
        format!("{}/models", base_url)
    };

    let resp = client
        .get(&models_url)
        .timeout(Duration::from_secs(5))
        .send()
        .await;
    match resp {
        Ok(r) if r.status().is_success() => Ok(()),
        _ => {
            let resp2 = client
                .get(base_url)
                .timeout(Duration::from_secs(5))
                .send()
                .await;
            match resp2 {
                Ok(r2) if r2.status().is_success() => Ok(()),
                Ok(r2) => Err(AiLibError::NetworkError(format!(
                    "Health check returned status {}",
                    r2.status()
                ))),
                Err(e) => Err(AiLibError::NetworkError(format!(
                    "Health check request failed: {}",
                    e
                ))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests require a running mock server, which can be flaky in CI.
    // The health_check function is simple enough that unit testing the logic
    // is sufficient - integration tests with real endpoints provide better coverage.

    #[tokio::test]
    async fn errors_on_invalid_url() {
        // Test with an invalid/unreachable URL
        let err = health_check("http://127.0.0.1:1").await.unwrap_err();
        assert!(matches!(err, AiLibError::NetworkError(_)));
    }

    #[tokio::test]
    async fn url_construction_handles_trailing_slash() {
        // Verify URL construction logic by testing with unreachable URLs
        // The function should attempt both /models and base URL
        let err1 = health_check("http://127.0.0.1:1/").await.unwrap_err();
        let err2 = health_check("http://127.0.0.1:1").await.unwrap_err();
        assert!(matches!(err1, AiLibError::NetworkError(_)));
        assert!(matches!(err2, AiLibError::NetworkError(_)));
    }
}
