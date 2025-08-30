use ai_lib::provider::utils;
// include test utilities
mod tests_utils {
    include!("./utils/mock_transport.rs");
}
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

    let res = utils::upload_file_with_transport(
        Some(mock_ref),
        upload_url,
        tmp.to_str().unwrap(),
        "file",
    )
    .await
    .unwrap();
    assert_eq!(res, "https://cdn.example.com/uploaded.png");
}

#[tokio::test]
async fn upload_returns_id() {
    let upload_url = "http://example.invalid/files";
    let tmp = std::env::temp_dir().join("ai_lib_test_upload2.png");
    std::fs::write(&tmp, b"hello").unwrap();

    let mock = MockTransport::new(json!({"id": "file_abc123"}));
    let mock_ref: MockTransportRef = Arc::new(mock);

    let res = utils::upload_file_with_transport(
        Some(mock_ref),
        upload_url,
        tmp.to_str().unwrap(),
        "file",
    )
    .await
    .unwrap();
    assert_eq!(res, "file_abc123");
}

#[tokio::test]
async fn upload_missing_url_or_id_returns_error() {
    let upload_url = "http://example.invalid/files";
    let tmp = std::env::temp_dir().join("ai_lib_test_upload3.png");
    std::fs::write(&tmp, b"hello").unwrap();

    // Return JSON without url/id
    let mock = MockTransport::new(json!({"ok": true}));
    let mock_ref: MockTransportRef = Arc::new(mock);

    let res = utils::upload_file_with_transport(
        Some(mock_ref),
        upload_url,
        tmp.to_str().unwrap(),
        "file",
    )
    .await;
    assert!(res.is_err());
}

#[tokio::test]
async fn upload_transport_failure_propagates() {
    let upload_url = "http://example.invalid/files";
    let tmp = std::env::temp_dir().join("ai_lib_test_upload4.png");
    std::fs::write(&tmp, b"hello").unwrap();

    let mock = MockTransport::failing(json!({"id": "file_abc123"}));
    let mock_ref: MockTransportRef = Arc::new(mock);

    let res = utils::upload_file_with_transport(
        Some(mock_ref),
        upload_url,
        tmp.to_str().unwrap(),
        "file",
    )
    .await;
    assert!(res.is_err());
}
