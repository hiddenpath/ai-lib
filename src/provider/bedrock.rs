use crate::api::{ChatApi, ChatCompletionChunk, ModelInfo, ModelPermission};
use crate::types::{ChatCompletionRequest, ChatCompletionResponse, AiLibError};
use crate::transport::{HttpTransport, DynHttpTransportRef};
use futures::stream::Stream;

pub struct BedrockAdapter {
    transport: DynHttpTransportRef,
    // Note: Region/credentials fields or AWS SDK client can be added in future versions
}

impl BedrockAdapter {
    pub fn new() -> Result<Self, AiLibError> {
        let boxed = HttpTransport::new().boxed();
        Ok(Self { transport: boxed })
    }

    /// Construct using object-safe transport reference for testing or SDK integration
    pub fn with_transport_ref(transport: DynHttpTransportRef) -> Result<Self, AiLibError> {
        Ok(Self { transport })
    }
}

#[async_trait::async_trait]
impl ChatApi for BedrockAdapter {
    async fn chat_completion(&self, _request: ChatCompletionRequest) -> Result<ChatCompletionResponse, AiLibError> {
        Err(AiLibError::ProviderError("AWS Bedrock adapter not implemented - requires AWS SDK or SigV4 signing".to_string()))
    }

    async fn chat_completion_stream(&self, _request: ChatCompletionRequest) -> Result<Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>, AiLibError> {
        Err(AiLibError::ProviderError("AWS Bedrock stream adapter not implemented".to_string()))
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        Err(AiLibError::ProviderError("AWS Bedrock list_models not implemented - integrate AWS SDK or implement SigV4 HTTP calls".to_string()))
    }

    async fn get_model_info(&self, _model_id: &str) -> Result<crate::api::ModelInfo, AiLibError> {
        Ok(ModelInfo { id: _model_id.to_string(), object: "model".to_string(), created: 0, owned_by: "aws".to_string(), permission: vec![ModelPermission { id: "default".to_string(), object: "model_permission".to_string(), created: 0, allow_create_engine: false, allow_sampling: true, allow_logprobs: false, allow_search_indices: false, allow_view: true, allow_fine_tuning: false, organization: "*".to_string(), group: None, is_blocking: false }] })
    }
}
