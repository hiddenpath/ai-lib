// Provider factory - unified adapter creation
//
// This module centralizes all provider adapter creation logic.
// Adding a new provider only requires modifying this file.

//! Provider Factory module.
//!
//! This module is the central place for creating provider adapters.
//! It decouples the `AiClient` from specific provider implementations.
//!
//! To add a new provider:
//! 1. Add variant to `Provider` enum
//! 2. Add configuration to `ProviderConfigs`
//! 3. Add match arm in `create`

use crate::api::ChatProvider;
use crate::client::Provider;
use crate::provider::chat_provider::AdapterProvider;
use crate::provider::{
    config::ProviderConfig, AI21Adapter, CohereAdapter, GeminiAdapter, GenericAdapter,
    MistralAdapter, OpenAiAdapter, PerplexityAdapter, ProviderConfigs,
};
use crate::transport::DynHttpTransportRef;
use crate::types::AiLibError;

pub struct ProviderFactory;

impl ProviderFactory {
    /// Create provider adapter - unified entry point
    ///
    /// # Arguments
    /// * `provider` - The provider enum variant
    /// * `api_key` - Optional API key override
    /// * `base_url` - Optional base URL override
    /// * `transport` - Optional transport override
    ///
    /// # Returns
    /// Boxed ChatProvider implementation for the provider
    pub fn create_adapter(
        provider: Provider,
        api_key: Option<String>,
        base_url: Option<String>,
        transport: Option<DynHttpTransportRef>,
    ) -> Result<Box<dyn ChatProvider>, AiLibError> {
        let adapter = match provider {
            // Config-driven providers using GenericAdapter
            Provider::Groq => create_generic(ProviderConfigs::groq(), api_key, base_url, transport),
            Provider::XaiGrok => {
                create_generic(ProviderConfigs::xai_grok(), api_key, base_url, transport)
            }
            Provider::Ollama => {
                create_generic(ProviderConfigs::ollama(), api_key, base_url, transport)
            }
            Provider::DeepSeek => {
                create_generic(ProviderConfigs::deepseek(), api_key, base_url, transport)
            }
            Provider::Qwen => create_generic(ProviderConfigs::qwen(), api_key, base_url, transport),
            Provider::Anthropic => {
                create_generic(ProviderConfigs::anthropic(), api_key, base_url, transport)
            }
            Provider::AzureOpenAI => create_generic(
                ProviderConfigs::azure_openai(),
                api_key,
                base_url,
                transport,
            ),
            Provider::HuggingFace => {
                create_generic(ProviderConfigs::huggingface(), api_key, base_url, transport)
            }
            Provider::TogetherAI => {
                create_generic(ProviderConfigs::together_ai(), api_key, base_url, transport)
            }
            Provider::OpenRouter => {
                create_generic(ProviderConfigs::openrouter(), api_key, base_url, transport)
            }
            Provider::Replicate => {
                create_generic(ProviderConfigs::replicate(), api_key, base_url, transport)
            }
            Provider::BaiduWenxin => create_generic(
                ProviderConfigs::baidu_wenxin(),
                api_key,
                base_url,
                transport,
            ),
            Provider::TencentHunyuan => create_generic(
                ProviderConfigs::tencent_hunyuan(),
                api_key,
                base_url,
                transport,
            ),
            Provider::IflytekSpark => create_generic(
                ProviderConfigs::iflytek_spark(),
                api_key,
                base_url,
                transport,
            ),
            Provider::Moonshot => {
                create_generic(ProviderConfigs::moonshot(), api_key, base_url, transport)
            }
            Provider::ZhipuAI => {
                create_generic(ProviderConfigs::zhipu_ai(), api_key, base_url, transport)
            }
            Provider::MiniMax => {
                create_generic(ProviderConfigs::minimax(), api_key, base_url, transport)
            }

            // Independent adapters with dedicated implementations
            Provider::OpenAI => create_openai_adapter(api_key, base_url, transport),
            Provider::Gemini => create_gemini_adapter(api_key, base_url, transport),
            Provider::Mistral => create_mistral_adapter(api_key, base_url, transport),
            Provider::Cohere => create_cohere_adapter(api_key, base_url, transport),
            Provider::Perplexity => create_perplexity_adapter(),
            Provider::AI21 => create_ai21_adapter(),
        }?;

        Ok(AdapterProvider::new(format!("{provider:?}"), adapter).boxed())
    }
}

/// Helper function to create GenericAdapter with configuration
fn create_generic(
    mut config: ProviderConfig,
    api_key: Option<String>,
    base_url: Option<String>,
    transport: Option<DynHttpTransportRef>,
) -> Result<Box<dyn ChatProvider>, AiLibError> {
    // Apply overrides to config
    if let Some(url) = base_url {
        config.base_url = url;
    }

    let adapter = match (transport, api_key) {
        (Some(t), Some(key)) => GenericAdapter::with_transport_ref_api_key(config, t, Some(key))?,
        (Some(t), None) => GenericAdapter::with_transport_ref(config, t)?,
        (None, Some(key)) => GenericAdapter::new_with_api_key(config, Some(key))?,
        (None, None) => GenericAdapter::new(config)?,
    };

    Ok(Box::new(adapter))
}

fn create_openai_adapter(
    api_key: Option<String>,
    base_url: Option<String>,
    transport: Option<DynHttpTransportRef>,
) -> Result<Box<dyn ChatProvider>, AiLibError> {
    const DEFAULT_BASE: &str = "https://api.openai.com/v1";

    if let Some(t) = transport {
        let key = match api_key {
            Some(k) => k,
            None => resolve_api_key("OPENAI_API_KEY")?,
        };
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE.to_string());
        let adapter = OpenAiAdapter::with_transport_ref(t, key, base)?;
        return Ok(Box::new(adapter));
    }

    if let Some(key) = api_key {
        return Ok(Box::new(OpenAiAdapter::new_with_overrides(key, base_url)?));
    }

    Ok(Box::new(OpenAiAdapter::new()?))
}

fn create_gemini_adapter(
    api_key: Option<String>,
    base_url: Option<String>,
    transport: Option<DynHttpTransportRef>,
) -> Result<Box<dyn ChatProvider>, AiLibError> {
    const DEFAULT_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

    if let Some(t) = transport {
        let key = match api_key {
            Some(k) => k,
            None => resolve_api_key("GEMINI_API_KEY")?,
        };
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE.to_string());
        let adapter = GeminiAdapter::with_transport_ref(t, key, base)?;
        return Ok(Box::new(adapter));
    }

    if let Some(key) = api_key {
        return Ok(Box::new(GeminiAdapter::new_with_overrides(key, base_url)?));
    }

    Ok(Box::new(GeminiAdapter::new()?))
}

fn create_mistral_adapter(
    api_key: Option<String>,
    base_url: Option<String>,
    transport: Option<DynHttpTransportRef>,
) -> Result<Box<dyn ChatProvider>, AiLibError> {
    if let Some(t) = transport {
        let base = base_url.unwrap_or_else(default_mistral_base);
        let adapter = MistralAdapter::with_transport(t, api_key, base)?;
        return Ok(Box::new(adapter));
    }

    if base_url.is_some() || api_key.is_some() {
        let adapter = MistralAdapter::new_with_overrides(api_key, base_url)?;
        return Ok(Box::new(adapter));
    }

    Ok(Box::new(MistralAdapter::new()?))
}

fn create_cohere_adapter(
    api_key: Option<String>,
    base_url: Option<String>,
    transport: Option<DynHttpTransportRef>,
) -> Result<Box<dyn ChatProvider>, AiLibError> {
    const DEFAULT_BASE: &str = "https://api.cohere.ai";

    if let Some(t) = transport {
        let key = match api_key {
            Some(k) => k,
            None => resolve_api_key("COHERE_API_KEY")?,
        };
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE.to_string());
        let adapter = CohereAdapter::with_transport_ref(t, key, base);
        return Ok(Box::new(adapter));
    }

    if let Some(key) = api_key {
        return Ok(Box::new(CohereAdapter::new_with_overrides(key, base_url)?));
    }

    Ok(Box::new(CohereAdapter::new()?))
}

fn create_perplexity_adapter() -> Result<Box<dyn ChatProvider>, AiLibError> {
    Ok(Box::new(PerplexityAdapter::new()?))
}

fn create_ai21_adapter() -> Result<Box<dyn ChatProvider>, AiLibError> {
    Ok(Box::new(AI21Adapter::new()?))
}

fn resolve_api_key(var: &str) -> Result<String, AiLibError> {
    std::env::var(var)
        .map_err(|_| AiLibError::AuthenticationError(format!("{var} environment variable not set")))
}

fn default_mistral_base() -> String {
    std::env::var("MISTRAL_BASE_URL").unwrap_or_else(|_| "https://api.mistral.ai".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_driven_provider_builds_generic_adapter() {
        std::env::set_var("GROQ_API_KEY", "test-key");
        let adapter = ProviderFactory::create_adapter(Provider::Groq, None, None, None)
            .expect("config-driven adapter builds");
        assert_eq!(adapter.name(), "Groq");
    }

    #[test]
    fn openai_provider_uses_env_key() {
        std::env::set_var("OPENAI_API_KEY", "test-key");
        let adapter = ProviderFactory::create_adapter(
            Provider::OpenAI,
            None,
            Some("https://api.openai.com/v1".to_string()),
            None,
        )
        .expect("independent adapter builds");
        assert_eq!(adapter.name(), "OpenAI");
        std::env::remove_var("OPENAI_API_KEY");
    }
}
