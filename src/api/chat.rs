use crate::types::{AiLibError, ChatCompletionRequest, ChatCompletionResponse};
use async_trait::async_trait;
use futures::stream::Stream;

/// Chat API module
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
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    >;

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

    /// Batch chat completion requests
    ///
    /// # Arguments
    /// * `requests` - Vector of chat completion requests
    /// * `concurrency_limit` - Optional concurrency limit for concurrent processing
    ///
    /// # Returns
    /// * `Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError>` - Returns vector of results
    async fn chat_completion_batch(
        &self,
        requests: Vec<ChatCompletionRequest>,
        concurrency_limit: Option<usize>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        batch_utils::process_batch_concurrent(self, requests, concurrency_limit).await
    }
}

/// Streaming response data chunk
#[derive(Debug, Clone)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChoiceDelta>,
}

/// Streaming response choice delta
#[derive(Debug, Clone)]
pub struct ChoiceDelta {
    pub index: u32,
    pub delta: MessageDelta,
    pub finish_reason: Option<String>,
}

/// Message delta
#[derive(Debug, Clone)]
pub struct MessageDelta {
    pub role: Option<Role>,
    pub content: Option<String>,
}

/// Model information
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

/// Batch processing result containing successful and failed responses
#[derive(Debug)]
pub struct BatchResult {
    pub successful: Vec<ChatCompletionResponse>,
    pub failed: Vec<(usize, AiLibError)>,
    pub total_requests: usize,
    pub total_successful: usize,
    pub total_failed: usize,
}

impl BatchResult {
    /// Create a new batch result
    pub fn new(total_requests: usize) -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
            total_requests,
            total_successful: 0,
            total_failed: 0,
        }
    }

    /// Add a successful response
    pub fn add_success(&mut self, response: ChatCompletionResponse) {
        self.successful.push(response);
        self.total_successful += 1;
    }

    /// Add a failed response with index
    pub fn add_failure(&mut self, index: usize, error: AiLibError) {
        self.failed.push((index, error));
        self.total_failed += 1;
    }

    /// Check if all requests were successful
    pub fn all_successful(&self) -> bool {
        self.total_failed == 0
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.total_successful as f64 / self.total_requests as f64) * 100.0
        }
    }
}

/// Batch processing utility functions
pub mod batch_utils {
    use super::*;
    use futures::stream::{self, StreamExt};
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    /// Default implementation for concurrent batch processing
    pub async fn process_batch_concurrent<T: ChatApi + ?Sized>(
        api: &T,
        requests: Vec<ChatCompletionRequest>,
        concurrency_limit: Option<usize>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let semaphore = concurrency_limit.map(|limit| Arc::new(Semaphore::new(limit)));

        let futures = requests.into_iter().enumerate().map(|(index, request)| {
            let api_ref = api;
            let semaphore_ref = semaphore.clone();

            async move {
                // Acquire permit if concurrency limit is set
                let _permit = if let Some(sem) = &semaphore_ref {
                    match sem.acquire().await {
                        Ok(permit) => Some(permit),
                        Err(_) => {
                            return (
                                index,
                                Err(AiLibError::ProviderError(
                                    "Failed to acquire semaphore permit".to_string(),
                                )),
                            )
                        }
                    }
                } else {
                    None
                };

                // Process the request
                let result = api_ref.chat_completion(request).await;

                // Return result with index for ordering
                (index, result)
            }
        });

        // Execute all futures concurrently
        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(concurrency_limit.unwrap_or(usize::MAX))
            .collect()
            .await;

        // Sort results by original index to maintain order
        let mut sorted_results = Vec::with_capacity(results.len());
        sorted_results.resize_with(results.len(), || {
            Err(AiLibError::ProviderError("Placeholder".to_string()))
        });
        for (index, result) in results {
            sorted_results[index] = result;
        }

        Ok(sorted_results)
    }

    /// Sequential batch processing implementation
    pub async fn process_batch_sequential<T: ChatApi + ?Sized>(
        api: &T,
        requests: Vec<ChatCompletionRequest>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            let result = api.chat_completion(request).await;
            results.push(result);
        }

        Ok(results)
    }

    /// Smart batch processing: automatically choose processing strategy based on request type and size
    pub async fn process_batch_smart<T: ChatApi + ?Sized>(
        api: &T,
        requests: Vec<ChatCompletionRequest>,
        concurrency_limit: Option<usize>,
    ) -> Result<Vec<Result<ChatCompletionResponse, AiLibError>>, AiLibError> {
        let request_count = requests.len();

        // For small batches, use sequential processing
        if request_count <= 3 {
            return process_batch_sequential(api, requests).await;
        }

        // For larger batches, use concurrent processing
        process_batch_concurrent(api, requests, concurrency_limit).await
    }
}
