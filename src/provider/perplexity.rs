use crate::api::ChatApi;
use crate::types::{
    ChatCompletionRequest, ChatCompletionResponse, Message, Role,
};
use crate::types::AiLibError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use futures::Stream;
use async_trait::async_trait;

/// Perplexity API adapter
/// 
/// Perplexity provides search-enhanced AI with a custom API format.
/// Documentation: https://docs.perplexity.ai/getting-started/overview
pub struct PerplexityAdapter {
    client: Client,
    api_key: String,
}

impl PerplexityAdapter {
    pub fn new() -> Result<Self, AiLibError> {
        let api_key = std::env::var("PERPLEXITY_API_KEY")
            .map_err(|_| AiLibError::ConfigurationError(
                "PERPLEXITY_API_KEY environment variable not set".to_string()
            ))?;

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        let perplexity_request = self.convert_request(&request)?;
        
        let response = self
            .client
            .post("https://api.perplexity.ai/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&perplexity_request)
            .send()
            .await
            .map_err(|e| AiLibError::NetworkError(format!("Perplexity API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiLibError::ProviderError(format!(
                "Perplexity API error {}: {}",
                status, error_text
            )));
        }

        let perplexity_response: PerplexityResponse = response
            .json()
            .await
            .map_err(|e| AiLibError::DeserializationError(format!("Failed to parse Perplexity response: {}", e)))?;

        self.convert_response(perplexity_response)
    }

    pub async fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<crate::api::ChatCompletionChunk, AiLibError>> + Send + Unpin>, AiLibError> {
        // For now, convert streaming request to non-streaming and return a single chunk
        let response = self.chat_completion(request.clone()).await?;
        
        // Create a single chunk from the response
        let chunk = crate::api::ChatCompletionChunk {
            id: response.id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: response.created,
            model: response.model.clone(),
            choices: response.choices.into_iter().map(|choice| {
                crate::api::ChoiceDelta {
                    index: choice.index,
                    delta: crate::api::MessageDelta {
                        role: Some(choice.message.role),
                        content: Some(match &choice.message.content {
                            crate::Content::Text(text) => text.clone(),
                            _ => "".to_string(),
                        }),
                    },
                    finish_reason: choice.finish_reason,
                }
            }).collect(),
        };
        
        let stream = futures::stream::once(async move { Ok(chunk) });
        Ok(Box::new(Box::pin(stream)))
    }

    fn convert_request(&self, request: &ChatCompletionRequest) -> Result<PerplexityRequest, AiLibError> {
        // Convert messages to Perplexity format
        let messages = request
            .messages
            .iter()
            .map(|msg| PerplexityMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: match &msg.content {
                    crate::Content::Text(text) => text.clone(),
                    _ => "Unsupported content type".to_string(),
                },
            })
            .collect();

        Ok(PerplexityRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: Some(false),
        })
    }

    fn convert_response(&self, response: PerplexityResponse) -> Result<ChatCompletionResponse, AiLibError> {
        let choice = response.choices.first()
            .ok_or_else(|| AiLibError::InvalidModelResponse("No choices in Perplexity response".to_string()))?;

        let message = Message {
            role: match choice.message.role.as_str() {
                "assistant" => Role::Assistant,
                "user" => Role::User,
                "system" => Role::System,
                _ => Role::Assistant,
            },
            content: crate::Content::Text(choice.message.content.clone().unwrap_or_default()),
            function_call: None,
        };

        Ok(ChatCompletionResponse {
            id: response.id,
            object: "chat.completion".to_string(),
            created: response.created,
            model: response.model,
            choices: vec![crate::types::Choice {
                index: 0,
                message,
                finish_reason: choice.finish_reason.clone(),
            }],
            usage: response.usage.map(|u| crate::types::Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }).unwrap_or_else(|| crate::types::Usage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            }),
            usage_status: crate::types::response::UsageStatus::Finalized,
        })
    }
}

#[async_trait]
impl ChatApi for PerplexityAdapter {
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AiLibError> {
        self.chat_completion(request).await
    }

    async fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<
        Box<dyn Stream<Item = Result<crate::api::ChatCompletionChunk, AiLibError>> + Send + Unpin>,
        AiLibError,
    > {
        self.chat_completion_stream(request).await
    }

    async fn list_models(&self) -> Result<Vec<String>, AiLibError> {
        // Return default models for Perplexity
        Ok(vec![
            "llama-3.1-sonar-small-128k-online".to_string(),
            "llama-3.1-sonar-large-128k-online".to_string(),
        ])
    }

    async fn get_model_info(&self, model_id: &str) -> Result<crate::api::ModelInfo, AiLibError> {
        Ok(crate::api::ModelInfo {
            id: model_id.to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "perplexity".to_string(),
            permission: vec![],
        })
    }
}

#[derive(Serialize)]
struct PerplexityRequest {
    model: String,
    messages: Vec<PerplexityMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    stream: Option<bool>,
}

#[derive(Serialize)]
struct PerplexityMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct PerplexityResponse {
    id: String,
    #[allow(dead_code)]
    object: String,
    created: u64,
    model: String,
    choices: Vec<PerplexityChoice>,
    usage: Option<PerplexityUsage>,
}

#[derive(Deserialize)]
struct PerplexityChoice {
    message: PerplexityMessageResponse,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct PerplexityMessageResponse {
    role: String,
    content: Option<String>,
}

#[derive(Deserialize)]
struct PerplexityUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}
