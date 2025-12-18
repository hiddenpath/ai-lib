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
) -> Result<String, AiLibError> {
    use reqwest::multipart;

    let p = Path::new(path);
    if !p.exists() {
        return Err(AiLibError::ProviderError("file not found".to_string()));
    }

    let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("file.bin");
    let bytes = fs::read(p).map_err(|e| AiLibError::ProviderError(format!("read error: {}", e)))?;

    let part = multipart::Part::bytes(bytes).file_name(file_name.to_string());
    let form = multipart::Form::new().part(field_name.to_string(), part);

    // Build HTTP client (feature-gated). Falls back to local builder if unified_transport is disabled.
    fn build_http_client() -> Result<Client, AiLibError> {
        #[cfg(feature = "unified_transport")]
        {
            crate::transport::client_factory::build_shared_client().map_err(|e| {
                AiLibError::NetworkError(format!("failed to build http client: {}", e))
            })
        }
        #[cfg(not(feature = "unified_transport"))]
        {
            let mut builder = reqwest::Client::builder();
            // respect AI_PROXY_URL if provided; otherwise no_proxy so local mocks work
            if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
                if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                    builder = builder.proxy(proxy);
                } else {
                    builder = builder.no_proxy();
                }
            } else {
                builder = builder.no_proxy();
            }
            builder.build().map_err(|e| {
                AiLibError::NetworkError(format!("failed to build http client: {}", e))
            })
        }
    }

    let client = build_http_client()?;
    let resp = client
        .post(upload_url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| AiLibError::NetworkError(format!("upload failed: {}", e)))?;
    let status = resp.status();
    if !status.is_success() {
        let txt = resp.text().await.unwrap_or_default();
        return Err(AiLibError::ProviderError(format!(
            "upload returned {}: {}",
            status, txt
        )));
    }

    // Naive: expect JSON response containing a 'url' field
    let j: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AiLibError::ProviderError(format!("parse response: {}", e)))?;
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
) -> Result<String, AiLibError> {
    use std::fs;
    use std::path::Path;

    let p = Path::new(path);
    if !p.exists() {
        return Err(AiLibError::ProviderError("file not found".to_string()));
    }
    let file_name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file.bin")
        .to_string();
    let bytes = fs::read(p).map_err(|e| AiLibError::ProviderError(format!("read error: {}", e)))?;

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
pub(crate) fn parse_upload_response(j: serde_json::Value) -> Result<String, AiLibError> {
    if let Some(url) = j.get("url").and_then(|v| v.as_str()) {
        return Ok(url.to_string());
    }
    if let Some(id) = j.get("id").and_then(|v| v.as_str()) {
        return Ok(id.to_string());
    }
    Err(AiLibError::ProviderError(
        "upload response missing url/id".to_string(),
    ))
}

// Tests moved to file end to satisfy clippy::items-after-test-module

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::BoxFuture;
    use serde_json::json;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use crate::transport::dyn_transport::{DynHttpTransport, StreamFuture};

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

    #[test]
    fn upload_file_inline_encodes_contents() {
        let path = temp_path("inline");
        fs::write(&path, b"hello-world").unwrap();
        let data = upload_file_inline(path.to_str().unwrap(), Some("text/plain")).unwrap();
        assert!(
            data.starts_with("data:text/plain;base64,"),
            "expected data URL, got {data}"
        );
        fs::remove_file(path).unwrap();
    }

    #[tokio::test]
    async fn upload_file_with_transport_invokes_injected_transport() {
        let path = temp_path("upload-with-transport");
        fs::write(&path, b"payload").unwrap();
        let transport = Arc::new(MockUploadTransport::default());

        let result = upload_file_with_transport(
            Some(transport.clone()),
            "https://mock.upload",
            path.to_str().unwrap(),
            "file",
        )
        .await
        .expect("upload should succeed");

        assert_eq!(result, "https://mock.upload/file");

        let recorded = transport
            .last_call
            .lock()
            .await
            .clone()
            .expect("call recorded");
        assert_eq!(recorded.url, "https://mock.upload");
        assert_eq!(recorded.field_name, "file");
        assert_eq!(
            recorded.file_name,
            path.file_name().unwrap().to_str().unwrap()
        );
        assert_eq!(recorded.bytes, b"payload");

        fs::remove_file(path).unwrap();
    }

    fn temp_path(prefix: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{unique}.bin"))
    }

    #[derive(Clone, Debug)]
    struct UploadInvocation {
        url: String,
        field_name: String,
        file_name: String,
        bytes: Vec<u8>,
    }

    #[derive(Clone, Default)]
    struct MockUploadTransport {
        last_call: Arc<Mutex<Option<UploadInvocation>>>,
    }

    impl DynHttpTransport for MockUploadTransport {
        fn get_json<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
        ) -> BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
            Box::pin(async {
                Err(AiLibError::UnsupportedFeature(
                    "get_json not implemented".to_string(),
                ))
            })
        }

        fn post_json<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
            _body: serde_json::Value,
        ) -> BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
            Box::pin(async {
                Err(AiLibError::UnsupportedFeature(
                    "post_json not implemented".to_string(),
                ))
            })
        }

        fn post_stream<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
            _body: serde_json::Value,
        ) -> StreamFuture<'a> {
            Box::pin(async {
                Err(AiLibError::UnsupportedFeature(
                    "post_stream not implemented".to_string(),
                ))
            })
        }

        fn upload_multipart<'a>(
            &'a self,
            url: &'a str,
            _headers: Option<HashMap<String, String>>,
            field_name: &'a str,
            file_name: &'a str,
            bytes: Vec<u8>,
        ) -> BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
            let last_call = self.last_call.clone();
            let url = url.to_string();
            let field_name = field_name.to_string();
            let file_name = file_name.to_string();
            Box::pin(async move {
                let mut lock = last_call.lock().await;
                *lock = Some(UploadInvocation {
                    url,
                    field_name,
                    file_name,
                    bytes,
                });
                Ok(json!({"url": "https://mock.upload/file"}))
            })
        }
    }
}
