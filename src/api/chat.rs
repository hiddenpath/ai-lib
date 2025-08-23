use async_trait::async_trait;
use crate::types::{ChatCompletionRequest, ChatCompletionResponse, AiLibError};
use futures::stream::Stream;

/// 通用的聊天API接口，定义所有AI服务的核心能力
/// 
/// Generic chat API interface
/// 
/// This trait defines the core capabilities that all AI services should have,
/// without depending on any specific model implementation details
#[async_trait]
pub trait ChatApi: Send + Sync {
    /// Send chat completion request
    /// 
    /// # Arguments
    /// * `request` - Generic chat completion request
    /// 
    /// # Returns
    /// * `Result<ChatCompletionResponse, AiLibError>` - Returns response on success, error on failure
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError>;
    
    /// Streaming chat completion request
    /// 
    /// # Arguments
    /// * `request` - Generic chat completion request
    /// 
    /// # Returns
    /// * `Result<impl Stream<Item = Result<ChatCompletionChunk, AiLibError>>, AiLibError>` - Returns streaming response on success
    async fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>, AiLibError>;
    
    /// Get list of supported models
    /// 
    /// # Returns
    /// * `Result<Vec<String>, AiLibError>` - Returns model list on success, error on failure
    async fn list_models(&self) -> Result<Vec<String>, AiLibError>;
    
    /// Get model information
    /// 
    /// # Arguments
    /// * `model_id` - Model ID
    /// 
    /// # Returns
    /// * `Result<ModelInfo, AiLibError>` - Returns model information on success, error on failure
    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo, AiLibError>;
}

/// 流式响应的数据块
/// 
/// Streaming response data chunk
#[derive(Debug, Clone)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChoiceDelta>,
}

/// 流式响应的选择项增量
/// 
/// Streaming response choice delta
#[derive(Debug, Clone)]
pub struct ChoiceDelta {
    pub index: u32,
    pub delta: MessageDelta,
    pub finish_reason: Option<String>,
}

/// 消息增量
/// 
/// Message delta
#[derive(Debug, Clone)]
pub struct MessageDelta {
    pub role: Option<Role>,
    pub content: Option<String>,
}

/// 模型信息
/// 
/// Model information
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
    pub permission: Vec<ModelPermission>,
}

/// 模型权限
/// 
/// Model permission
#[derive(Debug, Clone)]
pub struct ModelPermission {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub allow_create_engine: bool,
    pub allow_sampling: bool,
    pub allow_logprobs: bool,
    pub allow_search_indices: bool,
    pub allow_view: bool,
    pub allow_fine_tuning: bool,
    pub organization: String,
    pub group: Option<String>,
    pub is_blocking: bool,
}

// Re-export Role type as it's also needed in streaming responses
use crate::types::Role;
