use crate::api::{
    ChatApi, ChatCompletionChunk, ChoiceDelta, MessageDelta, ModelInfo, ModelPermission,
};
use crate::metrics::{Metrics, NoopMetrics};
use crate::transport::{DynHttpTransportRef, HttpTransport};
use crate::types::{
    AiLibError, ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Role, Usage,
};
use futures::stream::Stream;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Mistral adapter (conservative HTTP implementation).
///
/// Note: Mistral provides an official Rust SDK (https://github.com/ivangabriele/mistralai-client-rs).
/// We keep this implementation HTTP-based for now and can swap to the SDK later.
pub struct MistralAdapter {
    #[allow(dead_code)] // Kept for backward compatibility, now using direct reqwest
    transport: DynHttpTransportRef,
    api_key: Option<String>,
    base_url: String,
    metrics: Arc<dyn Metrics>,
}

impl MistralAdapter {
    pub fn new() -> Result<Self, AiLibError> {
        let api_key = std::env::var("MISTRAL_API_KEY").ok();
        let base_url = std::env::var("MISTRAL_BASE_URL")
            .unwrap_or_else(|_| "https://api.mistral.ai".to_string());
        let boxed = HttpTransport::new().boxed();
        Ok(Self {
            transport: boxed,
            api_key,
            base_url,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Explicit API key / base_url overrides.
    pub fn new_with_overrides(
        api_key: Option<String>,
        base_url: Option<String>,
    ) -> Result<Self, AiLibError> {
        let boxed = HttpTransport::new().boxed();
        Ok(Self {
            transport: boxed,
            api_key,
            base_url: base_url.unwrap_or_else(|| {
                std::env::var("MISTRAL_BASE_URL")
                    .unwrap_or_else(|_| "https://api.mistral.ai".to_string())
            }),
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Construct using an injected object-safe transport reference (for testing/SDKs)
    pub fn with_transport(
        transport: DynHttpTransportRef,
        api_key: Option<String>,
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
    pub fn with_transport_and_metrics(
        transport: DynHttpTransportRef,
        api_key: Option<String>,
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

    fn convert_request(&self, request: &ChatCompletionRequest) -> serde_json::Value {
        let msgs: Vec<serde_json::Value> = request.messages.iter().map(|msg| {
            serde_json::json!({
                "role": match msg.role { Role::System => "system", Role::User => "user", Role::Assistant => "assistant" },
                "content": msg.content.as_text()
            })
        }).collect();

        let mut body = serde_json::json!({ "model": request.model, "messages": msgs });
        if let Some(temp) = request.temperature {
            body["temperature"] =
                serde_json::Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::Value::Number(serde_json::Number::from(max_tokens));
        }

        // Function calling (OpenAI-compatible schema supported by Mistral chat/completions)
        if let Some(funcs) = &request.functions {
            let mapped: Vec<serde_json::Value> = funcs
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters.clone().unwrap_or(serde_json::json!({}))
                    })
                })
                .collect();
            body["functions"] = serde_json::Value::Array(mapped);
        }
        if let Some(policy) = &request.function_call {
            match policy {
                crate::types::FunctionCallPolicy::Auto(name) => {
                    if name == "auto" {
                        body["function_call"] = serde_json::Value::String("auto".to_string());
                    } else {
                        body["function_call"] = serde_json::json!({"name": name});
                    }
                }
                crate::types::FunctionCallPolicy::None => {
                    body["function_call"] = serde_json::Value::String("none".to_string());
                }
            }
        }

        body
    }

    fn parse_response(
        &self,
        response: serde_json::Value,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        let choices = response["choices"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .enumerate()
            .map(|(index, choice)| {
                let message = choice["message"].as_object().ok_or_else(|| {
                    AiLibError::ProviderError("Invalid choice format".to_string())
                })?;
                let role = match message["role"].as_str().unwrap_or("user") {
                    "system" => Role::System,
                    "assistant" => Role::Assistant,
                    _ => Role::User,
                };
                let content = message["content"].as_str().unwrap_or("").to_string();

                // try to parse function_call if present
                let mut function_call: Option<crate::types::function_call::FunctionCall> = None;
                if let Some(fc_val) = message.get("function_call") {
                    if let Ok(fc) = serde_json::from_value::<
                        crate::types::function_call::FunctionCall,
                    >(fc_val.clone())
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
                }

                Ok(Choice {
                    index: index as u32,
                    message: Message {
                        role,
                        content: crate::types::common::Content::Text(content),
                        function_call,
                    },
                    finish_reason: choice["finish_reason"].as_str().map(|s| s.to_string()),
                })
            })
            .collect::<Result<Vec<_>, AiLibError>>()?;

        let usage = response["usage"].as_object().ok_or_else(|| {
            AiLibError::ProviderError("Invalid response format: usage not found".to_string())
        })?;
        let usage = Usage {
            prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
        };

        Ok(ChatCompletionResponse {
            id: response["id"].as_str().unwrap_or_default().to_string(),
            object: response["object"].as_str().unwrap_or_default().to_string(),
            created: response["created"].as_u64().unwrap_or(0),
            model: response["model"].as_str().unwrap_or_default().to_string(),
            choices,
            usage,
        })
    }
}

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

fn parse_chunk_data(data: &str) -> Result<Option<ChatCompletionChunk>, AiLibError> {
    let json: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| AiLibError::ProviderError(format!("JSON parse error: {}", e)))?;
    let mut choices_vec: Vec<ChoiceDelta> = Vec::new();
    if let Some(arr) = json["choices"].as_array() {
        for (index, choice) in arr.iter().enumerate() {
            let delta = &choice["delta"];
            let role = delta.get("role").and_then(|v| v.as_str()).map(|r| match r {
                "assistant" => Role::Assistant,
                "user" => Role::User,
                "system" => Role::System,
                _ => Role::Assistant,
            });
            let content = delta
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let md = MessageDelta { role, content };
            let cd = ChoiceDelta {
                index: index as u32,
                delta: md,
                finish_reason: choice
                    .get("finish_reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            };
            choices_vec.push(cd);
        }
    }

    Ok(Some(ChatCompletionChunk {
        id: json["id"].as_str().unwrap_or_default().to_string(),
        object: json["object"]
            .as_str()
            .unwrap_or("chat.completion.chunk")
            .to_string(),
        created: json["created"].as_u64().unwrap_or(0),
        model: json["model"].as_str().unwrap_or_default().to_string(),
        choices: choices_vec,
    }))
}

fn split_text_into_chunks(text: &str, max_len: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let s = text.as_bytes();
    while start < s.len() {
        let end = std::cmp::min(start + max_len, s.len());
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

#[async_trait::async_trait]
impl ChatApi for MistralAdapter {
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        self.metrics.incr_counter("mistral.requests", 1).await;
        let timer = self
            .metrics
            .start_timer("mistral.request_duration_ms")
            .await;

        // Use direct reqwest approach like groqai for better reliability
        let url = format!("{}{}", self.base_url, "/v1/chat/completions");
        
        // Create reqwest client with proxy support (like groqai does)
        let mut client_builder = reqwest::Client::builder();
        if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }
        let client = client_builder.build()
            .map_err(|e| AiLibError::ProviderError(format!("Failed to create HTTP client: {}", e)))?;

        // Build request directly with proper serialization
        let provider_request = self.convert_request(&request);
        
        let mut request_builder = client
            .post(&url)
            .header("Content-Type", "application/json");
            
        if let Some(key) = &self.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", key));
        }
        
        let response = request_builder
            .json(&provider_request)  // Use reqwest's json() method like groqai
            .send()
            .await
            .map_err(|e| AiLibError::ProviderError(format!("HTTP request failed: {}", e)))?;

        // Stop timer
        if let Some(t) = timer {
            t.stop();
        }

        // Handle response
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiLibError::ProviderError(format!(
                "HTTP request failed: {} - {}",
                status, error_text
            )));
        }

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| AiLibError::ProviderError(format!("Failed to parse response: {}", e)))?;

        self.parse_response(response_json)
    }

    async fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        let mut stream_request = self.convert_request(&request);
        stream_request["stream"] = serde_json::Value::Bool(true);

        let url = format!("{}{}", self.base_url, "/v1/chat/completions");

        // build client honoring proxy
        let mut client_builder = reqwest::Client::builder();
        if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }
        let client = client_builder
            .build()
            .map_err(|e| AiLibError::ProviderError(format!("Client error: {}", e)))?;

        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "text/event-stream".to_string());
        if let Some(key) = &self.api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", key));
        }

        let response = client.post(&url).json(&stream_request);
        let mut req = response;
        for (k, v) in headers.clone() {
            req = req.header(k, v);
        }

        let send_result = req.send().await;
        if let Ok(response) = send_result {
            if response.status().is_success() {
                let (tx, rx) = mpsc::unbounded_channel();

                tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    let mut stream = response.bytes_stream();

                    while let Some(item) = stream.next().await {
                        match item {
                            Ok(bytes) => {
                                buffer.extend_from_slice(&bytes);

                                // process complete events separated by boundaries
                                #[cfg(feature = "unified_sse")]
                                {
                                    while let Some(boundary) = crate::sse::parser::find_event_boundary(&buffer) {
                                        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
                                        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
                                            if let Some(parsed) = crate::sse::parser::parse_sse_event(event_text) {
                                                match parsed {
                                                    Ok(Some(chunk)) => { if tx.send(Ok(chunk)).is_err() { return; } }
                                                    Ok(None) => return,
                                                    Err(e) => { let _ = tx.send(Err(e)); return; }
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
                                                    Ok(Some(chunk)) => { if tx.send(Ok(chunk)).is_err() { return; } }
                                                    Ok(None) => return,
                                                    Err(e) => { let _ = tx.send(Err(e)); return; }
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
        }

        // fallback: call chat_completion and stream chunks
        let finished = self.chat_completion(request).await?;
        let text = finished
            .choices
            .first()
            .map(|c| c.message.content.as_text())
            .unwrap_or_default();
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let chunks = split_text_into_chunks(&text, 80);
            for chunk in chunks {
                let delta = ChoiceDelta {
                    index: 0,
                    delta: MessageDelta {
                        role: Some(Role::Assistant),
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
        // Mistral models endpoint
        let url = format!("{}/v1/models", self.base_url);
        let mut headers = HashMap::new();
        if let Some(key) = &self.api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", key));
        }
        // Use direct reqwest approach for better reliability
        let mut client_builder = reqwest::Client::builder();
        if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }
        let client = client_builder.build()
            .map_err(|e| AiLibError::ProviderError(format!("Failed to create HTTP client: {}", e)))?;

        let mut request_builder = client.get(&url);
        
        // Add headers
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }
        
        let response = request_builder
            .send()
            .await
            .map_err(|e| AiLibError::ProviderError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiLibError::ProviderError(format!(
                "HTTP request failed: {} - {}",
                status, error_text
            )));
        }

        let response: serde_json::Value = response.json().await
            .map_err(|e| AiLibError::ProviderError(format!("Failed to parse response: {}", e)))?;
        Ok(response["data"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
            .collect())
    }

    async fn get_model_info(&self, model_id: &str) -> Result<crate::api::ModelInfo, AiLibError> {
        Ok(ModelInfo {
            id: model_id.to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "mistral".to_string(),
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
