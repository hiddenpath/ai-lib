use ai_lib::provider::openai::OpenAiAdapter;
use ai_lib::types::{ChatCompletionRequest, Message, Role};
use std::sync::Arc;

#[path = "utils/mock_transport.rs"]
mod mock_transport;
use mock_transport::MockTransport;
use ai_lib::api::ChatApi;

#[tokio::test]
async fn openai_adapter_parses_function_call_from_response() {
    // Simulated provider response with a function_call on the first choice
    let resp = serde_json::json!({
        "id": "resp1",
        "object": "chat.completion",
        "created": 0,
        "model": "gpt-test",
        "usage": {"prompt_tokens":0, "completion_tokens":0, "total_tokens":0},
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "",
                    "function_call": { "name": "get_current_weather", "arguments": { "location": "Boston" } }
                },
                "finish_reason": "function_call"
            }
        ]
    });

    let mock = Arc::new(MockTransport::new(resp));
    // OpenAiAdapter expects transport + api_key + base_url in with_transport_ref
    let adapter = OpenAiAdapter::with_transport_ref(mock.clone(), "test-key".to_string(), "https://api.openai.com/v1".to_string()).unwrap();

    let req = ChatCompletionRequest::new("gpt-test".to_string(), vec![Message { role: Role::User, content: ai_lib::types::common::Content::Text("Hello".to_string()), function_call: None }]);

    let res = adapter.chat_completion(req).await.unwrap();
    let fc = &res.choices[0].message.function_call;
    assert!(fc.is_some());
    let fc = fc.as_ref().unwrap();
    assert_eq!(fc.name, "get_current_weather");
    assert!(fc.arguments.is_some());
    assert_eq!(fc.arguments.as_ref().unwrap()["location"], "Boston");
}
