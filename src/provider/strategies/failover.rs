use async_trait::async_trait;
use futures::stream::Stream;
use tracing::warn;

use crate::{
    api::{ChatCompletionChunk, ChatProvider, ModelInfo},
    types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse},
};

pub struct FailoverProvider {
    name: String,
    providers: Vec<Box<dyn ChatProvider>>,
}

impl FailoverProvider {
    pub fn new(providers: Vec<Box<dyn ChatProvider>>) -> Result<Self, AiLibError> {
        if providers.is_empty() {
            return Err(AiLibError::ConfigurationError(
                "failover strategy requires at least one provider".to_string(),
            ));
        }

        let composed_name = providers
            .iter()
            .map(|p| p.name().to_string())
            .collect::<Vec<_>>()
            .join("->");

        Ok(Self {
            name: format!("failover[{composed_name}]"),
            providers,
        })
    }

    fn should_retry(error: &AiLibError) -> bool {
        error.is_retryable() || matches!(error, AiLibError::TimeoutError(_))
    }

    fn log_fail_event(provider: &dyn ChatProvider, error: &AiLibError) {
        warn!(
            target = "ai_lib.failover",
            provider = provider.name(),
            error_code = %error.error_code_with_severity(),
            "failover candidate returned an error"
        );
    }
}

#[async_trait]
impl ChatProvider for FailoverProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn chat(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        let fallback_template = request.clone();
        let mut providers_iter = self.providers.iter();

        let first = providers_iter
            .next()
            .expect("validated during construction");

        let mut last_error = match first.chat(request).await {
            Ok(resp) => return Ok(resp),
            Err(err) => {
                if !Self::should_retry(&err) {
                    return Err(err);
                }
                Self::log_fail_event(first.as_ref(), &err);
                err
            }
        };

        for provider in providers_iter {
            match provider.chat(fallback_template.clone()).await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    if !Self::should_retry(&err) {
                        return Err(err);
                    }
                    Self::log_fail_event(provider.as_ref(), &err);
                    last_error = err;
                }
            }
        }

        Err(last_error)
    }

    async fn stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        let fallback_template = request.clone();
        let mut providers_iter = self.providers.iter();

        let first = providers_iter
            .next()
            .expect("validated during construction");

        let mut last_error = match first.stream(request).await {
            Ok(resp) => return Ok(resp),
            Err(err) => {
                if !Self::should_retry(&err) {
                    return Err(err);
                }
                Self::log_fail_event(first.as_ref(), &err);
                err
            }
        };

        for provider in providers_iter {
            match provider.stream(fallback_template.clone()).await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    if !Self::should_retry(&err) {
                        return Err(err);
                    }
                    Self::log_fail_event(provider.as_ref(), &err);
                    last_error = err;
                }
            }
        }

        Err(last_error)
    }

    async fn batch(
        &self,
        requests: Vec<ChatCompletionRequest>,
        concurrency_limit: Option<usize>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        let mut providers_iter = self.providers.iter();
        let first = providers_iter
            .next()
            .expect("validated during construction");

        let mut last_error = match first.batch(requests.clone(), concurrency_limit).await {
            Ok(resp) => return Ok(resp),
            Err(err) => {
                if !Self::should_retry(&err) {
                    return Err(err);
                }
                Self::log_fail_event(first.as_ref(), &err);
                err
            }
        };

        for provider in providers_iter {
            match provider.batch(requests.clone(), concurrency_limit).await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    if !Self::should_retry(&err) {
                        return Err(err);
                    }
                    Self::log_fail_event(provider.as_ref(), &err);
                    last_error = err;
                }
            }
        }

        Err(last_error)
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        let mut last_error = None;
        for provider in &self.providers {
            match provider.list_models().await {
                Ok(models) => return Ok(models),
                Err(err) => {
                    if !Self::should_retry(&err) {
                        return Err(err);
                    }
                    Self::log_fail_event(provider.as_ref(), &err);
                    last_error = Some(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            AiLibError::ConfigurationError(
                "failover strategy could not contact any provider".to_string(),
            )
        }))
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
            "model {model_id} not available in failover chain"
        )))
    }
}
