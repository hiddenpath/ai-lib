//! Config merger module
//!
//! This module handles merging of provider configurations with builder overrides.
//! It extracts configuration merging logic from builder.rs to improve modularity.

use crate::registry::model::ProviderConfig;

/// Configuration merger for combining registry config with builder overrides
pub struct ConfigMerger;

impl ConfigMerger {
    /// Merge provider configuration with builder overrides
    ///
    /// Priority: explicit override > registry config > default base_url
    ///
    /// # Arguments
    /// * `registry_config` - Configuration from registry
    /// * `base_url_override` - Optional base URL override from builder
    /// * `fallback_base_url` - Fallback base URL if config doesn't have one
    ///
    /// # Returns
    /// * Merged ProviderConfig
    pub fn merge_provider_config(
        mut registry_config: ProviderConfig,
        base_url_override: Option<String>,
        fallback_base_url: String,
    ) -> ProviderConfig {
        // Apply Builder Overrides to Config
        if let Some(url) = base_url_override {
            registry_config.base_url = Some(url);
        }
        // Ensure base_url is set from the resolved one if config didn't have it (or overridden)
        if registry_config.base_url.is_none() {
            registry_config.base_url = Some(fallback_base_url);
        }

        registry_config
    }
}
