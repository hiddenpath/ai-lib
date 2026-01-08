//! Manifest-Driven AI Client
//!
//! 这个客户端完全由Manifest驱动，不依赖硬编码的配置

use crate::api::ChatProvider;
use crate::builder::payload::PayloadBuilder;
use crate::manifest::schema::{Capability, Manifest, ModelDefinition, ProviderDefinition};
use crate::streaming::events::StreamingEvent;
use crate::streaming::pipeline::StreamProcessor;
use crate::types::{ChatCompletionRequest, ChatCompletionResponse};
use crate::Result;

use futures::{Stream, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::adapter::dynamic::ConfigDrivenAdapter;
use crate::utils::path_mapper::PathMapper;
use uuid::Uuid;

/// Manifest驱动的AI客户端
pub struct ManifestClient {
    /// Manifest引用
    manifest: Arc<Manifest>,

    /// 当前使用的provider定义
    current_provider: Option<ProviderDefinition>,

    /// 当前使用的model定义
    current_model: Option<ModelDefinition>,

    /// Payload构建器缓存
    payload_builders: HashMap<String, PayloadBuilder>,

    /// Chat provider实例
    chat_provider: Option<Box<dyn ChatProvider + Send + Sync>>,
}

impl ManifestClient {
    /// 从Manifest创建客户端
    pub fn new(manifest: Arc<Manifest>) -> Self {
        Self {
            manifest,
            current_provider: None,
            current_model: None,
            payload_builders: HashMap::new(),
            chat_provider: None,
        }
    }

    /// 选择模型和provider
    pub fn select_model(&mut self, model_id: &str) -> Result<()> {
        // 从manifest中查找模型定义
        let model_def = self.manifest.models.get(model_id).ok_or_else(|| {
            crate::AiLibError::ConfigurationError(format!(
                "Model '{}' not found in manifest",
                model_id
            ))
        })?;

        // 查找对应的provider定义
        let provider_def = self
            .manifest
            .providers
            .get(&model_def.provider)
            .ok_or_else(|| {
                crate::AiLibError::ConfigurationError(format!(
                    "Provider '{}' not found in manifest",
                    model_def.provider
                ))
            })?;

        self.current_model = Some(model_def.clone());
        self.current_provider = Some(provider_def.clone());

        // 创建或更新payload builder（使用manifest映射规则）
        let payload_builder = PayloadBuilder::new(
            convert_mapping_rules(&provider_def.parameter_mappings),
            provider_def.payload_format.clone(),
            provider_def.response_format.clone(),
        );
        self.payload_builders
            .insert(model_id.to_string(), payload_builder);

        // 创建对应的ChatProvider实例
        let adapter =
            ConfigDrivenAdapter::new(self.manifest.clone(), &model_def.provider, model_id)?;
        self.chat_provider = Some(Box::new(adapter));

        Ok(())
    }

    /// 发送chat请求
    pub async fn chat(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        let provider = self.chat_provider.as_ref().ok_or_else(|| {
            crate::AiLibError::ConfigurationError(
                "No provider selected. Call select_model() first.".to_string(),
            )
        })?;

        self.validate_request(&request)?;

        // 直接调用底层provider
        provider.chat(request).await
    }

    /// 发送streaming chat请求
    pub async fn chat_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamingEvent>> + Send + '_>>> {
        let provider = self.chat_provider.as_ref().ok_or_else(|| {
            crate::AiLibError::ConfigurationError(
                "No provider selected. Call select_model() first.".to_string(),
            )
        })?;

        self.validate_request(&request)?;

        // 按 manifest.streaming 配置映射增量事件（算子化 StreamProcessor）
        let streaming_cfg_owned = self.current_provider.as_ref().map(|p| p.streaming.clone());
        let processor = streaming_cfg_owned
            .as_ref()
            .map(|cfg| Arc::new(Mutex::new(StreamProcessor::new(cfg.clone()))));
        let provider_def = self.current_provider.clone();
        let raw_stream = provider.stream(request).await?;
        let mapped = raw_stream.map(move |item| {
            let proc = processor.clone();
            let cfg_ref = streaming_cfg_owned.as_ref();
            let prov_ref = provider_def.as_ref();
            item.map(|chunk| {
                if let Some(p) = proc.as_ref() {
                    if let Ok(value) = serde_json::to_value(&chunk) {
                        if let Ok(mut guard) = p.lock() {
                            if let Some(ev) = guard.process(&value) {
                                return ev;
                            }
                        }
                    }
                }
                map_chunk_to_event(&chunk, cfg_ref, prov_ref)
            })
        });
        Ok(Box::pin(mapped))
    }

    /// 获取当前选择的模型
    pub fn current_model(&self) -> Option<&ModelDefinition> {
        self.current_model.as_ref()
    }

    /// 获取当前选择的provider
    pub fn current_provider(&self) -> Option<&ProviderDefinition> {
        self.current_provider.as_ref()
    }

    /// 检查是否已选择模型
    pub fn has_selected_model(&self) -> bool {
        self.current_model.is_some() && self.current_provider.is_some()
    }

    /// 获取支持的模型列表
    pub fn available_models(&self) -> Vec<String> {
        self.manifest.models.keys().cloned().collect()
    }

    /// 获取支持的provider列表
    pub fn available_providers(&self) -> Vec<String> {
        self.manifest.providers.keys().cloned().collect()
    }

    fn validate_request(&self, request: &ChatCompletionRequest) -> Result<()> {
        let caps = self
            .current_model
            .as_ref()
            .map(|m| &m.capabilities)
            .or_else(|| self.current_provider.as_ref().map(|p| &p.capabilities));

        if let Some(capabilities) = caps {
            if request.stream.unwrap_or(false) && !capabilities.contains(&Capability::Streaming) {
                return Err(crate::AiLibError::UnsupportedFeature(
                    "streaming not supported by selected model/provider".to_string(),
                ));
            }

            if let Some(funcs) = request.functions.as_ref() {
                if !funcs.is_empty() && !capabilities.contains(&Capability::Tools) {
                    return Err(crate::AiLibError::UnsupportedFeature(
                        "tools/function calling not supported by selected model/provider"
                            .to_string(),
                    ));
                }
                if funcs.len() > 1 && !capabilities.contains(&Capability::ParallelTools) {
                    return Err(crate::AiLibError::UnsupportedFeature(
                        "parallel tools not supported by selected model/provider".to_string(),
                    ));
                }
            }
        }

        Ok(())
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

/// 根据 manifest streaming 配置解析 chunk
fn map_chunk_to_event(
    chunk: &crate::api::ChatCompletionChunk,
    streaming_cfg: Option<&crate::manifest::schema::StreamingConfig>,
    provider: Option<&ProviderDefinition>,
) -> StreamingEvent {
    if let Some(cfg) = streaming_cfg {
        if let Ok(value) = serde_json::to_value(chunk) {
            // Operator-style event_map handling
            if let Some(ev) = map_event_with_rules(&value, cfg) {
                return ev;
            }

            // 若有响应映射配置，优先尝试工具调用映射
            if let Some(provider) = provider {
                if let Some(features) = provider.features.as_ref() {
                    if let Some(resp_map) = features.response_mapping.as_ref() {
                        if let Some(tc_map) = resp_map.tool_calls.as_ref() {
                            if let Some(ev) = map_tool_call_with_mapping(&value, tc_map) {
                                return ev;
                            }
                        }
                    }
                }
            }
            // Legacy Fallback Paths: These are preserved for backward compatibility
            // but the operator-style event_map (above) or explicit content_path/tool_call_path
            // in aimanifest.yaml should be preferred.
            let event_format = cfg.event_format.as_deref().unwrap_or("");
            let content_paths: Vec<&str> = match event_format {
                "anthropic_sse" => {
                    vec!["delta.text", "content[0].text", "choices[0].delta.content"]
                }
                "gemini_json" => vec![
                    "candidates[0].content.parts[0].text",
                    "candidates[0].content.parts[0].functionCall.args",
                ],
                "cohere_stream" => vec!["text", "generations[0].text"],
                "responses_api" => vec!["output[0].content[0].text", "choices[0].delta.content"],
                _ => vec!["choices[0].delta.content"],
            };
            let content_paths = cfg
                .content_path
                .as_ref()
                .map(|p| vec![p.as_str()])
                .unwrap_or(content_paths);

            for path in content_paths {
                if let Some(delta) = crate::utils::path_mapper::PathMapper::get_path(&value, path)
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                {
                    let finish_reason = cfg
                        .finish_reason_path
                        .as_ref()
                        .and_then(|fp| {
                            crate::utils::path_mapper::PathMapper::get_path(&value, fp)
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                        })
                        .or_else(|| {
                            crate::utils::path_mapper::PathMapper::get_path(
                                &value,
                                "choices[0].finish_reason",
                            )
                            .and_then(|v| v.as_str().map(|s| s.to_string()))
                        });

                    return StreamingEvent::PartialContentDelta(
                        crate::streaming::events::PartialContentDelta {
                            delta,
                            choice_index: 0,
                            finish_reason,
                            candidate_index: candidate_index(streaming_cfg, &value),
                        },
                    );
                }
            }

            // Legacy Tool Paths: Preserved for compatibility in case manifest configuration is incomplete.
            let tool_paths: Vec<&str> = match event_format {
                "anthropic_sse" => vec!["delta.partial_json", "content[0].tool_calls"],
                "gemini_json" => vec!["candidates[0].content.parts[0].functionCall.args"],
                "cohere_stream" => vec!["tool_calls[0].function.arguments", "tool_calls[0].args"],
                _ => vec!["choices[0].delta.tool_calls[0].function.arguments"],
            };
            let tool_paths = cfg
                .tool_call_path
                .as_ref()
                .map(|p| vec![p.as_str()])
                .unwrap_or(tool_paths);

            for path in tool_paths {
                if let Some(args) = crate::utils::path_mapper::PathMapper::get_path(&value, path)
                    .and_then(|v| {
                        if v.is_string() {
                            v.as_str().map(|s| s.to_string())
                        } else {
                            serde_json::to_string(v).ok()
                        }
                    })
                {
                    let tool_call_id = crate::utils::path_mapper::PathMapper::get_path(
                        &value,
                        "choices[0].delta.tool_calls[0].id",
                    )
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "tool_call".to_string());

                    let function_name = crate::utils::path_mapper::PathMapper::get_path(
                        &value,
                        "choices[0].delta.tool_calls[0].function.name",
                    )
                    .and_then(|v| v.as_str().map(|s| s.to_string()));

                    return StreamingEvent::PartialToolCall(
                        crate::streaming::events::PartialToolCall {
                            tool_call_id,
                            function_name_delta: function_name,
                            arguments_delta: Some(args),
                            tool_name: None,
                            candidate_index: candidate_index(streaming_cfg, &value),
                        },
                    );
                }
            }

            // metadata fallback
            if let Some(meta_path) = cfg.extra_metadata_path.as_ref() {
                if let Some(val) = get_value_by_path(&value, meta_path) {
                    return StreamingEvent::Metadata(crate::streaming::events::StreamMetadata {
                        data: val.clone(),
                        candidate_index: candidate_index(streaming_cfg, &value),
                    });
                }
            }
        }
    }
    StreamingEvent::from(chunk.clone())
}

/// 使用manifest定义的映射规则解析工具调用
fn map_tool_call_with_mapping(
    root: &serde_json::Value,
    tc_map: &crate::manifest::schema::ToolCallsMapping,
) -> Option<StreamingEvent> {
    let val = crate::utils::path_mapper::PathMapper::get_path(root, &tc_map.path)?;

    let items: Vec<&serde_json::Value> = if let Some(arr) = val.as_array() {
        arr.iter().collect()
    } else {
        vec![val]
    };

    for item in items {
        let id = if matches!(
            tc_map.fields.id_strategy,
            Some(crate::manifest::schema::IdStrategy::GenerateUuid)
        ) || tc_map.fields.id == "_generate_uuid"
        {
            Uuid::new_v4().to_string()
        } else {
            crate::utils::path_mapper::PathMapper::get_path(item, &tc_map.fields.id)
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "tool_call".to_string())
        };

        let name = crate::utils::path_mapper::PathMapper::get_path(item, &tc_map.fields.name)
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        let args = crate::utils::path_mapper::PathMapper::get_path(item, &tc_map.fields.args)
            .and_then(|v| {
                if v.is_string() {
                    v.as_str().map(|s| s.to_string())
                } else {
                    serde_json::to_string(v).ok()
                }
            });

        return Some(StreamingEvent::PartialToolCall(
            crate::streaming::events::PartialToolCall {
                tool_call_id: id,
                function_name_delta: name,
                arguments_delta: args,
                tool_name: None,
                candidate_index: None,
            },
        ));
    }

    None
}

/// 使用算子化 event_map 解析流事件
fn map_event_with_rules(
    root: &serde_json::Value,
    cfg: &crate::manifest::schema::StreamingConfig,
) -> Option<StreamingEvent> {
    if cfg.event_map.is_empty() {
        return None;
    }

    for rule in &cfg.event_map {
        if evaluate_match(&rule.matcher, root) {
            if let Some(ev) = build_event_from_rule(rule, root, cfg) {
                return Some(ev);
            }
        }
    }

    // fallback: emit Metadata if配置提供了额外元数据路径
    if let Some(meta_path) = cfg.extra_metadata_path.as_ref() {
        if let Some(val) = get_value_by_path(root, meta_path) {
            return Some(StreamingEvent::Metadata(
                crate::streaming::events::StreamMetadata {
                    data: val.clone(),
                    candidate_index: candidate_index(Some(cfg), root),
                },
            ));
        }
    }

    None
}

fn build_event_from_rule(
    rule: &crate::manifest::schema::StreamingEventRule,
    root: &serde_json::Value,
    cfg: &crate::manifest::schema::StreamingConfig,
) -> Option<StreamingEvent> {
    match rule.emit.as_str() {
        "PartialContentDelta" => {
            let content_path = rule.fields.get("content")?;
            let delta = get_string_by_path(root, content_path)?;
            let finish_reason = rule
                .fields
                .get("finish_reason")
                .and_then(|p| get_string_by_path(root, p));
            let choice_index = rule
                .fields
                .get("choice_index")
                .and_then(|p| get_string_by_path(root, p))
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0);
            Some(StreamingEvent::PartialContentDelta(
                crate::streaming::events::PartialContentDelta {
                    delta,
                    choice_index,
                    finish_reason,
                    candidate_index: candidate_index(Some(cfg), root),
                },
            ))
        }
        "PartialToolCall" => {
            let args_path = rule
                .fields
                .get("arguments")
                .or_else(|| rule.fields.get("args"))
                .or_else(|| rule.fields.get("partial_json"))?;
            let arguments_delta = get_string_by_path(root, args_path);
            let id_path = rule.fields.get("tool_call_id");
            let tool_call_id = if let Some(idp) = id_path {
                if idp == "_generate_uuid" {
                    Uuid::new_v4().to_string()
                } else {
                    get_string_by_path(root, idp).unwrap_or_else(|| "tool_call".to_string())
                }
            } else {
                "tool_call".to_string()
            };
            let function_name = rule
                .fields
                .get("function_name")
                .and_then(|p| get_string_by_path(root, p));
            Some(StreamingEvent::PartialToolCall(
                crate::streaming::events::PartialToolCall {
                    tool_call_id,
                    function_name_delta: function_name,
                    arguments_delta,
                    tool_name: None,
                    candidate_index: candidate_index(Some(cfg), root),
                },
            ))
        }
        "ThinkingDelta" => {
            let thinking_path = rule.fields.get("thinking")?;
            let thinking = get_string_by_path(root, thinking_path)?;
            Some(StreamingEvent::ThinkingDelta(
                crate::streaming::events::ThinkingDelta {
                    thinking,
                    signature: None,
                },
            ))
        }
        "Metadata" => {
            let data_path = rule.fields.get("data")?;
            let data = get_value_by_path(root, data_path)?.clone();
            Some(StreamingEvent::Metadata(
                crate::streaming::events::StreamMetadata {
                    data,
                    candidate_index: candidate_index(Some(cfg), root),
                },
            ))
        }
        "StreamEnd" | "Finish" => Some(StreamingEvent::StreamEnd),
        _ => None,
    }
}

fn evaluate_match(expr: &str, root: &serde_json::Value) -> bool {
    let or_parts: Vec<&str> = expr.split("||").collect();
    for or_part in or_parts {
        let mut ok = true;
        let and_parts: Vec<&str> = or_part.split("&&").collect();
        for part in and_parts {
            let cond = part.trim();
            if cond.is_empty() {
                continue;
            }
            if cond.starts_with("exists(") && cond.ends_with(')') {
                let path = cond.trim_start_matches("exists(").trim_end_matches(')');
                if get_value_by_path(root, path).is_none() {
                    ok = false;
                    break;
                }
                continue;
            }
            if let Some(idx) = cond.find(" in ") {
                let (path, rest) = cond.split_at(idx);
                let path = path.trim();
                let list_str = rest.trim_start_matches(" in ").trim();
                let list_str = list_str.trim_start_matches('[').trim_end_matches(']');
                let values: Vec<String> = list_str
                    .split(',')
                    .filter_map(|v| v.trim().trim_matches('\'').trim_matches('"').parse().ok())
                    .collect();
                let actual = get_string_by_path(root, path);
                if !actual.map(|a| values.contains(&a)).unwrap_or(false) {
                    ok = false;
                    break;
                }
                continue;
            }
            if let Some(idx) = cond.find("==") {
                let (path, value_part) = cond.split_at(idx);
                let path = path.trim();
                let target = value_part
                    .trim_start_matches("==")
                    .trim()
                    .trim_matches('\'')
                    .trim_matches('"');
                let actual = get_string_by_path(root, path);
                if actual.as_deref() != Some(target) {
                    ok = false;
                    break;
                }
                continue;
            }
        }
        if ok {
            return true;
        }
    }
    false
}

fn get_string_by_path(root: &serde_json::Value, path: &str) -> Option<String> {
    get_value_by_path(root, path).and_then(|v| {
        if v.is_string() {
            v.as_str().map(|s| s.to_string())
        } else {
            serde_json::to_string(v).ok()
        }
    })
}

fn get_value_by_path<'a>(root: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let normalized = path.trim().trim_start_matches("$.").to_string();
    PathMapper::get_path(root, &normalized)
}

fn candidate_index(
    cfg: Option<&crate::manifest::schema::StreamingConfig>,
    root: &serde_json::Value,
) -> Option<usize> {
    cfg.and_then(|c| {
        c.candidate
            .as_ref()
            .and_then(|cand| cand.candidate_id_path.as_ref())
            .and_then(|p| get_string_by_path(root, p))
            .and_then(|s| s.parse::<usize>().ok())
    })
}

impl Default for ManifestClient {
    fn default() -> Self {
        // 创建空的manifest用于默认构造
        let manifest = Arc::new(Manifest {
            version: "1.1".to_string(),
            metadata: crate::manifest::schema::ManifestMetadata {
                description: Some("Default empty manifest".to_string()),
                authors: vec![],
                last_updated: None,
            },
            standard_schema: crate::manifest::schema::StandardSchema {
                parameters: std::collections::HashMap::new(),
                tools: Default::default(),
                response_format: Default::default(),
                multimodal: Default::default(),
                agentic_loop: None,
                streaming_events: None,
            },
            providers: HashMap::new(),
            models: HashMap::new(),
        });

        Self::new(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::schema::StandardSchema;
    use std::collections::HashMap;

    #[test]
    fn test_manifest_client_creation() {
        let manifest = Arc::new(Manifest {
            version: "1.1".to_string(),
            metadata: crate::manifest::schema::ManifestMetadata {
                description: Some("Test manifest".to_string()),
                authors: vec![],
                last_updated: None,
            },
            standard_schema: StandardSchema {
                parameters: HashMap::new(),
                multimodal: Default::default(),
                response_format: Default::default(),
                tools: Default::default(),
                agentic_loop: Default::default(),
                streaming_events: Default::default(),
            },
            providers: HashMap::new(),
            models: HashMap::new(),
        });

        let client = ManifestClient::new(manifest);
        assert!(!client.has_selected_model());
        assert!(client.available_models().is_empty());
        assert!(client.available_providers().is_empty());
    }

    #[test]
    fn test_model_selection_not_found() {
        let manifest = Arc::new(Manifest {
            version: "1.1".to_string(),
            metadata: crate::manifest::schema::ManifestMetadata {
                description: Some("Test manifest".to_string()),
                authors: vec![],
                last_updated: None,
            },
            standard_schema: StandardSchema {
                parameters: HashMap::new(),
                multimodal: Default::default(),
                response_format: Default::default(),
                tools: Default::default(),
                agentic_loop: Default::default(),
                streaming_events: Default::default(),
            },
            providers: HashMap::new(),
            models: HashMap::new(),
        });

        let mut client = ManifestClient::new(manifest);
        let result = client.select_model("nonexistent-model");
        assert!(result.is_err());
        assert!(!client.has_selected_model());
    }
}
