use ai_lib::{
    api::ModelInfo,
    provider::strategies::{FailoverProvider, RoundRobinProvider},
    types::{
        common::{Content, Message},
        error::AiLibError,
        request::ChatCompletionRequest,
        response::{ChatCompletionResponse, Usage, UsageStatus},
        Choice, Role,
    },
    ChatProvider,
};
use async_trait::async_trait;
use futures::stream::Stream;

struct StaticProvider {
    name: &'static str,
    result: Result<ChatCompletionResponse, AiLibError>,
}

impl StaticProvider {
    fn new(name: &'static str, result: Result<ChatCompletionResponse, AiLibError>) -> Self {
        Self { name, result }
    }
}

#[async_trait]
impl ChatProvider for StaticProvider {
    fn name(&self) -> &str {
        self.name
    }

    async fn chat(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        self.result.clone()
    }

    async fn stream(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ai_lib::api::ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        Err(AiLibError::UnsupportedFeature("not implemented".into()))
    }

    async fn batch(
        &self,
        _requests: Vec<ChatCompletionRequest>,
        _concurrency_limit: Option<usize>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        Err(AiLibError::UnsupportedFeature("not implemented".into()))
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        Ok(vec![])
    }

    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo, AiLibError> {
        Ok(ModelInfo {
            id: model_id.to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: self.name.to_string(),
            permission: vec![],
        })
    }
}

fn dummy_request() -> ChatCompletionRequest {
    ChatCompletionRequest::new(
        "dummy-model".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("ping".into()),
            function_call: None,
        }],
    )
}

fn response_with_content(text: &str) -> ChatCompletionResponse {
    ChatCompletionResponse {
        id: "test".into(),
        object: "chat.completion".into(),
        created: 0,
        model: "dummy".into(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: Role::Assistant,
                content: Content::Text(text.into()),
                function_call: None,
            },
            finish_reason: None,
        }],
        usage: Usage {
            prompt_tokens: 1,
            completion_tokens: 1,
            total_tokens: 2,
        },
        usage_status: UsageStatus::Finalized,
    }
}

#[tokio::test]
async fn failover_provider_advances_on_retryable_error() {
    let failing = StaticProvider::new("failing", Err(AiLibError::NetworkError("boom".into())));
    let succeeding = StaticProvider::new("success", Ok(response_with_content("ok")));

    let provider = FailoverProvider::new(vec![Box::new(failing), Box::new(succeeding)])
        .expect("strategy builds");

    let resp = provider
        .chat(dummy_request())
        .await
        .expect("second provider succeeds");
    assert_eq!(
        resp.first_text().unwrap(),
        "ok",
        "failover should eventually return fallback response"
    );
}

#[tokio::test]
async fn round_robin_provider_rotates_providers() {
    let first = StaticProvider::new("one", Ok(response_with_content("one")));
    let second = StaticProvider::new("two", Ok(response_with_content("two")));

    let provider =
        RoundRobinProvider::new(vec![Box::new(first), Box::new(second)]).expect("strategy builds");

    let r1 = provider.chat(dummy_request()).await.expect("first call");
    let r2 = provider.chat(dummy_request()).await.expect("second call");

    assert_eq!(r1.first_text().unwrap(), "one");
    assert_eq!(r2.first_text().unwrap(), "two");
}
