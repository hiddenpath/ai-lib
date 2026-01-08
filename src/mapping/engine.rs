//! Mapping引擎核心实现
//!
//! 实现了复杂的数据映射逻辑，支持路径解析、模板替换、条件映射等

use crate::manifest::schema::PayloadFormat;
use crate::types::{common::Content, request::ChatCompletionRequest};
use serde_json::{json, Value};
use std::collections::HashMap;

use super::errors::{MappingError, MappingResult};
use super::rules::{ConditionalMapping, MappingRule, ParameterTransform, TransformType};

/// Mapping引擎 - 将StandardRequest转换为Provider-specific格式
#[derive(Clone)]
pub struct MappingEngine {
    /// 参数映射规则
    parameter_mappings: HashMap<String, MappingRule>,

    /// 负载格式策略
    payload_format: PayloadFormat,

    /// 路径重写规则
    path_rewrites: Vec<(String, String)>,

    /// 模板变量
    template_vars: HashMap<String, Value>,
}

impl MappingEngine {
    /// 创建新的Mapping引擎
    pub fn new(
        parameter_mappings: HashMap<String, MappingRule>,
        payload_format: PayloadFormat,
    ) -> Self {
        Self {
            parameter_mappings,
            payload_format,
            path_rewrites: Vec::new(),
            template_vars: HashMap::new(),
        }
    }

    /// 设置路径重写规则
    pub fn with_path_rewrites(mut self, rewrites: Vec<(String, String)>) -> Self {
        self.path_rewrites = rewrites;
        self
    }

    /// 设置模板变量
    pub fn with_template_vars(mut self, vars: HashMap<String, Value>) -> Self {
        self.template_vars = vars;
        self
    }

    /// 将标准请求转换为提供商特定的请求
    pub fn transform_request(&mut self, request: &ChatCompletionRequest) -> MappingResult<Value> {
        let mut result = json!({});

        // 应用参数映射
        let mappings = self.parameter_mappings.clone();
        for (param_name, mapping_rule) in &mappings {
            self.apply_mapping_rule(&mut result, param_name, mapping_rule, request)?;
        }

        // 应用负载格式特定的转换
        self.apply_payload_format(&mut result, request)?;

        Ok(result)
    }

    /// 应用映射规则
    fn apply_mapping_rule(
        &mut self,
        target: &mut Value,
        param_name: &str,
        rule: &MappingRule,
        request: &ChatCompletionRequest,
    ) -> MappingResult<()> {
        match rule {
            MappingRule::Direct(target_path) => {
                self.apply_direct_mapping(target, param_name, target_path, request)?;
            }
            MappingRule::Conditional(conditions) => {
                self.apply_conditional_mapping(target, param_name, conditions, request)?;
            }
            MappingRule::Transform(transform) => {
                self.apply_transform_mapping(target, param_name, transform, request)?;
            }
        }
        Ok(())
    }

    /// 应用直接映射
    fn apply_direct_mapping(
        &mut self,
        target: &mut Value,
        param_name: &str,
        target_path: &str,
        request: &ChatCompletionRequest,
    ) -> MappingResult<()> {
        let source_value = self.extract_param_value(param_name, request)?;
        self.set_path_value(target, target_path, source_value)?;
        Ok(())
    }

    /// 应用条件映射
    fn apply_conditional_mapping(
        &mut self,
        target: &mut Value,
        param_name: &str,
        conditions: &[ConditionalMapping],
        request: &ChatCompletionRequest,
    ) -> MappingResult<()> {
        for condition in conditions {
            if self.evaluate_condition(&condition.condition, request)? {
                let target_path = &condition.target_path;
                let source_value = self.extract_param_value(param_name, request)?;

                // 应用可选的转换
                let final_value = if let Some(transform) = &condition.transform {
                    self.apply_transform(&source_value, transform)?
                } else {
                    source_value
                };

                self.set_path_value(target, target_path, final_value)?;
                return Ok(());
            }
        }

        // 如果没有条件匹配，使用默认值
        Err(MappingError::ConfigurationError {
            message: format!("No matching condition for parameter: {}", param_name),
        })
    }

    /// 应用转换映射
    fn apply_transform_mapping(
        &mut self,
        target: &mut Value,
        param_name: &str,
        transform: &ParameterTransform,
        request: &ChatCompletionRequest,
    ) -> MappingResult<()> {
        let source_value = self.extract_param_value(param_name, request)?;
        let transformed_value = self.apply_transform(&source_value, transform)?;

        let target_path = transform
            .target_path
            .as_deref()
            .unwrap_or_else(|| self.generate_target_path(param_name));

        self.set_path_value(target, target_path, transformed_value)?;
        Ok(())
    }

    /// 应用转换规则
    fn apply_transform(
        &self,
        value: &Value,
        transform: &ParameterTransform,
    ) -> MappingResult<Value> {
        match transform.transform_type {
            TransformType::Scale => self.apply_scale_transform(value, transform),
            TransformType::Format => self.apply_format_transform(value, transform),
            TransformType::EnumMap => self.apply_enum_map_transform(value, transform),
            TransformType::PathRewrite => self.apply_path_rewrite_transform(value, transform),
            TransformType::TypeCast => self.apply_type_cast_transform(value, transform),
            TransformType::Custom => Err(MappingError::ConfigurationError {
                message: "Custom transforms not implemented yet".to_string(),
            }),
        }
    }

    /// 应用缩放转换
    fn apply_scale_transform(
        &self,
        value: &Value,
        transform: &ParameterTransform,
    ) -> MappingResult<Value> {
        let factor = transform
            .params
            .get("factor")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| MappingError::ConfigurationError {
                message: "Scale transform requires 'factor' parameter".to_string(),
            })?;

        match value {
            Value::Number(n) => {
                let scaled = n.as_f64().unwrap_or(0.0) * factor;
                Ok(json!(scaled))
            }
            _ => Err(MappingError::TypeConversionFailed {
                from: value.to_string(),
                to: "number".to_string(),
            }),
        }
    }

    /// 应用格式化转换
    fn apply_format_transform(
        &self,
        value: &Value,
        transform: &ParameterTransform,
    ) -> MappingResult<Value> {
        let template = transform
            .params
            .get("template")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MappingError::ConfigurationError {
                message: "Format transform requires 'template' parameter".to_string(),
            })?;

        // 简单的模板替换实现
        let mut result = template.to_string();
        if let Some(var_name) = template.strip_prefix("{{") {
            if let Some(var_name) = var_name.strip_suffix("}}") {
                if let Some(replacement) = self.template_vars.get(var_name) {
                    result = replacement.to_string();
                } else if var_name == "value" {
                    result = value.to_string().trim_matches('"').to_string();
                }
            }
        }

        Ok(json!(result))
    }

    /// 应用枚举映射转换
    fn apply_enum_map_transform(
        &self,
        value: &Value,
        transform: &ParameterTransform,
    ) -> MappingResult<Value> {
        let mappings = transform
            .params
            .get("mappings")
            .and_then(|v| v.as_object())
            .ok_or_else(|| MappingError::ConfigurationError {
                message: "EnumMap transform requires 'mappings' parameter".to_string(),
            })?;

        let binding = value.to_string();
        let key = binding.trim_matches('"');
        if let Some(mapped_value) = mappings.get(key) {
            Ok(mapped_value.clone())
        } else {
            // 使用默认值或返回原值
            Ok(value.clone())
        }
    }

    /// 应用路径重写转换
    fn apply_path_rewrite_transform(
        &self,
        value: &Value,
        _transform: &ParameterTransform,
    ) -> MappingResult<Value> {
        // 简单的路径重写逻辑
        // 实际实现会更复杂，支持通配符等
        Ok(value.clone())
    }

    /// 应用类型转换
    fn apply_type_cast_transform(
        &self,
        value: &Value,
        transform: &ParameterTransform,
    ) -> MappingResult<Value> {
        let target_type = transform
            .params
            .get("target_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MappingError::ConfigurationError {
                message: "TypeCast transform requires 'target_type' parameter".to_string(),
            })?;

        match target_type {
            "string" => Ok(json!(value.to_string())),
            "number" => {
                if let Some(s) = value.as_str() {
                    if let Ok(n) = s.parse::<f64>() {
                        Ok(json!(n))
                    } else {
                        Err(MappingError::TypeConversionFailed {
                            from: s.to_string(),
                            to: "number".to_string(),
                        })
                    }
                } else {
                    Ok(value.clone())
                }
            }
            _ => Err(MappingError::ConfigurationError {
                message: format!("Unsupported target type: {}", target_type),
            }),
        }
    }

    /// 提取参数值
    fn extract_param_value(
        &self,
        param_name: &str,
        request: &ChatCompletionRequest,
    ) -> MappingResult<Value> {
        match param_name {
            "model" => Ok(json!(request.model)),
            "messages" => self.extract_messages(request),
            "temperature" => Ok(json!(request.temperature)),
            "max_tokens" => Ok(json!(request.max_tokens)),
            "stream" => Ok(json!(request.stream)),
            "top_p" => Ok(json!(request.top_p)),
            "top_k" => Ok(json!(request.top_k)),
            "stop_sequences" => Ok(json!(request.stop_sequences)),
            "logprobs" => Ok(json!(request.logprobs)),
            "top_logprobs" => Ok(json!(request.top_logprobs)),
            "seed" => Ok(json!(request.seed)),
            "response_format_mode" => Ok(json!(request.response_format_mode.clone())),
            "functions" => Ok(json!(request.functions)),
            "function_call" | "tool_choice" => Ok(json!(request.function_call)),
            _ => Err(MappingError::MissingRequiredParameter {
                parameter: param_name.to_string(),
            }),
        }
    }

    /// 提取消息内容
    fn extract_messages(&self, request: &ChatCompletionRequest) -> MappingResult<Value> {
        let messages: Vec<Value> = request
            .messages
            .iter()
            .map(|msg| {
                json!({
                    "role": msg.role,
                    "content": match &msg.content {
                        Content::Text(text) => json!(text),
                        Content::Image { .. } => json!("[IMAGE]"),
                        Content::Audio { .. } => json!("[AUDIO]"),
                        Content::Json(json) => json.clone(),
                    }
                })
            })
            .collect();

        Ok(json!(messages))
    }

    /// 评估条件表达式
    fn evaluate_condition(
        &self,
        condition: &str,
        request: &ChatCompletionRequest,
    ) -> MappingResult<bool> {
        // 简化的条件评估实现
        // 实际应该支持更复杂的表达式
        if condition.contains("stream=true") {
            Ok(request.stream.unwrap_or(false))
        } else if condition.contains("has_functions") {
            Ok(request.functions.as_ref().map_or(false, |f| !f.is_empty()))
        } else {
            Ok(true) // 默认匹配
        }
    }

    /// 设置路径值
    fn set_path_value(&self, target: &mut Value, path: &str, value: Value) -> MappingResult<()> {
        let parts: Vec<&str> = path.split('.').collect();
        self.set_nested_value(target, &parts, value)
    }

    /// 递归设置嵌套值
    fn set_nested_value(
        &self,
        target: &mut Value,
        parts: &[&str],
        value: Value,
    ) -> MappingResult<()> {
        if parts.is_empty() {
            *target = value;
            return Ok(());
        }

        let obj = target
            .as_object_mut()
            .ok_or_else(|| MappingError::InvalidPathSyntax {
                path: parts.join("."),
            })?;

        let key = parts[0];
        if parts.len() == 1 {
            obj.insert(key.to_string(), value);
        } else {
            if !obj.contains_key(key) {
                obj.insert(key.to_string(), json!({}));
            }
            let next = obj.get_mut(key).unwrap();
            self.set_nested_value(next, &parts[1..], value)?;
        }

        Ok(())
    }

    /// 生成目标路径
    fn generate_target_path<'a>(&self, param_name: &'a str) -> &'a str {
        // 简单的路径生成逻辑
        match param_name {
            "temperature" => "generationConfig.temperature",
            "max_tokens" => "generationConfig.maxOutputTokens",
            "messages" => "contents",
            _ => param_name,
        }
    }

    /// 应用负载格式特定的转换
    fn apply_payload_format(
        &self,
        target: &mut Value,
        request: &ChatCompletionRequest,
    ) -> MappingResult<()> {
        match self.payload_format {
            PayloadFormat::OpenaiStyle => {
                // OpenAI格式已经是标准格式，无需额外转换
            }
            PayloadFormat::AnthropicStyle => {
                self.apply_anthropic_format(target, request)?;
            }
            PayloadFormat::GeminiStyle => {
                self.apply_gemini_format(target, request)?;
            }
            PayloadFormat::CohereNative => {
                // Cohere V2格式处理 - 基本与OpenAI类似，但有一些差异
                // 主要差异在ensure_cohere_format中处理
            }
            PayloadFormat::Custom(_) => {
                // 自定义格式，保持不变
            }
        }
        Ok(())
    }

    /// 应用Anthropic格式转换
    fn apply_anthropic_format(
        &self,
        target: &mut Value,
        _request: &ChatCompletionRequest,
    ) -> MappingResult<()> {
        // 将system prompt从messages中提取到顶层
        if let Some(messages) = target.get_mut("messages") {
            if let Some(msgs) = messages.as_array_mut() {
                let mut system_prompt = None;
                let mut filtered_messages = Vec::new();

                for msg in msgs.drain(..) {
                    if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
                        if let Some(content) = msg.get("content") {
                            system_prompt = Some(content.clone());
                        }
                    } else {
                        filtered_messages.push(msg);
                    }
                }

                *msgs = filtered_messages;

                if let Some(system) = system_prompt {
                    target["system"] = system;
                }
            }
        }
        Ok(())
    }

    /// 应用Gemini格式转换
    fn apply_gemini_format(
        &self,
        target: &mut Value,
        _request: &ChatCompletionRequest,
    ) -> MappingResult<()> {
        // Gemini格式转换逻辑
        if let Some(messages) = target.get_mut("messages") {
            if let Some(msgs) = messages.as_array() {
                let contents: Vec<Value> = msgs.iter().map(|msg| {
                    json!({
                        "role": if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                            "model"
                        } else {
                            "user"
                        },
                        "parts": [{
                            "text": msg.get("content")
                        }]
                    })
                }).collect();

                *target.get_mut("contents").unwrap_or(&mut json!([])) = json!(contents);
                target.as_object_mut().unwrap().remove("messages");
            }
        }
        Ok(())
    }
}
