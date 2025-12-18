use crate::api::{ChatCompletionChunk, ChatProvider, ModelInfo, ModelPermission};
use crate::metrics::{Metrics, NoopMetrics};
use crate::transport::{DynHttpTransportRef, HttpTransport};
use crate::types::{
    AiLibError, ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Role, Usage,
    UsageStatus,
};
use futures::stream::Stream;
use futures::StreamExt;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
#[cfg(feature = "unified_transport")]
use std::time::Duration;

/// Google Gemini independent adapter, supporting multimodal AI services
///
/// Google Gemini independent adapter for multimodal AI service
///
/// Gemini API is completely different from OpenAI format, requires independent adapter:
/// - Endpoint: /v1beta/models/{model}:generateContent
/// - Request body: contents array instead of messages
/// - Response: candidates\[0\].content.parts\[0\].text
/// - Authentication: URL parameter ?key=<API_KEY>
pub struct GeminiAdapter {
    #[allow(dead_code)] // Kept for backward compatibility, now using direct reqwest
    transport: DynHttpTransportRef,
    api_key: String,
    base_url: String,
    metrics: Arc<dyn Metrics>,
}

impl GeminiAdapter {
    #[allow(dead_code)]
    fn build_default_timeout_secs() -> u64 {
        std::env::var("AI_HTTP_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30)
    }

    fn build_default_transport() -> Result<DynHttpTransportRef, AiLibError> {
        #[cfg(feature = "unified_transport")]
        {
            let timeout = Duration::from_secs(Self::build_default_timeout_secs());
            let client = crate::transport::client_factory::build_shared_client().map_err(|e| {
                AiLibError::NetworkError(format!("Failed to build http client: {}", e))
            })?;
            let t = HttpTransport::with_reqwest_client(client, timeout);
            Ok(t.boxed())
        }
        #[cfg(not(feature = "unified_transport"))]
        {
            let t = HttpTransport::new();
            return Ok(t.boxed());
        }
    }

    pub fn new() -> Result<Self, AiLibError> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
            AiLibError::AuthenticationError(
                "GEMINI_API_KEY environment variable not set".to_string(),
            )
        })?;

        Ok(Self {
            transport: Self::build_default_transport()?,
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Explicit overrides for api_key and optional base_url (takes precedence over env vars)
    pub fn new_with_overrides(
        api_key: String,
        base_url: Option<String>,
    ) -> Result<Self, AiLibError> {
        Ok(Self {
            transport: Self::build_default_transport()?,
            api_key,
            base_url: base_url
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string()),
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Construct using object-safe transport reference
    pub fn with_transport_ref(
        transport: DynHttpTransportRef,
        api_key: String,
        base_url: String,
    ) -> Result<Self, AiLibError> {
        Ok(Self {
            transport,
            api_key,
            base_url,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Construct with an injected transport and metrics implementation
    pub fn with_transport_ref_and_metrics(
        transport: DynHttpTransportRef,
        api_key: String,
        base_url: String,
        metrics: Arc<dyn Metrics>,
    ) -> Result<Self, AiLibError> {
        Ok(Self {
            transport,
            api_key,
            base_url,
            metrics,
        })
    }

    /// Convert generic request to Gemini format
    fn convert_to_gemini_request(&self, request: &ChatCompletionRequest) -> serde_json::Value {
        let contents: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    Role::User => "user",
                    Role::Assistant => "model", // Gemini uses "model" instead of "assistant"
                    Role::System => "user",     // Gemini has no system role, convert to user
                };

                serde_json::json!({
                    "role": role,
                    "parts": [{"text": msg.content.as_text()}]
                })
            })
            .collect();

        let mut gemini_request = serde_json::json!({
            "contents": contents
        });

        // Gemini generation configuration
        let mut generation_config = serde_json::json!({});

        if let Some(temp) = request.temperature {
            generation_config["temperature"] =
                serde_json::Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
        }
        if let Some(max_tokens) = request.max_tokens {
            generation_config["maxOutputTokens"] =
                serde_json::Value::Number(serde_json::Number::from(max_tokens));
        }
        if let Some(top_p) = request.top_p {
            generation_config["topP"] =
                serde_json::Value::Number(serde_json::Number::from_f64(top_p.into()).unwrap());
        }

        if !generation_config.as_object().unwrap().is_empty() {
            gemini_request["generationConfig"] = generation_config;
        }

        request.apply_extensions(&mut gemini_request);

        gemini_request
    }

    /// Parse Gemini response to generic format
    fn parse_gemini_response(
        &self,
        response: serde_json::Value,
        model: &str,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        let candidates = response["candidates"].as_array().ok_or_else(|| {
            AiLibError::ProviderError("No candidates in Gemini response".to_string())
        })?;

        let choices: Result<Vec<Choice>, AiLibError> = candidates
            .iter()
            .enumerate()
            .map(|(index, candidate)| {
                let content = candidate["content"]["parts"][0]["text"]
                    .as_str()
                    .ok_or_else(|| {
                        AiLibError::ProviderError("No text in Gemini candidate".to_string())
                    })?;

                // Try to parse a function_call if the provider returned one. Gemini's
                // response shape may place structured data under candidate["function_call"]
                // or nested inside candidate["content"]["function_call"]. We try both.
                let mut function_call: Option<crate::types::function_call::FunctionCall> = None;
                if let Some(fc_val) = candidate.get("function_call").cloned().or_else(|| {
                    candidate
                        .get("content")
                        .and_then(|c| c.get("function_call"))
                        .cloned()
                }) {
                    if let Ok(fc) = serde_json::from_value::<
                        crate::types::function_call::FunctionCall,
                    >(fc_val.clone())
                    {
                        function_call = Some(fc);
                    } else {
                        // Fallback: extract name + arguments (arguments may be a JSON string)
                        if let Some(name) = fc_val
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                        {
                            let args = fc_val.get("arguments").and_then(|a| {
                                if a.is_string() {
                                    serde_json::from_str::<serde_json::Value>(a.as_str().unwrap())
                                        .ok()
                                } else {
                                    Some(a.clone())
                                }
                            });
                            function_call = Some(crate::types::function_call::FunctionCall {
                                name,
                                arguments: args,
                            });
                        }
                    }
                }

                let finish_reason = candidate["finishReason"].as_str().map(|r| match r {
                    "STOP" => "stop".to_string(),
                    "MAX_TOKENS" => "length".to_string(),
                    _ => r.to_string(),
                });

                Ok(Choice {
                    index: index as u32,
                    message: Message {
                        role: Role::Assistant,
                        content: crate::types::common::Content::Text(content.to_string()),
                        function_call,
                    },
                    finish_reason,
                })
            })
            .collect();

        let usage = Usage {
            prompt_tokens: response["usageMetadata"]["promptTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
            completion_tokens: response["usageMetadata"]["candidatesTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
            total_tokens: response["usageMetadata"]["totalTokenCount"]
                .as_u64()
                .unwrap_or(0) as u32,
        };

        Ok(ChatCompletionResponse {
            id: format!("gemini-{}", chrono::Utc::now().timestamp()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: model.to_string(),
            choices: choices?,
            usage,
            usage_status: UsageStatus::Finalized, // Gemini provides accurate usage data
        })
    }
}

#[async_trait::async_trait]
impl ChatProvider for GeminiAdapter {
    fn name(&self) -> &str {
        "Gemini"
    }

    async fn chat(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        self.metrics.incr_counter("gemini.requests", 1).await;
        let timer = self.metrics.start_timer("gemini.request_duration_ms").await;

        let gemini_request = self.convert_to_gemini_request(&request);

        // Gemini uses URL parameter authentication, not headers
        let url = format!("{}/models/{}:generateContent", self.base_url, request.model);

        let headers = HashMap::from([
            ("Content-Type".to_string(), "application/json".to_string()),
            ("x-goog-api-key".to_string(), self.api_key.clone()),
        ]);

        // Use unified transport
        let response_json = self
            .transport
            .post_json(&url, Some(headers), gemini_request)
            .await
            .map_err(|e| e.with_context(&format!("Gemini chat request to {}", url)))?;
        if let Some(t) = timer {
            t.stop();
        }
        self.parse_gemini_response(response_json, &request.model)
    }

    async fn stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        // Try native SSE first per Gemini API streamGenerateContent
        let url = format!(
            "{}/models/{}:streamGenerateContent",
            self.base_url, request.model
        );
        let gemini_request = self.convert_to_gemini_request(&request);
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Accept".to_string(), "text/event-stream".to_string());
        headers.insert("x-goog-api-key".to_string(), self.api_key.clone());

        if let Ok(mut byte_stream) = self
            .transport
            .post_stream(&url, Some(headers), gemini_request)
            .await
        {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            tokio::spawn(async move {
                let mut buffer = Vec::new();
                while let Some(item) = byte_stream.next().await {
                    match item {
                        Ok(bytes) => {
                            buffer.extend_from_slice(&bytes);
                            #[cfg(feature = "unified_sse")]
                            {
                                while let Some(boundary) =
                                    crate::sse::parser::find_event_boundary(&buffer)
                                {
                                    let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
                                    if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
                                        for line in event_text.lines() {
                                            let line = line.trim();
                                            if let Some(data) = line.strip_prefix("data: ") {
                                                if data.is_empty() {
                                                    continue;
                                                }
                                                if data == "[DONE]" {
                                                    return;
                                                }
                                                match serde_json::from_str::<serde_json::Value>(
                                                    data,
                                                ) {
                                                    Ok(json) => {
                                                        let text = json
                                                            .get("candidates")
                                                            .and_then(|c| c.as_array())
                                                            .and_then(|arr| arr.first())
                                                            .and_then(|cand| {
                                                                cand.get("content")
                                                                    .and_then(|c| c.get("parts"))
                                                                    .and_then(|p| p.as_array())
                                                                    .and_then(|parts| parts.first())
                                                                    .and_then(|part| {
                                                                        part.get("text")
                                                                    })
                                                                    .and_then(|t| t.as_str())
                                                            })
                                                            .map(|s| s.to_string());
                                                        if let Some(tdelta) = text {
                                                            let delta = crate::api::ChoiceDelta { index: 0, delta: crate::api::MessageDelta { role: Some(crate::types::Role::Assistant), content: Some(tdelta) }, finish_reason: json.get("candidates").and_then(|c| c.as_array()).and_then(|arr| arr.first()).and_then(|cand| cand.get("finishReason").or_else(|| json.get("finishReason"))).and_then(|v| v.as_str()).map(|r| match r { "STOP" => "stop".to_string(), "MAX_TOKENS" => "length".to_string(), other => other.to_string() }) };
                                                            let chunk_obj = ChatCompletionChunk {
                                                                id: json
                                                                    .get("responseId")
                                                                    .and_then(|v| v.as_str())
                                                                    .unwrap_or("")
                                                                    .to_string(),
                                                                object: "chat.completion.chunk"
                                                                    .to_string(),
                                                                created: 0,
                                                                model: request.model.clone(),
                                                                choices: vec![delta],
                                                            };
                                                            if tx.send(Ok(chunk_obj)).is_err() {
                                                                return;
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        let _ = tx.send(Err(
                                                            AiLibError::ProviderError(format!(
                                                                "Gemini SSE JSON parse error: {}",
                                                                e
                                                            )),
                                                        ));
                                                        return;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            #[cfg(not(feature = "unified_sse"))]
                            {
                                fn find_event_boundary(buffer: &[u8]) -> Option<usize> {
                                    let mut i = 0;
                                    while i + 1 < buffer.len() {
                                        if buffer[i] == b'\n' && buffer[i + 1] == b'\n' {
                                            return Some(i + 2);
                                        }
                                        if i + 3 < buffer.len()
                                            && buffer[i] == b'\r'
                                            && buffer[i + 1] == b'\n'
                                            && buffer[i + 2] == b'\r'
                                            && buffer[i + 3] == b'\n'
                                        {
                                            return Some(i + 4);
                                        }
                                        i += 1;
                                    }
                                    None
                                }
                                while let Some(boundary) = find_event_boundary(&buffer) {
                                    let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
                                    if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
                                        for line in event_text.lines() {
                                            let line = line.trim();
                                            if let Some(data) = line.strip_prefix("data: ") {
                                                if data.is_empty() {
                                                    continue;
                                                }
                                                if data == "[DONE]" {
                                                    return;
                                                }
                                                match serde_json::from_str::<serde_json::Value>(
                                                    data,
                                                ) {
                                                    Ok(json) => {
                                                        let text = json
                                                            .get("candidates")
                                                            .and_then(|c| c.as_array())
                                                            .and_then(|arr| arr.first())
                                                            .and_then(|cand| {
                                                                cand.get("content")
                                                                    .and_then(|c| c.get("parts"))
                                                                    .and_then(|p| p.as_array())
                                                                    .and_then(|parts| parts.first())
                                                                    .and_then(|part| {
                                                                        part.get("text")
                                                                    })
                                                                    .and_then(|t| t.as_str())
                                                            })
                                                            .map(|s| s.to_string());
                                                        if let Some(tdelta) = text {
                                                            let delta = crate::api::ChoiceDelta { index: 0, delta: crate::api::MessageDelta { role: Some(crate::types::Role::Assistant), content: Some(tdelta) }, finish_reason: None };
                                                            let chunk_obj = ChatCompletionChunk {
                                                                id: json
                                                                    .get("responseId")
                                                                    .and_then(|v| v.as_str())
                                                                    .unwrap_or("")
                                                                    .to_string(),
                                                                object: "chat.completion.chunk"
                                                                    .to_string(),
                                                                created: 0,
                                                                model: request.model.clone(),
                                                                choices: vec![delta],
                                                            };
                                                            if tx.send(Ok(chunk_obj)).is_err() {
                                                                return;
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        let _ = tx.send(Err(
                                                            AiLibError::ProviderError(format!(
                                                                "Gemini SSE JSON parse error: {}",
                                                                e
                                                            )),
                                                        ));
                                                        return;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(Err(AiLibError::ProviderError(format!(
                                "Stream error: {}",
                                e
                            ))));
                            break;
                        }
                    }
                }
            });
            let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
            return Ok(Box::new(Box::pin(stream)));
        }

        // Fallback to non-streaming + simulated chunks
        fn split_text_into_chunks(text: &str, max_len: usize) -> Vec<String> {
            let mut chunks = Vec::new();
            let mut start = 0;
            let bytes = text.as_bytes();
            while start < bytes.len() {
                let end = std::cmp::min(start + max_len, bytes.len());
                let mut cut = end;
                if end < bytes.len() {
                    if let Some(pos) = text[start..end].rfind(' ') {
                        cut = start + pos;
                    }
                }
                if cut == start {
                    cut = end;
                }
                chunks.push(String::from_utf8_lossy(&bytes[start..cut]).to_string());
                start = cut;
                if start < bytes.len() && bytes[start] == b' ' {
                    start += 1;
                }
            }
            chunks
        }

        let finished = self.chat(request).await?;
        let text = finished
            .choices
            .first()
            .map(|c| c.message.content.as_text())
            .unwrap_or_default();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            let chunks = split_text_into_chunks(&text, 80);
            for chunk in chunks {
                let delta = crate::api::ChoiceDelta {
                    index: 0,
                    delta: crate::api::MessageDelta {
                        role: Some(crate::types::Role::Assistant),
                        content: Some(chunk.clone()),
                    },
                    finish_reason: None,
                };
                let chunk_obj = ChatCompletionChunk {
                    id: "simulated".to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: 0,
                    model: finished.model.clone(),
                    choices: vec![delta],
                };
                if tx.send(Ok(chunk_obj)).is_err() {
                    return;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        });
        let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
        Ok(Box::new(Box::pin(stream)))
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        // Common Gemini models
        Ok(vec![
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
            "gemini-1.0-pro".to_string(),
        ])
    }

    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo, AiLibError> {
        Ok(ModelInfo {
            id: model_id.to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "google".to_string(),
            permission: vec![ModelPermission {
                id: "default".to_string(),
                object: "model_permission".to_string(),
                created: 0,
                allow_create_engine: false,
                allow_sampling: true,
                allow_logprobs: false,
                allow_search_indices: false,
                allow_view: true,
                allow_fine_tuning: false,
                organization: "*".to_string(),
                group: None,
                is_blocking: false,
            }],
        })
    }
}
