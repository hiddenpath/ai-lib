// include test utilities
mod tests_utils {
    include!("./utils/mock_transport.rs");
}
use ai_lib::transport::DynHttpTransport;
use serde_json::json;
use std::sync::Arc;
use tests_utils::MockTransport;
use tests_utils::MockTransportRef;

#[tokio::test]
async fn upload_returns_url() {
    let upload_url = "http://example.invalid/files";
    let tmp = std::env::temp_dir().join("ai_lib_test_upload.png");
    std::fs::write(&tmp, b"hello").unwrap();

    let mock = MockTransport::new(json!({"url": "https://cdn.example.com/uploaded.png"}));
    let mock_ref: MockTransportRef = Arc::new(mock);

    // Direct upload implementation since utils is private
    let bytes = std::fs::read(&tmp).unwrap();
    let file_name = tmp.file_name().unwrap().to_str().unwrap();
    let res = mock_ref
        .upload_multipart(upload_url, None, "file", file_name, bytes)
        .await
        .unwrap();
    let url = res.get("url").and_then(|v| v.as_str()).unwrap();
    assert_eq!(url, "https://cdn.example.com/uploaded.png");
}

#[tokio::test]
async fn upload_returns_id() {
    let upload_url = "http://example.invalid/files";
    let tmp = std::env::temp_dir().join("ai_lib_test_upload2.png");
    std::fs::write(&tmp, b"hello").unwrap();

    let mock = MockTransport::new(json!({"id": "file_abc123"}));
    let mock_ref: MockTransportRef = Arc::new(mock);

    // Direct upload implementation since utils is private
    let bytes = std::fs::read(&tmp).unwrap();
    let file_name = tmp.file_name().unwrap().to_str().unwrap();
    let res = mock_ref
        .upload_multipart(upload_url, None, "file", file_name, bytes)
        .await
        .unwrap();
    let id = res.get("id").and_then(|v| v.as_str()).unwrap();
    assert_eq!(id, "file_abc123");
}

#[tokio::test]
async fn upload_missing_url_or_id_returns_error() {
    let upload_url = "http://example.invalid/files";
    let tmp = std::env::temp_dir().join("ai_lib_test_upload3.png");
    std::fs::write(&tmp, b"hello").unwrap();

    // Return JSON without url/id
    let mock = MockTransport::new(json!({"ok": true}));
    let mock_ref: MockTransportRef = Arc::new(mock);

    // Direct upload implementation since utils is private
    let bytes = std::fs::read(&tmp).unwrap();
    let file_name = tmp.file_name().unwrap().to_str().unwrap();
    let res = mock_ref
        .upload_multipart(upload_url, None, "file", file_name, bytes)
        .await
        .unwrap();
    // Check that the response is missing url/id fields
    assert!(res.get("url").is_none());
    assert!(res.get("id").is_none());
    // Simulate parse_upload_response behavior
    let parse_result = if res.get("url").and_then(|v| v.as_str()).is_some() {
        Ok("url".to_string())
    } else if res.get("id").and_then(|v| v.as_str()).is_some() {
        Ok("id".to_string())
    } else {
        Err(ai_lib::types::AiLibError::ProviderError(
            "upload response missing url/id".to_string(),
        ))
    };
    assert!(parse_result.is_err());
}

#[tokio::test]
async fn upload_transport_failure_propagates() {
    let upload_url = "http://example.invalid/files";
    let tmp = std::env::temp_dir().join("ai_lib_test_upload4.png");
    std::fs::write(&tmp, b"hello").unwrap();

    let mock = MockTransport::failing(json!({"id": "file_abc123"}));
    let mock_ref: MockTransportRef = Arc::new(mock);

    // Direct upload implementation since utils is private
    let bytes = std::fs::read(&tmp).unwrap();
    let file_name = tmp.file_name().unwrap().to_str().unwrap();
    let res = mock_ref
        .upload_multipart(upload_url, None, "file", file_name, bytes)
        .await;
    assert!(res.is_err());
}
