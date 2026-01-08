//! Configuration provider trait
//!
//! This module defines the ConfigProvider trait that abstracts configuration sources.
//! Implementations can load configurations from embedded data, files, remote URLs, etc.

use crate::registry::model::{ModelInfo, ProviderConfig};
use crate::types::AiLibError;
use async_trait::async_trait;

/// Error type for configuration operations
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Provider not found in configuration
    ProviderNotFound(String),
    /// Model not found in configuration
    ModelNotFound(String),
    /// Failed to read configuration file
    FileRead(String),
    /// Invalid configuration format
    InvalidFormat(String),
    /// Failed to fetch remote configuration
    RemoteFetch(String),
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ProviderNotFound(id) => write!(f, "Provider not found: {}", id),
            ConfigError::ModelNotFound(id) => write!(f, "Model not found: {}", id),
            ConfigError::FileRead(err) => write!(f, "Failed to read config file: {}", err),
            ConfigError::InvalidFormat(err) => write!(f, "Invalid config format: {}", err),
            ConfigError::RemoteFetch(err) => write!(f, "Failed to fetch remote config: {}", err),
            ConfigError::Internal(err) => write!(f, "Internal error: {}", err),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<ConfigError> for AiLibError {
    fn from(err: ConfigError) -> Self {
        AiLibError::ConfigurationError(err.to_string())
    }
}

/// Configuration provider trait
///
/// This trait abstracts the source of configuration data, allowing implementations
/// to load from embedded resources, files, remote URLs, or other sources.
///
/// # Examples
///
/// ```rust,ignore
/// use ai_lib::config::ConfigProvider;
///
/// async fn use_provider(provider: &dyn ConfigProvider) {
///     let model = provider.resolve_model("gpt-4").await.unwrap();
///     println!("Model: {}", model.id);
/// }
/// ```
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    /// Resolve provider configuration by ID
    ///
    /// # Arguments
    /// * `id` - Provider identifier (e.g., "openai", "anthropic")
    ///
    /// # Returns
    /// * `Ok(ProviderConfig)` - Provider configuration
    /// * `Err(ConfigError)` - If provider not found or error occurred
    async fn resolve_provider(&self, id: &str) -> Result<ProviderConfig, ConfigError>;

    /// Resolve model configuration by ID
    ///
    /// # Arguments
    /// * `id` - Model identifier (e.g., "gpt-4", "claude-3-opus")
    ///
    /// # Returns
    /// * `Ok(ModelInfo)` - Model information
    /// * `Err(ConfigError)` - If model not found or error occurred
    async fn resolve_model(&self, id: &str) -> Result<ModelInfo, ConfigError>;

    /// List all available provider IDs
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of provider identifiers
    /// * `Err(ConfigError)` - If error occurred
    async fn list_providers(&self) -> Result<Vec<String>, ConfigError>;

    /// List all available models
    ///
    /// # Returns
    /// * `Ok(Vec<ModelInfo>)` - List of model information
    /// * `Err(ConfigError)` - If error occurred
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ConfigError>;

    /// Check if configuration has been updated
    ///
    /// This is useful for implementing hot-reload functionality.
    /// Default implementation returns `false` (no updates).
    ///
    /// # Returns
    /// * `Ok(true)` - Configuration has been updated
    /// * `Ok(false)` - No updates available
    /// * `Err(ConfigError)` - If error occurred
    async fn check_update(&self) -> Result<bool, ConfigError> {
        Ok(false)
    }

    /// Reload configuration from source
    ///
    /// This method should re-read the configuration and update internal state.
    /// Default implementation does nothing.
    ///
    /// # Returns
    /// * `Ok(())` - Configuration reloaded successfully
    /// * `Err(ConfigError)` - If reload failed
    async fn reload(&self) -> Result<(), ConfigError> {
        Ok(())
    }
}
