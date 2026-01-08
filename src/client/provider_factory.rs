// Provider factory - unified adapter creation
//
// This module centralizes all provider adapter creation logic.
// v0.5.0 Update: Now Data-Driven based on Protocols.

use crate::api::ChatProvider;
use crate::client::Provider;
use crate::provider::chat_provider::AdapterProvider;
#[allow(deprecated)]
use crate::provider::{
    config::ProviderConfig as LegacyProviderConfig, AI21Adapter, CohereAdapter, GeminiAdapter,
    GenericAdapter, MistralAdapter, PerplexityAdapter,
};
use crate::registry::model::ProviderConfig;
use crate::transport::DynHttpTransportRef;
use crate::types::AiLibError;

pub struct ProviderFactory;

impl ProviderFactory {
    /// Create provider adapter based on Protocol and Configuration (v0.5.0 Core)
    ///
    /// # Arguments
    /// * `protocol` - The protocol ID (e.g., "openai", "anthropic")
    /// * `config` - The configuration from the Registry
    /// * `transport` - Optional transport override
    ///
    /// # Returns
    /// Boxed ChatProvider implementation
    pub fn create(
        protocol: &str,
        config: ProviderConfig,
        transport: Option<DynHttpTransportRef>,
    ) -> Result<Box<dyn ChatProvider>, AiLibError> {
        let adapter: Box<dyn ChatProvider> = match protocol {
            // Config-driven Generic Logic (OpenAI Compatible)
            "openai" => {
                use crate::adapter::dynamic::ConfigDrivenAdapter;
                use crate::manifest::schema::{
                    AuthConfig, ModelDefinition, PayloadFormat, ProviderDefinition, ResponseFormat,
                };
                use std::collections::HashMap;
                use std::sync::Arc;

                // 1. Synthesize ProviderDefinition from Registry Config
                let api_key_env = config
                    .api_env
                    .clone()
                    .unwrap_or("OPENAI_API_KEY".to_string());
                let auth = AuthConfig::Bearer {
                    token_env: api_key_env,
                    extra_headers: vec![],
                };

                let provider_def = ProviderDefinition {
                    version: "1.0".to_string(),
                    base_url: config.base_url.clone(),
                    base_url_template: None,
                    connection_vars: None,
                    auth,
                    payload_format: PayloadFormat::OpenaiStyle,
                    parameter_mappings: HashMap::new(),
                    special_handling: HashMap::new(),
                    response_format: ResponseFormat::OpenaiStyle,
                    response_paths: HashMap::new(),
                    streaming: Default::default(),
                    experimental_features: vec![],
                    capabilities: vec![],
                    response_strategy: None,
                    tools_mapping: None,
                    prompt_caching: None,
                    service_tier: None,
                    reasoning_tokens: None,
                    features: None,
                };

                let model_def = ModelDefinition {
                    provider: protocol.to_string(),
                    model_id: "default".to_string(),
                    display_name: None,
                    context_window: 128000,
                    capabilities: vec![],
                    pricing: None,
                    overrides: HashMap::new(),
                    status: Default::default(),
                    tags: vec![],
                    agentic_capabilities: None,
                };

                // 2. Create Dummy Manifest (Mock) - required by new_raw if it expects strong type
                let manifest = Arc::new(crate::manifest::schema::Manifest {
                    version: "1.0".to_string(),
                    metadata: Default::default(),
                    standard_schema: crate::manifest::schema::StandardSchema {
                        parameters: HashMap::new(),
                        tools: Default::default(),
                        response_format: Default::default(),
                        multimodal: Default::default(),
                        agentic_loop: None,
                        streaming_events: None,
                    },
                    providers: HashMap::new(),
                    models: HashMap::new(),
                });

                // 3. Instantiate ConfigDrivenAdapter
                // Note: Transport override is ignored in this phase as ConfigDrivenAdapter manages its own client.
                let adapter = ConfigDrivenAdapter::new_raw(manifest, provider_def, model_def)?;

                Box::new(adapter)
            }

            // Native Protocols - Legacy paths preserved for specific optimized providers for now
            "anthropic" => create_generic_native("ANTHROPIC_API_KEY", config, transport)?,

            // Independent Adapters
            "gemini" => create_native_adapter(
                |k, u, t| GeminiAdapter::with_transport_ref(t, k, u),
                |k, u| GeminiAdapter::new_with_overrides(k, Some(u)),
                GeminiAdapter::new,
                "GEMINI_API_KEY",
                "https://generativelanguage.googleapis.com/v1beta",
                config,
                transport,
            )?,

            "mistral" => {
                let base = config
                    .base_url
                    .unwrap_or_else(|| "https://api.mistral.ai".to_string());
                // Priority: config.api_key > env var
                let api_key = config
                    .api_key
                    .or_else(|| std::env::var("MISTRAL_API_KEY").ok());

                if let Some(t) = transport {
                    Box::new(MistralAdapter::with_transport(t, api_key, base)?)
                } else if api_key.is_some() {
                    Box::new(MistralAdapter::new_with_overrides(api_key, Some(base))?)
                } else {
                    Box::new(MistralAdapter::new()?)
                }
            }

            "cohere" => create_native_adapter(
                |k, u, t| Ok(CohereAdapter::with_transport_ref(t, k, u)),
                |k, u| CohereAdapter::new_with_overrides(k, Some(u)),
                CohereAdapter::new,
                "COHERE_API_KEY",
                "https://api.cohere.ai",
                config,
                transport,
            )?,

            "perplexity" => Box::new(PerplexityAdapter::new()?),
            "ai21" => Box::new(AI21Adapter::new()?),

            _ => {
                return Err(AiLibError::ConfigurationError(format!(
                    "Unknown protocol: {}",
                    protocol
                )))
            }
        };

        Ok(AdapterProvider::new(protocol.to_string(), adapter).boxed())
    }

    /// Legacy Compatibility Wrapper
    ///
    /// Converts the `Provider` enum into a Protocol + Default Config call.
    pub fn create_adapter(
        provider: Provider,
        api_key_override: Option<String>,
        base_url_override: Option<String>,
        transport: Option<DynHttpTransportRef>,
    ) -> Result<Box<dyn ChatProvider>, AiLibError> {
        let protocol = provider.as_protocol();

        // Construct temporary ConnectionOptions to use the converter
        // This bridges the legacy arguments to the new unified converter
        let options = crate::config::ConnectionOptions {
            base_url: base_url_override,
            api_key: api_key_override,
            proxy: None,
            timeout: None,
            disable_proxy: false,
        };

        // Use unified converter
        let config =
            crate::config::converter::ConfigConverter::convert_legacy(provider, Some(&options));

        Self::create(protocol, config, transport)
    }
}

// Helper to reduce boilerplate for Native Adapters
fn create_native_adapter<F1, F2, F3, E>(
    with_transport: F1,
    with_override: F2,
    new_default: F3,
    env_key: &str,
    default_base: &str,
    config: ProviderConfig,
    transport: Option<DynHttpTransportRef>,
) -> Result<Box<dyn ChatProvider>, AiLibError>
where
    F1: Fn(String, String, DynHttpTransportRef) -> Result<E, AiLibError>,
    F2: Fn(String, String) -> Result<E, AiLibError>,
    F3: Fn() -> Result<E, AiLibError>,
    E: ChatProvider + 'static,
{
    let base = config.base_url.unwrap_or_else(|| default_base.to_string());
    // Priority: config.api_key > env var
    let api_key = config.api_key.or_else(|| std::env::var(env_key).ok());

    if let Some(t) = transport {
        let key = api_key
            .ok_or_else(|| AiLibError::AuthenticationError(format!("{} not set", env_key)))?;
        Ok(Box::new(with_transport(key, base, t)?))
    } else if let Some(key) = api_key {
        Ok(Box::new(with_override(key, base)?))
    } else {
        Ok(Box::new(new_default()?))
    }
}

// Helper for Anthropic/Other Generic-like natives
#[allow(deprecated)]
fn create_generic_native(
    env_key: &str,
    config: ProviderConfig,
    transport: Option<DynHttpTransportRef>,
) -> Result<Box<dyn ChatProvider>, AiLibError> {
    let legacy_conf = LegacyProviderConfig {
        base_url: config
            .base_url
            .ok_or(AiLibError::ConfigurationError("Missing baseUrl".into()))?,
        api_key_env: env_key.to_string(),
        chat_model: "placeholder".to_string(), // Placeholder for validation, not used in adapter init
        multimodal_model: None,
        headers: config.headers,
        models_endpoint: None,
        chat_endpoint: "/chat/completions".to_string(), // Default assumes OpenAI-like structure or overriding
        upload_endpoint: None,
        upload_size_limit: None,
        field_mapping: crate::provider::config::FieldMapping {
            messages_field: "messages".to_string(),
            model_field: "model".to_string(),
            role_mapping: std::collections::HashMap::from([
                ("System".to_string(), "system".to_string()),
                ("User".to_string(), "user".to_string()),
                ("Assistant".to_string(), "assistant".to_string()),
            ]),
            response_content_path: "choices.0.message.content".to_string(),
        },
    };

    // Pass api_key from config if available (priority: config.api_key > env var)
    let adapter = match transport {
        Some(t) => Box::new(GenericAdapter::with_transport_ref_api_key(
            legacy_conf,
            t,
            config.api_key,
        )?) as Box<dyn ChatProvider>,
        None => Box::new(GenericAdapter::new_with_api_key(
            legacy_conf,
            config.api_key,
        )?) as Box<dyn ChatProvider>,
    };

    Ok(adapter)
}
