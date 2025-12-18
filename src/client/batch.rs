//! Batch processing module.
//!
//! This module handles concurrent execution of multiple chat completion requests.
//! It supports:
//! - Fixed concurrency limits
//! - Smart batching (auto-tuning concurrency)

use super::AiClient;
use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};

pub async fn chat_completion_batch(
    client: &AiClient,
    requests: Vec<ChatCompletionRequest>,
    concurrency_limit: Option<usize>,
) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
    client
        .chat_provider
        .batch(requests, concurrency_limit)
        .await
}

pub async fn chat_completion_batch_smart(
    client: &AiClient,
    requests: Vec<ChatCompletionRequest>,
) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
    // Use sequential processing for small batches, concurrent processing for large batches
    let concurrency_limit = if requests.len() <= 3 { None } else { Some(10) };
    chat_completion_batch(client, requests, concurrency_limit).await
}
