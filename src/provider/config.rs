use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::types::AiLibError;

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
    /// Optional file upload endpoint path (e.g. OpenAI: "/v1/files")
    pub upload_endpoint: Option<String>,
    /// Optional file size limit (bytes) above which files should be uploaded instead of inlined
    pub upload_size_limit: Option<u64>,
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
            upload_endpoint: Some("/v1/files".to_string()),
            upload_size_limit: Some(1024 * 64),
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

    /// 验证配置的完整性和正确性
    ///
    /// # Returns
    /// * `Result<(), AiLibError>` - 验证成功返回Ok，失败返回错误信息
    pub fn validate(&self) -> Result<(), AiLibError> {
        // 验证base_url
        if self.base_url.is_empty() {
            return Err(AiLibError::ConfigurationError("base_url cannot be empty".to_string()));
        }
        
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(AiLibError::ConfigurationError(
                "base_url must be a valid HTTP/HTTPS URL".to_string()
            ));
        }

        // 验证api_key_env
        if self.api_key_env.is_empty() {
            return Err(AiLibError::ConfigurationError("api_key_env cannot be empty".to_string()));
        }

        // 验证chat_endpoint
        if self.chat_endpoint.is_empty() {
            return Err(AiLibError::ConfigurationError("chat_endpoint cannot be empty".to_string()));
        }

        // 验证field_mapping
        self.field_mapping.validate()?;

        // 验证headers中的Content-Type
        if let Some(content_type) = self.headers.get("Content-Type") {
            if content_type != "application/json" && content_type != "multipart/form-data" {
                return Err(AiLibError::ConfigurationError(
                    "Content-Type header must be 'application/json' or 'multipart/form-data'".to_string()
                ));
            }
        }

        Ok(())
    }

    /// 获取完整的聊天完成URL
    pub fn chat_url(&self) -> String {
        format!("{}{}", self.base_url, self.chat_endpoint)
    }

    /// 获取完整的模型列表URL
    pub fn models_url(&self) -> Option<String> {
        self.models_endpoint.as_ref().map(|endpoint| {
            format!("{}{}", self.base_url, endpoint)
        })
    }

    /// 获取完整的文件上传URL
    pub fn upload_url(&self) -> Option<String> {
        self.upload_endpoint.as_ref().map(|endpoint| {
            format!("{}{}", self.base_url, endpoint)
        })
    }
}

impl FieldMapping {
    /// 验证字段映射配置
    pub fn validate(&self) -> Result<(), AiLibError> {
        if self.messages_field.is_empty() {
            return Err(AiLibError::ConfigurationError("messages_field cannot be empty".to_string()));
        }

        if self.model_field.is_empty() {
            return Err(AiLibError::ConfigurationError("model_field cannot be empty".to_string()));
        }

        if self.response_content_path.is_empty() {
            return Err(AiLibError::ConfigurationError("response_content_path cannot be empty".to_string()));
        }

        // 验证role_mapping不为空
        if self.role_mapping.is_empty() {
            return Err(AiLibError::ConfigurationError("role_mapping cannot be empty".to_string()));
        }

        // 验证必需的role映射
        let required_roles = ["System", "User", "Assistant"];
        for role in &required_roles {
            if !self.role_mapping.contains_key(*role) {
                return Err(AiLibError::ConfigurationError(
                    format!("role_mapping must contain '{}' role", role)
                ));
            }
        }

        Ok(())
    }
}
