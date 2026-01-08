#[cfg(test)]
mod tests {
    use ai_lib::types::common::Content;
    use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

    #[test]
    fn test_provider_enum() {
        // Test Provider enum
        let provider = Provider::Groq;
        assert!(matches!(provider, Provider::Groq));
    }

    #[test]
    fn test_client_creation() {
        // Test client creation (without actual API calls)
        let client_result = AiClient::new(Provider::Groq);
        if let Err(e) = &client_result {
            eprintln!("Client creation failed: {}", e);
        }
        assert!(client_result.is_ok(), "Client creation should succeed");

        let client = client_result.unwrap();
        assert_eq!(client.provider_name(), "Groq");
    }

    #[test]
    fn test_request_builder() {
        // Test request builder
        let request = ChatCompletionRequest::new(
            "test-model".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text("Hello".to_string()),
                function_call: None,
            }],
        )
        .with_temperature(0.7)
        .with_max_tokens(100);

        assert_eq!(request.model, "test-model");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(100));
    }

    #[test]
    fn test_role_enum() {
        // Test Role enum
        let user_role = Role::User;
        let system_role = Role::System;
        let assistant_role = Role::Assistant;

        assert!(matches!(user_role, Role::User));
        assert!(matches!(system_role, Role::System));
        assert!(matches!(assistant_role, Role::Assistant));
    }
}
