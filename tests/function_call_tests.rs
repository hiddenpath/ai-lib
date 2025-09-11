use ai_lib::provider::config::ProviderConfig;
use ai_lib::provider::GenericAdapter;
use ai_lib::transport::DynHttpTransport;
use ai_lib::types::common::Content;
use ai_lib::types::function_call::FunctionCallPolicy;
use ai_lib::types::function_call::Tool;
use ai_lib::types::{ChatCompletionRequest, Message, Role};
use ai_lib::AiLibError;
use ai_lib::ChatApi;
use bytes::Bytes;
use futures::stream::{self};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

struct MockTransport {
    response: serde_json::Value,
}

impl MockTransport {
    fn new(response: serde_json::Value) -> Self {
        Self { response }
    }
}

impl DynHttpTransport for MockTransport {
    fn get_json<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<serde_json::Value, AiLibError>> + Send + 'a>,
    > {
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
    }

    fn post_json<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        _body: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<serde_json::Value, AiLibError>> + Send + 'a>,
    > {
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
    }

    fn post_stream<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        _body: serde_json::Value,
    ) -> std::pin::Pin<
        Box<
            dyn futures::Future<
                    Output = Result<
                        std::pin::Pin<
                            Box<dyn futures::Stream<Item = Result<Bytes, AiLibError>> + Send>,
                        >,
                        AiLibError,
                    >,
                > + Send
                + 'a,
        >,
    > {
        // For tests we don't use streaming; return an empty stream
        let stream: std::pin::Pin<
            Box<dyn futures::Stream<Item = Result<Bytes, AiLibError>> + Send>,
        > = Box::pin(stream::empty());
        Box::pin(async move { Ok(stream) })
    }

    fn upload_multipart<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        _field_name: &'a str,
        _file_name: &'a str,
        _bytes: Vec<u8>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<serde_json::Value, AiLibError>> + Send + 'a>,
    > {
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
    }
}

#[tokio::test]
async fn generic_adapter_parses_function_call_object_arguments() {
    // Build a provider response where function_call.arguments is an object
    let response = json!({
        "id": "1",
        "object": "chat.completion",
        "created": 0,
        "model": "test-model",
        "choices": [
            {
                "message": {
                    "role": "assistant",
                    "content": "",
                    "function_call": {
                        "name": "ascii_horse",
                        "arguments": { "size": 5 }
                    }
                },
                "finish_reason": null
            }
        ],
        "usage": { "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 }
    });

    let transport = Arc::new(MockTransport::new(response));
    let config =
        ProviderConfig::openai_compatible("http://example", "API_KEY", "gpt-3.5-turbo", None);
    let adapter = GenericAdapter::with_transport_ref(config, transport).expect("create adapter");

    let msg = Message {
        role: Role::User,
        content: Content::Text("Hello".to_string()),
        function_call: None,
    };
    let req = ChatCompletionRequest::new("test-model".to_string(), vec![msg]);

    let resp = adapter.chat_completion(req).await.expect("chat completion");
    let fc = &resp.choices[0].message.function_call;
    assert!(fc.is_some(), "function_call should be populated");
    let fc = fc.as_ref().unwrap();
    assert_eq!(fc.name, "ascii_horse");
    assert_eq!(
        fc.arguments
            .as_ref()
            .and_then(|v| v.get("size"))
            .and_then(|s| s.as_i64()),
        Some(5)
    );
}

#[tokio::test]
async fn generic_adapter_parses_function_call_stringified_arguments() {
    // Provider returns arguments as a JSON string; adapter should parse it
    let response = json!({
        "id": "2",
        "object": "chat.completion",
        "created": 0,
        "model": "test-model",
        "choices": [
            {
                "message": {
                    "role": "assistant",
                    "content": "",
                    "function_call": {
                        "name": "ascii_horse",
                        "arguments": "{\"size\":10}"
                    }
                },
                "finish_reason": null
            }
        ],
        "usage": { "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 }
    });

    let transport = Arc::new(MockTransport::new(response));
    let config =
        ProviderConfig::openai_compatible("http://example", "API_KEY", "gpt-3.5-turbo", None);
    let adapter = GenericAdapter::with_transport_ref(config, transport).expect("create adapter");

    let msg = Message {
        role: Role::User,
        content: Content::Text("Hello".to_string()),
        function_call: None,
    };
    let req = ChatCompletionRequest::new("test-model".to_string(), vec![msg]);

    let resp = adapter.chat_completion(req).await.expect("chat completion");
    let fc = &resp.choices[0].message.function_call;
    assert!(fc.is_some(), "function_call should be populated");
    let fc = fc.as_ref().unwrap();
    assert_eq!(fc.name, "ascii_horse");
    // arguments should be parsed into a JSON object with size=10
    assert_eq!(
        fc.arguments
            .as_ref()
            .and_then(|v| v.get("size"))
            .and_then(|s| s.as_i64()),
        Some(10)
    );
}

#[tokio::test]
async fn generic_adapter_parses_tool_calls_first_function() {
    // OpenAI tool_calls format; arguments provided as JSON string
    let response = json!({
        "id": "3",
        "object": "chat.completion",
        "created": 0,
        "model": "test-model",
        "choices": [
            {
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [
                        {"type":"function","function": {"name": "ascii_horse", "arguments": "{\"size\":7}"}}
                    ]
                },
                "finish_reason": null
            }
        ],
        "usage": {"prompt_tokens":0, "completion_tokens":0, "total_tokens":0}
    });

    let transport = Arc::new(MockTransport::new(response));
    let config =
        ProviderConfig::openai_compatible("http://example", "API_KEY", "gpt-3.5-turbo", None);
    let adapter = GenericAdapter::with_transport_ref(config, transport).expect("create adapter");

    let msg = Message { role: Role::User, content: Content::Text("Hello".to_string()), function_call: None };
    let req = ChatCompletionRequest::new("test-model".to_string(), vec![msg]);
    let resp = adapter.chat_completion(req).await.expect("chat completion");
    let fc = resp.choices[0].message.function_call.as_ref().expect("function_call present");
    assert_eq!(fc.name, "ascii_horse");
    assert_eq!(fc.arguments.as_ref().unwrap().get("size").and_then(|v| v.as_i64()), Some(7));
}

#[test]
fn chat_request_serializes_functions_field() {
    let tool = Tool {
        name: "ascii_horse".to_string(),
        description: Some("draw horse".to_string()),
        parameters: Some(json!({"type":"object"})),
    };
    let msg = Message {
        role: Role::User,
        content: Content::Text("Hello".to_string()),
        function_call: None,
    };
    let mut req = ChatCompletionRequest::new("m1".to_string(), vec![msg]);
    req.functions = Some(vec![tool]);
    req.function_call = Some(FunctionCallPolicy::Auto("auto".to_string()));

    let v = serde_json::to_value(&req).expect("serialize");
    assert!(v.get("functions").is_some(), "functions must be serialized");
    assert!(
        v.get("function_call").is_some(),
        "function_call must be serialized when set"
    );
}
