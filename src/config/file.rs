//! File-based configuration provider
//!
//! This provider loads configuration from a custom YAML or JSON file path.
//! Supports configuration reloading and update detection for hot-reload scenarios.

use super::provider_trait::{ConfigError, ConfigProvider};
use crate::registry::model::{ModelInfo, ProviderConfig, RegistryConfig};
use crate::registry::ModelRegistry;
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// File-based configuration provider
///
/// Loads configuration from a specified YAML or JSON file. Supports checking for file
/// updates and reloading configuration for hot-reload functionality.
///
/// # Examples
///
/// ```rust,ignore
/// use ai_lib::config::FileConfigProvider;
/// use std::path::Path;
///
/// let provider = FileConfigProvider::new("config/aimanifest.yaml").unwrap();
/// let model = provider.resolve_model("gpt-4o").await.unwrap();
///
/// // Check for updates and reload if necessary
/// if provider.check_update().await.unwrap() {
///     provider.reload().await.unwrap();
/// }
/// ```
pub struct FileConfigProvider {
    path: PathBuf,
    registry: Arc<RwLock<ModelRegistry>>,
    last_modified: Arc<RwLock<Option<SystemTime>>>,
}

impl FileConfigProvider {
    /// Create a new file-based configuration provider
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file (YAML or JSON)
    ///
    /// # Returns
    /// * `Ok(FileConfigProvider)` - Successfully created and loaded
    /// * `Err(ConfigError)` - If file cannot be read or parsed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let provider = FileConfigProvider::new("my-manifest.yaml")?;
    /// ```
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let path = path.into();
        let registry = Arc::new(RwLock::new(ModelRegistry::new()));

        // Initial load
        let config = Self::load_from_file(&path)?;
        registry.write().unwrap().merge_config(config);

        // Get initial modification time
        let last_modified = Self::get_modification_time(&path)?;

        Ok(Self {
            path,
            registry,
            last_modified: Arc::new(RwLock::new(Some(last_modified))),
        })
    }

    /// Load configuration from file
    fn load_from_file(path: &Path) -> Result<RegistryConfig, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::FileRead(format!("{}: {}", path.display(), e)))?;

        // 1. Try YAML (Manifest format)
        if let Ok(manifest) = serde_yaml::from_str::<crate::manifest::schema::Manifest>(&content) {
            return Ok(RegistryConfig::from_manifest(&manifest));
        }

        // 2. Fallback to JSON (Legacy Registry format)
        serde_json::from_str(&content)
            .map_err(|e| ConfigError::InvalidFormat(format!("{}: {}", path.display(), e)))
    }

    /// Get file modification time
    fn get_modification_time(path: &Path) -> Result<SystemTime, ConfigError> {
        let metadata = fs::metadata(path)
            .map_err(|e| ConfigError::FileRead(format!("{}: {}", path.display(), e)))?;

        metadata
            .modified()
            .map_err(|e| ConfigError::FileRead(format!("Cannot get modification time: {}", e)))
    }
}

#[async_trait]
impl ConfigProvider for FileConfigProvider {
    async fn resolve_provider(&self, id: &str) -> Result<ProviderConfig, ConfigError> {
        self.registry
            .read()
            .unwrap()
            .resolve_provider(id)
            .ok_or_else(|| ConfigError::ProviderNotFound(id.to_string()))
    }

    async fn resolve_model(&self, id: &str) -> Result<ModelInfo, ConfigError> {
        self.registry
            .read()
            .unwrap()
            .resolve_model(id)
            .ok_or_else(|| ConfigError::ModelNotFound(id.to_string()))
    }

    async fn list_providers(&self) -> Result<Vec<String>, ConfigError> {
        // Not implemented yet - requires ModelRegistry enhancement
        Ok(Vec::new())
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, ConfigError> {
        // Not implemented yet - requires ModelRegistry enhancement
        Ok(Vec::new())
    }

    async fn check_update(&self) -> Result<bool, ConfigError> {
        let current_modified = Self::get_modification_time(&self.path)?;
        let mut last = self.last_modified.write().unwrap();

        if last.map_or(true, |t| current_modified > t) {
            *last = Some(current_modified);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn reload(&self) -> Result<(), ConfigError> {
        let config = Self::load_from_file(&self.path)?;
        self.registry.write().unwrap().merge_config(config);

        // Update last modified time
        let modified = Self::get_modification_time(&self.path)?;
        *self.last_modified.write().unwrap() = Some(modified);

        Ok(())
    }
}
