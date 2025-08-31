use crate::api::{ChatApi, ChatCompletionChunk};
use crate::metrics::{Metrics, NoopMetrics};
use crate::provider::{
    CohereAdapter, GeminiAdapter, GenericAdapter, MistralAdapter, OpenAiAdapter, ProviderConfigs,
};
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};
use futures::stream::Stream;
use futures::Future;
use std::sync::Arc;
use tokio::sync::oneshot;

/// Unified AI client module
///
/// AI model provider enumeration
#[derive(Debug, Clone, Copy)]
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
        AiClientBuilder::new(provider).build()
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
        }
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
        let adapter: Box<dyn ChatApi> = match self.provider {
            // Use config-driven generic adapter
            Provider::Groq => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::XaiGrok => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::Ollama => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::DeepSeek => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::Qwen => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::BaiduWenxin => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::TencentHunyuan => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::IflytekSpark => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::Moonshot => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::Anthropic => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::AzureOpenAI => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::HuggingFace => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            Provider::TogetherAI => {
                if let Some(custom_transport) = transport {
                    Box::new(GenericAdapter::with_transport_ref(
                        config,
                        custom_transport,
                    )?)
                } else {
                    Box::new(GenericAdapter::new(config)?)
                }
            }
            // Use independent adapters (these don't support custom configuration)
            Provider::OpenAI => Box::new(OpenAiAdapter::new()?),
            Provider::Gemini => Box::new(GeminiAdapter::new()?),
            Provider::Mistral => Box::new(MistralAdapter::new()?),
            Provider::Cohere => Box::new(CohereAdapter::new()?),
        };

        // 7. Create AiClient
        let client = AiClient {
            provider: self.provider,
            adapter,
            metrics: self.metrics.unwrap_or_else(|| Arc::new(NoopMetrics::new())),
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
        match self.provider {
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
            // These providers don't support custom configuration
            Provider::OpenAI | Provider::Gemini | Provider::Mistral | Provider::Cohere => {
                Err(AiLibError::ConfigurationError(
                    "This provider does not support custom configuration".to_string(),
                ))
            }
        }
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
