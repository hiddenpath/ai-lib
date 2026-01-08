//! Streaming事件定义
//!
//! 定义统一的streaming事件模型，支持各种provider的streaming格式

use crate::api::ChatCompletionChunk;
use crate::types::{Choice, Usage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 统一的Streaming事件枚举
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "event_type", content = "data")]
pub enum StreamingEvent {
    /// 部分内容增量
    PartialContentDelta(PartialContentDelta),

    /// 思考过程增量 (Anthropic/Claude)
    ThinkingDelta(ThinkingDelta),

    /// 部分工具调用
    PartialToolCall(PartialToolCall),

    /// 工具调用开始
    ToolCallStarted(ToolCallStarted),

    /// 工具调用结束
    ToolCallEnded(ToolCallEnded),

    /// 元数据事件（如 citations、额外payload）
    Metadata(StreamMetadata),

    /// 引用块 (Citation chunk)
    CitationChunk(CitationChunk),

    /// 最终候选结果
    FinalCandidate(FinalCandidate),

    /// 错误事件
    Error(StreamError),

    /// 流结束
    StreamEnd,
}

/// 部分内容增量事件
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PartialContentDelta {
    /// 增量内容
    pub delta: String,

    /// 选择索引
    pub choice_index: usize,

    /// 是否完成
    pub finish_reason: Option<String>,

    /// 候选索引（用于多候选fan-out）
    #[serde(default)]
    pub candidate_index: Option<usize>,
}

/// 思考过程增量事件
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ThinkingDelta {
    /// 思考内容增量
    pub thinking: String,

    /// 签名 (如果支持)
    pub signature: Option<String>,
}

/// 部分工具调用事件
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PartialToolCall {
    /// 工具调用ID
    pub tool_call_id: String,

    /// 函数名增量
    pub function_name_delta: Option<String>,

    /// 参数增量 (JSON字符串)
    pub arguments_delta: Option<String>,

    /// 工具名称
    pub tool_name: Option<String>,

    /// 候选索引（用于多候选fan-out）
    #[serde(default)]
    pub candidate_index: Option<usize>,
}

/// 工具调用开始事件
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ToolCallStarted {
    /// 工具调用ID
    pub tool_call_id: String,

    /// 工具名称
    pub tool_name: String,

    /// 初始参数
    pub initial_arguments: Option<String>,
}

/// 工具调用结束事件
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ToolCallEnded {
    /// 工具调用ID
    pub tool_call_id: String,

    /// 最终结果
    pub result: Option<String>,
}

/// 元数据事件
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct StreamMetadata {
    /// 元数据主体
    pub data: serde_json::Value,

    /// 候选索引（用于多候选fan-out）
    #[serde(default)]
    pub candidate_index: Option<usize>,
}

/// 引用块事件
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CitationChunk {
    /// 引用索引
    pub index: usize,

    /// 引用内容
    pub citation: CitationData,
}

/// 引用数据
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CitationData {
    /// 引用ID
    pub id: Option<String>,

    /// 文档标题
    pub title: Option<String>,

    /// 引用片段
    pub snippet: String,

    /// 来源URL
    pub url: Option<String>,
}

/// 最终候选结果事件
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct FinalCandidate {
    /// 选择列表
    pub choices: Vec<Choice>,

    /// 使用统计
    pub usage: Option<Usage>,

    /// 完成原因
    pub finish_reason: Option<String>,

    /// 模型名称
    pub model: Option<String>,
}

/// 流错误事件
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct StreamError {
    /// 错误消息
    pub message: String,

    /// 错误类型
    pub error_type: String,

    /// 错误代码
    pub code: Option<i32>,
}

/// Streaming事件类型枚举 (用于快速判断)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StreamingEventType {
    PartialContentDelta,
    ThinkingDelta,
    PartialToolCall,
    ToolCallStarted,
    ToolCallEnded,
    Metadata,
    CitationChunk,
    FinalCandidate,
    Error,
    StreamEnd,
}

impl StreamingEvent {
    /// 获取事件类型
    pub fn event_type(&self) -> StreamingEventType {
        match self {
            StreamingEvent::PartialContentDelta(_) => StreamingEventType::PartialContentDelta,
            StreamingEvent::ThinkingDelta(_) => StreamingEventType::ThinkingDelta,
            StreamingEvent::PartialToolCall(_) => StreamingEventType::PartialToolCall,
            StreamingEvent::ToolCallStarted(_) => StreamingEventType::ToolCallStarted,
            StreamingEvent::ToolCallEnded(_) => StreamingEventType::ToolCallEnded,
            StreamingEvent::Metadata(_) => StreamingEventType::Metadata,
            StreamingEvent::CitationChunk(_) => StreamingEventType::CitationChunk,
            StreamingEvent::FinalCandidate(_) => StreamingEventType::FinalCandidate,
            StreamingEvent::Error(_) => StreamingEventType::Error,
            StreamingEvent::StreamEnd => StreamingEventType::StreamEnd,
        }
    }

    /// 检查是否为内容相关事件
    pub fn is_content_event(&self) -> bool {
        matches!(
            self.event_type(),
            StreamingEventType::PartialContentDelta | StreamingEventType::ThinkingDelta
        )
    }

    /// 检查是否为工具相关事件
    pub fn is_tool_event(&self) -> bool {
        matches!(
            self.event_type(),
            StreamingEventType::PartialToolCall
                | StreamingEventType::ToolCallStarted
                | StreamingEventType::ToolCallEnded
        )
    }

    /// 检查是否为终止事件
    pub fn is_terminal_event(&self) -> bool {
        matches!(
            self.event_type(),
            StreamingEventType::FinalCandidate
                | StreamingEventType::Error
                | StreamingEventType::StreamEnd
        )
    }
}

impl Default for StreamingEvent {
    fn default() -> Self {
        StreamingEvent::StreamEnd
    }
}

/// 从ChatCompletionChunk转换为StreamingEvent的辅助函数
impl From<ChatCompletionChunk> for StreamingEvent {
    fn from(chunk: ChatCompletionChunk) -> Self {
        if let Some(choice) = chunk.choices.first() {
            if let Some(delta) = Some(&choice.delta) {
                // 检查是否有内容增量
                if let Some(content) = &delta.content {
                    return StreamingEvent::PartialContentDelta(PartialContentDelta {
                        delta: content.clone(),
                        choice_index: choice.index as usize,
                        finish_reason: choice.finish_reason.clone(),
                        candidate_index: None,
                    });
                }

                // TODO: 检查是否有工具调用 - MessageDelta目前没有tool_calls字段
            }

            // 检查完成原因
            if let Some(finish_reason) = &choice.finish_reason {
                return StreamingEvent::FinalCandidate(FinalCandidate {
                    choices: vec![], // TODO: 需要转换ChoiceDelta到Choice
                    usage: None,     // ChatCompletionChunk没有usage字段
                    finish_reason: Some(finish_reason.clone()),
                    model: Some(chunk.model.clone()),
                });
            }
        }

        // 默认返回流结束
        StreamingEvent::StreamEnd
    }
}

/// Fallback conversion from StreamingEvent to ChatCompletionChunk for API compatibility
impl TryFrom<StreamingEvent> for ChatCompletionChunk {
    type Error = crate::AiLibError;

    fn try_from(event: StreamingEvent) -> Result<Self, Self::Error> {
        use crate::api::{ChoiceDelta, MessageDelta};
        use crate::types::Role;

        match event {
            StreamingEvent::PartialContentDelta(d) => Ok(ChatCompletionChunk {
                id: String::new(),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                model: String::new(),
                choices: vec![ChoiceDelta {
                    index: d.choice_index as u32,
                    delta: MessageDelta {
                        role: Some(Role::Assistant),
                        content: Some(d.delta),
                    },
                    finish_reason: d.finish_reason,
                }],
            }),
            StreamingEvent::ThinkingDelta(d) => Ok(ChatCompletionChunk {
                id: String::new(),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                model: String::new(),
                choices: vec![ChoiceDelta {
                    index: 0,
                    delta: MessageDelta {
                        role: Some(Role::Assistant),
                        content: Some(format!("<thinking>\n{}\n</thinking>", d.thinking)),
                    },
                    finish_reason: None,
                }],
            }),
            StreamingEvent::PartialToolCall(_d) => {
                // OpenAI style tool call delta (simplified)
                Ok(ChatCompletionChunk {
                    id: String::new(),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp() as u64,
                    model: String::new(),
                    choices: vec![ChoiceDelta {
                        index: 0,
                        delta: MessageDelta {
                            role: Some(Role::Assistant),
                            content: None,
                        },
                        finish_reason: None,
                    }],
                })
            }
            StreamingEvent::FinalCandidate(d) => Ok(ChatCompletionChunk {
                id: String::new(),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                model: d.model.unwrap_or_default(),
                choices: vec![ChoiceDelta {
                    index: 0,
                    delta: MessageDelta {
                        role: None,
                        content: None,
                    },
                    finish_reason: d.finish_reason,
                }],
            }),
            StreamingEvent::Error(e) => Err(crate::AiLibError::ProviderError(e.message)),
            StreamingEvent::StreamEnd => Ok(ChatCompletionChunk {
                id: String::new(),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp() as u64,
                model: String::new(),
                choices: vec![],
            }),
            _ => Err(crate::AiLibError::UnsupportedFeature(format!(
                "Cannot convert {:?} to ChatCompletionChunk",
                event.event_type()
            ))),
        }
    }
}
