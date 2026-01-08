//! Payload Builder - 构建不同格式的请求payload
//!
//! 支持标准格式和各种provider-specific格式的转换

use crate::manifest::schema::{PayloadFormat, ResponseFormat};
use crate::mapping::engine::MappingEngine;
use crate::types::request::ChatCompletionRequest;
use serde_json::Value;
use std::collections::HashMap;

use crate::mapping::rules::MappingRule;

/// Payload Builder - 负责构建不同格式的请求payload
pub struct PayloadBuilder {
    /// 参数映射规则（保留用于未来的高级映射）
    #[allow(dead_code)]
    parameter_mappings: HashMap<String, MappingRule>,

    /// 负载格式
    payload_format: PayloadFormat,

    /// 响应格式
    response_format: ResponseFormat,

    /// Mapping引擎
    mapping_engine: Option<MappingEngine>,
}

impl PayloadBuilder {
    /// 创建新的PayloadBuilder
    pub fn new(
        parameter_mappings: HashMap<String, MappingRule>,
        payload_format: PayloadFormat,
        response_format: ResponseFormat,
    ) -> Self {
        let mut mapping_engine =
            MappingEngine::new(parameter_mappings.clone(), payload_format.clone());

        // 设置基础模板变量
        let mut template_vars = HashMap::new();
        template_vars.insert(
            "payload_format".to_string(),
            serde_json::json!(payload_format),
        );
        template_vars.insert(
            "response_format".to_string(),
            serde_json::json!(response_format),
        );

        mapping_engine = mapping_engine.with_template_vars(template_vars);

        Self {
            parameter_mappings,
            payload_format,
            response_format,
            mapping_engine: Some(mapping_engine),
        }
    }

    /// 构建标准请求的payload
    pub fn build_payload(&self, request: &ChatCompletionRequest) -> crate::Result<Value> {
        let mut engine = self
            .mapping_engine
            .as_ref()
            .ok_or_else(|| {
                crate::AiLibError::ConfigurationError("Mapping engine not initialized".to_string())
            })?
            .clone();

        // 使用mapping引擎转换请求
        let mut payload = engine.transform_request(request)?;

        // 应用payload格式特定的后处理
        self.post_process_payload(&mut payload, request)?;

        Ok(payload)
    }

    /// 后处理payload
    fn post_process_payload(
        &self,
        payload: &mut Value,
        request: &ChatCompletionRequest,
    ) -> crate::Result<()> {
        match self.payload_format {
            PayloadFormat::OpenaiStyle => {
                self.ensure_openai_format(payload, request)?;
            }
            PayloadFormat::AnthropicStyle => {
                self.ensure_anthropic_format(payload, request)?;
            }
            PayloadFormat::GeminiStyle => {
                self.ensure_gemini_format(payload, request)?;
            }
            PayloadFormat::CohereNative => {
                self.ensure_cohere_format(payload, request)?;
            }
            PayloadFormat::Custom(_) => {
                // 自定义格式，不做额外处理
            }
        }

        // 验证payload完整性
        self.validate_payload(payload)?;

        Ok(())
    }

    /// 确保OpenAI格式
    fn ensure_openai_format(
        &self,
        payload: &mut Value,
        request: &ChatCompletionRequest,
    ) -> crate::Result<()> {
        let obj = payload.as_object_mut().ok_or_else(|| {
            crate::AiLibError::ConfigurationError("Payload must be an object".to_string())
        })?;

        // 确保基本字段存在
        if !obj.contains_key("model") {
            obj.insert("model".to_string(), serde_json::json!(request.model));
        }

        if !obj.contains_key("messages") {
            // 从原始消息构建
            let messages: Vec<Value> = request
                .messages
                .iter()
                .map(|msg| {
                    serde_json::json!({
                        "role": msg.role,
                        "content": msg.content.to_string()
                    })
                })
                .collect();
            obj.insert("messages".to_string(), serde_json::json!(messages));
        }

        // 确保流式参数正确
        if request.stream.unwrap_or(false) {
            obj.insert("stream".to_string(), serde_json::json!(true));
            obj.insert(
                "stream_options".to_string(),
                serde_json::json!({
                    "include_usage": true
                }),
            );
        }

        Ok(())
    }

    /// 确保Anthropic格式
    fn ensure_anthropic_format(
        &self,
        payload: &mut Value,
        _request: &ChatCompletionRequest,
    ) -> crate::Result<()> {
        let obj = payload.as_object_mut().ok_or_else(|| {
            crate::AiLibError::ConfigurationError("Payload must be an object".to_string())
        })?;

        // Anthropic使用不同的参数名
        if let Some(max_tokens) = obj.remove("max_tokens") {
            obj.insert("max_tokens".to_string(), max_tokens);
        }

        // 确保temperature在合理范围内
        if let Some(temp) = obj.get_mut("temperature") {
            if let Some(temp_val) = temp.as_f64() {
                // Anthropic建议温度范围0.0-1.0
                let clamped_temp = temp_val.max(0.0).min(1.0);
                *temp = serde_json::json!(clamped_temp);
            }
        }

        // 添加Anthropic特定字段
        obj.insert(
            "anthropic_version".to_string(),
            serde_json::json!("bedrock-2023-05-31"),
        );

        Ok(())
    }

    /// 确保Gemini格式
    fn ensure_gemini_format(
        &self,
        payload: &mut Value,
        _request: &ChatCompletionRequest,
    ) -> crate::Result<()> {
        let obj = payload.as_object_mut().ok_or_else(|| {
            crate::AiLibError::ConfigurationError("Payload must be an object".to_string())
        })?;

        // Gemini使用generationConfig而不是顶级参数
        if !obj.contains_key("generationConfig") {
            let mut gen_config = serde_json::Map::new();

            // 从顶级参数移动到generationConfig
            if let Some(temp) = obj.remove("temperature") {
                gen_config.insert("temperature".to_string(), temp);
            }

            if let Some(max_tokens) = obj.remove("max_tokens") {
                gen_config.insert("maxOutputTokens".to_string(), max_tokens);
            }

            if let Some(top_p) = obj.remove("top_p") {
                gen_config.insert("topP".to_string(), top_p);
            }

            if !gen_config.is_empty() {
                obj.insert(
                    "generationConfig".to_string(),
                    serde_json::json!(gen_config),
                );
            }
        }

        // 确保contents字段存在
        if !obj.contains_key("contents") && obj.contains_key("messages") {
            // 转换messages到contents格式
            if let Some(messages) = obj.remove("messages") {
                if let Some(msgs) = messages.as_array() {
                    let contents: Vec<Value> = msgs
                        .iter()
                        .map(|msg| {
                            let role =
                                if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                                    "model"
                                } else {
                                    "user"
                                };

                            let parts = if let Some(content) = msg.get("content") {
                                vec![serde_json::json!({"text": content})]
                            } else {
                                vec![]
                            };

                            serde_json::json!({
                                "role": role,
                                "parts": parts
                            })
                        })
                        .collect();

                    obj.insert("contents".to_string(), serde_json::json!(contents));
                }
            }
        }

        Ok(())
    }

    /// 确保Cohere V2格式
    fn ensure_cohere_format(
        &self,
        payload: &mut Value,
        request: &ChatCompletionRequest,
    ) -> crate::Result<()> {
        let obj = payload.as_object_mut().ok_or_else(|| {
            crate::AiLibError::ConfigurationError("Payload must be an object".to_string())
        })?;

        // Cohere V2 API uses "message" for single message or "messages" for multiple
        if !obj.contains_key("message") && !obj.contains_key("messages") {
            // Convert standard messages to Cohere format
            if request.messages.len() == 1 {
                let msg = &request.messages[0];
                obj.insert(
                    "message".to_string(),
                    serde_json::json!(msg.content.to_string()),
                );
            } else {
                let messages: Vec<Value> = request
                    .messages
                    .iter()
                    .map(|msg| {
                        serde_json::json!({
                            "role": match msg.role {
                                crate::Role::User => "user",
                                crate::Role::Assistant => "assistant",
                                crate::Role::System => "system",
                                _ => "user",
                            },
                            "content": msg.content.to_string()
                        })
                    })
                    .collect();
                obj.insert("messages".to_string(), serde_json::json!(messages));
            }
        }

        // Cohere V2 specific parameters
        if let Some(temperature) = request.temperature {
            obj.insert("temperature".to_string(), serde_json::json!(temperature));
        }

        if let Some(max_tokens) = request.max_tokens {
            obj.insert("max_tokens".to_string(), serde_json::json!(max_tokens));
        }

        // Cohere V2 supports connectors for RAG
        // This would be configured via special_handling in manifest

        Ok(())
    }

    /// 验证payload完整性
    fn validate_payload(&self, payload: &Value) -> crate::Result<()> {
        let obj = payload.as_object().ok_or_else(|| {
            crate::AiLibError::ConfigurationError("Payload must be a JSON object".to_string())
        })?;

        // 检查必需字段
        match self.payload_format {
            PayloadFormat::OpenaiStyle => {
                if !obj.contains_key("model") {
                    return Err(crate::AiLibError::ConfigurationError(
                        "OpenAI format requires 'model' field".to_string(),
                    ));
                }
                if !obj.contains_key("messages") {
                    return Err(crate::AiLibError::ConfigurationError(
                        "OpenAI format requires 'messages' field".to_string(),
                    ));
                }
            }
            PayloadFormat::AnthropicStyle => {
                if !obj.contains_key("messages") {
                    return Err(crate::AiLibError::ConfigurationError(
                        "Anthropic format requires 'messages' field".to_string(),
                    ));
                }
                if !obj.contains_key("max_tokens") {
                    return Err(crate::AiLibError::ConfigurationError(
                        "Anthropic format requires 'max_tokens' field".to_string(),
                    ));
                }
            }
            PayloadFormat::GeminiStyle => {
                if !obj.contains_key("contents") {
                    return Err(crate::AiLibError::ConfigurationError(
                        "Gemini format requires 'contents' field".to_string(),
                    ));
                }
            }
            PayloadFormat::CohereNative => {
                // Cohere V2 API format validation
                if !obj.contains_key("message") && !obj.contains_key("messages") {
                    return Err(crate::AiLibError::ConfigurationError(
                        "Cohere format requires 'message' or 'messages' field".to_string(),
                    ));
                }
            }
            PayloadFormat::Custom(_) => {
                // 自定义格式不做强制验证
            }
        }

        Ok(())
    }

    /// 获取payload格式
    pub fn payload_format(&self) -> &PayloadFormat {
        &self.payload_format
    }

    /// 获取响应格式
    pub fn response_format(&self) -> &ResponseFormat {
        &self.response_format
    }

    /// 克隆mapping引擎
    pub fn mapping_engine(&self) -> Option<&MappingEngine> {
        self.mapping_engine.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::Content;
    use crate::Message;
    use crate::Role;
    use std::collections::HashMap;

    #[test]
    fn test_openai_payload_builder() {
        let mut mappings = HashMap::new();
        mappings.insert("model".to_string(), MappingRule::direct("model"));
        mappings.insert("messages".to_string(), MappingRule::direct("messages"));
        mappings.insert(
            "temperature".to_string(),
            MappingRule::direct("temperature"),
        );

        let builder = PayloadBuilder::new(
            mappings,
            PayloadFormat::OpenaiStyle,
            ResponseFormat::OpenaiStyle,
        );

        let request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: Content::Text("Hello".to_string()),
                function_call: None,
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: Some(false),
            ..Default::default()
        };

        let payload = builder.build_payload(&request).unwrap();
        let obj = payload.as_object().unwrap();

        assert_eq!(obj["model"], "gpt-4");
        assert!(obj.contains_key("messages"));
        assert_eq!(obj["temperature"], 0.7);
    }

    #[test]
    fn test_anthropic_payload_builder() {
        let mut mappings = HashMap::new();
        mappings.insert("messages".to_string(), MappingRule::direct("messages"));
        mappings.insert(
            "temperature".to_string(),
            MappingRule::direct("temperature"),
        );
        mappings.insert("max_tokens".to_string(), MappingRule::direct("max_tokens"));

        let builder = PayloadBuilder::new(
            mappings,
            PayloadFormat::AnthropicStyle,
            ResponseFormat::AnthropicStyle,
        );

        let request = ChatCompletionRequest {
            model: "claude-3".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: Content::Text("Hello".to_string()),
                function_call: None,
            }],
            temperature: Some(1.5), // 超出Anthropic范围
            max_tokens: Some(100),
            stream: Some(false),
            ..Default::default()
        };

        let payload = builder.build_payload(&request).unwrap();
        let obj = payload.as_object().unwrap();

        // 温度应该被限制在1.0以内
        assert_eq!(obj["temperature"], 1.0);
        assert!(obj.contains_key("anthropic_version"));
    }
}
