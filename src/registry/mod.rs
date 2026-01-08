pub mod model;
pub use model::{ModelInfo, ModelPricing, ProviderConfig, RegistryConfig};

#[cfg(feature = "config_hot_reload")]
pub mod watcher;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static GLOBAL_REGISTRY: Lazy<ModelRegistry> = Lazy::new(ModelRegistry::new);

/// Model registry with concurrent access support
///
/// Phase 3 Enhancement: Uses DashMap for lock-free concurrent reads
pub struct ModelRegistry {
    // DashMap provides concurrent access without explicit locking
    models: Arc<DashMap<String, ModelInfo>>,
    providers: Arc<DashMap<String, ProviderConfig>>,
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRegistry {
    pub fn new() -> Self {
        let registry = Self {
            models: Arc::new(DashMap::new()),
            providers: Arc::new(DashMap::new()),
        };
        // Load embedded defaults immediately
        registry.load_embedded();
        registry
    }

    fn load_embedded(&self) {
        // Load from embedded YAML manifest (v0.5.0+ Standard)
        const YAML_MANIFEST: &str = include_str!("../defaults/aimanifest.yaml");
        if let Ok(manifest) =
            serde_yaml::from_str::<crate::manifest::schema::Manifest>(YAML_MANIFEST)
        {
            let config = RegistryConfig::from_manifest(&manifest);
            self.merge_config(config);
        } else {
            eprintln!("CRITICAL: Failed to parse embedded aimanifest.yaml");
        }
    }

    /// Merge configuration into registry
    ///
    /// Phase 3: Now uses DashMap for concurrent inserts
    pub fn merge_config(&self, config: RegistryConfig) {
        for model in config.models {
            self.models.insert(model.id.clone(), model);
        }

        for (id, provider) in config.providers {
            self.providers.insert(id, provider);
        }
    }

    /// Resolve model by ID
    ///
    /// Phase 3: Lock-free concurrent reads with DashMap
    pub fn resolve_model(&self, model_id: &str) -> Option<ModelInfo> {
        self.models.get(model_id).map(|entry| entry.clone())
    }

    /// Resolve provider by ID
    ///
    /// Phase 3: Lock-free concurrent reads with DashMap
    pub fn resolve_provider(&self, provider_id: &str) -> Option<ProviderConfig> {
        self.providers.get(provider_id).map(|entry| entry.clone())
    }

    /// List all model IDs (Phase 3 enhancement)
    pub fn list_model_ids(&self) -> Vec<String> {
        self.models
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// List all provider IDs (Phase 3 enhancement)
    pub fn list_provider_ids(&self) -> Vec<String> {
        self.providers
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
}
