use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    api::ChatProvider,
    client::{AiClient, AiClientBuilder, Provider},
    config::ResilienceConfig,
    metrics::Metrics,
    provider::{chat_provider::AdapterProvider, config::ProviderConfig, generic::GenericAdapter},
    transport::DynHttpTransportRef,
    types::AiLibError,
};

macro_rules! define_provider_builder {
    ($name:ident, $provider_variant:expr) => {
        pub struct $name {
            inner: AiClientBuilder,
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    inner: AiClientBuilder::new($provider_variant),
                }
            }

            pub fn with_base_url(mut self, base_url: &str) -> Self {
                self.inner = self.inner.with_base_url(base_url);
                self
            }

            pub fn with_proxy(mut self, proxy_url: Option<&str>) -> Self {
                self.inner = self.inner.with_proxy(proxy_url);
                self
            }

            pub fn without_proxy(mut self) -> Self {
                self.inner = self.inner.without_proxy();
                self
            }

            pub fn with_timeout(mut self, timeout: Duration) -> Self {
                self.inner = self.inner.with_timeout(timeout);
                self
            }

            pub fn with_pool_config(mut self, max_idle: usize, idle_timeout: Duration) -> Self {
                self.inner = self.inner.with_pool_config(max_idle, idle_timeout);
                self
            }

            pub fn with_metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
                self.inner = self.inner.with_metrics(metrics);
                self
            }

            pub fn with_default_chat_model(mut self, model: &str) -> Self {
                self.inner = self.inner.with_default_chat_model(model);
                self
            }

            pub fn with_default_multimodal_model(mut self, model: &str) -> Self {
                self.inner = self.inner.with_default_multimodal_model(model);
                self
            }

            pub fn with_smart_defaults(mut self) -> Self {
                self.inner = self.inner.with_smart_defaults();
                self
            }

            pub fn for_production(mut self) -> Self {
                self.inner = self.inner.for_production();
                self
            }

            pub fn for_development(mut self) -> Self {
                self.inner = self.inner.for_development();
                self
            }

            pub fn with_max_concurrency(mut self, max: usize) -> Self {
                self.inner = self.inner.with_max_concurrency(max);
                self
            }

            pub fn with_resilience_config(mut self, config: ResilienceConfig) -> Self {
                self.inner = self.inner.with_resilience_config(config);
                self
            }

            pub fn with_strategy(mut self, strategy: Box<dyn ChatProvider>) -> Self {
                self.inner = self.inner.with_strategy(strategy);
                self
            }

            pub fn build(self) -> Result<AiClient, AiLibError> {
                self.inner.build()
            }

            pub fn build_provider(self) -> Result<Box<dyn ChatProvider>, AiLibError> {
                self.inner.build_provider()
            }
        }
    };
}

define_provider_builder!(GroqBuilder, Provider::Groq);
define_provider_builder!(XaiGrokBuilder, Provider::XaiGrok);
define_provider_builder!(OllamaBuilder, Provider::Ollama);
define_provider_builder!(DeepSeekBuilder, Provider::DeepSeek);
define_provider_builder!(AnthropicBuilder, Provider::Anthropic);
define_provider_builder!(AzureOpenAiBuilder, Provider::AzureOpenAI);
define_provider_builder!(HuggingFaceBuilder, Provider::HuggingFace);
define_provider_builder!(TogetherAiBuilder, Provider::TogetherAI);
define_provider_builder!(OpenRouterBuilder, Provider::OpenRouter);
define_provider_builder!(ReplicateBuilder, Provider::Replicate);
define_provider_builder!(BaiduWenxinBuilder, Provider::BaiduWenxin);
define_provider_builder!(TencentHunyuanBuilder, Provider::TencentHunyuan);
define_provider_builder!(IflytekSparkBuilder, Provider::IflytekSpark);
define_provider_builder!(MoonshotBuilder, Provider::Moonshot);
define_provider_builder!(QwenBuilder, Provider::Qwen);
define_provider_builder!(ZhipuAiBuilder, Provider::ZhipuAI);
define_provider_builder!(MiniMaxBuilder, Provider::MiniMax);
define_provider_builder!(OpenAiBuilder, Provider::OpenAI);
define_provider_builder!(GeminiBuilder, Provider::Gemini);
define_provider_builder!(MistralBuilder, Provider::Mistral);
define_provider_builder!(CohereBuilder, Provider::Cohere);
define_provider_builder!(PerplexityBuilder, Provider::Perplexity);
define_provider_builder!(Ai21Builder, Provider::AI21);

/// Builder for OpenAI-compatible custom providers without editing the `Provider` enum.
///
/// This builder allows you to create a `ChatProvider` for any service that exposes
/// an OpenAI-compatible API, even if it's not natively supported by the library.
///
/// # Example
///
/// ```rust
/// # use ai_lib::provider::builders::CustomProviderBuilder;
/// # use ai_lib::AiClientBuilder;
/// # use ai_lib::Provider;
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a provider for a hypothetical service "MyService"
/// let my_provider = CustomProviderBuilder::new("MyService")
///     .with_base_url("https://api.myservice.com/v1")
///     .with_api_key_env("MY_SERVICE_API_KEY")
///     .with_default_chat_model("my-model-v1")
///     .build_provider()?;
///
/// // Inject it into the client
/// let client = AiClientBuilder::new(Provider::OpenAI) // Enum ignored
///     .with_strategy(my_provider)
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct CustomProviderBuilder {
    name: String,
    base_url: Option<String>,
    api_key_env: Option<String>,
    api_key_override: Option<String>,
    chat_model: Option<String>,
    multimodal_model: Option<String>,
    chat_endpoint: String,
    upload_endpoint: Option<String>,
    models_endpoint: Option<String>,
    headers: HashMap<String, String>,
    transport: Option<DynHttpTransportRef>,
}

impl CustomProviderBuilder {
    /// Create a new builder with the human-readable provider name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base_url: None,
            api_key_env: None,
            api_key_override: None,
            chat_model: None,
            multimodal_model: None,
            chat_endpoint: "/chat/completions".to_string(),
            upload_endpoint: Some("/v1/files".to_string()),
            models_endpoint: Some("/models".to_string()),
            headers: HashMap::new(),
            transport: None,
        }
    }

    /// Set the base URL (required) for the custom provider.
    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = Some(base_url.to_string());
        self
    }

    /// Override the environment variable used to fetch the API key at runtime.
    pub fn with_api_key_env(mut self, env_var: &str) -> Self {
        self.api_key_env = Some(env_var.to_string());
        self
    }

    /// Inject a literal API key instead of relying on environment variables.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key_override = Some(api_key.into());
        self
    }

    /// Override the default chat model used for simple helpers.
    pub fn with_default_chat_model(mut self, model: &str) -> Self {
        self.chat_model = Some(model.to_string());
        self
    }

    /// Override the default multimodal model (optional).
    pub fn with_default_multimodal_model(mut self, model: &str) -> Self {
        self.multimodal_model = Some(model.to_string());
        self
    }

    /// Override the chat completion endpoint (default: `/chat/completions`).
    pub fn with_chat_endpoint(mut self, endpoint: &str) -> Self {
        self.chat_endpoint = endpoint.to_string();
        self
    }

    /// Override the upload endpoint (default: `/v1/files`).
    pub fn with_upload_endpoint(mut self, endpoint: Option<&str>) -> Self {
        self.upload_endpoint = endpoint.map(|e| e.to_string());
        self
    }

    /// Override the models endpoint (default: `/models`).
    pub fn with_models_endpoint(mut self, endpoint: Option<&str>) -> Self {
        self.models_endpoint = endpoint.map(|e| e.to_string());
        self
    }

    /// Merge custom HTTP headers (e.g., vendor-specific auth scopes).
    pub fn with_headers<I, K, V>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in headers {
            self.headers.insert(k.into(), v.into());
        }
        self
    }

    /// Provide a pre-built transport (shared client, proxy, custom TLS, etc.).
    pub fn with_transport(mut self, transport: DynHttpTransportRef) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Build a boxed `ChatProvider` that can be passed to `AiClientBuilder::with_strategy`.
    pub fn build_provider(self) -> Result<Box<dyn ChatProvider>, AiLibError> {
        let base_url = self.base_url.ok_or_else(|| {
            AiLibError::ConfigurationError(
                "CustomProviderBuilder requires `with_base_url` to be set".to_string(),
            )
        })?;

        let chat_model = self
            .chat_model
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string());
        let env_key = self.api_key_env.unwrap_or_else(|| {
            let upper = self
                .name
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_uppercase()
                    } else {
                        '_'
                    }
                })
                .collect::<String>();
            format!("{upper}_API_KEY")
        });

        let mut config = ProviderConfig::openai_compatible(
            &base_url,
            &env_key,
            &chat_model,
            self.multimodal_model.as_deref(),
        );
        config.chat_endpoint = self.chat_endpoint;
        config.upload_endpoint = self.upload_endpoint;
        config.models_endpoint = self.models_endpoint;
        config.headers.extend(self.headers);

        let adapter = match (self.transport, self.api_key_override) {
            (Some(transport), api_key) => {
                GenericAdapter::with_transport_ref_api_key(config, transport, api_key)?
            }
            (None, api_key) => GenericAdapter::new_with_api_key(config, api_key)?,
        };

        Ok(AdapterProvider::new(self.name, Box::new(adapter)).boxed())
    }
}
