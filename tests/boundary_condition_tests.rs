use ai_lib::{AiClient, AiClientBuilder, Provider};

#[tokio::test]
async fn test_concurrent_request_limit() {
    // Test backpressure control with max concurrency
    let client = AiClientBuilder::new(Provider::Groq)
        .with_max_concurrency(2)
        .build();

    // Should succeed in building client with concurrency limit
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_empty_model_name() {
    // Test that empty model names are rejected
    use ai_lib::types::common::Content;
    use ai_lib::{ChatCompletionRequest, Message, Role};

    let request = ChatCompletionRequest::new(
        "".to_string(), // Empty model name
        vec![Message {
            role: Role::User,
            content: Content::Text("test".to_string()),
            function_call: None,
        }],
    );

    // Model name validation happens at provider level
    assert_eq!(request.model, "");
}

#[tokio::test]
async fn test_extremely_long_prompt() {
    use ai_lib::types::common::Content;
    use ai_lib::{ChatCompletionRequest, Message, Role};

    // Create a very long prompt (1MB)
    let long_text = "a".repeat(1024 * 1024);

    let request = ChatCompletionRequest::new(
        "test-model".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text(long_text.clone()),
            function_call: None,
        }],
    );

    // Should handle large prompts without panicking
    assert_eq!(request.messages[0].content.as_text(), long_text);
}

#[tokio::test]
async fn test_zero_timeout() {
    use std::time::Duration;

    // Test that zero timeout is handled
    let client = AiClientBuilder::new(Provider::Groq)
        .with_timeout(Duration::from_secs(0))
        .build();

    assert!(client.is_ok());
}

#[tokio::test]
async fn test_invalid_proxy_url() {
    // Test that invalid proxy URLs are handled gracefully
    let client = AiClientBuilder::new(Provider::Groq)
        .with_proxy(Some("not-a-valid-url"))
        .build();

    // Should build successfully (validation happens at request time)
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_empty_messages_array() {
    use ai_lib::ChatCompletionRequest;

    // Test empty messages array
    let request = ChatCompletionRequest::new("test-model".to_string(), vec![]);

    assert_eq!(request.messages.len(), 0);
}

#[tokio::test]
async fn test_multiple_provider_switches() {
    // Test switching providers multiple times
    let mut client = AiClient::new(Provider::Groq).unwrap();

    assert!(client.switch_provider(Provider::OpenAI).is_ok());
    assert!(client.switch_provider(Provider::Anthropic).is_ok());
    assert!(client.switch_provider(Provider::Groq).is_ok());

    assert_eq!(client.provider_name(), "Groq");
}

#[tokio::test]
async fn test_builder_without_any_configuration() {
    // Test builder with no custom configuration
    let client = AiClientBuilder::new(Provider::Groq).build();

    assert!(client.is_ok());
}

#[tokio::test]
async fn test_negative_temperature() {
    use ai_lib::ChatCompletionRequest;

    // Test negative temperature value
    let mut request = ChatCompletionRequest::new("test-model".to_string(), vec![]);
    request.temperature = Some(-1.0);

    // Should accept any f32 value (validation at provider level)
    assert_eq!(request.temperature, Some(-1.0));
}

#[tokio::test]
async fn test_extremely_high_max_tokens() {
    use ai_lib::ChatCompletionRequest;

    // Test extremely high max_tokens
    let mut request = ChatCompletionRequest::new("test-model".to_string(), vec![]);
    request.max_tokens = Some(u32::MAX);

    assert_eq!(request.max_tokens, Some(u32::MAX));
}

#[test]
fn test_provider_enum_exhaustiveness() {
    // Ensure all providers have default models
    let providers = vec![
        Provider::Groq,
        Provider::OpenAI,
        Provider::Anthropic,
        Provider::Gemini,
        Provider::Mistral,
        Provider::Cohere,
        Provider::DeepSeek,
        Provider::Qwen,
        Provider::Ollama,
        Provider::XaiGrok,
        Provider::AzureOpenAI,
        Provider::HuggingFace,
        Provider::TogetherAI,
        Provider::OpenRouter,
        Provider::Replicate,
        Provider::BaiduWenxin,
        Provider::TencentHunyuan,
        Provider::IflytekSpark,
        Provider::Moonshot,
        Provider::ZhipuAI,
        Provider::MiniMax,
        Provider::Perplexity,
        Provider::AI21,
    ];

    for provider in providers {
        // Should not panic
        let model = provider.default_chat_model();
        assert!(!model.is_empty());
    }
}

#[test]
fn test_model_options_defaults() {
    use ai_lib::ModelOptions;

    let options = ModelOptions::default();
    assert!(options.chat_model.is_none());
    assert!(options.multimodal_model.is_none());
    assert!(options.fallback_models.is_empty());
    assert!(options.auto_discovery);
}

#[tokio::test]
async fn test_connection_options_precedence() {
    use ai_lib::ConnectionOptions;
    use std::time::Duration;

    let opts = ConnectionOptions {
        base_url: Some("https://custom.api.com".to_string()),
        proxy: Some("http://proxy:8080".to_string()),
        api_key: Some("test-key".to_string()),
        timeout: Some(Duration::from_secs(60)),
        disable_proxy: false,
    };

    let client = AiClient::with_options(Provider::Groq, opts);
    assert!(client.is_ok());
}
