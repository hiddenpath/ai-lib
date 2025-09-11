use crate::api::{ChatApi, ChatCompletionChunk, ModelInfo, ModelPermission};
use crate::metrics::{Metrics, NoopMetrics};
use crate::transport::{DynHttpTransportRef, HttpTransport};
use crate::types::{
    AiLibError, ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Role, Usage,
};
use futures::stream::Stream;
use std::clone::Clone;
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "unified_transport")]
use std::time::Duration;

#[cfg(not(feature = "unified_sse"))]
fn find_event_boundary(buffer: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i < buffer.len().saturating_sub(1) {
        if buffer[i] == b'\n' && buffer[i + 1] == b'\n' {
            return Some(i + 2);
        }
        if i < buffer.len().saturating_sub(3)
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

#[cfg(not(feature = "unified_sse"))]
fn parse_sse_event(event_text: &str) -> Option<Result<Option<ChatCompletionChunk>, AiLibError>> {
    for line in event_text.lines() {
        let line = line.trim();
        if let Some(stripped) = line.strip_prefix("data: ") {
            let data = stripped;
            if data == "[DONE]" {
                return Some(Ok(None));
            }
            return Some(parse_chunk_data(data));
        }
    }
    None
}

#[cfg(not(feature = "unified_sse"))]
fn parse_chunk_data(data: &str) -> Result<Option<ChatCompletionChunk>, AiLibError> {
    match serde_json::from_str::<serde_json::Value>(data) {
        Ok(json) => {
            let choices = json["choices"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .enumerate()
                        .map(|(index, choice)| {
                            let delta = &choice["delta"];
                            crate::api::ChoiceDelta {
                                index: index as u32,
                                delta: crate::api::MessageDelta {
                                    role: delta["role"].as_str().map(|r| match r {
                                        "assistant" => crate::types::Role::Assistant,
                                        "user" => crate::types::Role::User,
                                        "system" => crate::types::Role::System,
                                        _ => crate::types::Role::Assistant,
                                    }),
                                    content: delta["content"].as_str().map(str::to_string),
                                },
                                finish_reason: choice["finish_reason"].as_str().map(str::to_string),
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            Ok(Some(ChatCompletionChunk {
                id: json["id"].as_str().unwrap_or_default().to_string(),
                object: json["object"]
                    .as_str()
                    .unwrap_or("chat.completion.chunk")
                    .to_string(),
                created: json["created"].as_u64().unwrap_or(0),
                model: json["model"].as_str().unwrap_or_default().to_string(),
                choices,
            }))
        }
        Err(e) => Err(AiLibError::ProviderError(format!(
            "JSON parse error: {}",
            e
        ))),
    }
}

fn split_text_into_chunks(text: &str, max_len: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let s = text.as_bytes();
    while start < s.len() {
        let end = std::cmp::min(start + max_len, s.len());
        // try to cut at last whitespace
        let mut cut = end;
        if end < s.len() {
            if let Some(pos) = text[start..end].rfind(' ') {
                cut = start + pos;
            }
        }
        if cut == start {
            cut = end;
        }
        let chunk = String::from_utf8_lossy(&s[start..cut]).to_string();
        chunks.push(chunk);
        start = cut;
        if start < s.len() && s[start] == b' ' {
            start += 1;
        }
    }
    chunks
}
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

pub struct CohereAdapter {
    #[allow(dead_code)] // Kept for backward compatibility, now using direct reqwest
    transport: DynHttpTransportRef,
    api_key: String,
    base_url: String,
    metrics: Arc<dyn Metrics>,
}

impl CohereAdapter {
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
            let client = crate::transport::client_factory::build_shared_client()
                .map_err(|e| AiLibError::NetworkError(format!("Failed to build http client: {}", e)))?;
            let t = HttpTransport::with_reqwest_client(client, timeout);
            return Ok(t.boxed());
        }
        #[cfg(not(feature = "unified_transport"))]
        {
            let t = HttpTransport::new();
            return Ok(t.boxed());
        }
    }

    /// Create Cohere adapter. Requires COHERE_API_KEY env var.
    pub fn new() -> Result<Self, AiLibError> {
        let api_key = std::env::var("COHERE_API_KEY").map_err(|_| {
            AiLibError::AuthenticationError(
                "COHERE_API_KEY environment variable not set".to_string(),
            )
        })?;
        let base_url = std::env::var("COHERE_BASE_URL")
            .unwrap_or_else(|_| "https://api.cohere.ai".to_string());
        Ok(Self {
            transport: Self::build_default_transport()?,
            api_key,
            base_url,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Explicit overrides for api_key and optional base_url (takes precedence over env vars)
    pub fn new_with_overrides(
        api_key: String,
        base_url: Option<String>,
    ) -> Result<Self, AiLibError> {
        let resolved_base = base_url.unwrap_or_else(|| {
            std::env::var("COHERE_BASE_URL").unwrap_or_else(|_| "https://api.cohere.ai".to_string())
        });
        Ok(Self {
            transport: Self::build_default_transport()?,
            api_key,
            base_url: resolved_base,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Create adapter with injectable transport (for testing)
    pub fn with_transport(transport: HttpTransport, api_key: String, base_url: String) -> Self {
        Self {
            transport: transport.boxed(),
            api_key,
            base_url,
            metrics: Arc::new(NoopMetrics::new()),
        }
    }

    /// Construct using object-safe transport reference
    pub fn with_transport_ref(
        transport: DynHttpTransportRef,
        api_key: String,
        base_url: String,
    ) -> Self {
        Self {
            transport,
            api_key,
            base_url,
            metrics: Arc::new(NoopMetrics::new()),
        }
    }

    /// Construct with an injected transport reference and metrics implementation
    pub fn with_transport_ref_and_metrics(
        transport: DynHttpTransportRef,
        api_key: String,
        base_url: String,
        metrics: Arc<dyn Metrics>,
    ) -> Self {
        Self {
            transport,
            api_key,
            base_url,
            metrics,
        }
    }

    fn convert_request(&self, request: &ChatCompletionRequest) -> serde_json::Value {
        // Build v2/chat format according to Cohere documentation
        let msgs: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": match msg.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                    },
                    "content": msg.content.as_text()
                })
            })
            .collect();

        let mut chat_body = serde_json::json!({
            "model": request.model,
            "messages": msgs,
        });

        // Add optional parameters
        if let Some(temp) = request.temperature {
            chat_body["temperature"] =
                serde_json::Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
        }
        if let Some(max_tokens) = request.max_tokens {
            chat_body["max_tokens"] =
                serde_json::Value::Number(serde_json::Number::from(max_tokens));
        }

        chat_body
    }

    fn parse_response(
        &self,
        response: serde_json::Value,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        // Try different response formats: OpenAI-like choices, Cohere v2/chat message, or Cohere v1 generations
        let content = if let Some(c) = response.get("choices") {
            // OpenAI format
            c[0]["message"]["content"]
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| c[0]["text"].as_str().map(|s| s.to_string()))
        } else if let Some(msg) = response.get("message") {
            // Cohere v2/chat format
            msg.get("content").and_then(|content_array| {
                content_array
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|content_obj| {
                        content_obj
                            .get("text")
                            .and_then(|t| t.as_str())
                            .map(|text| text.to_string())
                    })
            })
        } else if let Some(gens) = response.get("generations") {
            // Cohere v1 format
            gens[0]["text"].as_str().map(|s| s.to_string())
        } else {
            None
        };

        let content = content.unwrap_or_default();

        let mut function_call: Option<crate::types::function_call::FunctionCall> = None;
        if let Some(fc_val) = response.get("function_call") {
            if let Ok(fc) =
                serde_json::from_value::<crate::types::function_call::FunctionCall>(fc_val.clone())
            {
                function_call = Some(fc);
            } else if let Some(name) = fc_val
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
            {
                let args = fc_val.get("arguments").and_then(|a| {
                    if a.is_string() {
                        serde_json::from_str::<serde_json::Value>(a.as_str().unwrap()).ok()
                    } else {
                        Some(a.clone())
                    }
                });
                function_call = Some(crate::types::function_call::FunctionCall {
                    name,
                    arguments: args,
                });
            }
        } else if let Some(tool_calls) = response.get("tool_calls").and_then(|v| v.as_array()) {
            if let Some(first) = tool_calls.first() {
                if let Some(func) = first.get("function").or_else(|| first.get("function_call")) {
                    if let Some(name) = func.get("name").and_then(|v| v.as_str()) {
                        let mut args_opt = func.get("arguments").cloned();
                        if let Some(args_val) = &args_opt {
                            if args_val.is_string() {
                                if let Some(s) = args_val.as_str() {
                                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(s) {
                                        args_opt = Some(parsed);
                                    }
                                }
                            }
                        }
                        function_call = Some(crate::types::function_call::FunctionCall {
                            name: name.to_string(),
                            arguments: args_opt,
                        });
                    }
                }
            }
        }

        let choice = Choice {
            index: 0,
            message: Message {
                role: Role::Assistant,
                content: crate::types::common::Content::Text(content.clone()),
                function_call,
            },
            finish_reason: None,
        };

        let usage = Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        };

        Ok(ChatCompletionResponse {
            id: response["id"].as_str().unwrap_or_default().to_string(),
            object: response["object"].as_str().unwrap_or_default().to_string(),
            created: response["created"].as_u64().unwrap_or(0),
            model: response["model"].as_str().unwrap_or_default().to_string(),
            choices: vec![choice],
            usage,
        })
    }
}

#[async_trait::async_trait]
impl ChatApi for CohereAdapter {
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        self.metrics.incr_counter("cohere.requests", 1).await;
        let timer = self.metrics.start_timer("cohere.request_duration_ms").await;

        let _body = self.convert_request(&request);

        // Use v1/generate endpoint (fallback for older API keys)
        let url_generate = format!("{}/v1/generate", self.base_url);

        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Accept".to_string(), "application/json".to_string());

        // Convert messages to prompt string for v1/generate endpoint
        let prompt = request
            .messages
            .iter()
            .map(|msg| match msg.role {
                Role::System => format!("System: {}", msg.content.as_text()),
                Role::User => format!("Human: {}", msg.content.as_text()),
                Role::Assistant => format!("Assistant: {}", msg.content.as_text()),
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut generate_body = serde_json::json!({
            "model": request.model,
            "prompt": prompt,
        });

        if let Some(temp) = request.temperature {
            generate_body["temperature"] =
                serde_json::Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
        }
        if let Some(max_tokens) = request.max_tokens {
            generate_body["max_tokens"] =
                serde_json::Value::Number(serde_json::Number::from(max_tokens));
        }

        // Use unified transport
        let response_json = self
            .transport
            .post_json(&url_generate, Some(headers), generate_body)
            .await?;

        if let Some(t) = timer {
            t.stop();
        }

        self.parse_response(response_json)
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        // Build stream request similar to chat_completion but with stream=true
        let mut stream_request = self.convert_request(&_request);
        stream_request["stream"] = serde_json::Value::Bool(true);

        let url = format!("{}/v1/chat", self.base_url);

        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );
        // Try unified transport streaming first
        if let Ok(byte_stream) = self
            .transport
            .post_stream(&url, Some(headers.clone()), stream_request.clone())
            .await
        {
            let (tx, rx) = mpsc::unbounded_channel();
            tokio::spawn(async move {
                let mut buffer = Vec::new();
                futures::pin_mut!(byte_stream);
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
                                        if let Some(parsed) =
                                            crate::sse::parser::parse_sse_event(event_text)
                                        {
                                            match parsed {
                                                Ok(Some(chunk)) => {
                                                    if tx.send(Ok(chunk)).is_err() {
                                                        return;
                                                    }
                                                }
                                                Ok(None) => return,
                                                Err(e) => {
                                                    let _ = tx.send(Err(e));
                                                    return;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            #[cfg(not(feature = "unified_sse"))]
                            {
                                while let Some(boundary) = find_event_boundary(&buffer) {
                                    let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
                                    if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
                                        if let Some(parsed) = parse_sse_event(event_text) {
                                            match parsed {
                                                Ok(Some(chunk)) => {
                                                    if tx.send(Ok(chunk)).is_err() {
                                                        return;
                                                    }
                                                }
                                                Ok(None) => return,
                                                Err(e) => {
                                                    let _ = tx.send(Err(e));
                                                    return;
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
            let stream = UnboundedReceiverStream::new(rx);
            return Ok(Box::new(Box::pin(stream)));
        }

        // Fallback: call non-streaming chat_completion and stream the result in chunks
        let finished = self.chat_completion(_request.clone()).await?;
        let text = finished
            .choices
            .first()
            .map(|c| c.message.content.as_text())
            .unwrap_or_default();

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let chunks = split_text_into_chunks(&text, 80);
            for chunk in chunks {
                // construct ChatCompletionChunk with single delta
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

        let stream = UnboundedReceiverStream::new(rx);
        Ok(Box::new(Box::pin(stream)))
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        // Use v1/models endpoint for listing models via unified transport
        let url = format!("{}/v1/models", self.base_url);
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );

        let response = self.transport.get_json(&url, Some(headers)).await?;

        Ok(response["models"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| {
                m["id"]
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| m["name"].as_str().map(|s| s.to_string()))
            })
            .collect())
    }

    async fn get_model_info(&self, model_id: &str) -> Result<crate::api::ModelInfo, AiLibError> {
        Ok(ModelInfo {
            id: model_id.to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "cohere".to_string(),
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
