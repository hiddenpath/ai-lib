use super::config::ProviderConfig;
use crate::api::{
    ChatApi, ChatCompletionChunk, ChoiceDelta, MessageDelta, ModelInfo, ModelPermission,
};
use crate::metrics::{Metrics, NoopMetrics};
use crate::transport::{DynHttpTransportRef, HttpTransport};
use crate::types::{
    AiLibError, ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Role, Usage,
};
use futures::stream::{Stream, StreamExt};
use std::env;
use std::sync::Arc;
/// Configuration-driven generic adapter for OpenAI-compatible APIs
pub struct GenericAdapter {
    transport: DynHttpTransportRef,
    config: ProviderConfig,
    api_key: Option<String>,
    metrics: Arc<dyn Metrics>,
}

impl GenericAdapter {
    pub fn new(config: ProviderConfig) -> Result<Self, AiLibError> {
        // Validate configuration
        config.validate()?;

        // For generic/config-driven providers we treat the API key as optional.
        // Some deployments (e.g. local Ollama) don't require a key. If the env var
        // is missing we continue with None and callers will simply omit auth headers.
        let api_key = env::var(&config.api_key_env).ok();

        Ok(Self {
            transport: HttpTransport::new_without_proxy().boxed(),
            config,
            api_key,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Create adapter with an explicit API key override (takes precedence over env var).
    pub fn new_with_api_key(
        config: ProviderConfig,
        api_key_override: Option<String>,
    ) -> Result<Self, AiLibError> {
        config.validate()?;
        let api_key = api_key_override.or_else(|| env::var(&config.api_key_env).ok());
        Ok(Self {
            transport: HttpTransport::new_without_proxy().boxed(),
            config,
            api_key,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Create adapter with custom transport layer (for testing)
    pub fn with_transport(
        config: ProviderConfig,
        transport: HttpTransport,
    ) -> Result<Self, AiLibError> {
        // Validate configuration
        config.validate()?;

        let api_key = env::var(&config.api_key_env).ok();

        Ok(Self {
            transport: transport.boxed(),
            config,
            api_key,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Custom transport + API key override.
    pub fn with_transport_api_key(
        config: ProviderConfig,
        transport: HttpTransport,
        api_key_override: Option<String>,
    ) -> Result<Self, AiLibError> {
        config.validate()?;
        let api_key = api_key_override.or_else(|| env::var(&config.api_key_env).ok());
        Ok(Self {
            transport: transport.boxed(),
            config,
            api_key,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Accept an object-safe transport reference directly
    pub fn with_transport_ref(
        config: ProviderConfig,
        transport: DynHttpTransportRef,
    ) -> Result<Self, AiLibError> {
        // Validate configuration
        config.validate()?;

        let api_key = env::var(&config.api_key_env).ok();
        Ok(Self {
            transport,
            config,
            api_key,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Object-safe transport + API key override.
    pub fn with_transport_ref_api_key(
        config: ProviderConfig,
        transport: DynHttpTransportRef,
        api_key_override: Option<String>,
    ) -> Result<Self, AiLibError> {
        config.validate()?;
        let api_key = api_key_override.or_else(|| env::var(&config.api_key_env).ok());
        Ok(Self {
            transport,
            config,
            api_key,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// Create adapter with custom transport and an injected metrics implementation
    pub fn with_transport_ref_and_metrics(
        config: ProviderConfig,
        transport: DynHttpTransportRef,
        metrics: Arc<dyn Metrics>,
    ) -> Result<Self, AiLibError> {
        // Validate configuration
        config.validate()?;

        let api_key = env::var(&config.api_key_env).ok();
        Ok(Self {
            transport,
            config,
            api_key,
            metrics,
        })
    }

    /// Create adapter with injected metrics (uses default HttpTransport)
    pub fn with_metrics(
        config: ProviderConfig,
        metrics: Arc<dyn Metrics>,
    ) -> Result<Self, AiLibError> {
        // Validate configuration
        config.validate()?;

        let api_key = env::var(&config.api_key_env).ok();
        Ok(Self {
            transport: HttpTransport::new().boxed(),
            config,
            api_key,
            metrics,
        })
    }

    /// Convert generic request to provider-specific format (async: may upload local files)
    async fn convert_request(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<serde_json::Value, AiLibError> {
        let default_role = "user".to_string();

        // Build messages array; may perform uploads for local files
        let mut messages: Vec<serde_json::Value> = Vec::with_capacity(request.messages.len());
        for msg in request.messages.iter() {
            let role_key = format!("{:?}", msg.role);
            let mapped_role = self
                .config
                .field_mapping
                .role_mapping
                .get(&role_key)
                .unwrap_or(&default_role)
                .clone();

            // Handle multimodal: if image has no url but has a name and upload endpoint configured, upload it
            let content_val = match &msg.content {
                crate::types::common::Content::Image {
                    url,
                    mime: _mime,
                    name,
                } => {
                    if url.is_some() {
                        crate::provider::utils::content_to_provider_value(&msg.content)
                    } else if let Some(n) = name {
                        if let Some(upload_ep) = &self.config.upload_endpoint {
                            let upload_url = format!(
                                "{}{}",
                                self.config.base_url.trim_end_matches('/'),
                                upload_ep
                            );
                            // Decide whether to upload or inline based on configured size limit.
                            let should_upload = match self.config.upload_size_limit {
                                Some(limit) => match std::fs::metadata(n) {
                                    Ok(meta) => meta.len() > limit,
                                    Err(_) => true, // if we can't stat the file, attempt upload
                                },
                                None => true, // default: upload if no limit configured (preserve prior behavior)
                            };

                            if should_upload {
                                // Use the injected transport when available so tests can mock uploads.
                                match crate::provider::utils::upload_file_with_transport(
                                    Some(self.transport.clone()),
                                    &upload_url,
                                    n,
                                    "file",
                                )
                                .await
                                {
                                    Ok(remote_url) => {
                                        if remote_url.starts_with("http://")
                                            || remote_url.starts_with("https://")
                                            || remote_url.starts_with("data:")
                                        {
                                            serde_json::json!({"image": {"url": remote_url}})
                                        } else {
                                            serde_json::json!({"image": {"file_id": remote_url}})
                                        }
                                    }
                                    Err(_) => crate::provider::utils::content_to_provider_value(
                                        &msg.content,
                                    ),
                                }
                            } else {
                                // Inline small files as data URLs
                                crate::provider::utils::content_to_provider_value(&msg.content)
                            }
                        } else {
                            crate::provider::utils::content_to_provider_value(&msg.content)
                        }
                    } else {
                        crate::provider::utils::content_to_provider_value(&msg.content)
                    }
                }
                _ => crate::provider::utils::content_to_provider_value(&msg.content),
            };

            messages.push(serde_json::json!({"role": mapped_role, "content": content_val}));
        }

        // Use string literals as JSON keys
        let mut provider_request = serde_json::json!({
            "model": request.model,
            "messages": messages
        });

        // Add optional parameters
        if let Some(temp) = request.temperature {
            provider_request["temperature"] =
                serde_json::Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
        }
        if let Some(max_tokens) = request.max_tokens {
            provider_request["max_tokens"] =
                serde_json::Value::Number(serde_json::Number::from(max_tokens));
        }
        if let Some(top_p) = request.top_p {
            provider_request["top_p"] =
                serde_json::Value::Number(serde_json::Number::from_f64(top_p.into()).unwrap());
        }
        if let Some(freq_penalty) = request.frequency_penalty {
            provider_request["frequency_penalty"] = serde_json::Value::Number(
                serde_json::Number::from_f64(freq_penalty.into()).unwrap(),
            );
        }
        if let Some(presence_penalty) = request.presence_penalty {
            provider_request["presence_penalty"] = serde_json::Value::Number(
                serde_json::Number::from_f64(presence_penalty.into()).unwrap(),
            );
        }

        // Function calling (OpenAI-compatible). Many config-driven providers accept this schema.
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
            provider_request["functions"] = serde_json::Value::Array(mapped);
        }

        if let Some(policy) = &request.function_call {
            match policy {
                crate::types::FunctionCallPolicy::Auto(name) => {
                    if name == "auto" {
                        provider_request["function_call"] = serde_json::Value::String("auto".to_string());
                    } else {
                        provider_request["function_call"] = serde_json::json!({"name": name});
                    }
                }
                crate::types::FunctionCallPolicy::None => {
                    provider_request["function_call"] = serde_json::Value::String("none".to_string());
                }
            }
        }

        Ok(provider_request)
    }

    /// Find event boundary
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

    /// Parse SSE event
    fn parse_sse_event(
        event_text: &str,
    ) -> Option<Result<Option<ChatCompletionChunk>, AiLibError>> {
        for line in event_text.lines() {
            let line = line.trim();
            if let Some(stripped) = line.strip_prefix("data: ") {
                let data = stripped;
                if data == "[DONE]" {
                    return Some(Ok(None));
                }
                return Some(Self::parse_chunk_data(data));
            }
        }
        None
    }

    /// Parse chunk data
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
                                ChoiceDelta {
                                    index: index as u32,
                                    delta: MessageDelta {
                                        role: delta["role"].as_str().map(|r| match r {
                                            "assistant" => Role::Assistant,
                                            "user" => Role::User,
                                            "system" => Role::System,
                                            _ => Role::Assistant,
                                        }),
                                        content: delta["content"].as_str().map(str::to_string),
                                    },
                                    finish_reason: choice["finish_reason"]
                                        .as_str()
                                        .map(str::to_string),
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

    /// Parse response
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

                // try to parse a function_call if present
                let mut function_call: Option<crate::types::function_call::FunctionCall> = None;
                if let Some(fc_val) = message.get("function_call") {
                    // attempt full deserialization
                    if let Ok(mut fc) = serde_json::from_value::<
                        crate::types::function_call::FunctionCall,
                    >(fc_val.clone())
                    {
                        // If the provider deserialized arguments as a JSON string, try to parse it into structured JSON.
                        if let Some(arg_val) = &fc.arguments {
                            if arg_val.is_string() {
                                if let Some(s) = arg_val.as_str() {
                                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(s)
                                    {
                                        fc.arguments = Some(parsed);
                                    }
                                }
                            }
                        }
                        function_call = Some(fc);
                    } else {
                        // fallback: try to extract name + arguments (arguments may be a string)
                        let name = fc_val
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        if let Some(name) = name {
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
            id: response["id"].as_str().unwrap_or("").to_string(),
            object: response["object"].as_str().unwrap_or("").to_string(),
            created: response["created"].as_u64().unwrap_or(0),
            model: response["model"].as_str().unwrap_or("").to_string(),
            choices,
            usage,
        })
    }
}
#[async_trait::async_trait]
impl ChatApi for GenericAdapter {
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        // metrics: standardized keys
        let provider_key = "generic";
        self.metrics
            .incr_counter(&crate::metrics::keys::requests(provider_key), 1)
            .await;
        let timer = self
            .metrics
            .start_timer(&crate::metrics::keys::request_duration_ms(provider_key))
            .await;

        // Build request body & headers
        let url = self.config.chat_url();
        let provider_request = self.convert_request(&request).await?;
        let mut headers = self.config.headers.clone();
        if let Some(key) = &self.api_key {
            if self.config.base_url.contains("anthropic.com") {
                headers.insert("x-api-key".to_string(), key.clone());
            } else {
                headers.insert("Authorization".to_string(), format!("Bearer {}", key));
            }
        }

        // Use transport to allow mocking in tests
        let response_json = self
            .transport
            .post_json(&url, Some(headers), provider_request)
            .await?;

        // Stop timer
        if let Some(t) = timer {
            t.stop();
        }

        let parsed = self.parse_response(response_json)?;

        // optional cost metrics
        #[cfg(feature = "cost_metrics")]
        {
            let usd = crate::metrics::cost::estimate_usd(parsed.usage.prompt_tokens, parsed.usage.completion_tokens);
            crate::metrics::cost::record_cost(self.metrics.as_ref(), provider_key, &parsed.model, usd).await;
        }

        Ok(parsed)
    }

    async fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        let mut stream_request = self.convert_request(&request).await?;
        stream_request["stream"] = serde_json::Value::Bool(true);

        let url = self.config.chat_url();

        // Create HTTP client
        let mut client_builder = reqwest::Client::builder();
        if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }
        let client = client_builder
            .build()
            .map_err(|e| AiLibError::ProviderError(format!("Client error: {}", e)))?;

        let mut headers = self.config.headers.clone();
        headers.insert("Accept".to_string(), "text/event-stream".to_string());

        // Set different authentication methods based on provider when an API key is present
        if let Some(key) = &self.api_key {
            if self.config.base_url.contains("anthropic.com") {
                headers.insert("x-api-key".to_string(), key.clone());
            } else {
                headers.insert("Authorization".to_string(), format!("Bearer {}", key));
            }
        }

        let response = client.post(&url).json(&stream_request);

        let mut req = response;
        for (key, value) in headers {
            req = req.header(key, value);
        }

        let response = req
            .send()
            .await
            .map_err(|e| AiLibError::ProviderError(format!("Stream request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AiLibError::ProviderError(format!(
                "Stream error: {}",
                error_text
            )));
        }

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut stream = response.bytes_stream();

            while let Some(result) = stream.next().await {
                match result {
                    Ok(bytes) => {
                        buffer.extend_from_slice(&bytes);

                        #[cfg(feature = "unified_sse")]
                        {
                            while let Some(event_end) = crate::sse::parser::find_event_boundary(&buffer) {
                                let event_bytes = buffer.drain(..event_end).collect::<Vec<_>>();
                                if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
                                    if let Some(chunk) = crate::sse::parser::parse_sse_event(event_text) {
                                        match chunk {
                                            Ok(Some(c)) => { if tx.send(Ok(c)).is_err() { return; } }
                                            Ok(None) => return,
                                            Err(e) => { let _ = tx.send(Err(e)); return; }
                                        }
                                    }
                                }
                            }
                        }

                        #[cfg(not(feature = "unified_sse"))]
                        {
                            while let Some(event_end) = Self::find_event_boundary(&buffer) {
                                let event_bytes = buffer.drain(..event_end).collect::<Vec<_>>();
                                if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
                                    if let Some(chunk) = Self::parse_sse_event(event_text) {
                                        match chunk {
                                            Ok(Some(c)) => { if tx.send(Ok(c)).is_err() { return; } }
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

        let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
        Ok(Box::new(Box::pin(stream)))
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        if let Some(models_endpoint) = &self.config.models_endpoint {
            let url = format!("{}{}", self.config.base_url, models_endpoint);
            let mut headers = self.config.headers.clone();

            // Set different authentication methods based on provider when an API key is present
            if let Some(key) = &self.api_key {
                if self.config.base_url.contains("anthropic.com") {
                    headers.insert("x-api-key".to_string(), key.clone());
                } else {
                    headers.insert("Authorization".to_string(), format!("Bearer {}", key));
                }
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
                .filter_map(|model| model["id"].as_str().map(|s| s.to_string()))
                .collect())
        } else {
            Err(AiLibError::ProviderError(
                "Models endpoint not configured".to_string(),
            ))
        }
    }

    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo, AiLibError> {
        Ok(ModelInfo {
            id: model_id.to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "generic".to_string(),
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
