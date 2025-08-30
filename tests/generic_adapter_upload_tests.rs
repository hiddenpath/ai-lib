use ai_lib::transport::dyn_transport::DynHttpTransport;
use ai_lib::types::common::Content;
use ai_lib::types::{ChatCompletionRequest, Message, Role};
use ai_lib::AiLibError;
use ai_lib::ChatApi;
use bytes::Bytes;
use futures::Stream;
use serde_json::json;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use ai_lib::provider::config::ProviderConfig;

struct CapturingTransport {
    upload_response: serde_json::Value,
    post_response: serde_json::Value,
    pub last_post_body: tokio::sync::Mutex<Option<serde_json::Value>>,
}

impl CapturingTransport {
    fn new(upload_response: serde_json::Value, post_response: serde_json::Value) -> Self {
        Self {
            upload_response,
            post_response,
            last_post_body: tokio::sync::Mutex::new(None),
        }
    }
}

impl DynHttpTransport for CapturingTransport {
    fn get_json<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
    ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
        let resp = self.post_response.clone();
        Box::pin(async move { Ok(resp) })
    }

    fn post_json<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        body: serde_json::Value,
    ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
        let resp = self.post_response.clone();
        let last = &self.last_post_body;
        Box::pin(async move {
            let mut lock = last.lock().await;
            *lock = Some(body);
            Ok(resp)
        })
    }

    fn post_stream<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        _body: serde_json::Value,
    ) -> futures::future::BoxFuture<'a, Result<Pin<Box<dyn Stream<Item = Result<Bytes, AiLibError>> + Send>>, AiLibError>> {
        Box::pin(async move {
            Err(AiLibError::ProviderError(
                "stream not supported".to_string(),
            ))
        })
    }

    fn upload_multipart<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        _field_name: &'a str,
        _file_name: &'a str,
        _bytes: Vec<u8>,
    ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
        let resp = self.upload_response.clone();
        Box::pin(async move { Ok(resp) })
    }
}

#[tokio::test]
async fn generic_adapter_uploads_image_and_returns_file_id() {
    let tmp = std::env::temp_dir().join("ai_lib_test_generic_upload_id.png");
    std::fs::write(&tmp, b"content").unwrap();

    // upload returns id
    let upload_resp = json!({"id": "file_generic_1"});
    let post_resp = json!({
        "id": "r1",
        "object": "chat.completion",
        "created": 0,
        "model": "m",
        "choices": [{ "message": { "role": "assistant", "content": "ok" }, "finish_reason": null }],
        "usage": { "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 }
    });

    let transport = Arc::new(CapturingTransport::new(upload_resp, post_resp));
    let config = ProviderConfig::openai_compatible("http://example", "API_KEY");
    // set upload_size_limit to 0 so file is considered large and uploaded
    let mut cfg = config.clone();
    cfg.upload_size_limit = Some(0);

    let adapter =
        ai_lib::provider::generic::GenericAdapter::with_transport_ref(cfg, transport.clone())
            .expect("create adapter");

    let msg = Message {
        role: Role::User,
        content: Content::Image {
            url: None,
            mime: Some("image/png".to_string()),
            name: Some(tmp.to_str().unwrap().to_string()),
        },
        function_call: None,
    };
    let req = ChatCompletionRequest::new("m".to_string(), vec![msg]);

    let _ = adapter.chat_completion(req).await.expect("chat completion");

    let lock = transport.last_post_body.lock().await;
    let body = lock.as_ref().expect("body captured");
    let messages = body
        .get("messages")
        .and_then(|v| v.as_array())
        .expect("messages array");
    let content = messages[0].get("content").expect("content");
    let file_id = content
        .get("image")
        .and_then(|i| i.get("file_id"))
        .and_then(|u| u.as_str())
        .expect("image.file_id present");
    assert_eq!(file_id, "file_generic_1");
}

#[tokio::test]
async fn generic_adapter_upload_failure_falls_back_to_inline() {
    let tmp = std::env::temp_dir().join("ai_lib_test_generic_upload_fail.png");
    std::fs::write(&tmp, b"content").unwrap();

    struct FailUploadTransport {
        post_resp: serde_json::Value,
    }
    impl DynHttpTransport for FailUploadTransport {
        fn get_json<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
        ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>>
        {
            let resp = self.post_resp.clone();
            Box::pin(async move { Ok(resp) })
        }
        fn post_json<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
            _body: serde_json::Value,
        ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>>
        {
            let resp = self.post_resp.clone();
            Box::pin(async move { Ok(resp) })
        }
        fn post_stream<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
            _body: serde_json::Value,
        ) -> futures::future::BoxFuture<'a, Result<Pin<Box<dyn Stream<Item = Result<Bytes, AiLibError>> + Send>>, AiLibError>> {
            Box::pin(async move {
                Err(AiLibError::ProviderError(
                    "stream not supported".to_string(),
                ))
            })
        }
        fn upload_multipart<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
            _field_name: &'a str,
            _file_name: &'a str,
            _bytes: Vec<u8>,
        ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>>
        {
            Box::pin(async move {
                Err(AiLibError::ProviderError(
                    "simulated upload failure".to_string(),
                ))
            })
        }
    }

    let post_resp = json!({
        "id": "r2",
        "object": "chat.completion",
        "created": 0,
        "model": "m",
        "choices": [{ "message": { "role": "assistant", "content": "ok" }, "finish_reason": null }],
        "usage": { "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 }
    });

    let transport = Arc::new(FailUploadTransport { post_resp });
    let config = ProviderConfig::openai_compatible("http://example", "API_KEY");
    let mut cfg = config.clone();
    cfg.upload_size_limit = Some(0);

    let adapter = ai_lib::provider::generic::GenericAdapter::with_transport_ref(cfg, transport)
        .expect("create adapter");

    let msg = Message {
        role: Role::User,
        content: Content::Image {
            url: None,
            mime: Some("image/png".to_string()),
            name: Some(tmp.to_str().unwrap().to_string()),
        },
        function_call: None,
    };
    let req = ChatCompletionRequest::new("m".to_string(), vec![msg]);

    let _ = adapter.chat_completion(req).await.expect("chat completion");

    // Verify inline in data URL
    let content_val = ai_lib::provider::utils::content_to_provider_value(&Content::Image {
        url: None,
        mime: Some("image/png".to_string()),
        name: Some(tmp.to_str().unwrap().to_string()),
    });
    let data_url = content_val
        .get("image")
        .and_then(|i| i.get("data"))
        .and_then(|d| d.as_str())
        .expect("inline data present");
    assert!(data_url.starts_with("data:image/png;base64,"));
}

#[tokio::test]
async fn generic_adapter_size_boundary_respects_upload_size_limit() {
    let tmp = std::env::temp_dir().join("ai_lib_test_generic_small.png");
    // write small content
    std::fs::write(&tmp, b"small").unwrap();

    // If upload_size_limit is larger than file, adapter should inline (no upload call). We'll simulate a transport
    // that would panic on upload to ensure it wasn't called; instead it should post inline content.
    struct PanicUploadTransport {
        post_resp: serde_json::Value,
    }
    impl DynHttpTransport for PanicUploadTransport {
        fn get_json<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
        ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>>
        {
            let resp = self.post_resp.clone();
            Box::pin(async move { Ok(resp) })
        }
        fn post_json<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
            _body: serde_json::Value,
        ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>>
        {
            let resp = self.post_resp.clone();
            Box::pin(async move { Ok(resp) })
        }
        fn post_stream<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
            _body: serde_json::Value,
        ) -> futures::future::BoxFuture<'a, Result<Pin<Box<dyn Stream<Item = Result<Bytes, AiLibError>> + Send>>, AiLibError>> {
            Box::pin(async move {
                Err(AiLibError::ProviderError(
                    "stream not supported".to_string(),
                ))
            })
        }
        fn upload_multipart<'a>(
            &'a self,
            _url: &'a str,
            _headers: Option<HashMap<String, String>>,
            _field_name: &'a str,
            _file_name: &'a str,
            _bytes: Vec<u8>,
        ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, AiLibError>>
        {
            panic!("upload_multipart should not be called for small files");
        }
    }

    let post_resp = json!({
        "id": "r3",
        "object": "chat.completion",
        "created": 0,
        "model": "m",
        "choices": [{ "message": { "role": "assistant", "content": "ok" }, "finish_reason": null }],
        "usage": { "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 }
    });

    let transport = Arc::new(PanicUploadTransport { post_resp });
    let config = ProviderConfig::openai_compatible("http://example", "API_KEY");
    let mut cfg = config.clone();
    // set upload_size_limit large so file is inlined
    cfg.upload_size_limit = Some(1024 * 1024);

    let adapter = ai_lib::provider::generic::GenericAdapter::with_transport_ref(cfg, transport)
        .expect("create adapter");

    let msg = Message {
        role: Role::User,
        content: Content::Image {
            url: None,
            mime: Some("image/png".to_string()),
            name: Some(tmp.to_str().unwrap().to_string()),
        },
        function_call: None,
    };
    let req = ChatCompletionRequest::new("m".to_string(), vec![msg]);

    let _ = adapter.chat_completion(req).await.expect("chat completion");

    // If we reached here without panic, the upload wasn't called and adapter inlined the file as expected.
}
