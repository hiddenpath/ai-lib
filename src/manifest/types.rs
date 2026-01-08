//! 2025年Manifest-First核心类型定义
//!
//! 这个模块定义了ai-lib-manifest-first架构的核心类型，
//! 包括StandardRequest、UnifiedResponse、StreamingEvent等。

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 🆕 2025年：标准请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardRequest {
    /// 模型ID
    pub model: String,

    /// 消息列表
    pub messages: Vec<StandardMessage>,

    /// 推理参数
    pub inference_params: InferenceParams,

    /// 工具定义
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,

    /// 多模态内容
    #[serde(default)]
    pub multimodal: Vec<ContentPart>,

    /// 流式选项
    #[serde(default)]
    pub stream: bool,

    /// 请求级扩展
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// 🆕 2025年：标准消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardMessage {
    /// 消息角色
    pub role: MessageRole,

    /// 消息内容
    pub content: Vec<ContentPart>,

    /// 工具调用（助手消息）
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,

    /// 工具结果（工具消息）
    #[serde(default)]
    pub tool_result: Option<ToolResult>,

    /// 消息级扩展
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// 🆕 2025年：消息角色枚举
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// 🆕 2025年：内容部分（多模态支持）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text(String),

    #[serde(rename = "image")]
    Image(ImageContent),

    #[serde(rename = "audio")]
    Audio(AudioContent),

    #[serde(rename = "video")]
    Video(VideoContent),

    #[serde(rename = "document")]
    Document(DocumentContent),
}

/// 🆕 2025年：图像内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    pub url: Option<String>,
    pub base64: Option<String>,
    pub mime_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// 🆕 2025年：音频内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioContent {
    pub url: Option<String>,
    pub base64: Option<String>,
    pub format: String,
    pub duration: Option<f64>,
}

/// 🆕 2025年：视频内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoContent {
    pub url: Option<String>,
    pub base64: Option<String>,
    pub format: String,
    pub duration: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// 🆕 2025年：文档内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    pub url: Option<String>,
    pub base64: Option<String>,
    pub mime_type: String,
    pub pages: Option<Vec<u32>>,
    pub title: Option<String>,
}

/// 🆕 2025年：推理参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceParams {
    /// 温度
    pub temperature: Option<f64>,

    /// Top-P
    pub top_p: Option<f64>,

    /// Top-K
    pub top_k: Option<u32>,

    /// 最大tokens
    pub max_tokens: Option<u32>,

    /// 停止序列
    #[serde(default)]
    pub stop_sequences: Vec<String>,

    /// Logit bias
    #[serde(default)]
    pub logit_bias: HashMap<String, f64>,

    /// 随机种子
    pub seed: Option<u64>,

    /// 其他参数
    #[serde(flatten)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// 🆕 2025年：工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// 工具ID
    pub id: String,

    /// 工具名称
    pub name: String,

    /// 工具描述
    pub description: String,

    /// 输入Schema
    pub input_schema: serde_json::Value,

    /// 输出Schema
    #[serde(default)]
    pub output_schema: Option<serde_json::Value>,

    /// 调用风格
    #[serde(default)]
    pub invocation_style: ToolInvocationStyle,

    /// 工具级扩展
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// 🆕 2025年：工具调用风格
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum ToolInvocationStyle {
    /// 同步调用
    #[default]
    Sync,
    /// 异步调用
    Async,
    /// 回调模式
    Callback,
    /// 并行调用
    Parallel,
}

/// 🆕 2025年：工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// 调用ID
    pub id: String,

    /// 工具名称
    pub name: String,

    /// 调用参数
    pub arguments: serde_json::Value,

    /// 调用状态
    #[serde(default)]
    pub status: ToolCallStatus,
}

/// 🆕 2025年：工具调用状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ToolCallStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
}

/// 🆕 2025年：工具结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 工具调用ID
    pub tool_call_id: String,

    /// 执行结果
    pub result: serde_json::Value,

    /// 执行状态
    pub status: ToolExecutionStatus,

    /// 执行时间（毫秒）
    #[serde(default)]
    pub execution_time_ms: Option<u64>,

    /// 错误信息
    #[serde(default)]
    pub error: Option<String>,
}

/// 🆕 2025年：工具执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolExecutionStatus {
    Success,
    Error,
    Timeout,
    Cancelled,
}

/// 🆕 2025年：统一响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedResponse {
    /// 响应ID
    pub id: String,

    /// 对象类型
    pub object: String,

    /// 创建时间戳
    pub created: i64,

    /// 使用的模型
    pub model: String,

    /// 选择列表
    pub choices: Vec<Choice>,

    /// 使用统计
    pub usage: Usage,

    /// 推理使用统计（2025年）
    #[serde(default)]
    pub reasoning_usage: Option<ReasoningUsage>,

    /// 引用信息
    #[serde(default)]
    pub citations: Vec<Citation>,

    /// 响应级元数据
    #[serde(default)]
    pub metadata: ResponseMetadata,

    /// 响应级扩展
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// 🆕 2025年：选择结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// 选择索引
    pub index: usize,

    /// 消息
    pub message: StandardMessage,

    /// 完成原因
    pub finish_reason: Option<String>,

    /// 选择级扩展
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// 🆕 2025年：使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// 提示tokens
    pub prompt_tokens: Option<u32>,

    /// 完成tokens
    pub completion_tokens: Option<u32>,

    /// 总tokens
    pub total_tokens: Option<u32>,

    /// 是否估算
    #[serde(default)]
    pub estimated: bool,
}

/// 🆕 2025年：推理使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningUsage {
    /// 推理tokens
    pub reasoning_tokens: Option<u32>,

    /// Thinking tokens
    pub thinking_tokens: Option<u32>,

    /// Cached tokens
    pub cached_tokens: Option<u32>,
}

/// 🆕 2025年：引用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// 引用ID
    #[serde(default)]
    pub id: Option<String>,

    /// 来源
    pub source: String,

    /// 定位器（页码、时间戳等）
    pub locator: Option<String>,

    /// 摘要
    pub snippet: Option<String>,

    /// 置信度
    pub confidence: Option<f64>,

    /// 引用类型
    #[serde(default)]
    pub citation_type: CitationType,
}

/// 🆕 2025年：引用类型
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CitationType {
    #[default]
    Text,
    Image,
    Audio,
    Video,
    Document,
}

/// 🆕 2025年：响应元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseMetadata {
    /// HTTP状态码
    pub http_status: Option<u16>,

    /// 延迟（毫秒）
    pub latency_ms: Option<u64>,

    /// 提供商
    pub provider: Option<String>,

    /// Manifest版本
    pub manifest_version: Option<String>,

    /// 服务层级
    pub service_tier: Option<String>,

    /// 请求ID
    pub request_id: Option<String>,
}

/// 🆕 2025年：Streaming事件枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum StreamingEvent {
    /// 部分内容增量
    #[serde(rename = "partial_content_delta")]
    PartialContentDelta(PartialContentDelta),

    /// 推理增量
    #[serde(rename = "thinking_delta")]
    ThinkingDelta(ThinkingDelta),

    /// 部分工具调用
    #[serde(rename = "partial_tool_call")]
    PartialToolCall(PartialToolCall),

    /// 工具调用开始
    #[serde(rename = "tool_call_started")]
    ToolCallStarted(ToolCallStarted),

    /// 工具调用结束
    #[serde(rename = "tool_call_ended")]
    ToolCallEnded(ToolCallEnded),

    /// 引用块
    #[serde(rename = "citation_chunk")]
    CitationChunk(CitationChunk),

    /// 最终候选
    #[serde(rename = "final_candidate")]
    FinalCandidate(FinalCandidate),
}

/// 🆕 2025年：部分内容增量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialContentDelta {
    pub content: String,
    pub model: String,
    pub choice_index: usize,
}

/// 🆕 2025年：推理增量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingDelta {
    pub thinking: String,
    pub effort: String, // "low", "medium", "high", "auto"
    pub model: String,
}

/// 🆕 2025年：部分工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialToolCall {
    pub tool_call_id: String,
    pub function_name: Option<String>,
    pub arguments_delta: Option<String>,
    pub choice_index: usize,
}

/// 🆕 2025年：工具调用开始
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallStarted {
    pub tool_call_id: String,
    pub function_name: String,
    pub choice_index: usize,
}

/// 🆕 2025年：工具调用结束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEnded {
    pub tool_call_id: String,
    pub result: ToolResult,
    pub choice_index: usize,
}

/// 🆕 2025年：引用块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationChunk {
    pub citation: Citation,
    pub choice_index: usize,
}

/// 🆕 2025年：最终候选
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalCandidate {
    pub choice: Choice,
    pub usage: Usage,
    #[serde(default)]
    pub reasoning_usage: Option<ReasoningUsage>,
}

/// 🆕 2025年：Agentic响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgenticResponse {
    /// 最终响应
    pub final_response: UnifiedResponse,

    /// 迭代次数
    pub iterations: usize,

    /// 工具调用次数
    pub tool_calls_made: usize,

    /// 推理tokens使用量
    pub reasoning_tokens_used: Option<u32>,

    /// 执行时间（毫秒）
    pub execution_time_ms: u64,

    /// Agentic级扩展
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

/// 🆕 2025年：上传策略枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadStrategy {
    Multipart,
    Base64Inline,
    UrlReference,
}

// 🆕 2025年：工具相关类型将在Phase 2中实现
// 暂时简化，专注于核心manifest schema

impl StandardRequest {
    /// 创建新的标准请求
    pub fn new(model: String, messages: Vec<StandardMessage>) -> Self {
        Self {
            model,
            messages,
            inference_params: InferenceParams {
                temperature: None,
                top_p: None,
                top_k: None,
                max_tokens: None,
                stop_sequences: vec![],
                logit_bias: HashMap::new(),
                seed: None,
                extensions: HashMap::new(),
            },
            tools: vec![],
            multimodal: vec![],
            stream: false,
            extensions: HashMap::new(),
        }
    }

    /// 设置温度
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.inference_params.temperature = Some(temperature);
        self
    }

    /// 设置最大tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.inference_params.max_tokens = Some(max_tokens);
        self
    }

    /// 启用流式
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// 添加工具
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }
}

impl StandardMessage {
    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: vec![ContentPart::Text(content.into())],
            tool_calls: vec![],
            tool_result: None,
            extensions: HashMap::new(),
        }
    }

    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: vec![ContentPart::Text(content.into())],
            tool_calls: vec![],
            tool_result: None,
            extensions: HashMap::new(),
        }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: vec![ContentPart::Text(content.into())],
            tool_calls: vec![],
            tool_result: None,
            extensions: HashMap::new(),
        }
    }

    /// 创建工具消息
    pub fn tool(result: ToolResult) -> Self {
        Self {
            role: MessageRole::Tool,
            content: vec![],
            tool_calls: vec![],
            tool_result: Some(result),
            extensions: HashMap::new(),
        }
    }
}
