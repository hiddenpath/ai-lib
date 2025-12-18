#![cfg(feature = "interceptors")]

use ai_lib::api::{ChatProvider, ModelInfo};
use ai_lib::interceptors::{retry::RetryWrapper, RequestContext, RetryInterceptor};
use ai_lib::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse, Message, Role};
use ai_lib::{ChatCompletionChunk, Content};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Mock Adapter that fails N times then succeeds
struct MockRetryAdapter {
    failures_remaining: Arc<Mutex<u32>>,
    error_type: AiLibError,
}

impl MockRetryAdapter {
    fn new(failures: u32, error_type: AiLibError) -> Self {
        Self {
            failures_remaining: Arc::new(Mutex::new(failures)),
            error_type,
        }
    }
}

#[async_trait]
impl ChatProvider for MockRetryAdapter {
    fn name(&self) -> &str {
        "mock_retry"
    }

    async fn chat(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        let mut failures = self.failures_remaining.lock().unwrap();
        if *failures > 0 {
            *failures -= 1;
            return Err(self.error_type.clone());
        }
        Ok(ChatCompletionResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "test".to_string(),
            choices: vec![],
            usage: ai_lib::Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            usage_status: ai_lib::UsageStatus::Finalized,
        })
    }

    async fn stream(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<
        Box<
            dyn futures::stream::Stream<Item = Result<ChatCompletionChunk, AiLibError>>
                + Send
                + Unpin,
        >,
        AiLibError,
    > {
        unimplemented!()
    }

    async fn batch(
        &self,
        _requests: Vec<ChatCompletionRequest>,
        _concurrency_limit: Option<usize>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        unimplemented!()
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        Ok(vec!["test".to_string()])
    }

    async fn get_model_info(&self, _model_name: &str) -> Result<ModelInfo, AiLibError> {
        Ok(ModelInfo {
            id: "test".to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "test".to_string(),
            permission: vec![],
        })
    }
}

#[tokio::test]
async fn test_retry_success_after_failures() {
    // Setup: Fail 2 times with RateLimitExceeded, then succeed
    // Retry config: 3 retries
    let adapter = MockRetryAdapter::new(2, AiLibError::RateLimitExceeded("limit".to_string()));

    let retry_wrapper = RetryWrapper::new(RetryInterceptor::new(
        3,
        Duration::from_millis(1),
        Duration::from_millis(10),
    ));

    let ctx = RequestContext {
        provider: "test".to_string(),
        model: "test".to_string(),
    };

    let req = ChatCompletionRequest::new(
        "test".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("test".to_string()),
            function_call: None,
        }],
    );

    let adapter_failures = adapter.failures_remaining.clone();
    let adapter_error = adapter.error_type.clone();

    let result = retry_wrapper
        .execute(&ctx, &req, || {
            let adapter_clone = MockRetryAdapter {
                failures_remaining: adapter_failures.clone(),
                error_type: adapter_error.clone(),
            };
            let req_clone = req.clone();
            async move { adapter_clone.chat(req_clone).await }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(*adapter.failures_remaining.lock().unwrap(), 0);
}

#[tokio::test]
async fn test_retry_exhausted() {
    // Setup: Fail 4 times, but only retry 3 times (total 4 attempts)
    let adapter = MockRetryAdapter::new(5, AiLibError::RateLimitExceeded("limit".to_string()));

    let retry_wrapper = RetryWrapper::new(RetryInterceptor::new(
        3,
        Duration::from_millis(1),
        Duration::from_millis(10),
    ));

    let ctx = RequestContext {
        provider: "test".to_string(),
        model: "test".to_string(),
    };

    let req = ChatCompletionRequest::new(
        "test".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("test".to_string()),
            function_call: None,
        }],
    );

    let adapter_failures = adapter.failures_remaining.clone();
    let adapter_error = adapter.error_type.clone();

    let result = retry_wrapper
        .execute(&ctx, &req, || {
            let adapter_clone = MockRetryAdapter {
                failures_remaining: adapter_failures.clone(),
                error_type: adapter_error.clone(),
            };
            let req_clone = req.clone();
            async move { adapter_clone.chat(req_clone).await }
        })
        .await;

    assert!(result.is_err());
    // With 3 retries (4 total attempts), we should have 1 failure remaining (5 - 4 = 1)
    assert_eq!(*adapter.failures_remaining.lock().unwrap(), 1);
}

#[tokio::test]
async fn test_non_retryable_error() {
    // Setup: Fail with InvalidRequest (non-retryable)
    let adapter = MockRetryAdapter::new(2, AiLibError::InvalidRequest("invalid".to_string()));

    let retry_wrapper = RetryWrapper::new(RetryInterceptor::new(
        3,
        Duration::from_millis(1),
        Duration::from_millis(10),
    ));

    let ctx = RequestContext {
        provider: "test".to_string(),
        model: "test".to_string(),
    };

    let req = ChatCompletionRequest::new(
        "test".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("test".to_string()),
            function_call: None,
        }],
    );

    let adapter_failures = adapter.failures_remaining.clone();
    let adapter_error = adapter.error_type.clone();

    let result = retry_wrapper
        .execute(&ctx, &req, || {
            let adapter_clone = MockRetryAdapter {
                failures_remaining: adapter_failures.clone(),
                error_type: adapter_error.clone(),
            };
            let req_clone = req.clone();
            async move { adapter_clone.chat(req_clone).await }
        })
        .await;

    assert!(result.is_err());
    // Non-retryable error should not retry, so only 1 attempt (2 - 1 = 1 remaining)
    assert_eq!(*adapter.failures_remaining.lock().unwrap(), 1);
}
