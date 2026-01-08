//! Transport builder module
//!
//! This module handles HTTP transport configuration and creation.
//! It extracts transport-related logic from builder.rs to improve modularity.

use crate::client::Provider;
use crate::config::ConnectionOptions;
use crate::provider::classification::ProviderClassification;
use crate::transport::{DynHttpTransportRef, HttpTransport, HttpTransportConfig};
use crate::types::AiLibError;

/// Transport builder for creating HTTP transport instances
pub struct TransportBuilder;

impl TransportBuilder {
    /// Build transport from builder options
    ///
    /// # Arguments
    /// * `provider` - Provider enum for base URL resolution
    /// * `base_url_override` - Optional explicit base URL
    /// * `proxy_url` - Optional proxy URL
    /// * `timeout` - Optional timeout duration
    /// * `pool_max_idle` - Optional connection pool max idle connections
    /// * `pool_idle_timeout` - Optional connection pool idle timeout
    ///
    /// # Returns
    /// * `Ok((base_url, transport))` - Resolved base URL and optional transport
    pub fn build(
        provider: Provider,
        base_url_override: Option<String>,
        proxy_url: Option<String>,
        timeout: Option<std::time::Duration>,
        pool_max_idle: Option<usize>,
        pool_idle_timeout: Option<std::time::Duration>,
    ) -> Result<(String, Option<DynHttpTransportRef>), AiLibError> {
        let base_url = Self::determine_base_url(provider, base_url_override)?;
        let resolved_proxy = Self::determine_proxy_url(proxy_url);
        let transport = Self::create_custom_transport(
            resolved_proxy,
            timeout,
            pool_max_idle,
            pool_idle_timeout,
        )?;
        Ok((base_url, transport))
    }

    /// Determine base_url, priority: explicit setting > environment variable > default
    /// Determine base_url, priority: explicit setting > environment variable > default
    pub fn determine_base_url(
        provider: Provider,
        explicit: Option<String>,
    ) -> Result<String, AiLibError> {
        if let Some(url) = explicit {
            return Ok(url);
        }

        if let Some(env_var) = Self::base_url_env_var(provider) {
            if let Ok(value) = std::env::var(env_var) {
                return Ok(value);
            }
        }

        if provider.is_config_driven() {
            return provider.get_default_config().map(|config| config.base_url);
        }

        Self::default_base_url(provider)
    }

    fn base_url_env_var(provider: Provider) -> Option<&'static str> {
        match provider {
            Provider::Groq => Some("GROQ_BASE_URL"),
            Provider::XaiGrok => Some("GROK_BASE_URL"),
            Provider::Ollama => Some("OLLAMA_BASE_URL"),
            Provider::DeepSeek => Some("DEEPSEEK_BASE_URL"),
            Provider::Qwen => Some("DASHSCOPE_BASE_URL"),
            Provider::BaiduWenxin => Some("BAIDU_WENXIN_BASE_URL"),
            Provider::TencentHunyuan => Some("TENCENT_HUNYUAN_BASE_URL"),
            Provider::IflytekSpark => Some("IFLYTEK_BASE_URL"),
            Provider::Moonshot => Some("MOONSHOT_BASE_URL"),
            Provider::Anthropic => Some("ANTHROPIC_BASE_URL"),
            Provider::AzureOpenAI => Some("AZURE_OPENAI_BASE_URL"),
            Provider::HuggingFace => Some("HUGGINGFACE_BASE_URL"),
            Provider::TogetherAI => Some("TOGETHER_BASE_URL"),
            Provider::OpenRouter => Some("OPENROUTER_BASE_URL"),
            Provider::Replicate => Some("REPLICATE_BASE_URL"),
            Provider::ZhipuAI => Some("ZHIPU_BASE_URL"),
            Provider::MiniMax => Some("MINIMAX_BASE_URL"),
            Provider::Perplexity => Some("PERPLEXITY_BASE_URL"),
            Provider::AI21 => Some("AI21_BASE_URL"),
            // Independent adapters use fixed endpoints
            Provider::OpenAI | Provider::Gemini | Provider::Mistral | Provider::Cohere => None,
        }
    }

    fn default_base_url(provider: Provider) -> Result<String, AiLibError> {
        match provider {
            Provider::OpenAI => Ok("https://api.openai.com".to_string()),
            Provider::Gemini => Ok("https://generativelanguage.googleapis.com".to_string()),
            Provider::Mistral => Ok("https://api.mistral.ai".to_string()),
            Provider::Cohere => Ok("https://api.cohere.ai".to_string()),
            other => Err(AiLibError::ConfigurationError(format!(
                "Unknown provider for base URL determination: {other:?}"
            ))),
        }
    }

    /// Determine proxy_url, priority: explicit setting > environment variable
    pub fn determine_proxy_url(explicit: Option<String>) -> Option<String> {
        // 1. Explicitly set proxy_url
        if let Some(ref proxy_url) = explicit {
            // If proxy_url is empty string, it means explicitly no proxy
            if proxy_url.is_empty() {
                return None;
            }
            return Some(proxy_url.clone());
        }

        // 2. AI_PROXY_URL from environment variable
        std::env::var("AI_PROXY_URL").ok()
    }

    /// Create custom HttpTransport
    fn create_custom_transport(
        proxy_url: Option<String>,
        timeout: Option<std::time::Duration>,
        pool_max_idle: Option<usize>,
        pool_idle_timeout: Option<std::time::Duration>,
    ) -> Result<Option<DynHttpTransportRef>, AiLibError> {
        // If no custom configuration, return None (use default transport)
        if proxy_url.is_none() && pool_max_idle.is_none() && pool_idle_timeout.is_none() {
            return Ok(None);
        }

        // Create custom HttpTransportConfig
        let transport_config = HttpTransportConfig {
            timeout: timeout.unwrap_or_else(|| std::time::Duration::from_secs(30)),
            proxy: proxy_url,
            pool_max_idle_per_host: pool_max_idle,
            pool_idle_timeout,
        };

        // Create custom HttpTransport
        let transport = HttpTransport::new_with_config(transport_config)?;
        Ok(Some(transport.boxed()))
    }

    /// Build transport from ConnectionOptions
    pub fn from_options(
        opts: &ConnectionOptions,
    ) -> Result<Option<DynHttpTransportRef>, AiLibError> {
        let effective_proxy = if opts.disable_proxy {
            None
        } else {
            opts.proxy.clone()
        };

        if effective_proxy.is_none() && opts.timeout.is_none() {
            return Ok(None);
        }

        let transport_config = HttpTransportConfig {
            timeout: opts
                .timeout
                .unwrap_or_else(|| std::time::Duration::from_secs(30)),
            proxy: effective_proxy,
            pool_max_idle_per_host: None,
            pool_idle_timeout: None,
        };
        Ok(Some(
            HttpTransport::new_with_config(transport_config)?.boxed(),
        ))
    }
}
