use crate::api::{ChatApi, ChatCompletionChunk, ModelInfo, ModelPermission};
use crate::types::{ChatCompletionRequest, ChatCompletionResponse, AiLibError, Message, Role, Choice, Usage};
use crate::transport::{HttpTransport, DynHttpTransportRef};
use std::env;
use std::collections::HashMap;
use futures::stream::{self, Stream};

/// Google Gemini独立适配器，支持多模态AI服务
/// 
/// Google Gemini independent adapter for multimodal AI service
/// 
/// Gemini API is completely different from OpenAI format, requires independent adapter:
/// - Endpoint: /v1beta/models/{model}:generateContent
/// - Request body: contents array instead of messages
/// - Response: candidates[0].content.parts[0].text
/// - Authentication: URL parameter ?key=<API_KEY>
pub struct GeminiAdapter {
    transport: DynHttpTransportRef,
    api_key: String,
    base_url: String,
}

impl GeminiAdapter {
    pub fn new() -> Result<Self, AiLibError> {
        let api_key = env::var("GEMINI_API_KEY")
            .map_err(|_| AiLibError::AuthenticationError(
                "GEMINI_API_KEY environment variable not set".to_string()
            ))?;
        
        Ok(Self {
            transport: HttpTransport::new().boxed(),
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        })
    }

    /// Construct using object-safe transport reference
    pub fn with_transport_ref(transport: DynHttpTransportRef, api_key: String, base_url: String) -> Result<Self, AiLibError> {
        Ok(Self { transport, api_key, base_url })
    }

    /// Convert generic request to Gemini format
    fn convert_to_gemini_request(&self, request: &ChatCompletionRequest) -> serde_json::Value {
        let contents: Vec<serde_json::Value> = request.messages.iter().map(|msg| {
            let role = match msg.role {
                Role::User => "user",
                Role::Assistant => "model", // Gemini uses "model" instead of "assistant"
                Role::System => "user", // Gemini has no system role, convert to user
            };
            
            serde_json::json!({
                "role": role,
                "parts": [{"text": msg.content}]
            })
        }).collect();

        let mut gemini_request = serde_json::json!({
            "contents": contents
        });

        // Gemini generation configuration
        let mut generation_config = serde_json::json!({});
        
        if let Some(temp) = request.temperature {
            generation_config["temperature"] = serde_json::Value::Number(
                serde_json::Number::from_f64(temp.into()).unwrap()
            );
        }
        if let Some(max_tokens) = request.max_tokens {
            generation_config["maxOutputTokens"] = serde_json::Value::Number(
                serde_json::Number::from(max_tokens)
            );
        }
        if let Some(top_p) = request.top_p {
            generation_config["topP"] = serde_json::Value::Number(
                serde_json::Number::from_f64(top_p.into()).unwrap()
            );
        }

        if !generation_config.as_object().unwrap().is_empty() {
            gemini_request["generationConfig"] = generation_config;
        }

        gemini_request
    }

    /// Parse Gemini response to generic format
    fn parse_gemini_response(&self, response: serde_json::Value, model: &str) -> Result<ChatCompletionResponse, AiLibError> {
        let candidates = response["candidates"].as_array()
            .ok_or_else(|| AiLibError::ProviderError("No candidates in Gemini response".to_string()))?;

        let choices: Result<Vec<Choice>, AiLibError> = candidates.iter().enumerate().map(|(index, candidate)| {
            let content = candidate["content"]["parts"][0]["text"].as_str()
                .ok_or_else(|| AiLibError::ProviderError("No text in Gemini candidate".to_string()))?;
            
            let finish_reason = candidate["finishReason"].as_str().map(|r| match r {
                "STOP" => "stop".to_string(),
                "MAX_TOKENS" => "length".to_string(),
                _ => r.to_string(),
            });

            Ok(Choice {
                index: index as u32,
                message: Message {
                    role: Role::Assistant,
                    content: content.to_string(),
                },
                finish_reason,
            })
        }).collect();

        let usage = Usage {
            prompt_tokens: response["usageMetadata"]["promptTokenCount"].as_u64().unwrap_or(0) as u32,
            completion_tokens: response["usageMetadata"]["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
            total_tokens: response["usageMetadata"]["totalTokenCount"].as_u64().unwrap_or(0) as u32,
        };

        Ok(ChatCompletionResponse {
            id: format!("gemini-{}", chrono::Utc::now().timestamp()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: model.to_string(),
            choices: choices?,
            usage,
        })
    }
}

#[async_trait::async_trait]
impl ChatApi for GeminiAdapter {
    async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, AiLibError> {
        let gemini_request = self.convert_to_gemini_request(&request);
        
        // Gemini uses URL parameter authentication, not headers
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, request.model, self.api_key
        );

        let headers = HashMap::from([
            ("Content-Type".to_string(), "application/json".to_string()),
        ]);

        let response: serde_json::Value = self.transport
            .post_json(&url, Some(headers), gemini_request)
            .await?;

        self.parse_gemini_response(response, &request.model)
    }

    async fn chat_completion_stream(&self, _request: ChatCompletionRequest) -> Result<Box<dyn Stream<Item = Result<ChatCompletionChunk, AiLibError>> + Send + Unpin>, AiLibError> {
        // Gemini streaming response requires special handling, return empty stream for now
        let stream = stream::empty();
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