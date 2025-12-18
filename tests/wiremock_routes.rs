use ai_lib::{
    api::ChatProvider,
    metrics::NoopMetrics,
    provider::{configs::ProviderConfigs, GenericAdapter},
    transport::{DynHttpTransport, DynHttpTransportRef},
    types::{
        common::{Content, Message},
        AiLibError, ChatCompletionRequest, Role,
    },
};
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn stubbed_response() -> serde_json::Value {
    serde_json::json!({
        "id": "wiremock-test",
        "object": "chat.completion",
        "created": 0,
        "model": "stub-model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "wiremock-ok"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 5,
            "completion_tokens": 7,
            "total_tokens": 12
        }
    })
}

#[derive(Clone)]
struct StubTransport {
    last_url: Arc<Mutex<Option<String>>>,
    recorded_body: Arc<Mutex<Option<serde_json::Value>>>,
    response: Arc<serde_json::Value>,
}

impl StubTransport {
    fn new(response: serde_json::Value) -> Self {
        Self {
            last_url: Arc::new(Mutex::new(None)),
            recorded_body: Arc::new(Mutex::new(None)),
            response: Arc::new(response),
        }
    }

    fn last_url(&self) -> Option<String> {
        self.last_url.lock().unwrap().clone()
    }
}

impl DynHttpTransport for StubTransport {
    fn get_json<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
    ) -> BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
        Box::pin(async move {
            Err(AiLibError::UnsupportedFeature(
                "get_json not implemented in StubTransport".to_string(),
            ))
        })
    }

    fn post_json<'a>(
        &'a self,
        url: &'a str,
        _headers: Option<HashMap<String, String>>,
        body: serde_json::Value,
    ) -> BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
        let url = url.to_string();
        let response = self.response.clone();
        let last_url = self.last_url.clone();
        let recorded_body = self.recorded_body.clone();
        Box::pin(async move {
            *last_url.lock().unwrap() = Some(url);
            *recorded_body.lock().unwrap() = Some(body);
            Ok((*response).clone())
        })
    }

    fn post_stream<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        _body: serde_json::Value,
    ) -> futures::future::BoxFuture<
        'a,
        Result<ai_lib::transport::dyn_transport::BytesStream, AiLibError>,
    > {
        Box::pin(async move {
            Err(AiLibError::UnsupportedFeature(
                "post_stream not implemented in StubTransport".to_string(),
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
    ) -> BoxFuture<'a, Result<serde_json::Value, AiLibError>> {
        Box::pin(async move {
            Err(AiLibError::UnsupportedFeature(
                "upload_multipart not implemented in StubTransport".to_string(),
            ))
        })
    }
}

#[tokio::test]
async fn generic_adapter_handles_stubbed_transport() -> Result<(), AiLibError> {
    std::env::set_var("GROQ_API_KEY", "test-key");
    let stub = Arc::new(StubTransport::new(stubbed_response()));
    let transport: DynHttpTransportRef = stub.clone();

    let mut config = ProviderConfigs::groq();
    config.base_url = "https://mock.api".to_string();
    config.chat_endpoint = "/v1/chat/completions".to_string();

    let adapter = GenericAdapter::with_transport_ref_and_metrics(
        config,
        transport,
        Arc::new(NoopMetrics::new()),
    )?;

    let request = ChatCompletionRequest::new(
        "stub-model".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("ping".to_string()),
            function_call: None,
        }],
    );

    let response = ChatProvider::chat(&adapter, request).await?;
    assert_eq!(response.first_text().unwrap(), "wiremock-ok");
    assert_eq!(
        stub.last_url().as_deref(),
        Some("https://mock.api/v1/chat/completions")
    );
    Ok(())
}
