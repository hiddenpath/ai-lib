use crate::types::AiLibError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Provider configuration template defining API access parameters
///
/// This struct contains all necessary configuration for connecting to an AI provider,
/// including base URL, API endpoints, authentication, and model specifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Base URL for the provider's API
    pub base_url: String,
    /// Environment variable name for the API key
    pub api_key_env: String,
    /// Chat completion endpoint path
    pub chat_endpoint: String,
    /// Default chat model for this provider
    pub chat_model: String,
    /// Optional multimodal model for this provider (if supported)
    pub multimodal_model: Option<String>,
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

/// Field mapping configuration defining field mappings for different API formats
///
/// This struct maps the standard ai-lib field names to provider-specific field names,
/// allowing the library to work with different API formats seamlessly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Messages array field name (OpenAI: "messages", Gemini: "contents")
    pub messages_field: String,
    /// Model field name
    pub model_field: String,
    /// Role field mapping from ai-lib roles to provider roles
    pub role_mapping: HashMap<String, String>,
    /// Response content path (e.g. "choices.0.message.content")
    pub response_content_path: String,
}

impl ProviderConfig {
    /// OpenAI-compatible configuration template
    ///
    /// Creates a standard OpenAI-compatible configuration with default models.
    /// The default chat model is "gpt-3.5-turbo" and multimodal model is "gpt-4o".
    ///
    /// # Arguments
    /// * `base_url` - The base URL for the provider's API
    /// * `api_key_env` - Environment variable name for the API key
    /// * `chat_model` - Default chat model name
    /// * `multimodal_model` - Optional multimodal model name
    pub fn openai_compatible(
        base_url: &str,
        api_key_env: &str,
        chat_model: &str,
        multimodal_model: Option<&str>,
    ) -> Self {
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
            chat_model: chat_model.to_string(),
            multimodal_model: multimodal_model.map(|s| s.to_string()),
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

    /// OpenAI-compatible configuration template with default models
    ///
    /// This is a convenience method that uses standard default models.
    /// For custom models, use `openai_compatible()` with explicit model names.
    pub fn openai_compatible_default(base_url: &str, api_key_env: &str) -> Self {
        Self::openai_compatible(base_url, api_key_env, "gpt-3.5-turbo", Some("gpt-4o"))
    }

    /// Validate the configuration for completeness and correctness
    ///
    /// # Returns
    /// * `Result<(), AiLibError>` - Ok on success, error information on failure
    pub fn validate(&self) -> Result<(), AiLibError> {
        // Validate base_url
        if self.base_url.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "base_url cannot be empty".to_string(),
            ));
        }

        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(AiLibError::ConfigurationError(
                "base_url must be a valid HTTP/HTTPS URL".to_string(),
            ));
        }

        // Validate api_key_env
        if self.api_key_env.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "api_key_env cannot be empty".to_string(),
            ));
        }

        // Validate chat_endpoint
        if self.chat_endpoint.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "chat_endpoint cannot be empty".to_string(),
            ));
        }

        // Validate chat_model
        if self.chat_model.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "chat_model cannot be empty".to_string(),
            ));
        }

        // Validate field_mapping
        self.field_mapping.validate()?;

        // Validate headers Content-Type
        if let Some(content_type) = self.headers.get("Content-Type") {
            if content_type != "application/json" && content_type != "multipart/form-data" {
                return Err(AiLibError::ConfigurationError(
                    "Content-Type header must be 'application/json' or 'multipart/form-data'"
                        .to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get the complete chat completion URL
    pub fn chat_url(&self) -> String {
        format!("{}{}", self.base_url, self.chat_endpoint)
    }

    /// Get the complete models list URL
    pub fn models_url(&self) -> Option<String> {
        self.models_endpoint
            .as_ref()
            .map(|endpoint| format!("{}{}", self.base_url, endpoint))
    }

    /// Get the complete file upload URL
    pub fn upload_url(&self) -> Option<String> {
        self.upload_endpoint
            .as_ref()
            .map(|endpoint| format!("{}{}", self.base_url, endpoint))
    }

    /// Get the default chat model for this provider
    pub fn default_chat_model(&self) -> &str {
        &self.chat_model
    }

    /// Get the multimodal model if available
    pub fn multimodal_model(&self) -> Option<&str> {
        self.multimodal_model.as_deref()
    }
}

impl FieldMapping {
    /// Validate the field mapping configuration
    pub fn validate(&self) -> Result<(), AiLibError> {
        if self.messages_field.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "messages_field cannot be empty".to_string(),
            ));
        }

        if self.model_field.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "model_field cannot be empty".to_string(),
            ));
        }

        if self.response_content_path.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "response_content_path cannot be empty".to_string(),
            ));
        }

        // Validate role_mapping is not empty
        if self.role_mapping.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "role_mapping cannot be empty".to_string(),
            ));
        }

        // Validate required role mappings
        let required_roles = ["System", "User", "Assistant"];
        for role in &required_roles {
            if !self.role_mapping.contains_key(*role) {
                return Err(AiLibError::ConfigurationError(format!(
                    "role_mapping must contain '{}' role",
                    role
                )));
            }
        }

        Ok(())
    }
}
