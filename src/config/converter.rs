use crate::client::Provider;
use crate::config::ConnectionOptions;
use crate::registry::model::ProviderConfig;
use std::collections::HashMap;

/// Configuration converter for unifying legacy options with registry config
pub struct ConfigConverter;

impl ConfigConverter {
    /// Convert legacy parameters into a ProviderConfig
    pub fn convert_legacy(
        provider: Provider,
        options: Option<&ConnectionOptions>,
    ) -> ProviderConfig {
        let protocol = provider.as_protocol();
        let env_prefix = provider.env_prefix();

        // Extract overrides from options if present
        let (base_url_override, api_key_override) = if let Some(opts) = options {
            (opts.base_url.clone(), opts.api_key.clone())
        } else {
            (None, None)
        };

        ProviderConfig {
            protocol: protocol.to_string(),
            base_url: base_url_override
                .or_else(|| Self::default_base_url_str(provider).map(String::from)),
            api_env: Some(env_prefix.to_string()),
            api_key: api_key_override,
            headers: HashMap::new(),
            extra: HashMap::new(),
        }
    }

    /// Helper for Legacy URL defaults (Extracted from ProviderFactory)
    fn default_base_url_str(provider: Provider) -> Option<&'static str> {
        match provider.as_protocol() {
            "openai" => Some("https://api.openai.com/v1"), // Default for generic
            _ => None,
        }
    }
}
