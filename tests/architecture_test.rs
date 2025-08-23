#[cfg(test)]
mod tests {
    use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};

    #[test]
    fn test_provider_enum() {
        // 测试Provider枚举
        let provider = Provider::Groq;
        assert!(matches!(provider, Provider::Groq));
    }

    #[test]
    fn test_client_creation() {
        // 测试客户端创建（不实际调用API）
        let client_result = AiClient::new(Provider::Groq);
        assert!(client_result.is_ok());
        
        let client = client_result.unwrap();
        assert!(matches!(client.current_provider(), Provider::Groq));
    }

    #[test]
    fn test_request_builder() {
        // 测试请求构建器
        let request = ChatCompletionRequest::new(
            "test-model".to_string(),
            vec![Message {
                role: Role::User,
                content: "Hello".to_string(),
            }],
        ).with_temperature(0.7)
         .with_max_tokens(100);
        
        assert_eq!(request.model, "test-model");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(100));
    }

    #[test]
    fn test_role_enum() {
        // 测试Role枚举
        let user_role = Role::User;
        let system_role = Role::System;
        let assistant_role = Role::Assistant;
        
        assert!(matches!(user_role, Role::User));
        assert!(matches!(system_role, Role::System));
        assert!(matches!(assistant_role, Role::Assistant));
    }
}
