//! Registry resolver module
//!
//! This module handles resolution of models and providers from the registry.
//! It extracts the registry resolution logic from builder.rs to improve modularity.

use crate::client::Provider;
use crate::registry::{model::ProviderConfig, GLOBAL_REGISTRY};
use crate::types::AiLibError;

/// Registry resolver for model and provider configuration
pub struct RegistryResolver;

impl RegistryResolver {
    /// Resolve configuration using model-driven approach
    ///
    /// # Arguments
    /// * `model_id` - The model ID to look up in the registry
    ///
    /// # Returns
    /// * `Ok((protocol, config))` - Protocol string and provider configuration
    /// * `Err(AiLibError)` - If model or provider not found
    pub fn resolve_model_driven(model_id: &str) -> Result<(String, ProviderConfig), AiLibError> {
        // 1. Look up model
        let model_info = GLOBAL_REGISTRY.resolve_model(model_id).ok_or_else(|| {
            AiLibError::ConfigurationError(format!("Model '{}' not found in registry", model_id))
        })?;

        // 2. Look up provider
        let provider_conf = GLOBAL_REGISTRY
            .resolve_provider(&model_info.provider)
            .ok_or_else(|| {
                AiLibError::ConfigurationError(format!(
                    "Provider '{}' for model '{}' not found in registry",
                    model_info.provider, model_id
                ))
            })?;

        Ok((provider_conf.protocol.clone(), provider_conf))
    }

    /// Resolve configuration using provider-driven approach
    ///
    /// # Arguments
    /// * `provider` - The provider enum variant
    ///
    /// # Returns
    /// * `Ok((protocol, config))` - Protocol string and optional provider configuration
    ///   If provider is not in registry, config will be None and protocol will be from enum mapping
    pub fn resolve_provider_driven(provider: Provider) -> (String, Option<ProviderConfig>) {
        let key = provider.as_registry_key();
        if let Some(provider_conf) = GLOBAL_REGISTRY.resolve_provider(key) {
            (provider_conf.protocol.clone(), Some(provider_conf))
        } else {
            // If NOT in registry (rare, or new enum variant), use Protocol mapping
            (provider.as_protocol().to_string(), None)
        }
    }
}
