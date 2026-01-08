// AI Client Builder Module
//
// This module contains the AiClientBuilder implementation with progressive
// custom configuration options for creating AiClient instances.

use super::config_merger::ConfigMerger;
use super::metadata::metadata_from_provider;
use super::registry_resolver::RegistryResolver;
use super::transport_builder::TransportBuilder;
use super::{AiClient, Provider, ProviderFactory};
use crate::api::ChatProvider;
use crate::config::{ConnectionOptions, ResilienceConfig};
use crate::metrics::{Metrics, NoopMetrics};
use crate::model::ModelResolver;
use crate::provider::strategies::{FailoverProvider, RoundRobinProvider, RoutingStrategyBuilder};
use crate::rate_limiter::BackpressureController;
// use crate::transport::DynHttpTransportRef;
use crate::types::AiLibError;
use std::sync::Arc;

pub mod interceptors;

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
    // v0.5.0: Specific model ID to resolve from Registry
    model_id: Option<String>,
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
            model_id: None,
        }
    }

    /// Set custom base URL
    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = Some(base_url.to_string());
        self
    }

    /// Select a specific model from the Registry (v0.5.0)
    pub fn with_model(mut self, model_id: &str) -> Self {
        self.model_id = Some(model_id.to_string());
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

        // v0.5.0: Registry Resolution Logic using RegistryResolver
        let (resolved_protocol, resolved_provider_config) =
            if let Some(ref model_id) = self.model_id {
                // Path A: Model-driven (Preferred)
                let (protocol, config) = RegistryResolver::resolve_model_driven(model_id)?;
                (protocol, Some(config))
            } else {
                // Path B: Legacy Provider-driven
                let (protocol, config) = RegistryResolver::resolve_provider_driven(self.provider);
                (protocol, config)
            };

        // Setup Transport using TransportBuilder
        let timeout = self
            .timeout
            .unwrap_or_else(|| std::time::Duration::from_secs(30));
        let (base_url, transport) = TransportBuilder::build(
            self.provider,
            self.base_url.clone(),
            self.proxy_url.clone(),
            Some(timeout),
            self.pool_max_idle,
            self.pool_idle_timeout,
        )?;

        // Factory Call
        if let Some(config) = resolved_provider_config {
            // Merge config with builder overrides
            let merged_config = ConfigMerger::merge_provider_config(
                config,
                self.base_url.clone(),
                base_url.clone(),
            );

            ProviderFactory::create(&resolved_protocol, merged_config, transport)
        } else {
            // Legacy Fallback
            ProviderFactory::create_adapter(self.provider, None, Some(base_url), transport)
        }
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
    pub fn with_strategy(mut self, strategy: Box<dyn ChatProvider>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Compose a round-robin strategy from the provided providers.
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
        let adapters = super::helpers::build_strategy_chain(providers)?;
        let rr = RoundRobinProvider::new(adapters)?;
        self.strategy = Some(Box::new(rr));
        Ok(self)
    }

    /// Compose a failover strategy from built-in `Provider` variants.
    pub fn with_failover_chain(mut self, providers: Vec<Provider>) -> Result<Self, AiLibError> {
        let adapters = super::helpers::build_strategy_chain(providers)?;
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
        // 1-3. Setup Transport using TransportBuilder
        let timeout = builder
            .timeout
            .unwrap_or_else(|| std::time::Duration::from_secs(30));
        let (base_url, transport) = TransportBuilder::build(
            builder.provider,
            builder.base_url.clone(),
            builder.proxy_url.clone(),
            Some(timeout),
            builder.pool_max_idle,
            builder.pool_idle_timeout,
        )?;

        // 4. Create provider strategy (custom or factory-built)
        let chat_provider = if let Some(strategy) = builder.strategy.take() {
            strategy
        } else {
            // Reuse the shared build_provider logic using new modules
            // v0.5.0: Registry Resolution Logic using RegistryResolver
            let (resolved_protocol, resolved_provider_config) =
                if let Some(ref model_id) = builder.model_id {
                    // Path A: Model-driven
                    let (protocol, config) = RegistryResolver::resolve_model_driven(model_id)?;
                    (protocol, Some(config))
                } else {
                    // Path B: Legacy Provider-driven
                    let (protocol, config) =
                        RegistryResolver::resolve_provider_driven(builder.provider);
                    (protocol, config)
                };

            if let Some(config) = resolved_provider_config {
                // Merge config with builder overrides
                let merged_config = ConfigMerger::merge_provider_config(
                    config,
                    builder.base_url.clone(),
                    base_url.clone(),
                );
                ProviderFactory::create(&resolved_protocol, merged_config, transport)?
            } else {
                ProviderFactory::create_adapter(
                    builder.provider,
                    None,
                    Some(base_url.clone()),
                    transport,
                )?
            }
        };

        // 5. Build backpressure controller if configured
        let bp_ctrl: Option<Arc<BackpressureController>> = builder
            .resilience_config
            .backpressure
            .as_ref()
            .map(|cfg| Arc::new(BackpressureController::new(cfg.max_concurrent_requests)));

        // Use pre-built strategy if available
        let chat_provider = if let Some(strategy_provider) = builder.strategy {
            strategy_provider
        } else {
            chat_provider
        };

        // 6. Create AiClient
        let metadata = metadata_from_provider(
            builder.provider,
            chat_provider.name().to_string(),
            Some(base_url.clone()),
            None,
            None,
        );

        let options = ConnectionOptions {
            base_url: builder.base_url.clone(),
            proxy: builder.proxy_url.clone(),
            timeout: builder.timeout,
            api_key: None,        // Builder doesn't store API key directly yet
            disable_proxy: false, // Default
        };

        let client = AiClient {
            chat_provider,
            metadata,
            metrics: builder
                .metrics
                .unwrap_or_else(|| Arc::new(NoopMetrics::new())),
            model_resolver: builder
                .model_resolver
                .unwrap_or_else(|| Arc::new(ModelResolver::new())),
            connection_options: Some(options),
            custom_default_chat_model: builder.default_chat_model,
            custom_default_multimodal_model: builder.default_multimodal_model,
            // Backpressure logic moved to rate_limiter module, but builder sets it up
            backpressure: bp_ctrl,
            #[cfg(feature = "interceptors")]
            interceptor_pipeline: builder
                .interceptor_pipeline
                .or_else(|| builder.interceptor_builder.map(|b| b.build())),
        };

        Ok(client)
    }
}
