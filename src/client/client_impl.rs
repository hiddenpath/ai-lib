use crate::api::ChatCompletionChunk;
use crate::api::ChatProvider;
use crate::config::ConnectionOptions;
use crate::metrics::{Metrics, NoopMetrics};
use crate::model::{ModelResolution, ModelResolutionSource, ModelResolver};
use crate::rate_limiter::BackpressureController;
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};
use futures::stream::Stream;
use std::sync::Arc;

use super::builder::AiClientBuilder;
use super::helpers;
use super::metadata::{metadata_from_provider, ClientMetadata};
use super::model_options::ModelOptions;
use super::provider::Provider;
use super::stream::CancelHandle;
use super::{batch, request, stream, ProviderFactory};

/// 统一的AI客户端，提供跨厂商的AI服务访问接口
///
/// Unified AI client
///
/// Usage example:
/// ```rust
/// use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Switch model provider by changing Provider value
///     let client = AiClient::new(Provider::Groq)?;
///     
///     let request = ChatCompletionRequest::new(
///         "test-model".to_string(),
///         vec![Message {
///             role: Role::User,
///             content: ai_lib::types::common::Content::Text("Hello".to_string()),
///             function_call: None,
///         }],
///     );
///     
///     // Note: Set GROQ_API_KEY environment variable for actual API calls
///     // Optional: Set AI_PROXY_URL environment variable to use proxy server
///     // let response = client.chat_completion(request).await?;
///     
///     println!(
///         "Client created successfully with provider: {}",
///         client.provider_name()
///     );
///     println!("Request prepared for model: {}", request.model);
///     
///     Ok(())
/// }
/// ```
///
/// # Proxy Configuration
///
/// Configure proxy server by setting the `AI_PROXY_URL` environment variable:
///
/// ```bash
/// export AI_PROXY_URL=http://proxy.example.com:8080
/// ```
///
/// Supported proxy formats:
/// - HTTP proxy: `http://proxy.example.com:8080`
/// - HTTPS proxy: `https://proxy.example.com:8080`  
/// - With authentication: `http://user:pass@proxy.example.com:8080`
pub struct AiClient {
    pub(crate) chat_provider: Box<dyn ChatProvider>,
    pub(crate) metadata: ClientMetadata,
    pub(crate) metrics: Arc<dyn Metrics>,
    pub(crate) model_resolver: Arc<ModelResolver>,
    pub(crate) connection_options: Option<ConnectionOptions>,
    #[cfg(feature = "interceptors")]
    pub(crate) interceptor_pipeline: Option<crate::interceptors::InterceptorPipeline>,
    // Custom default models (override provider defaults)
    pub(crate) custom_default_chat_model: Option<String>,
    pub(crate) custom_default_multimodal_model: Option<String>,
    // Optional backpressure controller
    pub(crate) backpressure: Option<Arc<BackpressureController>>,
}

impl AiClient {
    /// Get the effective default chat model for this client (honors custom override)
    pub fn default_chat_model(&self) -> String {
        self.custom_default_chat_model
            .clone()
            .or_else(|| self.metadata.default_chat_model().map(|s| s.to_string()))
            .expect("AiClient metadata missing default chat model")
    }

    /// Create a new AI client
    pub fn new(provider: Provider) -> Result<Self, AiLibError> {
        AiClientBuilder::new(provider).build()
    }

    /// Create a new AI client builder
    pub fn builder(provider: Provider) -> AiClientBuilder {
        AiClientBuilder::new(provider)
    }

    /// Create AiClient with injected metrics implementation
    pub fn new_with_metrics(
        provider: Provider,
        metrics: Arc<dyn Metrics>,
    ) -> Result<Self, AiLibError> {
        AiClientBuilder::new(provider).with_metrics(metrics).build()
    }

    /// Create client with minimal explicit options (base_url/proxy/timeout).
    ///
    /// Fields left as `None` in `ConnectionOptions` will fall back to environment
    /// variables (e.g., `OPENAI_API_KEY`, `AI_PROXY_URL`, `AI_TIMEOUT_SECS`).
    /// Set `disable_proxy: true` to prevent automatic proxy detection from `AI_PROXY_URL`.
    pub fn with_options(provider: Provider, opts: ConnectionOptions) -> Result<Self, AiLibError> {
        // Hydrate unset fields from environment variables
        let opts = opts.hydrate_with_env(provider.env_prefix());

        let resolved_base_url = super::builder::resolve_base_url(provider, opts.base_url.clone())?;

        // Determine effective proxy: None if disable_proxy is true, otherwise use hydrated value
        let effective_proxy = if opts.disable_proxy {
            None
        } else {
            opts.proxy.clone()
        };

        let transport = if effective_proxy.is_some() || opts.timeout.is_some() {
            let transport_config = crate::transport::HttpTransportConfig {
                timeout: opts.timeout.unwrap_or(std::time::Duration::from_secs(30)),
                proxy: effective_proxy,
                pool_max_idle_per_host: None,
                pool_idle_timeout: None,
            };
            Some(crate::transport::HttpTransport::new_with_config(transport_config)?.boxed())
        } else {
            None
        };

        let chat_provider = ProviderFactory::create_adapter(
            provider,
            opts.api_key.clone(),
            Some(resolved_base_url.clone()),
            transport,
        )?;

        let metadata = metadata_from_provider(
            provider,
            chat_provider.name().to_string(),
            Some(resolved_base_url),
            None,
            None,
        );

        Ok(AiClient {
            chat_provider,
            metadata,
            metrics: Arc::new(NoopMetrics::new()),
            model_resolver: Arc::new(ModelResolver::new()),
            connection_options: Some(opts),
            custom_default_chat_model: None,
            custom_default_multimodal_model: None,
            backpressure: None,
            #[cfg(feature = "interceptors")]
            interceptor_pipeline: None,
        })
    }

    pub fn connection_options(&self) -> Option<&ConnectionOptions> {
        self.connection_options.as_ref()
    }

    /// Set metrics implementation on client
    pub fn with_metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
        self.metrics = metrics;
        self
    }

    /// Access the model resolver (advanced customization).
    pub fn model_resolver(&self) -> Arc<ModelResolver> {
        self.model_resolver.clone()
    }

    /// Send chat completion request
    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        request::chat_completion(self, request).await
    }

    #[cfg(feature = "response_parser")]
    /// Send a chat completion request and parse the response using the provided parser.
    ///
    /// This is a convenience method that pairs `chat_completion` with a `ResponseParser`.
    /// It ensures the response contains text content and then extracts structured data
    /// defined by the parser (e.g., JSON, Markdown sections).
    pub async fn chat_completion_parsed<P>(
        &self,
        request: ChatCompletionRequest,
        parser: P,
    ) -> Result<P::Output, AiLibError>
    where
        P: crate::response_parser::ResponseParser + Send + Sync,
    {
        let response = self.chat_completion(request).await?;
        let content = response
            .choices
            .first()
            .map(|c| c.message.content.as_text())
            .ok_or_else(|| {
                AiLibError::InvalidModelResponse(
                    "Response contained no text content to parse".to_string(),
                )
            })?;

        parser.parse(&content).await.map_err(|e| {
            AiLibError::InvalidModelResponse(format!("Failed to parse response: {}", e))
        })
    }
    /// Streaming chat completion request
    pub async fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        stream::chat_completion_stream(self, request).await
    }

    /// Streaming chat completion request with cancel control
    pub async fn chat_completion_stream_with_cancel(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<
        (
            Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
            CancelHandle,
        ),
        AiLibError,
    > {
        stream::chat_completion_stream_with_cancel(self, request).await
    }

    /// Batch chat completion requests
    pub async fn chat_completion_batch(
        &self,
        requests: Vec<ChatCompletionRequest>,
        concurrency_limit: Option<usize>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        batch::chat_completion_batch(self, requests, concurrency_limit).await
    }

    /// Smart batch processing
    pub async fn chat_completion_batch_smart(
        &self,
        requests: Vec<ChatCompletionRequest>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        batch::chat_completion_batch_smart(self, requests).await
    }

    /// Get list of supported models
    pub async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        helpers::list_models(self).await
    }

    /// Switch AI model provider
    pub fn switch_provider(&mut self, provider: Provider) -> Result<(), AiLibError> {
        helpers::switch_provider(self, provider)
    }

    /// Get the active provider name reported by the underlying strategy.
    pub fn provider_name(&self) -> &str {
        self.metadata.provider_name()
    }

    /// Get the current provider enum value
    pub fn provider(&self) -> Provider {
        self.metadata.provider()
    }

    /// Convenience helper: construct a request with the provider's default chat model.
    pub fn build_simple_request<S: Into<String>>(&self, prompt: S) -> ChatCompletionRequest {
        helpers::build_simple_request(self, prompt)
    }

    /// Convenience helper: construct a request with an explicitly specified chat model.
    pub fn build_simple_request_with_model<S: Into<String>>(
        &self,
        prompt: S,
        model: S,
    ) -> ChatCompletionRequest {
        helpers::build_simple_request_with_model(self, prompt, model)
    }

    /// Convenience helper: construct a request with the provider's default multimodal model.
    pub fn build_multimodal_request<S: Into<String>>(
        &self,
        prompt: S,
    ) -> Result<ChatCompletionRequest, AiLibError> {
        helpers::build_multimodal_request(self, prompt)
    }

    /// Convenience helper: construct a request with an explicitly specified multimodal model.
    pub fn build_multimodal_request_with_model<S: Into<String>>(
        &self,
        prompt: S,
        model: S,
    ) -> ChatCompletionRequest {
        helpers::build_multimodal_request_with_model(self, prompt, model)
    }

    /// One-shot helper: create a client for `provider`, send a single user prompt
    pub async fn quick_chat_text<P: Into<String>>(
        provider: Provider,
        prompt: P,
    ) -> Result<String, AiLibError> {
        helpers::quick_chat_text(provider, prompt).await
    }

    /// One-shot helper with model
    pub async fn quick_chat_text_with_model<P: Into<String>, M: Into<String>>(
        provider: Provider,
        prompt: P,
        model: M,
    ) -> Result<String, AiLibError> {
        helpers::quick_chat_text_with_model(provider, prompt, model).await
    }

    /// One-shot helper multimodal
    pub async fn quick_multimodal_text<P: Into<String>>(
        provider: Provider,
        prompt: P,
    ) -> Result<String, AiLibError> {
        helpers::quick_multimodal_text(provider, prompt).await
    }

    /// One-shot helper multimodal with model
    pub async fn quick_multimodal_text_with_model<P: Into<String>, M: Into<String>>(
        provider: Provider,
        prompt: P,
        model: M,
    ) -> Result<String, AiLibError> {
        helpers::quick_multimodal_text_with_model(provider, prompt, model).await
    }

    /// One-shot helper with model options
    pub async fn quick_chat_text_with_options<P: Into<String>>(
        provider: Provider,
        prompt: P,
        options: ModelOptions,
    ) -> Result<String, AiLibError> {
        helpers::quick_chat_text_with_options(provider, prompt, options).await
    }

    /// Upload a local file
    pub async fn upload_file(&self, path: &str) -> Result<String, AiLibError> {
        helpers::upload_file(self, path).await
    }

    pub(crate) fn provider_id(&self) -> Provider {
        self.metadata.provider()
    }

    pub(crate) fn prepare_chat_request(
        &self,
        mut request: ChatCompletionRequest,
    ) -> ChatCompletionRequest {
        if should_use_auto_token(&request.model) {
            if let Some(custom) = &self.custom_default_chat_model {
                request.model = custom.clone();
            } else {
                let resolution = self
                    .model_resolver
                    .resolve_chat_model(self.provider_id(), None);
                request.model = resolution.model;
            }
        }
        request
    }

    pub(crate) fn fallback_model_after_invalid(
        &self,
        failed_model: &str,
    ) -> Option<ModelResolution> {
        if let Some(custom) = &self.custom_default_chat_model {
            if !custom.eq_ignore_ascii_case(failed_model) {
                return Some(ModelResolution::new(
                    custom.clone(),
                    ModelResolutionSource::CustomDefault,
                    self.model_resolver.doc_url(self.provider_id()),
                ));
            }
        }

        self.model_resolver
            .fallback_after_invalid(self.provider_id(), failed_model)
    }
}

fn should_use_auto_token(model: &str) -> bool {
    let trimmed = model.trim();
    trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("auto")
        || trimmed.eq_ignore_ascii_case("default")
        || trimmed.eq_ignore_ascii_case("provider_default")
}
