use crate::api::{ChatApi, ChatCompletionChunk};
use crate::types::{ChatCompletionRequest, ChatCompletionResponse, AiLibError};
use crate::provider::{OpenAiAdapter, GeminiAdapter, GenericAdapter, ProviderConfigs};
use futures::stream::Stream;
use futures::Future;
use tokio::sync::oneshot;

/// 支持的AI模型提供商枚举
/// 
/// AI model provider enumeration
#[derive(Debug, Clone, Copy)]
pub enum Provider {
    Groq,
    OpenAI,
    DeepSeek,
    Anthropic,
    Gemini, // 独立适配器
}

/// 统一的AI客户端，支持多个提供商的混合架构实现
/// 
/// Unified AI client for multiple providers with hybrid architecture
/// 
/// Usage example:
/// ```rust
/// use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Switch model providers by just changing the Provider value
///     let client = AiClient::new(Provider::Groq)?;
///     
///     let request = ChatCompletionRequest::new(
///         "test-model".to_string(),
///         vec![Message {
///             role: Role::User,
///             content: "Hello".to_string(),
///         }],
///     );
///     
///     // Note: GROQ_API_KEY environment variable must be set to actually call the API
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
/// # Proxy Server Configuration
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
}

impl AiClient {
    /// Create a new AI client
    /// 
    /// # Arguments
    /// * `provider` - Choose the AI model provider to use
    /// 
    /// # Returns
    /// * `Result<Self, AiLibError>` - Returns client instance on success, error on failure
    /// 
    /// # Example
/// ```rust
/// use ai_lib::{AiClient, Provider};
/// 
/// let client = AiClient::new(Provider::Groq)?;
/// # Ok::<(), ai_lib::AiLibError>(())
/// ```
    pub fn new(provider: Provider) -> Result<Self, AiLibError> {
        let adapter: Box<dyn ChatApi> = match provider {
            // Configuration-driven generic adapters - proving OpenAI compatibility
            Provider::Groq => Box::new(GenericAdapter::new(ProviderConfigs::groq_as_generic())?),
            Provider::DeepSeek => Box::new(GenericAdapter::new(ProviderConfigs::deepseek())?),
            Provider::Anthropic => Box::new(GenericAdapter::new(ProviderConfigs::anthropic())?),
            
            // Independent adapters (special API formats)
            Provider::OpenAI => Box::new(OpenAiAdapter::new()?),
            Provider::Gemini => Box::new(GeminiAdapter::new()?),
        };
        
        Ok(Self { provider, adapter })
    }
    
    /// Send chat completion request
    /// 
    /// # Arguments
    /// * `request` - Chat completion request
    /// 
    /// # Returns
    /// * `Result<ChatCompletionResponse, AiLibError>` - Returns response on success, error on failure
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
    /// * `Result<impl Stream<Item = Result<ChatCompletionChunk, AiLibError>>, AiLibError>` - Returns streaming response on success
    pub async fn chat_completion_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>, AiLibError> {
        request.stream = Some(true);
        self.adapter.chat_completion_stream(request).await
    }
    
    /// Streaming chat completion request with cancellation control
    /// 
    /// # Arguments
    /// * `request` - Chat completion request
    /// 
    /// # Returns
    /// * `(Stream, CancelHandle)` - Streaming response and cancel handle
    pub async fn chat_completion_stream_with_cancel(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<(Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>, CancelHandle), AiLibError> {
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let stream = self.chat_completion_stream(request).await?;
        
        let cancel_handle = CancelHandle { sender: Some(cancel_tx) };
        let controlled_stream = ControlledStream::new(stream, cancel_rx);
        
        Ok((Box::new(Box::pin(controlled_stream)), cancel_handle))
    }
    
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
            Provider::Groq => Box::new(GenericAdapter::new(ProviderConfigs::groq_as_generic())?),
            Provider::DeepSeek => Box::new(GenericAdapter::new(ProviderConfigs::deepseek())?),
            Provider::Anthropic => Box::new(GenericAdapter::new(ProviderConfigs::anthropic())?),
            Provider::OpenAI => Box::new(OpenAiAdapter::new()?),
            Provider::Gemini => Box::new(GeminiAdapter::new()?),
        };
        
        self.provider = provider;
        self.adapter = new_adapter;
        Ok(())
    }
    
    /// Get the currently used provider
    pub fn current_provider(&self) -> Provider {
        self.provider
    }
}

/// 流式响应取消句柄
/// 
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

/// 可控制的流式响应
/// 
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
                    return Poll::Ready(Some(Err(AiLibError::ProviderError("Stream cancelled".to_string()))));
                }
                Poll::Pending => {}
            }
        }
        
        // Poll inner stream
        self.inner.poll_next_unpin(cx)
    }
}
