use async_trait::async_trait;
use futures::stream::Stream;

use crate::api::ChatCompletionChunk;
pub use crate::api::ChatProvider;
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};

pub struct AdapterProvider {
    name: String,
    inner: Box<dyn ChatProvider>,
}

impl AdapterProvider {
    pub fn new(name: impl Into<String>, inner: Box<dyn ChatProvider>) -> Self {
        Self {
            name: name.into(),
            inner,
        }
    }

    pub fn boxed(self) -> Box<dyn ChatProvider> {
        Box::new(self)
    }
}

#[async_trait]
impl ChatProvider for AdapterProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn chat(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        self.inner.chat(request).await
    }

    async fn stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        self.inner.stream(request).await
    }

    async fn batch(
        &self,
        requests: Vec<ChatCompletionRequest>,
        concurrency_limit: Option<usize>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        self.inner.batch(requests, concurrency_limit).await
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        self.inner.list_models().await
    }

    async fn get_model_info(&self, model_id: &str) -> Result<crate::api::ModelInfo, AiLibError> {
        self.inner.get_model_info(model_id).await
    }
}
