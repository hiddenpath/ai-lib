use super::config::ProviderConfig;

/// 预定义的提供商配置，支持多种AI服务
/// 
/// Predefined provider configurations for multiple AI services
pub struct ProviderConfigs;

impl ProviderConfigs {
    pub fn groq() -> ProviderConfig {
        ProviderConfig::openai_compatible(
            "https://api.groq.com/openai/v1",
            "GROQ_API_KEY"
        )
    }

    pub fn openai() -> ProviderConfig {
        ProviderConfig::openai_compatible(
            "https://api.openai.com/v1",
            "OPENAI_API_KEY"
        )
    }

    pub fn deepseek() -> ProviderConfig {
        ProviderConfig::openai_compatible(
            "https://api.deepseek.com/v1",
            "DEEPSEEK_API_KEY"
        )
    }
    
    /// Groq configuration - proving OpenAI compatibility
    pub fn groq_as_generic() -> ProviderConfig {
        ProviderConfig::openai_compatible(
            "https://api.groq.com/openai/v1",
            "GROQ_API_KEY"
        )
    }
    
    /// Anthropic Claude configuration - requires special handling
    pub fn anthropic() -> ProviderConfig {
        use std::collections::HashMap;
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        // Note: Anthropic uses x-api-key instead of Authorization: Bearer
        
        let mut role_mapping = HashMap::new();
        role_mapping.insert("System".to_string(), "system".to_string());
        role_mapping.insert("User".to_string(), "user".to_string());
        role_mapping.insert("Assistant".to_string(), "assistant".to_string());

        ProviderConfig {
            base_url: "https://api.anthropic.com/v1".to_string(),
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
            chat_endpoint: "/messages".to_string(),
            models_endpoint: None, // Claude doesn't have a public model list endpoint
            headers,
            field_mapping: crate::provider::config::FieldMapping {
                messages_field: "messages".to_string(),
                model_field: "model".to_string(),
                role_mapping,
                response_content_path: "content.0.text".to_string(), // Claude's response format is different
            },
        }
    }
}