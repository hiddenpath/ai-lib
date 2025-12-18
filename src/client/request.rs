//! Request execution module.
//!
//! This module handles single synchronous chat completion requests.
//! It coordinates with:
//! - `ProviderFactory`: Creates provider adapters
//! - `InterceptorPipeline`: Applies interceptors

use super::AiClient;
use crate::rate_limiter::BackpressurePermit;
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};
use tracing::warn;

#[allow(unused_mut)]
pub async fn chat_completion(
    client: &AiClient,
    request: ChatCompletionRequest,
) -> Result<ChatCompletionResponse, AiLibError> {
    // Acquire backpressure permit if configured
    let _bp_permit: Option<BackpressurePermit> = if let Some(ctrl) = &client.backpressure {
        match ctrl.acquire_permit().await {
            Ok(p) => Some(p),
            Err(_) => {
                return Err(AiLibError::RateLimitExceeded(
                    "Backpressure: no permits available".to_string(),
                ))
            }
        }
    } else {
        None
    };

    let provider = client.provider_id();
    let mut processed_request = client.prepare_chat_request(request);

    loop {
        match execute_once(client, &processed_request).await {
            Ok(resp) => return Ok(resp),
            Err(err) => {
                if client.model_resolver.looks_like_invalid_model(&err) {
                    if let Some(resolution) =
                        client.fallback_model_after_invalid(&processed_request.model)
                    {
                        warn!(
                            target = "ai_lib.model",
                            provider = ?provider,
                            failed_model = %processed_request.model,
                            fallback_model = %resolution.model,
                            source = ?resolution.source,
                            "Retrying request with fallback model"
                        );
                        processed_request.model = resolution.model;
                        continue;
                    }
                    let decorated = client.model_resolver.decorate_invalid_model_error(
                        provider,
                        &processed_request.model,
                        err,
                    );
                    return Err(decorated.with_context("AiClient::chat_completion"));
                }

                return Err(err.with_context("AiClient::chat_completion"));
            }
        }
    }
}

async fn execute_once(
    client: &AiClient,
    request: &ChatCompletionRequest,
) -> Result<ChatCompletionResponse, AiLibError> {
    #[cfg(feature = "interceptors")]
    if let Some(p) = &client.interceptor_pipeline {
        let ctx = crate::interceptors::RequestContext {
            provider: client.provider_name().to_lowercase(),
            model: request.model.clone(),
        };
        return p
            .execute(&ctx, request, || client.chat_provider.chat(request.clone()))
            .await;
    }

    client.chat_provider.chat(request.clone()).await
}
