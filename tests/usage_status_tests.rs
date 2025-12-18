use ai_lib::{ChatCompletionResponse, Choice, Content, Message, Role, Usage, UsageStatus};

#[tokio::test]
async fn test_usage_status_enum() {
    // Test that UsageStatus enum works correctly
    let status = UsageStatus::Finalized;
    assert_eq!(status, UsageStatus::Finalized);

    let status = UsageStatus::Estimated;
    assert_eq!(status, UsageStatus::Estimated);

    let status = UsageStatus::Pending;
    assert_eq!(status, UsageStatus::Pending);

    let status = UsageStatus::Unsupported;
    assert_eq!(status, UsageStatus::Unsupported);
}

#[tokio::test]
async fn test_usage_status_serialization() {
    use serde_json;

    // Test serialization
    let status = UsageStatus::Finalized;
    let serialized = serde_json::to_string(&status).unwrap();
    assert_eq!(serialized, "\"finalized\"");

    let status = UsageStatus::Estimated;
    let serialized = serde_json::to_string(&status).unwrap();
    assert_eq!(serialized, "\"estimated\"");

    let status = UsageStatus::Pending;
    let serialized = serde_json::to_string(&status).unwrap();
    assert_eq!(serialized, "\"pending\"");

    let status = UsageStatus::Unsupported;
    let serialized = serde_json::to_string(&status).unwrap();
    assert_eq!(serialized, "\"unsupported\"");
}

#[tokio::test]
async fn test_usage_status_deserialization() {
    use serde_json;

    // Test deserialization
    let status: UsageStatus = serde_json::from_str("\"finalized\"").unwrap();
    assert_eq!(status, UsageStatus::Finalized);

    let status: UsageStatus = serde_json::from_str("\"estimated\"").unwrap();
    assert_eq!(status, UsageStatus::Estimated);

    let status: UsageStatus = serde_json::from_str("\"pending\"").unwrap();
    assert_eq!(status, UsageStatus::Pending);

    let status: UsageStatus = serde_json::from_str("\"unsupported\"").unwrap();
    assert_eq!(status, UsageStatus::Unsupported);
}

#[tokio::test]
async fn test_chat_completion_response_with_usage_status() {
    // Test that ChatCompletionResponse includes usage_status field
    let response = ChatCompletionResponse {
        id: "test-id".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "test-model".to_string(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: Role::Assistant,
                content: Content::new_text("Test response"),
                function_call: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
        usage: Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
        },
        usage_status: UsageStatus::Finalized,
    };

    assert_eq!(response.usage_status, UsageStatus::Finalized);
    assert_eq!(response.usage.prompt_tokens, 10);
    assert_eq!(response.usage.completion_tokens, 20);
    assert_eq!(response.usage.total_tokens, 30);
}
