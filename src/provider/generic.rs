use crate::api::{ChatApi, ChatCompletionChunk, ChoiceDelta, MessageDelta, ModelInfo, ModelPermission};
use crate::types::{ChatCompletionRequest, ChatCompletionResponse, AiLibError, Message, Role, Choice, Usage};
use crate::transport::{HttpClient, HttpTransport};
use super::config::ProviderConfig;
use std::env;
use futures::stream::{Stream, StreamExt};

/// 配置驱动的通用适配器，支持OpenAI兼容API
/// 
/// Configuration-driven generic adapter for OpenAI-compatible APIs
pub struct GenericAdapter {
    transport: HttpTransport,
    config: ProviderConfig,
    api_key: String,
}

impl GenericAdapter {
    pub fn new(config: ProviderConfig) -> Result<Self, AiLibError> {
        let api_key = env::var(&config.api_key_env)
            .map_err(|_| AiLibError::AuthenticationError(
                format!("{} environment variable not set", config.api_key_env)
            ))?;
        
        Ok(Self {
            transport: HttpTransport::new(),
            config,
            api_key,
        })
    }
    
    /// Create adapter with custom transport layer (for testing)
    pub fn with_transport(config: ProviderConfig, transport: HttpTransport) -> Result<Self, AiLibError> {
        let api_key = env::var(&config.api_key_env)
            .map_err(|_| AiLibError::AuthenticationError(
                format!("{} environment variable not set", config.api_key_env)
            ))?;
        
        Ok(Self {
            transport,
            config,
            api_key,
        })
    }

    /// Convert generic request to provider-specific format
    fn convert_request(&self, request: &ChatCompletionRequest) -> serde_json::Value {
        let default_role = "user".to_string();
        
        // Build messages array
        let messages: Vec<serde_json::Value> = request.messages.iter().map(|msg| {
            let role_key = format!("{:?}", msg.role);
            let mapped_role = self.config.field_mapping.role_mapping
                .get(&role_key)
                .unwrap_or(&default_role);
            serde_json::json!({
                "role": mapped_role,
                "content": msg.content
            })
        }).collect();
        
        // Use string literals as JSON keys
        let mut provider_request = serde_json::json!({
            "model": request.model,
            "messages": messages
        });

        // Add optional parameters
        if let Some(temp) = request.temperature {
            provider_request["temperature"] = serde_json::Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
        }
        if let Some(max_tokens) = request.max_tokens {
            provider_request["max_tokens"] = serde_json::Value::Number(serde_json::Number::from(max_tokens));
        }
        if let Some(top_p) = request.top_p {
            provider_request["top_p"] = serde_json::Value::Number(serde_json::Number::from_f64(top_p.into()).unwrap());
        }
        if let Some(freq_penalty) = request.frequency_penalty {
            provider_request["frequency_penalty"] = serde_json::Value::Number(serde_json::Number::from_f64(freq_penalty.into()).unwrap());
        }
        if let Some(presence_penalty) = request.presence_penalty {
            provider_request["presence_penalty"] = serde_json::Value::Number(serde_json::Number::from_f64(presence_penalty.into()).unwrap());
        }

        provider_request
    }

    /// Find event boundary
    fn find_event_boundary(buffer: &[u8]) -> Option<usize> {
        let mut i = 0;
        while i < buffer.len().saturating_sub(1) {
            if buffer[i] == b'\n' && buffer[i + 1] == b'\n' {
                return Some(i + 2);
            }
            if i < buffer.len().saturating_sub(3) 
                && buffer[i] == b'\r' && buffer[i + 1] == b'\n' 
                && buffer[i + 2] == b'\r' && buffer[i + 3] == b'\n' {
                return Some(i + 4);
            }
            i += 1;
        }
        None
    }
    
    /// Parse SSE event
    fn parse_sse_event(event_text: &str) -> Option<Result<Option<ChatCompletionChunk>, AiLibError>> {
        for line in event_text.lines() {
            let line = line.trim();
            if line.starts_with("data: ") {
                let data = &line[6..];
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
                let choices = json["choices"].as_array()
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
                                    finish_reason: choice["finish_reason"].as_str().map(str::to_string),
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                
                Ok(Some(ChatCompletionChunk {
                    id: json["id"].as_str().unwrap_or_default().to_string(),
                    object: json["object"].as_str().unwrap_or("chat.completion.chunk").to_string(),
                    created: json["created"].as_u64().unwrap_or(0),
                    model: json["model"].as_str().unwrap_or_default().to_string(),
                    choices,
                }))
            }
            Err(e) => Err(AiLibError::ProviderError(format!("JSON parse error: {}", e)))
        }
    }
    
    /// Parse response
    fn parse_response(&self, response: serde_json::Value) -> Result<ChatCompletionResponse, AiLibError> {
        let choices = response["choices"]
            .as_array()
            .ok_or_else(|| AiLibError::ProviderError("Invalid response format: choices not found".to_string()))?
            .iter()
            .enumerate()
            .map(|(index, choice)| {
                let message = choice["message"].as_object()
                    .ok_or_else(|| AiLibError::ProviderError("Invalid choice format".to_string()))?;
                
                let role = match message["role"].as_str().unwrap_or("user") {
                    "system" => Role::System,
                    "assistant" => Role::Assistant,
                    _ => Role::User,
                };
                
                let content = message["content"].as_str()
                    .unwrap_or("")
                    .to_string();
                
                Ok(Choice {
                    index: index as u32,
                    message: Message { role, content },
                    finish_reason: choice["finish_reason"].as_str().map(|s| s.to_string()),
                })
            })
            .collect::<Result<Vec<_>, AiLibError>>()?;
        
        let usage = response["usage"].as_object()
            .ok_or_else(|| AiLibError::ProviderError("Invalid response format: usage not found".to_string()))?;
        
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
    async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, AiLibError> {
        let provider_request = self.convert_request(&request);
        let url = format!("{}{}", self.config.base_url, self.config.chat_endpoint);
        
        let mut headers = self.config.headers.clone();
        
        // Set different authentication methods based on provider
        if self.config.base_url.contains("anthropic.com") {
            headers.insert("x-api-key".to_string(), self.api_key.clone());
        } else {
            headers.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));
        }
        
        let response: serde_json::Value = self.transport
            .post(&url, Some(headers), &provider_request)
            .await?;
        
        self.parse_response(response)
    }

    async fn chat_completion_stream(&self, request: ChatCompletionRequest) -> Result<Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>, AiLibError> {
        let mut stream_request = self.convert_request(&request);
        stream_request["stream"] = serde_json::Value::Bool(true);
        
        let url = format!("{}{}", self.config.base_url, self.config.chat_endpoint);
        
        // Create HTTP client
        let mut client_builder = reqwest::Client::builder();
        if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
            if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }
        let client = client_builder.build()
            .map_err(|e| AiLibError::ProviderError(format!("Client error: {}", e)))?;
        
        let mut headers = self.config.headers.clone();
        headers.insert("Accept".to_string(), "text/event-stream".to_string());
        
        // Set different authentication methods based on provider
        if self.config.base_url.contains("anthropic.com") {
            headers.insert("x-api-key".to_string(), self.api_key.clone());
        } else {
            headers.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));
        }
        
        let response = client
            .post(&url)
            .json(&stream_request);
        
        let mut req = response;
        for (key, value) in headers {
            req = req.header(key, value);
        }
        
        let response = req.send().await
            .map_err(|e| AiLibError::ProviderError(format!("Stream request failed: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AiLibError::ProviderError(format!("Stream error: {}", error_text)));
        }
        
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut stream = response.bytes_stream();
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(bytes) => {
                        buffer.extend_from_slice(&bytes);
                        
                        while let Some(event_end) = Self::find_event_boundary(&buffer) {
                            let event_bytes = buffer.drain(..event_end).collect::<Vec<_>>();
                            
                            if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
                                if let Some(chunk) = Self::parse_sse_event(event_text) {
                                    match chunk {
                                        Ok(Some(c)) => {
                                            if tx.send(Ok(c)).is_err() {
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
                    Err(e) => {
                        let _ = tx.send(Err(AiLibError::ProviderError(format!("Stream error: {}", e))));
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
            
            // Set different authentication methods based on provider
            if self.config.base_url.contains("anthropic.com") {
                headers.insert("x-api-key".to_string(), self.api_key.clone());
            } else {
                headers.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));
            }
            
            let response: serde_json::Value = self.transport
                .get(&url, Some(headers))
                .await?;
            
            Ok(response["data"].as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|model| model["id"].as_str().map(|s| s.to_string()))
                .collect())
        } else {
            Err(AiLibError::ProviderError("Models endpoint not configured".to_string()))
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