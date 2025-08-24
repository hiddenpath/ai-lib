use crate::api::{ChatApi, ChatCompletionChunk};
use crate::provider::{OpenAiAdapter, GenericAdapter, ProviderConfigs};
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};
use futures::stream::Stream;
use futures::Future;
use tokio::sync::oneshot;

/// AI模型提供商枚举
#[derive(Debug, Clone, Copy)]
pub enum Provider {
    Groq,
    OpenAI,
    DeepSeek,
    // 特殊适配器
    // Gemini,  // 需要独立适配器
}

/// 统一AI客户端
///
/// 使用示例：
/// ```rust
/// use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // 切换模型提供商，只需更改 Provider 的值
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
///     // 注意：这里需要设置GROQ_API_KEY环境变量才能实际调用API
///     // 可选：设置AI_PROXY_URL环境变量使用代理服务器
///     // let response = client.chat_completion(request).await?;
///     
///     println!("Client created successfully with provider: {:?}", client.current_provider());
///     println!("Request prepared for model: {}", request.model);
///     
///     Ok(())
/// }
/// ```
///
/// # 代理服务器配置
///
/// 通过设置 `AI_PROXY_URL` 环境变量来配置代理服务器：
///
/// ```bash
/// export AI_PROXY_URL=http://proxy.example.com:8080
/// ```
///
/// 支持的代理格式：
/// - HTTP代理: `http://proxy.example.com:8080`
/// - HTTPS代理: `https://proxy.example.com:8080`  
/// - 带认证: `http://user:pass@proxy.example.com:8080`
pub struct AiClient {
    provider: Provider,
    adapter: Box<dyn ChatApi>,
}

impl AiClient {
    /// 创建新的AI客户端
    ///
    /// # Arguments
    /// * `provider` - 选择要使用的AI模型提供商
    ///
    /// # Returns
    /// * `Result<Self, AiLibError>` - 成功时返回客户端实例，失败时返回错误
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
            // 使用配置驱动的通用适配器
            Provider::Groq => Box::new(GenericAdapter::new(ProviderConfigs::groq())?),
            Provider::DeepSeek => Box::new(GenericAdapter::new(ProviderConfigs::deepseek())?),
            
            // 使用独立适配器
            Provider::OpenAI => Box::new(OpenAiAdapter::new()?),
        };

        Ok(Self { provider, adapter })
    }

    /// 发送聊天完成请求
    ///
    /// # Arguments
    /// * `request` - 聊天完成请求
    ///
    /// # Returns
    /// * `Result<ChatCompletionResponse, AiLibError>` - 成功时返回响应，失败时返回错误
    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        self.adapter.chat_completion(request).await
    }

    /// 流式聊天完成请求
    ///
    /// # Arguments
    /// * `request` - 聊天完成请求
    ///
    /// # Returns
    /// * `Result<impl Stream<Item = Result<ChatCompletionChunk, AiLibError>>, AiLibError>` - 成功时返回流式响应
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

    /// 带取消控制的流式聊天完成请求
    ///
    /// # Arguments
    /// * `request` - 聊天完成请求
    ///
    /// # Returns
    /// * `(Stream, CancelHandle)` - 流式响应和取消句柄
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
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let stream = self.chat_completion_stream(request).await?;

        let cancel_handle = CancelHandle {
            sender: Some(cancel_tx),
        };
        let controlled_stream = ControlledStream::new(stream, cancel_rx);

        Ok((Box::new(Box::pin(controlled_stream)), cancel_handle))
    }

    /// 获取支持的模型列表
    ///
    /// # Returns
    /// * `Result<Vec<String>, AiLibError>` - 成功时返回模型列表，失败时返回错误
    pub async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        self.adapter.list_models().await
    }

    /// 切换AI模型提供商
    ///
    /// # Arguments
    /// * `provider` - 新的提供商
    ///
    /// # Returns
    /// * `Result<(), AiLibError>` - 成功时返回()，失败时返回错误
    ///
    /// # Example
    /// ```rust
    /// use ai_lib::{AiClient, Provider};
    ///
    /// let mut client = AiClient::new(Provider::Groq)?;
    /// // 从Groq切换到Groq（演示切换功能）
    /// client.switch_provider(Provider::Groq)?;
    /// # Ok::<(), ai_lib::AiLibError>(())
    /// ```
    pub fn switch_provider(&mut self, provider: Provider) -> Result<(), AiLibError> {
        let new_adapter: Box<dyn ChatApi> = match provider {
            Provider::Groq => Box::new(GenericAdapter::new(ProviderConfigs::groq())?),
            Provider::DeepSeek => Box::new(GenericAdapter::new(ProviderConfigs::deepseek())?),
            Provider::OpenAI => Box::new(OpenAiAdapter::new()?),
        };

        self.provider = provider;
        self.adapter = new_adapter;
        Ok(())
    }

    /// 获取当前使用的提供商
    pub fn current_provider(&self) -> Provider {
        self.provider
    }
}

/// 流式响应取消句柄
pub struct CancelHandle {
    sender: Option<oneshot::Sender<()>>,
}

impl CancelHandle {
    /// 取消流式响应
    pub fn cancel(mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(());
        }
    }
}

/// 可控制的流式响应
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

        // 检查是否被取消
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

        // 轮询内部流
        self.inner.poll_next_unpin(cx)
    }
}
