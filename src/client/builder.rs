// AI Client Builder Module
//
// This module contains the AiClientBuilder implementation with progressive
// custom configuration options for creating AiClient instances.

use super::metadata::metadata_from_provider;
use super::{AiClient, Provider, ProviderFactory};
use crate::api::ChatProvider;
use crate::config::{ConnectionOptions, ResilienceConfig};
use crate::metrics::{Metrics, NoopMetrics};
use crate::model::ModelResolver;
use crate::provider::classification::ProviderClassification;
use crate::provider::strategies::{FailoverProvider, RoundRobinProvider, RoutingStrategyBuilder};
use crate::rate_limiter::BackpressureController;
use crate::types::AiLibError;
use std::sync::Arc;

/// AI client builder with progressive custom configuration
pub struct AiClientBuilder {
    provider: Provider,
    base_url: Option<String>,
    proxy_url: Option<String>,
    timeout: Option<std::time::Duration>,
    pool_max_idle: Option<usize>,
    pool_idle_timeout: Option<std::time::Duration>,
    metrics: Option<Arc<dyn Metrics>>,
    // Model configuration options
    default_chat_model: Option<String>,
    default_multimodal_model: Option<String>,
    // Resilience configuration
    resilience_config: ResilienceConfig,
    #[cfg(feature = "interceptors")]
    interceptor_pipeline: Option<crate::interceptors::InterceptorPipeline>,
    #[cfg(feature = "interceptors")]
    interceptor_builder: Option<crate::interceptors::default::DefaultInterceptorsBuilder>,
    // Optional pre-built provider strategy (allows building provider directly)
    strategy: Option<Box<dyn ChatProvider>>,
    model_resolver: Option<Arc<ModelResolver>>,
}

impl AiClientBuilder {
    /// Create a new builder instance
    pub fn new(provider: Provider) -> Self {
        Self {
            provider,
            base_url: None,
            proxy_url: None,
            timeout: None,
            pool_max_idle: None,
            pool_idle_timeout: None,
            metrics: None,
            default_chat_model: None,
            default_multimodal_model: None,
            resilience_config: ResilienceConfig::default(),
            #[cfg(feature = "interceptors")]
            interceptor_pipeline: None,
            #[cfg(feature = "interceptors")]
            interceptor_builder: None,
            strategy: None,
            model_resolver: None,
        }
    }

    /// Set custom base URL
    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = Some(base_url.to_string());
        self
    }

    /// Set custom proxy URL
    pub fn with_proxy(mut self, proxy_url: Option<&str>) -> Self {
        self.proxy_url = proxy_url.map(|s| s.to_string());
        self
    }

    /// Explicitly disable proxy usage
    pub fn without_proxy(mut self) -> Self {
        self.proxy_url = Some("".to_string());
        self
    }

    /// Set custom timeout duration
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set connection pool configuration
    pub fn with_pool_config(mut self, max_idle: usize, idle_timeout: std::time::Duration) -> Self {
        self.pool_max_idle = Some(max_idle);
        self.pool_idle_timeout = Some(idle_timeout);
        self
    }

    /// Set custom metrics implementation
    pub fn with_metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn with_interceptor_pipeline(
        mut self,
        pipeline: crate::interceptors::InterceptorPipeline,
    ) -> Self {
        self.interceptor_pipeline = Some(pipeline);
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_default_interceptors(mut self) -> Self {
        let p = crate::interceptors::create_default_interceptors();
        self.interceptor_pipeline = Some(p);
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_minimal_interceptors(mut self) -> Self {
        let p = crate::interceptors::default::DefaultInterceptorsBuilder::new()
            .enable_circuit_breaker(false)
            .enable_rate_limit(false)
            .build();
        self.interceptor_pipeline = Some(p);
        self
    }

    // --- Interceptor Configuration Proxies ---

    #[cfg(feature = "interceptors")]
    pub fn with_rate_limit(mut self, requests_per_minute: u32) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.with_rate_limit(requests_per_minute));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn with_circuit_breaker(mut self, threshold: u32, recovery: std::time::Duration) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.with_circuit_breaker(threshold, recovery));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn with_retry(
        mut self,
        max_attempts: u32,
        base_delay: std::time::Duration,
        max_delay: std::time::Duration,
    ) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.with_retry(max_attempts, base_delay, max_delay));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn with_interceptor_timeout(mut self, duration: std::time::Duration) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.with_timeout(duration));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_retry(mut self, enable: bool) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.enable_retry(enable));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_circuit_breaker(mut self, enable: bool) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.enable_circuit_breaker(enable));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_rate_limit(mut self, enable: bool) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.enable_rate_limit(enable));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_interceptor_timeout(mut self, enable: bool) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.enable_timeout(enable));
        self
    }

    /// Set default chat model for the client
    pub fn with_default_chat_model(mut self, model: &str) -> Self {
        self.default_chat_model = Some(model.to_string());
        self
    }

    /// Set default multimodal model for the client
    pub fn with_default_multimodal_model(mut self, model: &str) -> Self {
        self.default_multimodal_model = Some(model.to_string());
        self
    }

    /// Inject a custom model resolver (advanced usage).
    pub fn with_model_resolver(mut self, resolver: Arc<ModelResolver>) -> Self {
        self.model_resolver = Some(resolver);
        self
    }

    /// Enable smart defaults for resilience features
    pub fn with_smart_defaults(mut self) -> Self {
        self.resilience_config = ResilienceConfig::smart_defaults();
        self
    }

    /// Configure for production environment
    pub fn for_production(mut self) -> Self {
        self.resilience_config = ResilienceConfig::production();
        self
    }

    /// Configure for development environment
    pub fn for_development(mut self) -> Self {
        self.resilience_config = ResilienceConfig::development();
        self
    }

    /// Build and return a boxed `ChatProvider` according to the current builder configuration.
    /// If a custom `strategy` was provided via `with_strategy`, it will be returned directly.
    pub fn build_provider(mut self) -> Result<Box<dyn ChatProvider>, AiLibError> {
        // If caller supplied a custom strategy, use it directly
        if let Some(p) = self.strategy.take() {
            return Ok(p);
        }

        // Otherwise, construct provider according to builder configuration
        let base_url = self.determine_base_url()?;
        let proxy_url = self.determine_proxy_url();
        let timeout = self
            .timeout
            .unwrap_or_else(|| std::time::Duration::from_secs(30));
        let transport = self.create_custom_transport(proxy_url, timeout)?;

        ProviderFactory::create_adapter(self.provider, None, Some(base_url), transport)
    }

    /// Configure a simple max concurrent requests backpressure guard
    pub fn with_max_concurrency(mut self, max_concurrent_requests: usize) -> Self {
        let mut cfg = self.resilience_config.clone();
        cfg.backpressure = Some(crate::config::BackpressureConfig {
            max_concurrent_requests,
        });
        self.resilience_config = cfg;
        self
    }

    /// Set custom resilience configuration
    pub fn with_resilience_config(mut self, config: ResilienceConfig) -> Self {
        self.resilience_config = config;
        self
    }

    /// Provide a custom provider strategy (boxed ChatProvider)
    ///
    /// This allows injecting a fully custom implementation of `ChatProvider`,
    /// bypassing the standard provider factory logic.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ai_lib::{AiClientBuilder, Provider};
    /// # use ai_lib::provider::strategies::RoundRobinProvider;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a custom strategy (e.g., manually built RoundRobin)
    /// let strategy = RoundRobinProvider::new(vec![])?;
    ///
    /// let client = AiClientBuilder::new(Provider::OpenAI) // Provider enum ignored here
    ///     .with_strategy(Box::new(strategy))
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_strategy(mut self, strategy: Box<dyn ChatProvider>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Compose a round-robin strategy from the provided providers.
    ///
    /// This method takes a collection of boxed `ChatProvider` instances and wraps them
    /// in a `RoundRobinProvider`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ai_lib::{AiClientBuilder, Provider, AiClient};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let p1 = AiClientBuilder::new(Provider::OpenAI).build_provider()?;
    /// let p2 = AiClientBuilder::new(Provider::Anthropic).build_provider()?;
    ///
    /// let client = AiClientBuilder::new(Provider::OpenAI)
    ///     .with_round_robin_strategy(vec![p1, p2])?
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_round_robin_strategy<I>(mut self, providers: I) -> Result<Self, AiLibError>
    where
        I: IntoIterator<Item = Box<dyn ChatProvider>>,
    {
        let providers_vec: Vec<_> = providers.into_iter().collect();
        let rr = RoundRobinProvider::new(providers_vec)?;
        self.strategy = Some(Box::new(rr));
        Ok(self)
    }

    /// Compose a failover strategy from the provided providers.
    pub fn with_failover_strategy<I>(mut self, providers: I) -> Result<Self, AiLibError>
    where
        I: IntoIterator<Item = Box<dyn ChatProvider>>,
    {
        let providers_vec: Vec<_> = providers.into_iter().collect();
        let failover = FailoverProvider::new(providers_vec)?;
        self.strategy = Some(Box::new(failover));
        Ok(self)
    }

    /// Compose a round-robin strategy from built-in `Provider` variants.
    pub fn with_round_robin_chain(mut self, providers: Vec<Provider>) -> Result<Self, AiLibError> {
        let adapters = Self::build_strategy_chain(providers)?;
        let rr = RoundRobinProvider::new(adapters)?;
        self.strategy = Some(Box::new(rr));
        Ok(self)
    }

    /// Compose a failover strategy from built-in `Provider` variants.
    pub fn with_failover_chain(mut self, providers: Vec<Provider>) -> Result<Self, AiLibError> {
        let adapters = Self::build_strategy_chain(providers)?;
        let failover = FailoverProvider::new(adapters)?;
        self.strategy = Some(Box::new(failover));
        Ok(self)
    }

    /// Use `RoutingStrategyBuilder` to configure a round-robin strategy inline.
    pub fn with_round_robin_builder(
        mut self,
        builder: RoutingStrategyBuilder,
    ) -> Result<Self, AiLibError> {
        let rr = builder.build_round_robin()?;
        self.strategy = Some(Box::new(rr));
        Ok(self)
    }

    /// Use `RoutingStrategyBuilder` to configure a failover strategy inline.
    pub fn with_failover_builder(
        mut self,
        builder: RoutingStrategyBuilder,
    ) -> Result<Self, AiLibError> {
        let failover = builder.build_failover()?;
        self.strategy = Some(Box::new(failover));
        Ok(self)
    }

    /// Build AiClient instance
    pub fn build(self) -> Result<AiClient, AiLibError> {
        let mut builder = self;
        // 1. Determine base_url: explicit setting > environment variable > default
        let base_url = builder.determine_base_url()?;

        // 2. Determine proxy_url: explicit setting > environment variable
        let proxy_url = builder.determine_proxy_url();

        // 3. Determine timeout: explicit setting > default
        let timeout = builder
            .timeout
            .unwrap_or_else(|| std::time::Duration::from_secs(30));

        // 4. Create provider strategy (custom or factory-built)
        let transport = builder.create_custom_transport(proxy_url, timeout)?;
        let chat_provider = if let Some(strategy) = builder.strategy.take() {
            strategy
        } else {
            ProviderFactory::create_adapter(
                builder.provider,
                None, // API key handled by config or env
                Some(base_url.clone()),
                transport,
            )?
        };

        // 5. Build backpressure controller if configured
        let bp_ctrl: Option<Arc<BackpressureController>> = builder
            .resilience_config
            .backpressure
            .as_ref()
            .map(|cfg| Arc::new(BackpressureController::new(cfg.max_concurrent_requests)));

        // 6. Create AiClient
        let metadata = metadata_from_provider(
            builder.provider,
            chat_provider.name().to_string(),
            Some(base_url.clone()),
            None,
            None,
        );

        let model_resolver = builder
            .model_resolver
            .unwrap_or_else(|| Arc::new(ModelResolver::new()));

        let client = AiClient {
            chat_provider,
            metadata,
            metrics: builder
                .metrics
                .unwrap_or_else(|| Arc::new(NoopMetrics::new())),
            model_resolver,
            connection_options: None,
            custom_default_chat_model: builder.default_chat_model,
            custom_default_multimodal_model: builder.default_multimodal_model,
            backpressure: bp_ctrl,
            #[cfg(feature = "interceptors")]
            interceptor_pipeline: builder
                .interceptor_pipeline
                .or_else(|| builder.interceptor_builder.map(|b| b.build())),
        };

        Ok(client)
    }

    /// Determine base_url, priority: explicit setting > environment variable > default
    fn determine_base_url(&self) -> Result<String, AiLibError> {
        resolve_base_url(self.provider, self.base_url.clone())
    }

    fn build_strategy_chain(
        providers: Vec<Provider>,
    ) -> Result<Vec<Box<dyn ChatProvider>>, AiLibError> {
        if providers.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "routing strategy requires at least one provider".to_string(),
            ));
        }
        providers
            .into_iter()
            .map(|provider| Self::create_adapter_from_env(provider))
            .collect()
    }

    fn create_adapter_from_env(provider: Provider) -> Result<Box<dyn ChatProvider>, AiLibError> {
        let opts = ConnectionOptions::default().hydrate_with_env(provider.env_prefix());
        let resolved_base_url = resolve_base_url(provider, opts.base_url.clone())?;
        let transport = Self::transport_from_options(&opts)?;
        ProviderFactory::create_adapter(
            provider,
            opts.api_key.clone(),
            Some(resolved_base_url),
            transport,
        )
    }

    fn transport_from_options(
        opts: &ConnectionOptions,
    ) -> Result<Option<crate::transport::DynHttpTransportRef>, AiLibError> {
        let effective_proxy = if opts.disable_proxy {
            None
        } else {
            opts.proxy.clone()
        };

        if effective_proxy.is_none() && opts.timeout.is_none() {
            return Ok(None);
        }

        let transport_config = crate::transport::HttpTransportConfig {
            timeout: opts
                .timeout
                .unwrap_or_else(|| std::time::Duration::from_secs(30)),
            proxy: effective_proxy,
            pool_max_idle_per_host: None,
            pool_idle_timeout: None,
        };
        Ok(Some(
            crate::transport::HttpTransport::new_with_config(transport_config)?.boxed(),
        ))
    }

    /// Determine proxy_url, priority: explicit setting > environment variable
    fn determine_proxy_url(&self) -> Option<String> {
        // 1. Explicitly set proxy_url
        if let Some(ref proxy_url) = self.proxy_url {
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
        &self,
        proxy_url: Option<String>,
        timeout: std::time::Duration,
    ) -> Result<Option<crate::transport::DynHttpTransportRef>, AiLibError> {
        // If no custom configuration, return None (use default transport)
        if proxy_url.is_none() && self.pool_max_idle.is_none() && self.pool_idle_timeout.is_none() {
            return Ok(None);
        }

        // Create custom HttpTransportConfig
        let transport_config = crate::transport::HttpTransportConfig {
            timeout,
            proxy: proxy_url,
            pool_max_idle_per_host: self.pool_max_idle,
            pool_idle_timeout: self.pool_idle_timeout,
        };

        // Create custom HttpTransport
        let transport = crate::transport::HttpTransport::new_with_config(transport_config)?;
        Ok(Some(transport.boxed()))
    }
}

pub(crate) fn resolve_base_url(
    provider: Provider,
    explicit: Option<String>,
) -> Result<String, AiLibError> {
    if let Some(url) = explicit {
        return Ok(url);
    }

    if let Some(env_var) = base_url_env_var(provider) {
        if let Ok(value) = std::env::var(env_var) {
            return Ok(value);
        }
    }

    if provider.is_config_driven() {
        return provider.get_default_config().map(|config| config.base_url);
    }

    default_base_url(provider)
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
