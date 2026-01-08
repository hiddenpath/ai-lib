use ai_lib::{AiClient, ChatCompletionRequest, Content, Message, Provider, Role};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_config_driven_openai_integration() {
    // 1. Start Mock Server
    std::env::set_var("NO_PROXY", "*");
    let mock_server = MockServer::start().await;

    // 2. Prepare Mock Response
    let response_body = serde_json::json!({
        "id": "cmpl-test-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "gpt-3.5-turbo",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello from ConfigDrivenAdapter!"
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    });

    // Mount mock
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    // SANITY CHECK: Call mock directly with reqwest
    let client_sanity = reqwest::Client::new();
    let sanity_url = format!("{}/chat/completions", mock_server.uri());
    eprintln!("DEBUG_SANITY: URL: {}", sanity_url);
    let sanity_res = client_sanity
        .post(&sanity_url)
        .json(&serde_json::json!({"model": "test"}))
        .send()
        .await
        .expect("Sanity request failed");
    eprintln!("DEBUG_SANITY: Status: {}", sanity_res.status());
    assert!(sanity_res.status().is_success(), "Sanity check failed");

    // 3. Configure Client to use Mock Server
    std::env::set_var("OPENAI_API_KEY", "sk-mock-key");

    let client = AiClient::builder(Provider::OpenAI)
        .with_base_url(&mock_server.uri()) // Override base_url to point to mock
        .build()
        .expect("Failed to build client");

    // 4. Send Request
    let request = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello".to_string()),
            function_call: None,
        }],
    );

    // FIXED: Use chat_completion inherent method
    let response = client
        .chat_completion(request)
        .await
        .expect("Chat request failed");

    // 5. Verify Response
    assert_eq!(response.choices.len(), 1);
    assert_eq!(
        response.choices[0].message.content,
        Content::Text("Hello from ConfigDrivenAdapter!".to_string())
    );
    assert_eq!(response.model, "default");
}

#[cfg(feature = "unified_sse")]
#[tokio::test]
async fn test_config_driven_openai_streaming() {
    // 1. Start Mock Server
    std::env::set_var("NO_PROXY", "*");
    let mock_server = MockServer::start().await;

    // 2. Prepare Mock Streaming Response
    let chunk1 = serde_json::json!({
        "id": "cmpl-stream-1",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-3.5-turbo",
        "choices": [{"index": 0, "delta": {"role": "assistant", "content": "Stream"}, "finish_reason": null}]
    });
    let chunk2 = serde_json::json!({
        "id": "cmpl-stream-1",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-3.5-turbo",
        "choices": [{"index": 0, "delta": {"content": "ing"}, "finish_reason": null}]
    });
    let chunk3 = serde_json::json!({
        "id": "cmpl-stream-1",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "gpt-3.5-turbo",
        "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}]
    });

    let body = format!(
        "data: {}\n\ndata: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        serde_json::to_string(&chunk1).unwrap(),
        serde_json::to_string(&chunk2).unwrap(),
        serde_json::to_string(&chunk3).unwrap()
    );

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&mock_server)
        .await;

    std::env::set_var("OPENAI_API_KEY", "sk-mock-key");

    let client = AiClient::builder(Provider::OpenAI)
        .with_base_url(&mock_server.uri())
        .build()
        .expect("Failed to build client");

    let request = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Stream me".to_string()),
            function_call: None,
        }],
    );

    use futures::StreamExt;
    // FIXED: Use chat_completion_stream inherent method
    let mut stream = client
        .chat_completion_stream(request)
        .await
        .expect("Stream request failed");

    let mut collected_content = String::new();
    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.expect("Chunk error");
        if let Some(delta) = chunk.choices.first().and_then(|c| c.delta.content.as_ref()) {
            collected_content.push_str(delta);
        }
    }

    assert_eq!(collected_content, "Streaming");
}
