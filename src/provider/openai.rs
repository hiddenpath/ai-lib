use crate::api::{ChatApi, ChatCompletionChunk, ModelInfo, ModelPermission};
use crate::types::{ChatCompletionRequest, ChatCompletionResponse, AiLibError, Message, Role, Choice, Usage};
use crate::transport::{HttpTransport, DynHttpTransportRef};
use std::env;
use std::collections::HashMap;
use futures::stream::{self, Stream};

/// OpenAI适配器，支持GPT系列模型
/// 
/// OpenAI adapter supporting GPT series models
pub struct OpenAiAdapter {
    transport: DynHttpTransportRef,
    api_key: String,
    base_url: String,
}

impl OpenAiAdapter {
    pub fn new() -> Result<Self, AiLibError> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| AiLibError::AuthenticationError(
                "OPENAI_API_KEY environment variable not set".to_string()
            ))?;
        
        Ok(Self {
            transport: HttpTransport::new().boxed(),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }

    /// Construct with an injected object-safe transport reference
    pub fn with_transport_ref(transport: DynHttpTransportRef, api_key: String, base_url: String) -> Result<Self, AiLibError> {
        Ok(Self { transport, api_key, base_url })
    }

    fn convert_request(&self, request: &ChatCompletionRequest) -> serde_json::Value {
        let mut openai_request = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|msg| {
                serde_json::json!({
                    "role": match msg.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                    },
                    "content": msg.content
                })
            }).collect::<Vec<_>>()
        });

        if let Some(temp) = request.temperature {
            openai_request["temperature"] = serde_json::Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
        }
        if let Some(max_tokens) = request.max_tokens {
            openai_request["max_tokens"] = serde_json::Value::Number(serde_json::Number::from(max_tokens));
        }
        if let Some(top_p) = request.top_p {
            openai_request["top_p"] = serde_json::Value::Number(serde_json::Number::from_f64(top_p.into()).unwrap());
        }
        if let Some(freq_penalty) = request.frequency_penalty {
            openai_request["frequency_penalty"] = serde_json::Value::Number(serde_json::Number::from_f64(freq_penalty.into()).unwrap());
        }
        if let Some(presence_penalty) = request.presence_penalty {
            openai_request["presence_penalty"] = serde_json::Value::Number(serde_json::Number::from_f64(presence_penalty.into()).unwrap());
        }

        openai_request
    }

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
impl ChatApi for OpenAiAdapter {
    async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, AiLibError> {
        let openai_request = self.convert_request(&request);
        let url = format!("{}/chat/completions", self.base_url);
        

        
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        let response: serde_json::Value = self.transport
            .post_json(&url, Some(headers), openai_request)
            .await?;
        
        self.parse_response(response)
    }

    async fn chat_completion_stream(&self, _request: ChatCompletionRequest) -> Result<Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>, AiLibError> {
        let stream = stream::empty();
        Ok(Box::new(Box::pin(stream)))
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        let url = format!("{}/models", self.base_url);
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), format!("Bearer {}", self.api_key));
        
        let response: serde_json::Value = self.transport
            .get_json(&url, Some(headers))
            .await?;
        
        Ok(response["data"].as_array()
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