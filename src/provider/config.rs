use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 提供商配置模板，定义API访问参数
/// 
/// Provider configuration template defining API access parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Base URL
    pub base_url: String,
    /// API key environment variable name
    pub api_key_env: String,
    /// Chat completion endpoint path
    pub chat_endpoint: String,
    /// Model list endpoint path
    pub models_endpoint: Option<String>,
    /// Request headers template
    pub headers: HashMap<String, String>,
    /// Field mapping configuration
    pub field_mapping: FieldMapping,
}

/// 字段映射配置，定义不同API格式的字段映射
/// 
/// Field mapping configuration defining field mappings for different API formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Messages array field name (OpenAI: "messages", Gemini: "contents")
    pub messages_field: String,
    /// Model field name
    pub model_field: String,
    /// Role field mapping
    pub role_mapping: HashMap<String, String>,
    /// Response content path (e.g. "choices.0.message.content")
    pub response_content_path: String,
}

impl ProviderConfig {
    /// OpenAI-compatible configuration template
    pub fn openai_compatible(base_url: &str, api_key_env: &str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        let mut role_mapping = HashMap::new();
        role_mapping.insert("System".to_string(), "system".to_string());
        role_mapping.insert("User".to_string(), "user".to_string());
        role_mapping.insert("Assistant".to_string(), "assistant".to_string());

        Self {
            base_url: base_url.to_string(),
            api_key_env: api_key_env.to_string(),
            chat_endpoint: "/chat/completions".to_string(),
            models_endpoint: Some("/models".to_string()),
            headers,
            field_mapping: FieldMapping {
                messages_field: "messages".to_string(),
                model_field: "model".to_string(),
                role_mapping,
                response_content_path: "choices.0.message.content".to_string(),
            },
        }
    }
}