//! Embedded configuration provider
//!
//! This provider loads configuration from embedded resources (aimanifest.yaml).
//! It uses the existing ModelRegistry internally for backward compatibility.

use super::provider_trait::{ConfigError, ConfigProvider};
use crate::registry::model::{ModelInfo, ProviderConfig};
use crate::registry::ModelRegistry;
use async_trait::async_trait;
use std::sync::Arc;

/// Embedded configuration provider
///
/// Loads configuration from the embedded `defaults/aimanifest.yaml` file.
/// This is the default configuration source and ensures zero external dependencies.
///
/// # Examples
///
/// ```rust,ignore
/// use ai_lib::config::EmbeddedConfigProvider;
///
/// let provider = EmbeddedConfigProvider::new();
/// let model = provider.resolve_model("gpt-4o").await.unwrap();
/// ```
pub struct EmbeddedConfigProvider {
    registry: Arc<ModelRegistry>,
}

impl EmbeddedConfigProvider {
    /// Create a new embedded configuration provider
    ///
    /// This will load the embedded `aimanifest.yaml` configuration immediately.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(ModelRegistry::new()),
        }
    }
}

impl Default for EmbeddedConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConfigProvider for EmbeddedConfigProvider {
    async fn resolve_provider(&self, id: &str) -> Result<ProviderConfig, ConfigError> {
        self.registry
            .resolve_provider(id)
            .ok_or_else(|| ConfigError::ProviderNotFound(id.to_string()))
    }

    async fn resolve_model(&self, id: &str) -> Result<ModelInfo, ConfigError> {
        self.registry
            .resolve_model(id)
            .ok_or_else(|| ConfigError::ModelNotFound(id.to_string()))
    }

    async fn list_providers(&self) -> Result<Vec<String>, ConfigError> {
        // Note: ModelRegistry doesn't currently expose a list method
        // For now, return empty vec; this can be enhanced later
        Ok(Vec::new())
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ConfigError> {
        // Note: ModelRegistry doesn't currently expose a list method
        // For now, return empty vec; this can be enhanced later
        Ok(Vec::new())
    }

    // check_update and reload use default implementations (no updates for embedded)
}
