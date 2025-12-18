use ai_lib::api::{ChatCompletionChunk, ChatProvider, ModelInfo};
use ai_lib::provider::strategies::RoutingStrategyBuilder;
use ai_lib::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};
use async_trait::async_trait;
use futures::stream::Stream;

struct DummyProvider {
    name: &'static str,
}

impl DummyProvider {
    fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[async_trait]
impl ChatProvider for DummyProvider {
    fn name(&self) -> &str {
        self.name
    }

    async fn chat(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        Err(AiLibError::UnsupportedFeature(format!(
            "{} is a test provider",
            self.name
        )))
    }

    async fn stream(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        Err(AiLibError::UnsupportedFeature(
            "stream unsupported in dummy provider".to_string(),
        ))
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        Ok(vec![])
    }

    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo, AiLibError> {
        Err(AiLibError::ModelNotFound(model_id.to_string()))
    }
}

#[test]
fn routing_builder_produces_round_robin_name() {
    let builder = RoutingStrategyBuilder::new()
        .with_provider(Box::new(DummyProvider::new("primary")))
        .with_provider(Box::new(DummyProvider::new("secondary")));

    let round_robin = builder.build_round_robin().expect("round robin");
    assert!(
        round_robin.name().starts_with("round_robin["),
        "expected descriptive name, got {}",
        round_robin.name()
    );
}

#[test]
fn routing_builder_produces_failover_name() {
    let builder = RoutingStrategyBuilder::new()
        .with_provider(Box::new(DummyProvider::new("primary")))
        .with_provider(Box::new(DummyProvider::new("secondary")));

    let failover = builder.build_failover().expect("failover");
    assert!(
        failover.name().starts_with("failover["),
        "expected descriptive name, got {}",
        failover.name()
    );
}
