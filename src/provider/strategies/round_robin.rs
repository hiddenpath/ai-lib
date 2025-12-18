use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use futures::stream::Stream;

use crate::{
    api::{ChatCompletionChunk, ChatProvider, ModelInfo},
    types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse},
};

pub struct RoundRobinProvider {
    name: String,
    providers: Vec<Box<dyn ChatProvider>>,
    cursor: AtomicUsize,
}

impl RoundRobinProvider {
    pub fn new(providers: Vec<Box<dyn ChatProvider>>) -> Result<Self, AiLibError> {
        if providers.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "round_robin strategy requires at least one provider".to_string(),
            ));
        }

        let composed_name = providers
            .iter()
            .map(|p| p.name().to_string())
            .collect::<Vec<_>>()
            .join(",");

        Ok(Self {
            name: format!("round_robin[{composed_name}]"),
            providers,
            cursor: AtomicUsize::new(0),
        })
    }

    fn select(&self) -> &dyn ChatProvider {
        let idx = self.cursor.fetch_add(1, Ordering::Relaxed) % self.providers.len();
        self.providers[idx].as_ref()
    }
}

#[async_trait]
impl ChatProvider for RoundRobinProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn chat(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        self.select().chat(request).await
    }

    async fn stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        self.select().stream(request).await
    }

    async fn batch(
        &self,
        requests: Vec<ChatCompletionRequest>,
        concurrency_limit: Option<usize>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        self.select().batch(requests, concurrency_limit).await
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        self.select().list_models().await
    }

    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo, AiLibError> {
        for provider in &self.providers {
            match provider.get_model_info(model_id).await {
                Ok(info) => return Ok(info),
                Err(err) => {
                    if matches!(err, AiLibError::ModelNotFound(_)) {
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(AiLibError::ModelNotFound(format!(
            "model {model_id} not available in round robin chain"
        )))
    }
}
