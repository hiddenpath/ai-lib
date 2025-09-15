use crate::client::Provider;
use crate::provider::config::ProviderConfig;
use crate::provider::configs::ProviderConfigs;

/// Provider classification trait defining behavior patterns
pub trait ProviderClassification {
    /// Check if this provider is config-driven (uses GenericAdapter)
    fn is_config_driven(&self) -> bool;

    /// Check if this provider supports custom configuration
    fn supports_custom_config(&self) -> bool;

    /// Get the adapter type for this provider
    fn adapter_type(&self) -> AdapterType;

    /// Get the default configuration for this provider
    fn get_default_config(&self) -> Result<ProviderConfig, crate::types::AiLibError>;
}

/// Types of adapters used by providers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterType {
    /// Uses GenericAdapter with ProviderConfig
    ConfigDriven,
    /// Uses independent adapter (OpenAI, Gemini, Mistral, Cohere)
    Independent,
}

impl ProviderClassification for Provider {
    fn is_config_driven(&self) -> bool {
        CONFIG_DRIVEN_PROVIDERS.contains(self)
    }

    fn supports_custom_config(&self) -> bool {
        CONFIG_DRIVEN_PROVIDERS.contains(self)
    }

    fn adapter_type(&self) -> AdapterType {
        if CONFIG_DRIVEN_PROVIDERS.contains(self) {
            AdapterType::ConfigDriven
        } else {
            AdapterType::Independent
        }
    }

    fn get_default_config(&self) -> Result<ProviderConfig, crate::types::AiLibError> {
        match self {
            // Config-driven providers
            Provider::Groq => Ok(ProviderConfigs::groq()),
            Provider::XaiGrok => Ok(ProviderConfigs::xai_grok()),
            Provider::Ollama => Ok(ProviderConfigs::ollama()),
            Provider::DeepSeek => Ok(ProviderConfigs::deepseek()),
            Provider::Qwen => Ok(ProviderConfigs::qwen()),
            Provider::BaiduWenxin => Ok(ProviderConfigs::baidu_wenxin()),
            Provider::TencentHunyuan => Ok(ProviderConfigs::tencent_hunyuan()),
            Provider::IflytekSpark => Ok(ProviderConfigs::iflytek_spark()),
            Provider::Moonshot => Ok(ProviderConfigs::moonshot()),
            Provider::Anthropic => Ok(ProviderConfigs::anthropic()),
            Provider::AzureOpenAI => Ok(ProviderConfigs::azure_openai()),
            Provider::HuggingFace => Ok(ProviderConfigs::huggingface()),
            Provider::TogetherAI => Ok(ProviderConfigs::together_ai()),
            Provider::OpenRouter => Ok(ProviderConfigs::openrouter()),
            Provider::Replicate => Ok(ProviderConfigs::replicate()),
            Provider::ZhipuAI => Ok(ProviderConfigs::zhipu_ai()),
            Provider::MiniMax => Ok(ProviderConfigs::minimax()),

            // Independent providers don't support custom configuration
            Provider::OpenAI | Provider::Gemini | Provider::Mistral | Provider::Cohere | 
            Provider::Perplexity | Provider::AI21 => {
                Err(crate::types::AiLibError::ConfigurationError(
                    "This provider does not support custom configuration".to_string(),
                ))
            }
        }
    }
}

/// System-level provider classification constants
/// These arrays define the authoritative source of truth for provider behavior.
/// All modules should use these constants instead of hardcoding provider lists.
/// Providers that use GenericAdapter with ProviderConfig
pub const CONFIG_DRIVEN_PROVIDERS: &[Provider] = &[
    // Core config-driven providers
    Provider::Groq,
    Provider::XaiGrok,
    Provider::Ollama,
    Provider::DeepSeek,
    Provider::Anthropic,
    Provider::AzureOpenAI,
    Provider::HuggingFace,
    Provider::TogetherAI,
    Provider::OpenRouter,
    Provider::Replicate,
    // Chinese providers (config-driven)
    Provider::BaiduWenxin,
    Provider::TencentHunyuan,
    Provider::IflytekSpark,
    Provider::Moonshot,
    Provider::Qwen,
    Provider::ZhipuAI,
    Provider::MiniMax,
];

/// Providers that use independent adapters
pub const INDEPENDENT_PROVIDERS: &[Provider] = &[
    Provider::OpenAI,
    Provider::Gemini,
    Provider::Mistral,
    Provider::Cohere,
    Provider::Perplexity,
    Provider::AI21,
];

/// All supported providers
pub const ALL_PROVIDERS: &[Provider] = &[
    // Config-driven providers
    Provider::Groq,
    Provider::XaiGrok,
    Provider::Ollama,
    Provider::DeepSeek,
    Provider::Anthropic,
    Provider::AzureOpenAI,
    Provider::HuggingFace,
    Provider::TogetherAI,
    Provider::OpenRouter,
    Provider::Replicate,
    // Chinese providers
    Provider::BaiduWenxin,
    Provider::TencentHunyuan,
    Provider::IflytekSpark,
    Provider::Moonshot,
    Provider::Qwen,
    Provider::ZhipuAI,
    Provider::MiniMax,
    // Independent providers
    Provider::OpenAI,
    Provider::Gemini,
    Provider::Mistral,
    Provider::Cohere,
    Provider::Perplexity,
    Provider::AI21,
];

/// Helper functions for provider classification
impl Provider {
    /// Check if this provider is config-driven
    pub fn is_config_driven(&self) -> bool {
        CONFIG_DRIVEN_PROVIDERS.contains(self)
    }

    /// Check if this provider is independent
    pub fn is_independent(&self) -> bool {
        INDEPENDENT_PROVIDERS.contains(self)
    }

    /// Get all config-driven providers
    pub fn config_driven_providers() -> &'static [Provider] {
        CONFIG_DRIVEN_PROVIDERS
    }

    /// Get all independent providers
    pub fn independent_providers() -> &'static [Provider] {
        INDEPENDENT_PROVIDERS
    }

    /// Get all supported providers
    pub fn all_providers() -> &'static [Provider] {
        ALL_PROVIDERS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_classification() {
        // Test config-driven providers
        assert!(Provider::Groq.is_config_driven());
        assert!(Provider::Anthropic.is_config_driven());
        assert!(Provider::BaiduWenxin.is_config_driven());

        // Test independent providers
        assert!(Provider::OpenAI.is_independent());
        assert!(Provider::Gemini.is_independent());
        assert!(Provider::Mistral.is_independent());
        assert!(Provider::Cohere.is_independent());

        // Test adapter types
        assert_eq!(Provider::Groq.adapter_type(), AdapterType::ConfigDriven);
        assert_eq!(Provider::OpenAI.adapter_type(), AdapterType::Independent);
    }

    #[test]
    fn test_provider_arrays() {
        // Ensure all providers are covered
        let config_driven_count = CONFIG_DRIVEN_PROVIDERS.len();
        let independent_count = INDEPENDENT_PROVIDERS.len();
        let all_count = ALL_PROVIDERS.len();

        assert_eq!(config_driven_count + independent_count, all_count);

        // Ensure no duplicates by checking each provider appears only once
        for provider in ALL_PROVIDERS {
            let count = ALL_PROVIDERS.iter().filter(|&&p| p == *provider).count();
            assert_eq!(count, 1, "Provider {:?} appears {} times", provider, count);
        }
    }
}
