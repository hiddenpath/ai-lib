//! Streaming operator-based pipeline
//!
//! 通过 Manifest StreamingConfig 的算子化描述，对增量 JSON 帧执行
//! 过滤、累积、多候选标记以及事件映射，生成统一的 StreamingEvent。

use crate::manifest::schema::{StreamingCandidateConfig, StreamingConfig, StreamingEventRule};
use crate::streaming::events::StreamingEvent;
use crate::types::Usage;
use crate::utils::path_mapper::PathMapper;
use serde_json::Value;
use uuid::Uuid;

/// 流式事件处理器（无厂商分支，完全由 Manifest 驱动）
pub struct StreamProcessor {
    cfg: StreamingConfig,
    /// 分片累积缓冲（用于 stateful_tool_parsing）
    accumulated: String,
}

impl StreamProcessor {
    pub fn new(cfg: StreamingConfig) -> Self {
        Self {
            cfg,
            accumulated: String::new(),
        }
    }

    /// 处理单帧 JSON 值，返回可选 StreamingEvent
    pub fn process(&mut self, root: &Value) -> Option<StreamingEvent> {
        // 帧过滤
        if let Some(sel) = self.cfg.frame_selector.as_ref() {
            if !evaluate_match(sel, root) {
                return None;
            }
        }

        // 累积器：收集分片
        if let Some(acc) = self.cfg.accumulator.as_ref() {
            if let Some(key) = acc.key_path.as_ref() {
                if let Some(fragment) = get_string_by_path(root, key) {
                    self.accumulated.push_str(&fragment);
                }
            }
        }

        // 事件映射规则（优先于 stop_condition）
        for rule in &self.cfg.event_map {
            if evaluate_match(&rule.matcher, root) {
                if let Some(ev) =
                    build_event_from_rule(rule, root, &self.cfg, &mut self.accumulated)
                {
                    return Some(ev);
                }
            }
        }

        // stop_condition：满足则结束流（在 event_map 之后检查）
        if let Some(stop) = self.cfg.stop_condition.as_ref() {
            if evaluate_match(stop, root) {
                return Some(StreamingEvent::StreamEnd);
            }
        }

        // flush_on：若匹配则把已累积的参数作为一次工具增量输出
        if let Some(acc) = self.cfg.accumulator.as_ref() {
            if acc
                .flush_on
                .as_ref()
                .map(|c| evaluate_match(c, root))
                .unwrap_or(false)
            {
                if !self.accumulated.is_empty() {
                    let args = self.accumulated.clone();
                    self.accumulated.clear();
                    return Some(StreamingEvent::PartialToolCall(
                        crate::streaming::events::PartialToolCall {
                            tool_call_id: "tool_call".to_string(),
                            function_name_delta: None,
                            arguments_delta: Some(args),
                            tool_name: None,
                            candidate_index: candidate_index(self.cfg.candidate.as_ref(), root),
                        },
                    ));
                }
            }
        }

        // extra_metadata_path 支持
        if let Some(meta_path) = self.cfg.extra_metadata_path.as_ref() {
            if let Some(val) = get_value_by_path(root, meta_path) {
                return Some(StreamingEvent::Metadata(
                    crate::streaming::events::StreamMetadata {
                        data: val.clone(),
                        candidate_index: candidate_index(self.cfg.candidate.as_ref(), root),
                    },
                ));
            }
        }

        None
    }
}

fn build_event_from_rule(
    rule: &StreamingEventRule,
    root: &Value,
    cfg: &StreamingConfig,
    accumulated: &mut String,
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
                    candidate_index: candidate_index(cfg.candidate.as_ref(), root),
                },
            ))
        }
        "PartialToolCall" => {
            let args_path = rule
                .fields
                .get("arguments")
                .or_else(|| rule.fields.get("args"))
                .or_else(|| rule.fields.get("partial_json"));
            let mut arguments_delta = args_path.and_then(|p| get_string_by_path(root, p));
            if arguments_delta.is_none() && !accumulated.is_empty() {
                arguments_delta = Some(accumulated.clone());
                accumulated.clear();
            }
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
                    candidate_index: candidate_index(cfg.candidate.as_ref(), root),
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
        "ToolCallStarted" => {
            let tool_call_id = rule
                .fields
                .get("tool_call_id")
                .and_then(|p| get_string_by_path(root, p))
                .unwrap_or_else(|| "tool_call".to_string());
            let tool_name = rule
                .fields
                .get("tool_name")
                .and_then(|p| get_string_by_path(root, p))
                .unwrap_or_else(|| "tool".to_string());
            let initial_arguments = rule
                .fields
                .get("arguments")
                .and_then(|p| get_string_by_path(root, p));
            Some(StreamingEvent::ToolCallStarted(
                crate::streaming::events::ToolCallStarted {
                    tool_call_id,
                    tool_name,
                    initial_arguments,
                },
            ))
        }
        "ToolCallEnded" => {
            let tool_call_id = rule
                .fields
                .get("tool_call_id")
                .and_then(|p| get_string_by_path(root, p))
                .unwrap_or_else(|| "tool_call".to_string());
            let result = rule
                .fields
                .get("result")
                .and_then(|p| get_string_by_path(root, p));
            Some(StreamingEvent::ToolCallEnded(
                crate::streaming::events::ToolCallEnded {
                    tool_call_id,
                    result,
                },
            ))
        }
        "Metadata" => {
            let data_path = rule.fields.get("data")?;
            let data = get_value_by_path(root, data_path)?.clone();
            Some(StreamingEvent::Metadata(
                crate::streaming::events::StreamMetadata {
                    data,
                    candidate_index: candidate_index(cfg.candidate.as_ref(), root),
                },
            ))
        }
        "Finish" => {
            let finish_reason = rule
                .fields
                .get("finish_reason")
                .and_then(|p| get_string_by_path(root, p));
            let usage = rule
                .fields
                .get("usage")
                .and_then(|p| get_value_by_path(root, p))
                .and_then(|v| serde_json::from_value::<Usage>(v.clone()).ok());
            Some(StreamingEvent::FinalCandidate(
                crate::streaming::events::FinalCandidate {
                    choices: vec![],
                    usage,
                    finish_reason,
                    model: None,
                },
            ))
        }
        "StreamEnd" => Some(StreamingEvent::StreamEnd),
        _ => None,
    }
}

fn get_string_by_path(root: &Value, path: &str) -> Option<String> {
    get_value_by_path(root, path).and_then(|v| {
        if v.is_string() {
            v.as_str().map(|s| s.to_string())
        } else {
            serde_json::to_string(v).ok()
        }
    })
}

fn get_value_by_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let normalized = path.trim().trim_start_matches("$.").to_string();
    PathMapper::get_path(root, &normalized)
}

fn candidate_index(
    candidate_cfg: Option<&StreamingCandidateConfig>,
    root: &Value,
) -> Option<usize> {
    candidate_cfg
        .and_then(|c| c.candidate_id_path.as_ref())
        .and_then(|p| get_string_by_path(root, p))
        .and_then(|s| s.parse::<usize>().ok())
}

fn evaluate_match(expr: &str, root: &Value) -> bool {
    let or_parts: Vec<&str> = expr.split("||").collect();
    for or_part in or_parts {
        let mut ok = true;
        let and_parts: Vec<&str> = or_part.split("&&").collect();
        for part in and_parts {
            let cond = part.trim();
            if cond.is_empty() {
                continue;
            }
            // exists() check
            if cond.starts_with("exists(") && cond.ends_with(')') {
                let path = cond.trim_start_matches("exists(").trim_end_matches(')');
                if get_value_by_path(root, path).is_none() {
                    ok = false;
                    break;
                }
                continue;
            }
            // "in" list check
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
            // "!= null" check (value is not null)
            if let Some(idx) = cond.find("!= null") {
                let path = cond[..idx].trim();
                let val = get_value_by_path(root, path);
                if val.is_none() || val == Some(&Value::Null) {
                    ok = false;
                    break;
                }
                continue;
            }
            // "== null" check (value is null)
            if let Some(idx) = cond.find("== null") {
                let path = cond[..idx].trim();
                let val = get_value_by_path(root, path);
                if val.is_some() && val != Some(&Value::Null) {
                    ok = false;
                    break;
                }
                continue;
            }
            // "==" equality check (must come after null checks)
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
            // "!=" inequality check
            if let Some(idx) = cond.find("!=") {
                let (path, value_part) = cond.split_at(idx);
                let path = path.trim();
                let target = value_part
                    .trim_start_matches("!=")
                    .trim()
                    .trim_matches('\'')
                    .trim_matches('"');
                let actual = get_string_by_path(root, path);
                if actual.as_deref() == Some(target) {
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
