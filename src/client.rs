use crate::api::{ChatApi, ChatCompletionChunk};
use crate::config::ConnectionOptions;
use crate::metrics::{Metrics, NoopMetrics};
use crate::provider::{
    classification::ProviderClassification,
    CohereAdapter, GeminiAdapter, GenericAdapter, MistralAdapter, OpenAiAdapter, ProviderConfigs,
};
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};
use futures::stream::Stream;
use futures::Future;
use std::sync::Arc;
use tokio::sync::oneshot;

/// Model configuration options for explicit model selection
#[derive(Debug, Clone)]
pub struct ModelOptions {
    pub chat_model: Option<String>,
    pub multimodal_model: Option<String>,
    pub fallback_models: Vec<String>,
    pub auto_discovery: bool,
}

impl ModelOptions {
    /// Create default model options
    pub fn default() -> Self {
        Self {
            chat_model: None,
            multimodal_model: None,
            fallback_models: Vec::new(),
            auto_discovery: true,
        }
    }
    
    /// Set chat model
    pub fn with_chat_model(mut self, model: &str) -> Self {
        self.chat_model = Some(model.to_string());
        self
    }
    
    /// Set multimodal model
    pub fn with_multimodal_model(mut self, model: &str) -> Self {
        self.multimodal_model = Some(model.to_string());
        self
    }
    
    /// Set fallback models
    pub fn with_fallback_models(mut self, models: Vec<&str>) -> Self {
        self.fallback_models = models.into_iter().map(|s| s.to_string()).collect();
        self
    }
    
    /// Enable or disable auto discovery
    pub fn with_auto_discovery(mut self, enabled: bool) -> Self {
        self.auto_discovery = enabled;
        self
    }
}

/// Helper function to create GenericAdapter with optional custom transport
fn create_generic_adapter(
    config: crate::provider::config::ProviderConfig,
    transport: Option<crate::transport::DynHttpTransportRef>,
) -> Result<Box<dyn ChatApi>, AiLibError> {
    if let Some(custom_transport) = transport {
        Ok(Box::new(GenericAdapter::with_transport_ref(config, custom_transport)?))
    } else {
        Ok(Box::new(GenericAdapter::new(config)?))
    }
}

/// Unified AI client module
///
/// AI model provider enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Provider {
    // Config-driven providers
    Groq,
    XaiGrok,
    Ollama,
    DeepSeek,
    Anthropic,
    AzureOpenAI,
    HuggingFace,
    TogetherAI,
    // Chinese providers (OpenAI-compatible or config-driven)
    BaiduWenxin,
    TencentHunyuan,
    IflytekSpark,
    Moonshot,
    // Independent adapters
    OpenAI,
    Qwen,
    Gemini,
    Mistral,
    Cohere,
    // Bedrock removed (deferred)
}

impl Provider {
    /// Get the provider's preferred default chat model.
    /// These should mirror the values used inside `ProviderConfigs`.
    pub fn default_chat_model(&self) -> &'static str {
        match self {
            Provider::Groq => "llama-3.1-8b-instant",
            Provider::XaiGrok => "grok-beta",
            Provider::Ollama => "llama3-8b",
            Provider::DeepSeek => "deepseek-chat",
            Provider::Anthropic => "claude-3-5-sonnet-20241022",
            Provider::AzureOpenAI => "gpt-35-turbo",
            Provider::HuggingFace => "microsoft/DialoGPT-medium",
            Provider::TogetherAI => "meta-llama/Llama-3-8b-chat-hf",
            Provider::BaiduWenxin => "ernie-3.5",
            Provider::TencentHunyuan => "hunyuan-standard",
            Provider::IflytekSpark => "spark-v3.0",
            Provider::Moonshot => "moonshot-v1-8k",
            Provider::OpenAI => "gpt-3.5-turbo",
            Provider::Qwen => "qwen-turbo",
            Provider::Gemini => "gemini-pro", // widely used text/chat model
            Provider::Mistral => "mistral-small", // generic default
            Provider::Cohere => "command-r", // chat-capable model
        }
    }

    /// Get the provider's preferred multimodal model (if any).
    pub fn default_multimodal_model(&self) -> Option<&'static str> {
        match self {
            Provider::OpenAI => Some("gpt-4o"),
            Provider::AzureOpenAI => Some("gpt-4o"),
            Provider::Anthropic => Some("claude-3-5-sonnet-20241022"),
            Provider::Groq => None, // No multimodal model currently available
            Provider::Gemini => Some("gemini-1.5-flash"),
            Provider::Cohere => Some("command-r-plus"),
            // Others presently have no clearly documented multimodal endpoint or are not yet wired.
            _ => None,
        }
    }
}

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
///     println!("Client created successfully with provider: {:?}", client.current_provider());
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
    provider: Provider,
    adapter: Box<dyn ChatApi>,
    metrics: Arc<dyn Metrics>,
    connection_options: Option<ConnectionOptions>,
    // Custom default models (override provider defaults)
    custom_default_chat_model: Option<String>,
    custom_default_multimodal_model: Option<String>,
}

impl AiClient {
    /// Create a new AI client
    ///
    /// # Arguments
    /// * `provider` - The AI model provider to use
    ///
    /// # Returns
    /// * `Result<Self, AiLibError>` - Client instance on success, error on failure
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::{AiClient, Provider};
    ///
    /// let client = AiClient::new(Provider::Groq)?;
    /// # Ok::<(), ai_lib::AiLibError>(())
    /// ```
    pub fn new(provider: Provider) -> Result<Self, AiLibError> {
        // Use the new builder to create client with automatic environment variable detection
        let mut c = AiClientBuilder::new(provider).build()?;
        c.connection_options = None;
        Ok(c)
    }

    /// Create client with minimal explicit options (base_url/proxy/timeout). Not all providers
    /// support overrides; unsupported providers ignore unspecified fields gracefully.
    pub fn with_options(provider: Provider, opts: ConnectionOptions) -> Result<Self, AiLibError> {
        let config_driven = provider.is_config_driven();
        let need_builder = config_driven
            && (opts.base_url.is_some()
                || opts.proxy.is_some()
                || opts.timeout.is_some()
                || opts.disable_proxy);
        if need_builder {
            let mut b = AiClient::builder(provider);
            if let Some(ref base) = opts.base_url {
                b = b.with_base_url(base);
            }
            if opts.disable_proxy {
                b = b.without_proxy();
            } else if let Some(ref proxy) = opts.proxy {
                if proxy.is_empty() {
                    b = b.without_proxy();
                } else {
                    b = b.with_proxy(Some(proxy));
                }
            }
            if let Some(t) = opts.timeout {
                b = b.with_timeout(t);
            }
            let mut client = b.build()?;
            // If api_key override + generic provider path: re-wrap adapter using override
            if opts.api_key.is_some() {
                // Only applies to config-driven generic adapter providers
                let new_adapter: Option<Box<dyn ChatApi>> = match provider {
                    Provider::Groq => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::groq(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::XaiGrok => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::xai_grok(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::Ollama => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::ollama(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::DeepSeek => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::deepseek(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::Qwen => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::qwen(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::BaiduWenxin => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::baidu_wenxin(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::TencentHunyuan => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::tencent_hunyuan(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::IflytekSpark => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::iflytek_spark(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::Moonshot => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::moonshot(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::Anthropic => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::anthropic(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::AzureOpenAI => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::azure_openai(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::HuggingFace => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::huggingface(),
                        opts.api_key.clone(),
                    )?)),
                    Provider::TogetherAI => Some(Box::new(GenericAdapter::new_with_api_key(
                        ProviderConfigs::together_ai(),
                        opts.api_key.clone(),
                    )?)),
                    _ => None,
                };
                if let Some(a) = new_adapter {
                    client.adapter = a;
                }
            }
            client.connection_options = Some(opts);
            return Ok(client);
        }

        // Independent adapters: OpenAI / Gemini / Mistral / Cohere
        if provider.is_independent() {
            let adapter: Box<dyn ChatApi> = match provider {
                Provider::OpenAI => {
                    if let Some(ref k) = opts.api_key {
                        let inner =
                            OpenAiAdapter::new_with_overrides(k.clone(), opts.base_url.clone())?;
                        Box::new(inner)
                    } else {
                        let inner = OpenAiAdapter::new()?;
                        Box::new(inner)
                    }
                }
                Provider::Gemini => {
                    if let Some(ref k) = opts.api_key {
                        let inner =
                            GeminiAdapter::new_with_overrides(k.clone(), opts.base_url.clone())?;
                        Box::new(inner)
                    } else {
                        let inner = GeminiAdapter::new()?;
                        Box::new(inner)
                    }
                }
                Provider::Mistral => {
                    if opts.api_key.is_some() || opts.base_url.is_some() {
                        let inner = MistralAdapter::new_with_overrides(
                            opts.api_key.clone(),
                            opts.base_url.clone(),
                        )?;
                        Box::new(inner)
                    } else {
                        let inner = MistralAdapter::new()?;
                        Box::new(inner)
                    }
                }
                Provider::Cohere => {
                    if let Some(ref k) = opts.api_key {
                        let inner =
                            CohereAdapter::new_with_overrides(k.clone(), opts.base_url.clone())?;
                        Box::new(inner)
                    } else {
                        let inner = CohereAdapter::new()?;
                        Box::new(inner)
                    }
                }
                _ => unreachable!(),
            };
            return Ok(AiClient {
                provider,
                adapter,
                metrics: Arc::new(NoopMetrics::new()),
                connection_options: Some(opts),
                custom_default_chat_model: None,
                custom_default_multimodal_model: None,
            });
        }

        // Simple config-driven without overrides
        let mut client = AiClient::new(provider)?;
        if let Some(ref k) = opts.api_key {
            let override_adapter: Option<Box<dyn ChatApi>> = match provider {
                Provider::Groq => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::groq(),
                    Some(k.clone()),
                )?)),
                Provider::XaiGrok => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::xai_grok(),
                    Some(k.clone()),
                )?)),
                Provider::Ollama => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::ollama(),
                    Some(k.clone()),
                )?)),
                Provider::DeepSeek => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::deepseek(),
                    Some(k.clone()),
                )?)),
                Provider::Qwen => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::qwen(),
                    Some(k.clone()),
                )?)),
                Provider::BaiduWenxin => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::baidu_wenxin(),
                    Some(k.clone()),
                )?)),
                Provider::TencentHunyuan => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::tencent_hunyuan(),
                    Some(k.clone()),
                )?)),
                Provider::IflytekSpark => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::iflytek_spark(),
                    Some(k.clone()),
                )?)),
                Provider::Moonshot => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::moonshot(),
                    Some(k.clone()),
                )?)),
                Provider::Anthropic => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::anthropic(),
                    Some(k.clone()),
                )?)),
                Provider::AzureOpenAI => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::azure_openai(),
                    Some(k.clone()),
                )?)),
                Provider::HuggingFace => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::huggingface(),
                    Some(k.clone()),
                )?)),
                Provider::TogetherAI => Some(Box::new(GenericAdapter::new_with_api_key(
                    ProviderConfigs::together_ai(),
                    Some(k.clone()),
                )?)),
                _ => None,
            };
            if let Some(a) = override_adapter {
                client.adapter = a;
            }
        }
        client.connection_options = Some(opts);
        Ok(client)
    }

    pub fn connection_options(&self) -> Option<&ConnectionOptions> {
        self.connection_options.as_ref()
    }

    /// Create a new AI client builder
    ///
    /// The builder pattern allows more flexible client configuration:
    /// - Automatic environment variable detection
    /// - Support for custom base_url and proxy
    /// - Support for custom timeout and connection pool configuration
    ///
    /// # Arguments
    /// * `provider` - The AI model provider to use
    ///
    /// # Returns
    /// * `AiClientBuilder` - Builder instance
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::{AiClient, Provider};
    ///
    /// // Simplest usage - automatic environment variable detection
    /// let client = AiClient::builder(Provider::Groq).build()?;
    ///
    /// // Custom base_url and proxy
    /// let client = AiClient::builder(Provider::Groq)
    ///     .with_base_url("https://custom.groq.com")
    ///     .with_proxy(Some("http://proxy.example.com:8080"))
    ///     .build()?;
    /// # Ok::<(), ai_lib::AiLibError>(())
    /// ```
    pub fn builder(provider: Provider) -> AiClientBuilder {
        AiClientBuilder::new(provider)
    }

    /// Create AiClient with injected metrics implementation
    pub fn new_with_metrics(
        provider: Provider,
        metrics: Arc<dyn Metrics>,
    ) -> Result<Self, AiLibError> {
        let adapter: Box<dyn ChatApi> = match provider {
            Provider::Groq => Box::new(GenericAdapter::new(ProviderConfigs::groq())?),
            Provider::XaiGrok => Box::new(GenericAdapter::new(ProviderConfigs::xai_grok())?),
            Provider::Ollama => Box::new(GenericAdapter::new(ProviderConfigs::ollama())?),
            Provider::DeepSeek => Box::new(GenericAdapter::new(ProviderConfigs::deepseek())?),
            Provider::Qwen => Box::new(GenericAdapter::new(ProviderConfigs::qwen())?),
            Provider::Anthropic => Box::new(GenericAdapter::new(ProviderConfigs::anthropic())?),
            Provider::BaiduWenxin => {
                Box::new(GenericAdapter::new(ProviderConfigs::baidu_wenxin())?)
            }
            Provider::TencentHunyuan => {
                Box::new(GenericAdapter::new(ProviderConfigs::tencent_hunyuan())?)
            }
            Provider::IflytekSpark => {
                Box::new(GenericAdapter::new(ProviderConfigs::iflytek_spark())?)
            }
            Provider::Moonshot => Box::new(GenericAdapter::new(ProviderConfigs::moonshot())?),
            Provider::AzureOpenAI => {
                Box::new(GenericAdapter::new(ProviderConfigs::azure_openai())?)
            }
            Provider::HuggingFace => Box::new(GenericAdapter::new(ProviderConfigs::huggingface())?),
            Provider::TogetherAI => Box::new(GenericAdapter::new(ProviderConfigs::together_ai())?),
            Provider::OpenAI => Box::new(OpenAiAdapter::new()?),
            Provider::Gemini => Box::new(GeminiAdapter::new()?),
            Provider::Mistral => Box::new(MistralAdapter::new()?),
            Provider::Cohere => Box::new(CohereAdapter::new()?),
        };

        Ok(Self {
            provider,
            adapter,
            metrics,
            connection_options: None,
            custom_default_chat_model: None,
            custom_default_multimodal_model: None,
        })
    }

    /// Set metrics implementation on client
    pub fn with_metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
        self.metrics = metrics;
        self
    }

    /// Send chat completion request
    ///
    /// # Arguments
    /// * `request` - Chat completion request
    ///
    /// # Returns
    /// * `Result<ChatCompletionResponse, AiLibError>` - Response on success, error on failure
    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        self.adapter.chat_completion(request).await
    }

    /// Streaming chat completion request
    ///
    /// # Arguments
    /// * `request` - Chat completion request
    ///
    /// # Returns
    /// * `Result<impl Stream<Item = Result<ChatCompletionChunk, AiLibError>>, AiLibError>` - Stream response on success
    pub async fn chat_completion_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        request.stream = Some(true);
        self.adapter.chat_completion_stream(request).await
    }

    /// Streaming chat completion request with cancel control
    ///
    /// # Arguments
    /// * `request` - Chat completion request
    ///
    /// # Returns
    /// * `Result<(impl Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin, CancelHandle), AiLibError>` - Returns streaming response and cancel handle on success
    pub async fn chat_completion_stream_with_cancel(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<
        (
            Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
            CancelHandle,
        ),
        AiLibError,
    > {
        request.stream = Some(true);
        let stream = self.adapter.chat_completion_stream(request).await?;
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let cancel_handle = CancelHandle {
            sender: Some(cancel_tx),
        };

        let controlled_stream = ControlledStream::new(stream, cancel_rx);
        Ok((Box::new(controlled_stream), cancel_handle))
    }

    /// Batch chat completion requests
    ///
    /// # Arguments
    /// * `requests` - List of chat completion requests
    /// * `concurrency_limit` - Maximum concurrent request count (None means unlimited)
    ///
    /// # Returns
    /// * `Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError>` - Returns response results for all requests
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
    /// use ai_lib::types::common::Content;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = AiClient::new(Provider::Groq)?;
    ///     
    ///     let requests = vec![
    ///         ChatCompletionRequest::new(
    ///             "llama3-8b-8192".to_string(),
    ///             vec![Message {
    ///                 role: Role::User,
    ///                 content: Content::Text("Hello".to_string()),
    ///                 function_call: None,
    ///             }],
    ///         ),
    ///         ChatCompletionRequest::new(
    ///             "llama3-8b-8192".to_string(),
    ///             vec![Message {
    ///                 role: Role::User,
    ///                 content: Content::Text("How are you?".to_string()),
    ///                 function_call: None,
    ///             }],
    ///         ),
    ///     ];
    ///     
    ///     // Limit concurrency to 5
    ///     let responses = client.chat_completion_batch(requests, Some(5)).await?;
    ///     
    ///     for (i, response) in responses.iter().enumerate() {
    ///         match response {
    ///             Ok(resp) => println!("Request {}: {}", i, resp.choices[0].message.content.as_text()),
    ///             Err(e) => println!("Request {} failed: {}", i, e),
    ///         }
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn chat_completion_batch(
        &self,
        requests: Vec<ChatCompletionRequest>,
        concurrency_limit: Option<usize>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        self.adapter
            .chat_completion_batch(requests, concurrency_limit)
            .await
    }

    /// Smart batch processing: automatically choose processing strategy based on request count
    ///
    /// # Arguments
    /// * `requests` - List of chat completion requests
    ///
    /// # Returns
    /// * `Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError>` - Returns response results for all requests
    pub async fn chat_completion_batch_smart(
        &self,
        requests: Vec<ChatCompletionRequest>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        // Use sequential processing for small batches, concurrent processing for large batches
        let concurrency_limit = if requests.len() <= 3 { None } else { Some(10) };
        self.chat_completion_batch(requests, concurrency_limit)
            .await
    }

    /// Batch chat completion requests
    ///
    /// # Arguments
    /// * `requests` - List of chat completion requests
    /// * `concurrency_limit` - Maximum concurrent request count (None means unlimited)
    ///
    /// # Returns
    /// * `Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError>` - Returns response results for all requests
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
    /// use ai_lib::types::common::Content;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = AiClient::new(Provider::Groq)?;
    ///     
    ///     let requests = vec![
    ///         ChatCompletionRequest::new(
    ///             "llama3-8b-8192".to_string(),
    ///             vec![Message {
    ///                 role: Role::User,
    ///                 content: Content::Text("Hello".to_string()),
    ///                 function_call: None,
    ///             }],
    ///         ),
    ///         ChatCompletionRequest::new(
    ///             "llama3-8b-8192".to_string(),
    ///             vec![Message {
    ///                 role: Role::User,
    ///                 content: Content::Text("How are you?".to_string()),
    ///                 function_call: None,
    ///             }],
    ///         ),
    ///     ];
    ///     
    ///     // Limit concurrency to 5
    ///     let responses = client.chat_completion_batch(requests, Some(5)).await?;
    ///     
    ///     for (i, response) in responses.iter().enumerate() {
    ///         match response {
    ///             Ok(resp) => println!("Request {}: {}", i, resp.choices[0].message.content.as_text()),
    ///             Err(e) => println!("Request {} failed: {}", i, e),
    ///         }
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Get list of supported models
    ///
    /// # Returns
    /// * `Result<Vec<String>, AiLibError>` - Returns model list on success, error on failure
    pub async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        self.adapter.list_models().await
    }

    /// Switch AI model provider
    ///
    /// # Arguments
    /// * `provider` - New provider
    ///
    /// # Returns
    /// * `Result<(), AiLibError>` - Returns () on success, error on failure
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::{AiClient, Provider};
    ///
    /// let mut client = AiClient::new(Provider::Groq)?;
    /// // Switch from Groq to Groq (demonstrating switch functionality)
    /// client.switch_provider(Provider::Groq)?;
    /// # Ok::<(), ai_lib::AiLibError>(())
    /// ```
    pub fn switch_provider(&mut self, provider: Provider) -> Result<(), AiLibError> {
        let new_adapter: Box<dyn ChatApi> = match provider {
            Provider::Groq => Box::new(GenericAdapter::new(ProviderConfigs::groq())?),
            Provider::XaiGrok => Box::new(GenericAdapter::new(ProviderConfigs::xai_grok())?),
            Provider::Ollama => Box::new(GenericAdapter::new(ProviderConfigs::ollama())?),
            Provider::DeepSeek => Box::new(GenericAdapter::new(ProviderConfigs::deepseek())?),
            Provider::Qwen => Box::new(GenericAdapter::new(ProviderConfigs::qwen())?),
            Provider::OpenAI => Box::new(OpenAiAdapter::new()?),
            Provider::Anthropic => Box::new(GenericAdapter::new(ProviderConfigs::anthropic())?),
            Provider::BaiduWenxin => {
                Box::new(GenericAdapter::new(ProviderConfigs::baidu_wenxin())?)
            }
            Provider::TencentHunyuan => {
                Box::new(GenericAdapter::new(ProviderConfigs::tencent_hunyuan())?)
            }
            Provider::IflytekSpark => {
                Box::new(GenericAdapter::new(ProviderConfigs::iflytek_spark())?)
            }
            Provider::Moonshot => Box::new(GenericAdapter::new(ProviderConfigs::moonshot())?),
            Provider::Gemini => Box::new(GeminiAdapter::new()?),
            Provider::AzureOpenAI => {
                Box::new(GenericAdapter::new(ProviderConfigs::azure_openai())?)
            }
            Provider::HuggingFace => Box::new(GenericAdapter::new(ProviderConfigs::huggingface())?),
            Provider::TogetherAI => Box::new(GenericAdapter::new(ProviderConfigs::together_ai())?),
            Provider::Mistral => Box::new(MistralAdapter::new()?),
            Provider::Cohere => Box::new(CohereAdapter::new()?),
            // Provider::Bedrock => Box::new(BedrockAdapter::new()?),
        };

        self.provider = provider;
        self.adapter = new_adapter;
        Ok(())
    }

    /// Get current provider
    pub fn current_provider(&self) -> Provider {
        self.provider
    }

    /// Convenience helper: construct a request with the provider's default chat model.
    /// This does NOT send the request.
    /// Uses custom default model if set via AiClientBuilder, otherwise uses provider default.
    pub fn build_simple_request<S: Into<String>>(&self, prompt: S) -> ChatCompletionRequest {
        let model = self.custom_default_chat_model
            .clone()
            .unwrap_or_else(|| self.provider.default_chat_model().to_string());
            
        ChatCompletionRequest::new(
            model,
            vec![crate::types::Message {
                role: crate::types::Role::User,
                content: crate::types::common::Content::Text(prompt.into()),
                function_call: None,
            }],
        )
    }
    
    /// Convenience helper: construct a request with an explicitly specified chat model.
    /// This does NOT send the request.
    pub fn build_simple_request_with_model<S: Into<String>>(
        &self, 
        prompt: S, 
        model: S
    ) -> ChatCompletionRequest {
        ChatCompletionRequest::new(
            model.into(),
            vec![crate::types::Message {
                role: crate::types::Role::User,
                content: crate::types::common::Content::Text(prompt.into()),
                function_call: None,
            }],
        )
    }
    
    /// Convenience helper: construct a request with the provider's default multimodal model.
    /// This does NOT send the request.
    /// Uses custom default model if set via AiClientBuilder, otherwise uses provider default.
    pub fn build_multimodal_request<S: Into<String>>(
        &self, 
        prompt: S
    ) -> Result<ChatCompletionRequest, AiLibError> {
        let model = self.custom_default_multimodal_model
            .clone()
            .or_else(|| self.provider.default_multimodal_model().map(|s| s.to_string()))
            .ok_or_else(|| AiLibError::ConfigurationError(
                format!("No multimodal model available for provider {:?}", self.provider)
            ))?;
            
        Ok(ChatCompletionRequest::new(
            model,
            vec![crate::types::Message {
                role: crate::types::Role::User,
                content: crate::types::common::Content::Text(prompt.into()),
                function_call: None,
            }],
        ))
    }
    
    /// Convenience helper: construct a request with an explicitly specified multimodal model.
    /// This does NOT send the request.
    pub fn build_multimodal_request_with_model<S: Into<String>>(
        &self, 
        prompt: S, 
        model: S
    ) -> ChatCompletionRequest {
        ChatCompletionRequest::new(
            model.into(),
            vec![crate::types::Message {
                role: crate::types::Role::User,
                content: crate::types::common::Content::Text(prompt.into()),
                function_call: None,
            }],
        )
    }

    /// One-shot helper: create a client for `provider`, send a single user prompt using the
    /// default chat model, and return plain text content (first choice).
    pub async fn quick_chat_text<P: Into<String>>(
        provider: Provider,
        prompt: P,
    ) -> Result<String, AiLibError> {
        let client = AiClient::new(provider)?;
        let req = client.build_simple_request(prompt.into());
        let resp = client.chat_completion(req).await?;
        resp.first_text().map(|s| s.to_string())
    }
    
    /// One-shot helper: create a client for `provider`, send a single user prompt using an
    /// explicitly specified chat model, and return plain text content (first choice).
    pub async fn quick_chat_text_with_model<P: Into<String>, M: Into<String>>(
        provider: Provider,
        prompt: P,
        model: M,
    ) -> Result<String, AiLibError> {
        let client = AiClient::new(provider)?;
        let req = client.build_simple_request_with_model(prompt.into(), model.into());
        let resp = client.chat_completion(req).await?;
        resp.first_text().map(|s| s.to_string())
    }
    
    /// One-shot helper: create a client for `provider`, send a single user prompt using the
    /// default multimodal model, and return plain text content (first choice).
    pub async fn quick_multimodal_text<P: Into<String>>(
        provider: Provider,
        prompt: P,
    ) -> Result<String, AiLibError> {
        let client = AiClient::new(provider)?;
        let req = client.build_multimodal_request(prompt.into())?;
        let resp = client.chat_completion(req).await?;
        resp.first_text().map(|s| s.to_string())
    }
    
    /// One-shot helper: create a client for `provider`, send a single user prompt using an
    /// explicitly specified multimodal model, and return plain text content (first choice).
    pub async fn quick_multimodal_text_with_model<P: Into<String>, M: Into<String>>(
        provider: Provider,
        prompt: P,
        model: M,
    ) -> Result<String, AiLibError> {
        let client = AiClient::new(provider)?;
        let req = client.build_multimodal_request_with_model(prompt.into(), model.into());
        let resp = client.chat_completion(req).await?;
        resp.first_text().map(|s| s.to_string())
    }
    
    /// One-shot helper with model options: create a client for `provider`, send a single user prompt
    /// using specified model options, and return plain text content (first choice).
    pub async fn quick_chat_text_with_options<P: Into<String>>(
        provider: Provider,
        prompt: P,
        options: ModelOptions,
    ) -> Result<String, AiLibError> {
        let client = AiClient::new(provider)?;
        
        // Determine which model to use based on options
        let model = if let Some(chat_model) = options.chat_model {
            chat_model
        } else {
            provider.default_chat_model().to_string()
        };
        
        let req = client.build_simple_request_with_model(prompt.into(), model);
        let resp = client.chat_completion(req).await?;
        resp.first_text().map(|s| s.to_string())
    }
}

/// Streaming response cancel handle
pub struct CancelHandle {
    sender: Option<oneshot::Sender<()>>,
}

impl CancelHandle {
    /// Cancel streaming response
    pub fn cancel(mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(());
        }
    }
}

/// AI client builder with progressive custom configuration
///
/// Usage examples:
/// ```rust
/// use ai_lib::{AiClientBuilder, Provider};
///
/// // Simplest usage - automatic environment variable detection
/// let client = AiClientBuilder::new(Provider::Groq).build()?;
///
/// // Custom base_url and proxy
/// let client = AiClientBuilder::new(Provider::Groq)
///     .with_base_url("https://custom.groq.com")
///     .with_proxy(Some("http://proxy.example.com:8080"))
///     .build()?;
///
/// // Full custom configuration
/// let client = AiClientBuilder::new(Provider::Groq)
///     .with_base_url("https://custom.groq.com")
///     .with_proxy(Some("http://proxy.example.com:8080"))
///     .with_timeout(std::time::Duration::from_secs(60))
///     .with_pool_config(32, std::time::Duration::from_secs(90))
///     .build()?;
/// # Ok::<(), ai_lib::AiLibError>(())
/// ```
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
}

impl AiClientBuilder {
    /// Create a new builder instance
    ///
    /// # Arguments
    /// * `provider` - The AI model provider to use
    ///
    /// # Returns
    /// * `Self` - Builder instance
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
        }
    }

    /// Check if provider is config-driven (uses GenericAdapter)
    fn is_config_driven_provider(provider: Provider) -> bool {
        provider.is_config_driven()
    }

    /// Set custom base URL
    ///
    /// # Arguments
    /// * `base_url` - Custom base URL
    ///
    /// # Returns
    /// * `Self` - Builder instance for method chaining
    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = Some(base_url.to_string());
        self
    }

    /// Set custom proxy URL
    ///
    /// # Arguments
    /// * `proxy_url` - Custom proxy URL, or None to use AI_PROXY_URL environment variable
    ///
    /// # Returns
    /// * `Self` - Builder instance for method chaining
    ///
    /// # Examples
    /// ```rust
    /// use ai_lib::{AiClientBuilder, Provider};
    ///
    /// // Use specific proxy URL
    /// let client = AiClientBuilder::new(Provider::Groq)
    ///     .with_proxy(Some("http://proxy.example.com:8080"))
    ///     .build()?;
    ///
    /// // Use AI_PROXY_URL environment variable
    /// let client = AiClientBuilder::new(Provider::Groq)
    ///     .with_proxy(None)
    ///     .build()?;
    /// # Ok::<(), ai_lib::AiLibError>(())
    /// ```
    pub fn with_proxy(mut self, proxy_url: Option<&str>) -> Self {
        self.proxy_url = proxy_url.map(|s| s.to_string());
        self
    }

    /// Explicitly disable proxy usage
    ///
    /// This method ensures that no proxy will be used, regardless of environment variables.
    ///
    /// # Returns
    /// * `Self` - Builder instance for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::{AiClientBuilder, Provider};
    ///
    /// let client = AiClientBuilder::new(Provider::Groq)
    ///     .build()?;
    /// # Ok::<(), ai_lib::AiLibError>(())
    /// ```
    pub fn without_proxy(mut self) -> Self {
        self.proxy_url = Some("".to_string());
        self
    }

    /// Set custom timeout duration
    ///
    /// # Arguments
    /// * `timeout` - Custom timeout duration
    ///
    /// # Returns
    /// * `Self` - Builder instance for method chaining
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set connection pool configuration
    ///
    /// # Arguments
    /// * `max_idle` - Maximum idle connections per host
    /// * `idle_timeout` - Idle connection timeout duration
    ///
    /// # Returns
    /// * `Self` - Builder instance for method chaining
    pub fn with_pool_config(mut self, max_idle: usize, idle_timeout: std::time::Duration) -> Self {
        self.pool_max_idle = Some(max_idle);
        self.pool_idle_timeout = Some(idle_timeout);
        self
    }

    /// Set custom metrics implementation
    ///
    /// # Arguments
    /// * `metrics` - Custom metrics implementation
    ///
    /// # Returns
    /// * `Self` - Builder instance for method chaining
    pub fn with_metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Set default chat model for the client
    ///
    /// # Arguments
    /// * `model` - Default chat model name
    ///
    /// # Returns
    /// * `Self` - Builder instance for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::{AiClientBuilder, Provider};
    ///
    /// let client = AiClientBuilder::new(Provider::Groq)
    ///     .with_default_chat_model("llama-3.1-8b-instant")
    ///     .build()?;
    /// # Ok::<(), ai_lib::AiLibError>(())
    /// ```
    pub fn with_default_chat_model(mut self, model: &str) -> Self {
        self.default_chat_model = Some(model.to_string());
        self
    }

    /// Set default multimodal model for the client
    ///
    /// # Arguments
    /// * `model` - Default multimodal model name
    ///
    /// # Returns
    /// * `Self` - Builder instance for method chaining
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::{AiClientBuilder, Provider};
    ///
    /// let client = AiClientBuilder::new(Provider::Groq)
    ///     .with_default_multimodal_model("llama-3.2-11b-vision")
    ///     .build()?;
    /// # Ok::<(), ai_lib::AiLibError>(())
    /// ```
    pub fn with_default_multimodal_model(mut self, model: &str) -> Self {
        self.default_multimodal_model = Some(model.to_string());
        self
    }

    /// Build AiClient instance
    ///
    /// The build process applies configuration in the following priority order:
    /// 1. Explicitly set configuration (via with_* methods)
    /// 2. Environment variable configuration
    /// 3. Default configuration
    ///
    /// # Returns
    /// * `Result<AiClient, AiLibError>` - Returns client instance on success, error on failure
    pub fn build(self) -> Result<AiClient, AiLibError> {
        // 1. Determine base_url: explicit setting > environment variable > default
        let base_url = self.determine_base_url()?;

        // 2. Determine proxy_url: explicit setting > environment variable
        let proxy_url = self.determine_proxy_url();

        // 3. Determine timeout: explicit setting > default
        let timeout = self
            .timeout
            .unwrap_or_else(|| std::time::Duration::from_secs(30));

        // 4. Create custom ProviderConfig (if needed)
        let config = self.create_custom_config(base_url)?;

        // 5. Create custom HttpTransport (if needed)
        let transport = self.create_custom_transport(proxy_url.clone(), timeout)?;

        // 6. Create adapter
        let adapter: Box<dyn ChatApi> = if Self::is_config_driven_provider(self.provider) {
            // All config-driven providers use the same logic - much cleaner!
            create_generic_adapter(config, transport)?
        } else {
            // Independent adapters - simple one-liners
            match self.provider {
                Provider::OpenAI => Box::new(OpenAiAdapter::new()?),
                Provider::Gemini => Box::new(GeminiAdapter::new()?),
                Provider::Mistral => Box::new(MistralAdapter::new()?),
                Provider::Cohere => Box::new(CohereAdapter::new()?),
                _ => unreachable!("All providers should be handled by now"),
            }
        };

        // 7. Create AiClient
        let client = AiClient {
            provider: self.provider,
            adapter,
            metrics: self.metrics.unwrap_or_else(|| Arc::new(NoopMetrics::new())),
            connection_options: None,
            custom_default_chat_model: self.default_chat_model,
            custom_default_multimodal_model: self.default_multimodal_model,
        };

        Ok(client)
    }

    /// Determine base_url, priority: explicit setting > environment variable > default
    fn determine_base_url(&self) -> Result<String, AiLibError> {
        // 1. Explicitly set base_url
        if let Some(ref base_url) = self.base_url {
            return Ok(base_url.clone());
        }

        // 2. base_url from environment variable
        let env_var_name = self.get_base_url_env_var_name();
        if let Ok(base_url) = std::env::var(&env_var_name) {
            return Ok(base_url);
        }

        // 3. Use default configuration
        let default_config = self.get_default_provider_config()?;
        Ok(default_config.base_url)
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

    /// Get environment variable name for corresponding provider
    fn get_base_url_env_var_name(&self) -> String {
        match self.provider {
            Provider::Groq => "GROQ_BASE_URL".to_string(),
            Provider::XaiGrok => "GROK_BASE_URL".to_string(),
            Provider::Ollama => "OLLAMA_BASE_URL".to_string(),
            Provider::DeepSeek => "DEEPSEEK_BASE_URL".to_string(),
            Provider::Qwen => "DASHSCOPE_BASE_URL".to_string(),
            Provider::BaiduWenxin => "BAIDU_WENXIN_BASE_URL".to_string(),
            Provider::TencentHunyuan => "TENCENT_HUNYUAN_BASE_URL".to_string(),
            Provider::IflytekSpark => "IFLYTEK_BASE_URL".to_string(),
            Provider::Moonshot => "MOONSHOT_BASE_URL".to_string(),
            Provider::Anthropic => "ANTHROPIC_BASE_URL".to_string(),
            Provider::AzureOpenAI => "AZURE_OPENAI_BASE_URL".to_string(),
            Provider::HuggingFace => "HUGGINGFACE_BASE_URL".to_string(),
            Provider::TogetherAI => "TOGETHER_BASE_URL".to_string(),
            // These providers don't support custom base_url
            Provider::OpenAI | Provider::Gemini | Provider::Mistral | Provider::Cohere => {
                "".to_string()
            }
        }
    }

    /// Get default provider configuration
    fn get_default_provider_config(
        &self,
    ) -> Result<crate::provider::config::ProviderConfig, AiLibError> {
        self.provider.get_default_config()
    }

    /// Create custom ProviderConfig
    fn create_custom_config(
        &self,
        base_url: String,
    ) -> Result<crate::provider::config::ProviderConfig, AiLibError> {
        let mut config = self.get_default_provider_config()?;
        config.base_url = base_url;
        Ok(config)
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

/// Controllable streaming response
struct ControlledStream {
    inner: Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
    cancel_rx: Option<oneshot::Receiver<()>>,
}

impl ControlledStream {
    fn new(
        inner: Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        cancel_rx: oneshot::Receiver<()>,
    ) -> Self {
        Self {
            inner,
            cancel_rx: Some(cancel_rx),
        }
    }
}

impl Stream for ControlledStream {
    type Item = Result<ChatCompletionChunk, AiLibError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;
        use std::task::Poll;

        // Check if cancelled
        if let Some(ref mut cancel_rx) = self.cancel_rx {
            match Future::poll(std::pin::Pin::new(cancel_rx), cx) {
                Poll::Ready(_) => {
                    self.cancel_rx = None;
                    return Poll::Ready(Some(Err(AiLibError::ProviderError(
                        "Stream cancelled".to_string(),
                    ))));
                }
                Poll::Pending => {}
            }
        }

        // Poll inner stream
        self.inner.poll_next_unpin(cx)
    }
}
