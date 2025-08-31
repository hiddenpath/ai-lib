use crate::types::common::Content;
use base64::engine::general_purpose::STANDARD as base64_engine;
use base64::Engine as _;
use serde_json::json;
use std::path::Path;

/// Convert Content into a JSON value suitable for generic providers.
/// - If Content::Text -> string
/// - If Content::Json -> object
/// - If Content::Image with url -> {"image": {"url": ...}}
/// - If Content::Image without url -> inline base64 data
/// - If Content::Audio -> similar strategy
pub fn content_to_provider_value(content: &Content) -> serde_json::Value {
    match content {
        Content::Text(s) => serde_json::Value::String(s.clone()),
        Content::Json(v) => v.clone(),
        Content::Image {
            url,
            mime: _mime,
            name,
        } => {
            if let Some(u) = url {
                json!({"image": {"url": u}})
            } else {
                // No URL: attempt to treat name as path and inline base64 if readable
                if let Some(n) = name {
                    if let Some(data_url) = upload_file_inline(n, _mime.as_deref()) {
                        json!({"image": {"data": data_url}})
                    } else {
                        json!({"image": {"note": "no url, name not a readable path"}})
                    }
                } else {
                    json!({"image": {"note": "no url"}})
                }
            }
        }
        Content::Audio { url, mime: _mime } => {
            if let Some(u) = url {
                json!({"audio": {"url": u}})
            } else {
                json!({"audio": {"note": "no url"}})
            }
        }
    }
}
use crate::types::AiLibError;
use reqwest::Client;
use std::fs;
use std::time::Duration;

/// Upload or inline a local file path; current default behavior is to inline as data URL
/// Returns a data URL string if successful, or None on failure.
pub fn upload_file_inline(path: &str, mime: Option<&str>) -> Option<String> {
    let p = Path::new(path);
    if !p.exists() {
        return None;
    }
    if let Ok(bytes) = fs::read(p) {
        let b64 = base64_engine.encode(bytes);
        let mime = mime.unwrap_or("application/octet-stream");
        Some(format!("data:{};base64,{}", mime, b64))
    } else {
        None
    }
}

/// Upload a local file to the provider's upload endpoint using multipart/form-data.
/// This function is conservative: it only attempts an upload when an `upload_url` is provided.
/// Returns the provider-hosted file URL on success.
pub async fn upload_file_to_provider(
    upload_url: &str,
    path: &str,
    field_name: &str,
) -> Result<String, crate::types::AiLibError> {
    use reqwest::multipart;

    let p = Path::new(path);
    if !p.exists() {
        return Err(crate::types::AiLibError::ProviderError(
            "file not found".to_string(),
        ));
    }

    let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("file.bin");
    let bytes = fs::read(p)
        .map_err(|e| crate::types::AiLibError::ProviderError(format!("read error: {}", e)))?;

    let part = multipart::Part::bytes(bytes).file_name(file_name.to_string());
    let form = multipart::Form::new().part(field_name.to_string(), part);

    // Build client: respect AI_PROXY_URL if provided, otherwise disable proxy so
    // local mock servers (127.0.0.1) work even when system HTTP_PROXY is set.
    let client_builder = reqwest::Client::builder();
    let client_builder = if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            client_builder.proxy(proxy)
        } else {
            client_builder.no_proxy()
        }
    } else {
        client_builder.no_proxy()
    };

    let client = client_builder.build().map_err(|e| {
        crate::types::AiLibError::NetworkError(format!("failed to build http client: {}", e))
    })?;
    let resp = client
        .post(upload_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| crate::types::AiLibError::NetworkError(format!("upload failed: {}", e)))?;
    let status = resp.status();
    if !status.is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(crate::types::AiLibError::ProviderError(format!(
            "upload returned {}: {}",
            status, txt
        )));
    }

    // Naive: expect JSON response containing a 'url' field
    let j: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| crate::types::AiLibError::ProviderError(format!("parse response: {}", e)))?;
    parse_upload_response(j)
}

/// Upload a file using an injected transport if available.
/// If `transport` is Some, it will be used to post a multipart form. Otherwise,
/// falls back to `upload_file_to_provider` which constructs its own client.
pub async fn upload_file_with_transport(
    transport: Option<crate::transport::dyn_transport::DynHttpTransportRef>,
    upload_url: &str,
    path: &str,
    field_name: &str,
) -> Result<String, crate::types::AiLibError> {
    use std::fs;
    use std::path::Path;

    let p = Path::new(path);
    if !p.exists() {
        return Err(crate::types::AiLibError::ProviderError(
            "file not found".to_string(),
        ));
    }
    let file_name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file.bin")
        .to_string();
    let bytes = fs::read(p)
        .map_err(|e| crate::types::AiLibError::ProviderError(format!("read error: {}", e)))?;

    if let Some(t) = transport {
        // Use the injected transport to perform multipart upload
        let headers = None;
        let j = t
            .upload_multipart(upload_url, headers, field_name, &file_name, bytes)
            .await?;
        return parse_upload_response(j);
    }

    // Fallback to existing implementation
    upload_file_to_provider(upload_url, path, field_name).await
}

/// Parse a provider upload JSON response into a returned string.
/// Accepts either `{ "url": "..." }` or `{ "id": "..." }` and returns the url or id.
pub(crate) fn parse_upload_response(
    j: serde_json::Value,
) -> Result<String, crate::types::AiLibError> {
    if let Some(url) = j.get("url").and_then(|v| v.as_str()) {
        return Ok(url.to_string());
    }
    if let Some(id) = j.get("id").and_then(|v| v.as_str()) {
        return Ok(id.to_string());
    }
    Err(crate::types::AiLibError::ProviderError(
        "upload response missing url/id".to_string(),
    ))
}

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

    let client = client_builder
        .build()
        .map_err(|e| AiLibError::NetworkError(format!("Failed to build HTTP client: {}", e)))?;

    // Try models endpoint first
    let models_url = if base_url.ends_with('/') {
        format!("{}models", base_url)
    } else {
        format!("{}/models", base_url)
    };

    let resp = client.get(&models_url).send().await;
    match resp {
        Ok(r) if r.status().is_success() => Ok(()),
        _ => {
            // fallback to base url
            let resp2 = client.get(base_url).send().await;
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

// Tests moved to file end to satisfy clippy::items-after-test-module

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_upload_response_url() {
        let j = json!({"url": "https://cdn.example.com/file.png"});
        let res = parse_upload_response(j).unwrap();
        assert_eq!(res, "https://cdn.example.com/file.png");
    }

    #[test]
    fn parse_upload_response_id() {
        let j = json!({"id": "file_123"});
        let res = parse_upload_response(j).unwrap();
        assert_eq!(res, "file_123");
    }
}
