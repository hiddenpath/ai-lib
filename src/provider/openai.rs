use crate::api::{ChatCompletionChunk, ChatProvider, ModelInfo, ModelPermission};
#[cfg(not(feature = "unified_sse"))]
use crate::api::{ChoiceDelta, MessageDelta};
use crate::metrics::{Metrics, NoopMetrics};
use crate::transport::{DynHttpTransportRef, HttpTransport};
use crate::types::{
    AiLibError, ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Role, Usage,
    UsageStatus,
};
use futures::stream::{Stream, StreamExt};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
#[cfg(feature = "unified_transport")]
use std::time::Duration;

/// OpenAI adapter, supporting GPT series models
///
/// OpenAI adapter supporting GPT series models
pub struct OpenAiAdapter {
    transport: DynHttpTransportRef,
    api_key: String,
    base_url: String,
    metrics: Arc<dyn Metrics>,
}

impl OpenAiAdapter {
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
        let api_key = env::var("OPENAI_API_KEY").map_err(|_| {
            AiLibError::AuthenticationError(
                "OPENAI_API_KEY environment variable not set".to_string(),
            )
        })?;

        Ok(Self {
            transport: Self::build_default_transport()?,
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Explicit API key override (takes precedence over env var) + optional base_url override.
    pub fn new_with_overrides(
        api_key: String,
        base_url: Option<String>,
    ) -> Result<Self, AiLibError> {
        Ok(Self {
            transport: Self::build_default_transport()?,
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Construct with an injected object-safe transport reference
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

    pub fn with_metrics(
        api_key: String,
        base_url: String,
        metrics: Arc<dyn Metrics>,
    ) -> Result<Self, AiLibError> {
        Ok(Self {
            transport: Self::build_default_transport()?,
            api_key,
            base_url,
            metrics,
        })
    }

    #[allow(dead_code)]
    fn convert_request(&self, request: &ChatCompletionRequest) -> serde_json::Value {
        // Synchronous converter: do not perform provider uploads, just inline content
        let mut openai_request = serde_json::json!({
            "model": request.model,
            "messages": serde_json::Value::Array(vec![])
        });

        let mut msgs: Vec<serde_json::Value> = Vec::new();
        for msg in request.messages.iter() {
            let role = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            let content_val = crate::provider::utils::content_to_provider_value(&msg.content);
            msgs.push(serde_json::json!({"role": role, "content": content_val}));
        }
        openai_request["messages"] = serde_json::Value::Array(msgs);
        request.apply_extensions(&mut openai_request);
        openai_request
    }

    /// Async version that can upload local files to OpenAI before constructing the request
    async fn convert_request_async(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<serde_json::Value, AiLibError> {
        // Build the OpenAI-compatible request JSON. For now we avoid provider-specific
        // upload flows here and rely on the generic provider utils (which may inline files)
        // to produce content JSON values.
        let mut openai_request = serde_json::json!({
            "model": request.model,
            "messages": serde_json::Value::Array(vec![])
        });

        let mut msgs: Vec<serde_json::Value> = Vec::new();
        for msg in request.messages.iter() {
            let role = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
            };

            // If it's an Image with no URL but has a local `name`, attempt async upload to OpenAI
            let content_val = match &msg.content {
                crate::types::common::Content::Image { url, mime: _, name } => {
                    if url.is_some() {
                        crate::provider::utils::content_to_provider_value(&msg.content)
                    } else if let Some(n) = name {
                        // Try provider upload; fall back to inline behavior on error
                        let upload_url = format!("{}/files", self.base_url.trim_end_matches('/'));
                        match crate::provider::utils::upload_file_with_transport(
                            Some(self.transport.clone()),
                            &upload_url,
                            n,
                            "file",
                        )
                        .await
                        {
                            Ok(remote) => {
                                // remote may be a full URL, a data: URL, or a provider file id.
                                if remote.starts_with("http://")
                                    || remote.starts_with("https://")
                                    || remote.starts_with("data:")
                                {
                                    serde_json::json!({"image": {"url": remote}})
                                } else {
                                    // Treat as provider file id
                                    serde_json::json!({"image": {"file_id": remote}})
                                }
                            }
                            Err(_) => {
                                crate::provider::utils::content_to_provider_value(&msg.content)
                            }
                        }
                    } else {
                        crate::provider::utils::content_to_provider_value(&msg.content)
                    }
                }
                _ => crate::provider::utils::content_to_provider_value(&msg.content),
            };
            msgs.push(serde_json::json!({"role": role, "content": content_val}));
        }

        openai_request["messages"] = serde_json::Value::Array(msgs);

        // Optional params
        if let Some(temp) = request.temperature {
            openai_request["temperature"] =
                serde_json::Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
        }
        if let Some(max_tokens) = request.max_tokens {
            openai_request["max_tokens"] =
                serde_json::Value::Number(serde_json::Number::from(max_tokens));
        }
        if let Some(top_p) = request.top_p {
            openai_request["top_p"] =
                serde_json::Value::Number(serde_json::Number::from_f64(top_p.into()).unwrap());
        }
        if let Some(freq_penalty) = request.frequency_penalty {
            openai_request["frequency_penalty"] = serde_json::Value::Number(
                serde_json::Number::from_f64(freq_penalty.into()).unwrap(),
            );
        }
        if let Some(presence_penalty) = request.presence_penalty {
            openai_request["presence_penalty"] = serde_json::Value::Number(
                serde_json::Number::from_f64(presence_penalty.into()).unwrap(),
            );
        }

        // Add function calling definitions if provided
        if let Some(functions) = &request.functions {
            openai_request["functions"] =
                serde_json::to_value(functions).unwrap_or(serde_json::Value::Null);
        }

        // function_call policy may be set to control OpenAI behavior
        if let Some(policy) = &request.function_call {
            match policy {
                crate::types::function_call::FunctionCallPolicy::None => {
                    openai_request["function_call"] = serde_json::Value::String("none".to_string());
                }
                crate::types::function_call::FunctionCallPolicy::Auto(name) => {
                    if name == "auto" {
                        openai_request["function_call"] =
                            serde_json::Value::String("auto".to_string());
                    } else {
                        openai_request["function_call"] = serde_json::Value::String(name.clone());
                    }
                }
            }
        }

        request.apply_extensions(&mut openai_request);

        Ok(openai_request)
    }

    // Note: provider-specific upload helpers were removed to avoid blocking the async
    // runtime. Use `crate::provider::utils::upload_file_to_provider` (async) if provider
    // upload behavior is desired; it will be integrated in a future change.

    fn parse_response(
        &self,
        response: serde_json::Value,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        let choices = response["choices"]
            .as_array()
            .ok_or_else(|| {
                AiLibError::ProviderError("Invalid response format: choices not found".to_string())
            })?
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

                // Build the Message and try to populate a typed FunctionCall if provided by the provider
                let mut msg_obj = Message {
                    role,
                    content: crate::types::common::Content::Text(content.clone()),
                    function_call: None,
                };

                if let Some(fc_val) = message.get("function_call").cloned() {
                    // Try direct deserialization into our typed FunctionCall first
                    match serde_json::from_value::<crate::types::function_call::FunctionCall>(
                        fc_val.clone(),
                    ) {
                        Ok(fc) => {
                            msg_obj.function_call = Some(fc);
                        }
                        Err(_) => {
                            // Fallback: some providers return `arguments` as a JSON-encoded string.
                            let name = fc_val
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string();
                            let args_val = match fc_val.get("arguments") {
                                Some(a) if a.is_string() => {
                                    // Parse stringified JSON
                                    a.as_str()
                                        .and_then(|s| {
                                            serde_json::from_str::<serde_json::Value>(s).ok()
                                        })
                                        .unwrap_or(serde_json::Value::Null)
                                }
                                Some(a) => a.clone(),
                                None => serde_json::Value::Null,
                            };
                            msg_obj.function_call =
                                Some(crate::types::function_call::FunctionCall {
                                    name,
                                    arguments: Some(args_val),
                                });
                        }
                    }
                }
                Ok(Choice {
                    index: index as u32,
                    message: msg_obj,
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
            id: response["id"].as_str().unwrap_or("").to_string(),
            object: response["object"].as_str().unwrap_or("").to_string(),
            created: response["created"].as_u64().unwrap_or(0),
            model: response["model"].as_str().unwrap_or("").to_string(),
            choices,
            usage,
            usage_status: UsageStatus::Finalized, // OpenAI provides accurate usage data
        })
    }
}

#[async_trait::async_trait]
impl ChatProvider for OpenAiAdapter {
    fn name(&self) -> &str {
        "OpenAI"
    }

    async fn chat(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        // Record a request counter and start a timer using standardized keys
        self.metrics.incr_counter("openai.requests", 1).await;
        let timer = self.metrics.start_timer("openai.request_duration_ms").await;
        let url = format!("{}/chat/completions", self.base_url);

        // Build request body via converter
        let openai_request = self.convert_request_async(&request).await?;

        // Use unified transport
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response_json = self
            .transport
            .post_json(&url, Some(headers), openai_request)
            .await
            .map_err(|e| e.with_context(&format!("OpenAI chat request to {}", url)))?;

        if let Some(t) = timer {
            t.stop();
        }

        self.parse_response(response_json)
    }

    async fn stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        let url = format!("{}/chat/completions", self.base_url);

        // Build request body with stream=true
        let mut openai_request = self.convert_request_async(&request).await?;
        openai_request["stream"] = serde_json::Value::Bool(true);

        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Accept".to_string(), "text/event-stream".to_string());

        let byte_stream_res = self
            .transport
            .post_stream(&url, Some(headers), openai_request)
            .await;

        match byte_stream_res {
            Ok(mut byte_stream) => {
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    while let Some(result) = byte_stream.next().await {
                        match result {
                            Ok(bytes) => {
                                buffer.extend_from_slice(&bytes);
                                #[cfg(feature = "unified_sse")]
                                {
                                    while let Some(event_end) =
                                        crate::sse::parser::find_event_boundary(&buffer)
                                    {
                                        let event_bytes =
                                            buffer.drain(..event_end).collect::<Vec<_>>();
                                        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
                                            if let Some(chunk) =
                                                crate::sse::parser::parse_sse_event(event_text)
                                            {
                                                match chunk {
                                                    Ok(Some(c)) => {
                                                        if tx.send(Ok(c)).is_err() {
                                                            return;
                                                        }
                                                    }
                                                    Ok(None) => return, // [DONE] signal
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
                                    while let Some(event_end) = find_sse_event_boundary(&buffer) {
                                        let event_bytes =
                                            buffer.drain(..event_end).collect::<Vec<_>>();
                                        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
                                            if let Some(chunk) = parse_openai_sse_event(event_text)
                                            {
                                                match chunk {
                                                    Ok(Some(c)) => {
                                                        if tx.send(Ok(c)).is_err() {
                                                            return;
                                                        }
                                                    }
                                                    Ok(None) => return, // [DONE] signal
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
                let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
                Ok(Box::new(Box::pin(stream)))
            }
            Err(e) => Err(e),
        }
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        let url = format!("{}/models", self.base_url);
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );

        let response = self.transport.get_json(&url, Some(headers)).await?;

        Ok(response["data"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|model| model["id"].as_str().map(|s| s.to_string()))
            .collect())
    }

    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo, AiLibError> {
        Ok(ModelInfo {
            id: model_id.to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "openai".to_string(),
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

// Local SSE parsing functions for when unified_sse feature is not enabled
#[cfg(not(feature = "unified_sse"))]
fn find_sse_event_boundary(buffer: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 1 < buffer.len() {
        // LF LF
        if buffer[i] == b'\n' && buffer[i + 1] == b'\n' {
            return Some(i + 2);
        }
        // CR LF CR LF
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

#[cfg(not(feature = "unified_sse"))]
fn parse_openai_sse_event(
    event_text: &str,
) -> Option<Result<Option<ChatCompletionChunk>, AiLibError>> {
    for line in event_text.lines() {
        let line = line.trim();
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Some(Ok(None));
            }
            return Some(parse_openai_chunk_data(data));
        }
    }
    None
}

#[cfg(not(feature = "unified_sse"))]
fn parse_openai_chunk_data(data: &str) -> Result<Option<ChatCompletionChunk>, AiLibError> {
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
