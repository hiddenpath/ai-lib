//! Streaming request execution module.
//!
//! This module handles streaming chat completion requests.
//! It supports:
//! - Standard streaming
//! - Cancellable streaming via `CancelHandle`
//! - Backpressure control

use super::AiClient;
use crate::api::ChatCompletionChunk;
use crate::rate_limiter::BackpressurePermit;
use crate::types::{AiLibError, ChatCompletionRequest};
use futures::stream::Stream;
use futures::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;
use tracing::warn;

/// Streaming response cancel handle
pub struct CancelHandle {
    sender: Option<oneshot::Sender<()>>,
}

impl CancelHandle {
    /// Cancel streaming response
    pub fn cancel(mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(());
        }
    }
}

/// Controllable streaming response
struct ControlledStream {
    inner: Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
    cancel_rx: Option<oneshot::Receiver<()>>,
    // Hold a backpressure permit for the lifetime of the stream if present
    _bp_permit: Option<BackpressurePermit>,
}

impl ControlledStream {
    fn new_with_bp(
        inner: Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        cancel_rx: Option<oneshot::Receiver<()>>,
        bp_permit: Option<BackpressurePermit>,
    ) -> Self {
        Self {
            inner,
            cancel_rx,
            _bp_permit: bp_permit,
        }
    }
}

impl Stream for ControlledStream {
    type Item = Result<ChatCompletionChunk, AiLibError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;

        // Check if cancelled
        if let Some(ref mut cancel_rx) = self.cancel_rx {
            match Future::poll(Pin::new(cancel_rx), cx) {
                Poll::Ready(_) => {
                    self.cancel_rx = None;
                    return Poll::Ready(Some(Err(AiLibError::ProviderError(
                        "Stream cancelled".to_string(),
                    ))));
                }
                Poll::Pending => {}
            }
        }

        // Poll inner stream
        self.inner.poll_next_unpin(cx)
    }
}

#[allow(unused_mut)]
pub async fn chat_completion_stream(
    client: &AiClient,
    request: ChatCompletionRequest,
) -> Result<
    Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
    AiLibError,
> {
    let provider = client.provider_id();
    let mut processed_request = client.prepare_chat_request(request);
    processed_request.stream = Some(true);
    // Acquire backpressure permit if configured and hold it for the lifetime of the stream
    let bp_permit: Option<BackpressurePermit> = if let Some(ctrl) = &client.backpressure {
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
    loop {
        match client.chat_provider.stream(processed_request.clone()).await {
            Ok(inner) => {
                let cs = ControlledStream::new_with_bp(inner, None, bp_permit);
                return Ok(Box::new(cs));
            }
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
                            "Retrying stream with fallback model"
                        );
                        processed_request.model = resolution.model;
                        continue;
                    }
                    let decorated = client.model_resolver.decorate_invalid_model_error(
                        provider,
                        &processed_request.model,
                        err,
                    );
                    return Err(decorated.with_context("AiClient::chat_completion_stream"));
                }

                return Err(err.with_context("AiClient::chat_completion_stream"));
            }
        }
    }
}

pub async fn chat_completion_stream_with_cancel(
    client: &AiClient,
    request: ChatCompletionRequest,
) -> Result<
    (
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        CancelHandle,
    ),
    AiLibError,
> {
    let provider = client.provider_id();
    let mut processed_request = client.prepare_chat_request(request);
    processed_request.stream = Some(true);
    // Acquire backpressure permit if configured and hold it for the lifetime of the stream
    let bp_permit: Option<BackpressurePermit> = if let Some(ctrl) = &client.backpressure {
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
    let stream = loop {
        match client.chat_provider.stream(processed_request.clone()).await {
            Ok(stream) => break stream,
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
                            "Retrying cancellable stream with fallback model"
                        );
                        processed_request.model = resolution.model;
                        continue;
                    }
                    let decorated = client.model_resolver.decorate_invalid_model_error(
                        provider,
                        &processed_request.model,
                        err,
                    );
                    return Err(
                        decorated.with_context("AiClient::chat_completion_stream_with_cancel")
                    );
                }
                return Err(err.with_context("AiClient::chat_completion_stream_with_cancel"));
            }
        }
    };
    let (cancel_tx, cancel_rx) = oneshot::channel();
    let cancel_handle = CancelHandle {
        sender: Some(cancel_tx),
    };

    let controlled_stream = ControlledStream::new_with_bp(stream, Some(cancel_rx), bp_permit);
    Ok((Box::new(controlled_stream), cancel_handle))
}
