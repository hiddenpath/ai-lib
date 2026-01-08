//! 基于配置的动态AI适配器
//!
//! 这个模块实现了完全由YAML配置驱动的AI适配器，
//! 能够根据配置动态处理不同提供商的API异构性。
//!
//! Phase 2: 实现完整的ConfigDrivenAdapter功能

use crate::api::{ChatCompletionChunk, ChatProvider, ModelInfo};
use crate::builder::payload::PayloadBuilder;
use crate::manifest::schema::{Manifest, ModelDefinition, ProviderDefinition};
use crate::metrics::{Metrics, NoopMetrics};
use crate::streaming::pipeline::StreamProcessor;
use crate::utils::template::TemplateEngine;
use crate::{AiLibError, ChatCompletionRequest, ChatCompletionResponse, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// 基于配置的动态适配器
pub struct ConfigDrivenAdapter {
    /// Manifest引用
    _manifest: Arc<Manifest>,

    /// 提供商定义
    provider_def: ProviderDefinition,

    /// 模型定义
    model_def: ModelDefinition,

    /// HTTP客户端
    http_client: Client,

    /// Payload构建器
    payload_builder: PayloadBuilder,

    /// Metrics指标收集器
    metrics: Arc<dyn Metrics>,
}

impl ConfigDrivenAdapter {
    /// Create adapter from raw definitions (bypassing Manifest lookup)
    pub fn new_raw(
        manifest: Arc<Manifest>, // can be empty/dummy if not used for lookup
        provider_def: ProviderDefinition,
        model_def: ModelDefinition,
    ) -> Result<Self> {
        let parameter_mappings = convert_mapping_rules(&provider_def.parameter_mappings);
        let payload_builder = PayloadBuilder::new(
            parameter_mappings,
            provider_def.payload_format.clone(),
            provider_def.response_format.clone(),
        );

        let http_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| {
                AiLibError::ConfigurationError(format!("Failed to build http client: {}", e))
            })?;

        Ok(Self {
            _manifest: manifest,
            provider_def,
            model_def,
            http_client,
            payload_builder,
            metrics: Arc::new(NoopMetrics::new()),
        })
    }

    /// 创建新的动态适配器 (Standard Manifest Lookup)
    pub fn new(manifest: Arc<Manifest>, provider_id: &str, model_id: &str) -> Result<Self> {
        let provider_def = manifest
            .providers
            .get(provider_id)
            .ok_or_else(|| {
                AiLibError::ConfigurationError(format!(
                    "Provider '{}' not found in manifest",
                    provider_id
                ))
            })?
            .clone();

        let model_def = manifest
            .models
            .get(model_id)
            .ok_or_else(|| {
                AiLibError::ConfigurationError(format!(
                    "Model '{}' not found in manifest",
                    model_id
                ))
            })?
            .clone();

        // 验证模型属于正确的提供商
        if model_def.provider != provider_id {
            return Err(AiLibError::ConfigurationError(format!(
                "Model '{}' does not belong to provider '{}'",
                model_id, provider_id
            )));
        }

        Self::new_raw(manifest, provider_def, model_def)
    }

    /// Set metrics implementation
    pub fn with_metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
        self.metrics = metrics;
        self
    }

    /// 解析provider响应为标准格式
    fn parse_response(&self, response_json: &Value) -> Result<ChatCompletionResponse> {
        // 使用manifest配置的response_paths提取数据
        let content = self
            .extract_path_value(response_json, "content")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

        let tool_calls = self.extract_tool_calls(response_json);

        let usage = self.extract_usage(response_json);

        let finish_reason = self
            .extract_path_value(response_json, "finish_reason")
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        // 构建标准响应
        Ok(ChatCompletionResponse {
            id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: self.model_def.model_id.clone(),
            choices: vec![crate::types::Choice {
                index: 0,
                message: crate::types::Message {
                    role: crate::types::Role::Assistant,
                    content: crate::Content::Text(content),
                    function_call: tool_calls.first().cloned(),
                },
                finish_reason,
            }],
            usage: usage.unwrap_or_default(),
            usage_status: crate::types::UsageStatus::Finalized,
        })
    }

    /// 使用manifest的response_paths提取值
    fn extract_path_value(&self, json: &Value, key: &str) -> Option<Value> {
        use crate::utils::path_mapper::PathMapper;

        self.provider_def
            .response_paths
            .get(key)
            .and_then(|jp| PathMapper::get_path(json, &jp.0).cloned())
    }

    /// 提取工具调用
    fn extract_tool_calls(&self, json: &Value) -> Vec<crate::types::function_call::FunctionCall> {
        use crate::utils::path_mapper::PathMapper;

        // 从配置的response_paths中获取tool_calls路径
        if let Some(tool_calls_path) = self.provider_def.response_paths.get("tool_calls") {
            let path = &tool_calls_path.0;

            // 使用PathMapper获取工具调用数组
            if let Some(tool_calls_value) = PathMapper::get_path(json, path) {
                if let Some(tool_calls_array) = tool_calls_value.as_array() {
                    let mut result = Vec::new();

                    for tool_call_json in tool_calls_array {
                        // 解析单个工具调用
                        // OpenAI格式: {"id": "...", "function": {"name": "...", "arguments": "..."}}
                        if let Some(function) = tool_call_json.get("function") {
                            if let Some(name) = function.get("name").and_then(|v| v.as_str()) {
                                // arguments可能是字符串（JSON字符串）或对象
                                let arguments = if let Some(arg_str) =
                                    function.get("arguments").and_then(|v| v.as_str())
                                {
                                    // 尝试解析JSON字符串
                                    serde_json::from_str(arg_str).ok()
                                } else if let Some(arg_obj) = function.get("arguments") {
                                    Some(arg_obj.clone())
                                } else {
                                    None
                                };

                                result.push(crate::types::function_call::FunctionCall {
                                    name: name.to_string(),
                                    arguments,
                                });
                            }
                        }
                    }

                    return result;
                }
            }
        }

        vec![]
    }

    /// 提取使用统计
    fn extract_usage(&self, json: &Value) -> Option<crate::types::Usage> {
        // 优先使用配置的usage路径
        if let Some(v) = self.extract_path_value(json, "usage") {
            if let Some(obj) = v.as_object() {
                let prompt = obj
                    .get("prompt_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let completion = obj
                    .get("completion_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let total = obj
                    .get("total_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                return Some(crate::types::Usage {
                    prompt_tokens: prompt as u32,
                    completion_tokens: completion as u32,
                    total_tokens: total as u32,
                });
            }
        }
        None
    }

    /// 解析base URL（支持模板替换）
    fn resolve_base_url(&self) -> Result<String> {
        // 优先使用base_url_template（如果存在）
        if let Some(ref template) = self.provider_def.base_url_template {
            let vars = self
                .provider_def
                .connection_vars
                .as_ref()
                .cloned()
                .unwrap_or_default();

            TemplateEngine::replace(template, &vars).map_err(|e| {
                AiLibError::ConfigurationError(format!("Failed to resolve URL template: {}", e))
            })
        } else if let Some(ref base_url) = self.provider_def.base_url {
            Ok(base_url.clone())
        } else {
            Err(AiLibError::ConfigurationError(
                "Neither base_url nor base_url_template is set".to_string(),
            ))
        }
    }

    /// 统一的HTTP请求发送辅助函数
    async fn send_request(
        &self,
        url: &str,
        payload: &Value,
        stream: bool,
    ) -> Result<reqwest::Response> {
        let mut builder = self.http_client.post(url);

        // 设置Header
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        // 认证处理
        match &self.provider_def.auth {
            crate::manifest::schema::AuthConfig::Bearer { token_env, .. } => {
                if let Ok(token) = std::env::var(token_env) {
                    builder = builder.bearer_auth(token);
                }
            }
            crate::manifest::schema::AuthConfig::ApiKey {
                key_env,
                header_name,
            } => {
                if let Ok(key) = std::env::var(key_env) {
                    if let Some(h_name) = header_name {
                        if let Ok(h_val) =
                            reqwest::header::HeaderName::from_bytes(h_name.as_bytes())
                        {
                            headers.insert(h_val, key.parse().unwrap());
                        }
                    } else {
                        // Default to Bearer if no header name specified (fallback) or generic Auth header
                        builder = builder.bearer_auth(key);
                    }
                }
            }
            _ => {} // TODO: Implement other auth types
        }

        if stream {
            headers.insert("Accept", "text/event-stream".parse().unwrap());
        }

        builder = builder.headers(headers).json(payload);

        builder
            .send()
            .await
            .map_err(|e| AiLibError::NetworkError(format!("Request failed: {}", e)))
    }
}

/// 将manifest层的MappingRule转换为运行时MappingRule
fn convert_mapping_rules(
    manifest_rules: &HashMap<String, crate::manifest::schema::MappingRule>,
) -> HashMap<String, crate::mapping::rules::MappingRule> {
    manifest_rules
        .iter()
        .map(|(k, v)| (k.clone(), convert_rule(v)))
        .collect()
}

fn convert_rule(rule: &crate::manifest::schema::MappingRule) -> crate::mapping::rules::MappingRule {
    use crate::mapping::rules as r;
    match rule {
        crate::manifest::schema::MappingRule::Direct(p) => r::MappingRule::Direct(p.clone()),
        crate::manifest::schema::MappingRule::Conditional(conds) => {
            let converted = conds
                .iter()
                .map(|c| r::ConditionalMapping {
                    condition: c.condition.clone(),
                    target_path: c.target_path.clone(),
                    transform: c.transform.as_ref().map(convert_transform),
                })
                .collect();
            r::MappingRule::Conditional(converted)
        }
        crate::manifest::schema::MappingRule::Transform(t) => {
            r::MappingRule::Transform(convert_transform(t))
        }
    }
}

fn convert_transform(
    t: &crate::manifest::schema::ParameterTransform,
) -> crate::mapping::rules::ParameterTransform {
    use crate::mapping::rules::{ParameterTransform, TransformType};
    let transform_type = match t.transform_type {
        crate::manifest::schema::TransformType::Scale => TransformType::Scale,
        crate::manifest::schema::TransformType::Format => TransformType::Format,
        crate::manifest::schema::TransformType::EnumMap => TransformType::EnumMap,
        crate::manifest::schema::TransformType::Custom => TransformType::Custom,
    };

    ParameterTransform {
        transform_type,
        target_path: Some(t.target_path.clone()),
        params: t.params.clone(),
    }
}

#[async_trait]
impl ChatProvider for ConfigDrivenAdapter {
    async fn chat(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        // Metrics: Start Timer
        let timer = self.metrics.start_timer("chat_duration").await;
        self.metrics.incr_counter("chat_requests", 1).await;

        // 使用PayloadBuilder构建请求payload
        let payload = self.payload_builder.build_payload(&request)?;

        // 构建完整的请求URL（支持模板）
        let base_url = self.resolve_base_url()?;
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        // 发送HTTP请求
        let response = self.send_request(&url, &payload, false).await?;

        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AiLibError::ProviderError(format!(
                "Provider returned error {}: {}",
                status, error_text
            )));
        }

        // 解析响应
        let response_json: Value = response
            .json()
            .await
            .map_err(|e| AiLibError::ParseError(format!("Failed to parse response: {}", e)))?;

        // 转换为标准响应格式
        let result = self.parse_response(&response_json);

        // Metrics: Stop Timer
        if let Some(t) = timer {
            t.stop();
        }

        result
    }

    fn name(&self) -> &str {
        &self.model_def.model_id
    }

    async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo> {
        if model_id != self.model_def.model_id {
            return Err(AiLibError::InvalidRequest(format!(
                "Model '{}' not supported by this adapter",
                model_id
            )));
        }

        Ok(ModelInfo {
            id: self.model_def.model_id.clone(),
            object: "model".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            owned_by: self.provider_def.version.clone(),
            permission: vec![], // TODO: 添加权限信息
        })
    }

    async fn stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send + Unpin>> {
        // Metrics: Start Timer
        self.metrics.incr_counter("stream_requests", 1).await;

        // 1. 构建请求
        let mut payload = self.payload_builder.build_payload(&request)?;
        // 强制开启stream
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(true));
        }

        let base_url = self.resolve_base_url()?;
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        // 2. 发送请求 (Stream模式)
        let response = self.send_request(&url, &payload, true).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AiLibError::ProviderError(format!(
                "Provider returned stream error {}: {}",
                status, error_text
            )));
        }

        // 3. 构建Stream处理流 (使用 Manifest 配置驱动的算子化解析器)
        let mut byte_stream = response.bytes_stream();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Extract streaming config from manifest
        let streaming_cfg = self.provider_def.streaming.clone();
        let _decoder_cfg = streaming_cfg.decoder.clone();
        let _event_format = streaming_cfg.event_format.clone();

        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let _processor = StreamProcessor::new(streaming_cfg);

            #[cfg(feature = "unified_sse")]
            let parser = crate::sse::parser::ConfigDrivenParser::from_config(
                _decoder_cfg.as_ref(),
                _event_format.as_deref(),
            );

            #[cfg(feature = "unified_sse")]
            let delimiter = _decoder_cfg
                .as_ref()
                .and_then(|d| d.delimiter.clone())
                .unwrap_or_else(|| "\n\n".to_string());

            while let Some(item) = byte_stream.next().await {
                match item {
                    Ok(bytes) => {
                        buffer.extend_from_slice(&bytes);

                        #[cfg(feature = "unified_sse")]
                        {
                            // Use config-driven parser with manifest delimiter
                            while let Some(event_end) =
                                crate::sse::parser::find_event_boundary_with_delim(
                                    &buffer,
                                    Some(&delimiter),
                                )
                            {
                                let event_bytes = buffer.drain(..event_end).collect::<Vec<_>>();
                                if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
                                    // 1. Parse raw frame structure
                                    if let Some(json) = parser.parse_to_json(event_text) {
                                        // 2. Drive through Operator Pipeline
                                        if let Some(event) = processor.process(&json) {
                                            // 3. Convert to legacy chunk (for API compatibility)
                                            match ChatCompletionChunk::try_from(event) {
                                                Ok(chunk) => {
                                                    if tx.send(Ok(chunk)).is_err() {
                                                        return;
                                                    }
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(Err(e));
                                                    return;
                                                }
                                            }
                                        }
                                    } else if parser.is_done(event_text) {
                                        return; // [DONE]
                                    }
                                }
                            }
                        }

                        #[cfg(not(feature = "unified_sse"))]
                        {
                            let _ = tx.send(Err(AiLibError::UnsupportedFeature("Unified SSE feature required for ConfigDrivenAdapter streaming. Enable 'unified_sse' feature.".to_string())));
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(AiLibError::NetworkError(format!(
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

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![self.model_def.model_id.clone()])
    }
}
