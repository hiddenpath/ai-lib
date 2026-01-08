//! YAML Manifest Schema定义
//!
//! 这个模块定义了ai-lib-manifest.yaml的完整Rust类型表示，
//! 实现从标准接口到提供商异构映射的完整配置系统。

use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// 根Manifest结构
#[derive(Debug, Clone, Deserialize, Serialize, Validate, JsonSchema)]
pub struct Manifest {
    /// 配置版本
    pub version: String,

    /// 元数据
    #[serde(default)]
    pub metadata: ManifestMetadata,

    /// 标准接口定义（第一层）
    pub standard_schema: StandardSchema,

    /// 提供商映射定义（第二层）
    pub providers: HashMap<String, ProviderDefinition>,

    /// 模型实例定义（第三层）
    pub models: HashMap<String, ModelDefinition>,
}

/// Manifest元数据
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ManifestMetadata {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
    #[serde(default)]
    pub authors: Vec<String>,
}

impl Default for ManifestMetadata {
    fn default() -> Self {
        Self {
            description: Some("AI-Lib Provider Manifest".to_string()),
            last_updated: None,
            authors: vec!["AI-Lib Team".to_string()],
        }
    }
}

/// 标准接口定义（第一层）
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StandardSchema {
    /// 标准参数定义
    pub parameters: HashMap<String, ParameterDefinition>,

    /// 工具调用定义
    pub tools: ToolSchema,

    /// 响应格式定义
    pub response_format: ResponseFormatSchema,

    /// 多模态内容定义
    #[serde(default)]
    pub multimodal: MultimodalSchema,

    /// 🆕 2025年：Agentic Loop配置
    #[serde(default)]
    pub agentic_loop: Option<AgenticLoopSchema>,

    /// 🆕 2025年：Streaming事件模型
    #[serde(default)]
    pub streaming_events: Option<StreamingEventsSchema>,
}

/// 参数定义
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ParameterDefinition {
    /// 参数类型
    #[serde(rename = "type")]
    pub param_type: ParameterType,

    /// 类型约束
    #[serde(flatten)]
    pub constraints: ParameterConstraints,

    /// 默认值
    #[serde(default)]
    pub default: Option<serde_json::Value>,

    /// 描述
    #[serde(default)]
    pub description: Option<String>,
}

/// 参数类型枚举
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
}

/// 参数约束
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ParameterConstraints {
    /// 数值范围（用于数字类型）
    #[serde(default)]
    pub range: Option<[f64; 2]>,

    /// 整数范围
    #[serde(default)]
    pub min: Option<i64>,
    #[serde(default)]
    pub max: Option<i64>,

    /// 枚举值（用于string类型）
    #[serde(default)]
    pub values: Vec<String>,

    /// 正则表达式（用于string类型）
    #[serde(default)]
    pub pattern: Option<String>,
}

/// 工具调用Schema定义
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct ToolSchema {
    /// 标准工具定义格式
    pub schema: String,

    /// 选择策略枚举
    pub choice_policy: Vec<String>,

    /// 是否支持严格模式
    #[serde(default)]
    pub strict_mode: bool,

    /// 是否支持并行调用
    #[serde(default)]
    pub parallel_calls: bool,
}

/// 响应格式Schema定义
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct ResponseFormatSchema {
    /// 支持的响应类型
    pub types: Vec<String>,

    /// 是否支持Schema验证
    #[serde(default)]
    pub schema_validation: bool,
}

/// 多模态内容Schema定义
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct MultimodalSchema {
    /// 图像支持
    #[serde(default)]
    pub image: MediaTypeConfig,

    /// 音频支持
    #[serde(default)]
    pub audio: MediaTypeConfig,

    /// 视频支持
    #[serde(default)]
    pub video: MediaTypeConfig,
}

/// 媒体类型配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct MediaTypeConfig {
    /// 支持的格式
    pub formats: Vec<String>,

    /// 最大文件大小
    pub max_size: String,
}

impl Default for MediaTypeConfig {
    fn default() -> Self {
        Self {
            formats: vec![],
            max_size: "10MB".to_string(),
        }
    }
}

/// 🆕 2025年：Agentic Loop Schema
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AgenticLoopSchema {
    /// 最大迭代次数
    pub max_iterations: usize,

    /// 停止条件
    pub stop_conditions: Vec<String>,

    /// 推理强度
    pub reasoning_effort: ReasoningEffort,

    /// 支持thinking blocks
    #[serde(default)]
    pub thinking_blocks: bool,
}

/// 🆕 2025年：推理强度枚举
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    #[default]
    Auto,
    Low,
    Medium,
    High,
}

/// 🆕 2025年：Streaming事件Schema
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct StreamingEventsSchema {
    /// 支持的事件类型
    pub supported_events: Vec<String>,

    /// thinking blocks支持
    #[serde(default)]
    pub thinking_blocks: bool,

    /// citations支持
    #[serde(default)]
    pub citations_enabled: bool,

    /// 部分工具调用支持
    #[serde(default)]
    pub partial_tool_calls: bool,
}

/// 提供商定义（第二层）
#[derive(Debug, Clone, Deserialize, Serialize, Validate, JsonSchema)]
pub struct ProviderDefinition {
    /// API版本
    #[validate(length(min = 1))]
    pub version: String,

    /// 基础URL（静态）
    #[serde(default)]
    pub base_url: Option<String>,

    /// 基础URL模板（支持变量替换，如Azure OpenAI）
    #[serde(default)]
    pub base_url_template: Option<String>,

    /// 连接变量（用于URL模板替换）
    #[serde(default)]
    pub connection_vars: Option<HashMap<String, String>>,

    /// 认证配置
    pub auth: AuthConfig,

    /// 请求体格式
    pub payload_format: PayloadFormat,

    /// 参数映射规则
    pub parameter_mappings: HashMap<String, MappingRule>,

    /// 特殊处理规则
    #[serde(default)]
    pub special_handling: HashMap<String, SpecialHandling>,

    /// 响应格式
    pub response_format: ResponseFormat,

    /// 响应路径映射
    pub response_paths: HashMap<String, JsonPath>,

    /// 流式配置
    #[serde(default)]
    pub streaming: StreamingConfig,

    /// 实验性特性
    #[serde(default)]
    pub experimental_features: Vec<String>,

    /// 能力标识（自动推断或显式定义）
    #[serde(default)]
    pub capabilities: Vec<Capability>,

    /// 🆕 2025年：响应策略（Responses API等）
    #[serde(default)]
    pub response_strategy: Option<String>,

    /// 🆕 2025年：工具映射配置
    #[serde(default)]
    pub tools_mapping: Option<HashMap<String, ToolMappingConfig>>,

    /// 🆕 2025年：Prompt Caching配置
    #[serde(default)]
    pub prompt_caching: Option<PromptCachingConfig>,

    /// 🆕 2025年：服务层级配置
    #[serde(default)]
    pub service_tier: Option<ServiceTierConfig>,

    /// 🆕 2025年：推理tokens管理
    #[serde(default)]
    pub reasoning_tokens: Option<ReasoningTokensConfig>,

    /// 🆕 Provider特性配置（多候选、响应映射等）
    #[serde(default)]
    pub features: Option<ProviderFeatures>,
}

/// 认证配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum AuthConfig {
    /// Bearer token认证
    #[serde(rename = "bearer")]
    Bearer {
        token_env: String,
        #[serde(default)]
        extra_headers: Vec<HeaderDefinition>,
    },

    /// API key认证
    #[serde(rename = "api_key")]
    ApiKey {
        key_env: String,
        #[serde(default)]
        header_name: Option<String>,
    },

    /// 查询参数认证
    #[serde(rename = "query_param")]
    QueryParam {
        param_name: String,
        token_env: String,
    },

    /// OAuth2认证
    #[serde(rename = "oauth2")]
    OAuth2 {
        client_id_env: String,
        client_secret_env: String,
        token_url: String,
        #[serde(default)]
        scopes: Vec<String>,
    },

    /// Google Application Default Credentials
    #[serde(rename = "google_adc")]
    GoogleAdc {
        #[serde(default)]
        service_account_env: Option<String>,
    },
}

/// HTTP头定义
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct HeaderDefinition {
    pub name: String,
    pub value: String,
}

/// 请求体格式枚举
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PayloadFormat {
    OpenaiStyle,
    AnthropicStyle,
    GeminiStyle,
    /// Cohere V2 API native format
    CohereNative,
    Custom(String),
}

/// 参数映射规则
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum MappingRule {
    /// 直接映射到路径
    Direct(String),

    /// 条件映射
    Conditional(Vec<ConditionalMapping>),

    /// 转换映射
    Transform(ParameterTransform),
}

/// 条件映射
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ConditionalMapping {
    /// 条件表达式
    pub condition: String,

    /// 目标路径
    pub target_path: String,

    /// 转换规则（可选）
    #[serde(default)]
    pub transform: Option<ParameterTransform>,
}

/// 参数转换规则
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ParameterTransform {
    /// 转换类型
    #[serde(rename = "type")]
    pub transform_type: TransformType,

    /// 目标路径
    pub target_path: String,

    /// 转换参数
    #[serde(default)]
    pub params: serde_json::Map<String, serde_json::Value>,
}

/// 转换类型枚举
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransformType {
    /// 乘法转换（用于温度等参数）
    Scale,
    /// 字符串格式化
    Format,
    /// 枚举值映射
    EnumMap,
    /// 自定义转换
    Custom,
}

/// 🆕 Provider特性配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct ProviderFeatures {
    /// 多候选支持配置
    #[serde(default)]
    pub multi_candidate: Option<MultiCandidateFeature>,

    /// 响应映射配置（工具调用、错误映射等）
    #[serde(default)]
    pub response_mapping: Option<ResponseMapping>,
}

/// 多候选支持类型
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MultiCandidateSupport {
    Native,
    Simulated,
}

impl Default for MultiCandidateSupport {
    fn default() -> Self {
        MultiCandidateSupport::Native
    }
}

/// 多候选配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct MultiCandidateFeature {
    pub support_type: MultiCandidateSupport,
    #[serde(default)]
    pub param_name: Option<String>,
    #[serde(default)]
    pub max_concurrent: Option<usize>,
}

/// 响应映射配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct ResponseMapping {
    #[serde(default)]
    pub tool_calls: Option<ToolCallsMapping>,
    #[serde(default)]
    pub error: Option<ErrorMapping>,
    #[serde(default)]
    pub extra_metadata_path: Option<String>,
}

/// 工具调用映射配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct ToolCallsMapping {
    pub path: String,
    #[serde(default)]
    pub filter: Option<String>,
    pub fields: ToolCallFields,
    #[serde(default)]
    pub array_fan_out: bool,
}

/// 工具调用字段映射
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct ToolCallFields {
    pub id: String,
    pub name: String,
    pub args: String,
    #[serde(default)]
    pub id_strategy: Option<IdStrategy>,
}

/// 工具调用ID策略
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IdStrategy {
    GenerateUuid,
    Path,
}

/// 错误映射配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct ErrorMapping {
    #[serde(default)]
    pub message_path: Option<String>,
    #[serde(default)]
    pub code_path: Option<String>,
}

/// 特殊处理规则
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum SpecialHandling {
    /// 路径重定向
    PathRedirect(String),

    /// 结构转换
    StructureTransform {
        /// 转换类型
        transform_type: String,
        /// 参数
        params: HashMap<String, serde_json::Value>,
    },
}

/// 响应格式枚举
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    OpenaiStyle,
    AnthropicStyle,
    GeminiStyle,
    Custom(String),
}

/// JSON路径定义（支持点号和数组语法）
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct JsonPath(pub String);

/// 流式配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct StreamingConfig {
    /// 事件格式
    pub event_format: Option<String>,

    /// 内容路径
    pub content_path: Option<String>,

    /// 工具调用路径
    pub tool_call_path: Option<String>,

    /// 完成原因路径
    pub finish_reason_path: Option<String>,

    /// 流解码器配置（算子化）
    #[serde(default)]
    pub decoder: Option<StreamingDecoder>,

    /// 帧过滤器（JSONPath/表达式）
    #[serde(default)]
    pub frame_selector: Option<String>,

    /// 累积器配置（用于分片工具参数）
    #[serde(default)]
    pub accumulator: Option<StreamingAccumulator>,

    /// 候选拆分配置（fan-out）
    #[serde(default)]
    pub candidate: Option<StreamingCandidateConfig>,

    /// 事件映射规则表
    #[serde(default)]
    pub event_map: Vec<StreamingEventRule>,

    /// 停止条件
    #[serde(default)]
    pub stop_condition: Option<String>,

    /// 额外元数据收集路径（如citations）
    #[serde(default)]
    pub extra_metadata_path: Option<String>,
}

/// 流解码器配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct StreamingDecoder {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub delimiter: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub done_signal: Option<String>,
}

/// 流累积器配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct StreamingAccumulator {
    #[serde(default)]
    pub stateful_tool_parsing: bool,
    #[serde(default)]
    pub key_path: Option<String>,
    #[serde(default)]
    pub flush_on: Option<String>,
}

/// 多候选拆分配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct StreamingCandidateConfig {
    #[serde(default)]
    pub candidate_id_path: Option<String>,
    #[serde(default)]
    pub fan_out: bool,
}

/// 流事件映射规则
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct StreamingEventRule {
    #[serde(rename = "match")]
    pub matcher: String,
    pub emit: String,
    #[serde(default)]
    pub fields: HashMap<String, String>,
}

/// 能力标识枚举
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// 聊天能力
    Chat,
    /// 代码生成能力
    Code,
    /// 多模态能力（文本+图像/音频）
    Multimodal,
    /// 视觉能力
    Vision,
    /// 音频能力
    Audio,
    /// 视频能力
    Video,
    /// 函数调用能力
    Tools,
    /// 工具使用能力
    ToolUse,
    /// JSON模式能力
    JsonMode,
    /// 结构化输出能力
    StructuredOutput,
    /// 流式输出能力
    Streaming,
    /// 🆕 2025年：Agentic能力
    Agentic,
    /// 🆕 2025年：推理能力
    Reasoning,
    /// 🆕 2025年：并行工具调用
    ParallelTools,
    /// 🆕 2025年：内置工具链
    BuiltinTools,
}

/// 🆕 2025年：工具映射配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ToolMappingConfig {
    /// Provider中的工具名称
    pub provider_name: String,

    /// Schema路径映射
    pub schema_path: String,

    /// 支持并行调用
    #[serde(default)]
    pub parallel: bool,

    /// 调用风格
    #[serde(default)]
    pub invoke_style: ToolInvokeStyle,

    /// 最大并行度
    #[serde(default)]
    pub max_parallel: Option<usize>,

    /// 超时时间（秒）
    #[serde(default)]
    pub timeout: Option<u64>,
}

/// 🆕 2025年：工具调用风格
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum ToolInvokeStyle {
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

/// 🆕 2025年：Prompt Caching配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PromptCachingConfig {
    /// 是否启用
    #[serde(default)]
    pub enabled: bool,

    /// 缓存TTL（秒）
    #[serde(default)]
    pub ttl: Option<u64>,

    /// 缓存命名空间
    #[serde(default)]
    pub namespace: Option<String>,
}

/// 🆕 2025年：服务层级配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ServiceTierConfig {
    /// 优先级
    pub priority: ServicePriority,

    /// 支持批处理
    #[serde(default)]
    pub batch_supported: bool,
}

/// 🆕 2025年：服务优先级
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ServicePriority {
    Low,
    Medium,
    High,
}

/// 🆕 2025年：推理Tokens配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ReasoningTokensConfig {
    /// 预留推理tokens数量
    pub reserved: Option<u64>,

    /// 自动计算预留量
    #[serde(default)]
    pub auto_reserve: bool,

    /// 推理tokens计费倍数
    #[serde(default)]
    pub billing_multiplier: Option<f64>,
}

/// 🆕 2025年：Agentic能力配置
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AgenticCapabilities {
    /// 推理强度
    #[serde(default)]
    pub reasoning_effort: ReasoningEffort,

    /// 支持thinking blocks
    #[serde(default)]
    pub thinking_blocks: bool,

    /// 支持并行工具调用
    #[serde(default)]
    pub parallel_tools: bool,

    /// 最大工具并行度
    #[serde(default)]
    pub max_parallel_tools: Option<usize>,

    /// 支持内置工具链
    #[serde(default)]
    pub builtin_tools: Vec<String>,
}

/// 模型定义（第三层）
#[derive(Debug, Clone, Deserialize, Serialize, Validate, JsonSchema)]
pub struct ModelDefinition {
    /// 关联的提供商
    pub provider: String,

    /// 模型ID（API调用时使用）
    #[validate(length(min = 1))]
    pub model_id: String,

    /// 显示名称（UI友好）
    #[serde(default)]
    pub display_name: Option<String>,

    /// 上下文窗口大小
    #[validate(range(min = 1, max = 1000000))]
    pub context_window: usize,

    /// 模型能力列表
    pub capabilities: Vec<Capability>,

    /// 定价信息
    #[serde(default)]
    pub pricing: Option<PricingInfo>,

    /// 覆盖配置（覆盖provider的默认设置）
    #[serde(default)]
    pub overrides: HashMap<String, serde_json::Value>,

    /// 模型状态
    #[serde(default)]
    pub status: ModelStatus,

    /// 标签（用于分类和过滤）
    #[serde(default)]
    pub tags: Vec<String>,

    /// 🆕 2025年：Agentic能力配置
    #[serde(default)]
    pub agentic_capabilities: Option<AgenticCapabilities>,
}

/// 定价信息
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct PricingInfo {
    /// 输入token单价（USD）
    pub input_per_token: f64,

    /// 输出token单价（USD）
    pub output_per_token: f64,

    /// 货币单位
    #[serde(default)]
    pub currency: String,

    /// 计费单位（token, character, request等）
    #[serde(default)]
    pub unit: String,
}

impl Default for PricingInfo {
    fn default() -> Self {
        Self {
            input_per_token: 0.0,
            output_per_token: 0.0,
            currency: "USD".to_string(),
            unit: "token".to_string(),
        }
    }
}

/// 模型状态枚举
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    /// 活跃可用
    #[default]
    Active,
    /// 即将弃用
    Deprecated,
    /// 实验性
    Experimental,
    /// 不可用
    Disabled,
}

/// Manifest验证错误
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid reference: {0}")]
    InvalidReference(String),

    #[error("Capability mismatch: {0}")]
    CapabilityMismatch(String),
}

pub type ManifestResult<T> = Result<T, ManifestError>;

impl From<std::io::Error> for ManifestError {
    fn from(err: std::io::Error) -> Self {
        ManifestError::ValidationError(format!("IO error: {}", err))
    }
}

impl From<validator::ValidationErrors> for ManifestError {
    fn from(err: validator::ValidationErrors) -> Self {
        ManifestError::ValidationError(format!("Validation error: {}", err))
    }
}

/// Manifest验证器
pub struct ManifestValidator;

impl ManifestValidator {
    /// 验证完整Manifest
    pub fn validate_manifest(manifest: &Manifest) -> ManifestResult<()> {
        // 验证版本
        if manifest.version.is_empty() {
            return Err(ManifestError::MissingField("version".to_string()));
        }

        // 验证提供商引用
        for model in manifest.models.values() {
            if !manifest.providers.contains_key(&model.provider) {
                return Err(ManifestError::InvalidReference(format!(
                    "Model '{}' references unknown provider '{}'",
                    model.model_id, model.provider
                )));
            }
        }

        // 验证参数映射
        for provider in manifest.providers.values() {
            Self::validate_provider_mappings(provider, &manifest.standard_schema)?;
            Self::validate_response_paths(provider)?;
            Self::validate_streaming(provider)?;
            Self::validate_base_url_template(provider)?;
        }

        Ok(())
    }

    /// 验证提供商映射规则
    fn validate_provider_mappings(
        provider: &ProviderDefinition,
        standard: &StandardSchema,
    ) -> ManifestResult<()> {
        // 检查所有标准参数都有映射
        for param_name in standard.parameters.keys() {
            if !provider.parameter_mappings.contains_key(param_name) {
                // 对于可选参数，跳过验证
                continue;
            }
        }

        // 验证映射规则格式
        for (param_name, rule) in &provider.parameter_mappings {
            Self::validate_mapping_rule(param_name, rule)?;
        }

        Ok(())
    }

    /// 验证映射规则
    fn validate_mapping_rule(param_name: &str, rule: &MappingRule) -> ManifestResult<()> {
        match rule {
            MappingRule::Direct(path) => {
                if path.is_empty() {
                    return Err(ManifestError::ValidationError(format!(
                        "Empty path for parameter '{}'",
                        param_name
                    )));
                }
            }
            MappingRule::Conditional(conditions) => {
                if conditions.is_empty() {
                    return Err(ManifestError::ValidationError(format!(
                        "No conditions for parameter '{}'",
                        param_name
                    )));
                }
                for condition in conditions {
                    if condition.target_path.is_empty() {
                        return Err(ManifestError::ValidationError(format!(
                            "Empty target path in condition for parameter '{}'",
                            param_name
                        )));
                    }
                }
            }
            MappingRule::Transform(transform) => {
                if transform.target_path.is_empty() {
                    return Err(ManifestError::ValidationError(format!(
                        "Empty target path in transform for parameter '{}'",
                        param_name
                    )));
                }
            }
        }
        Ok(())
    }

    /// 验证响应路径配置
    fn validate_response_paths(provider: &ProviderDefinition) -> ManifestResult<()> {
        if !provider.response_paths.contains_key("content") {
            return Err(ManifestError::ValidationError(format!(
                "Provider '{}' missing response_paths.content",
                provider.version
            )));
        }
        Ok(())
    }

    /// 验证流式配置
    fn validate_streaming(provider: &ProviderDefinition) -> ManifestResult<()> {
        if let Some(event_format) = &provider.streaming.event_format {
            if event_format.is_empty() {
                return Err(ManifestError::ValidationError(
                    "streaming.event_format cannot be empty".to_string(),
                ));
            }
            if provider.streaming.content_path.is_none()
                && provider.streaming.tool_call_path.is_none()
            {
                return Err(ManifestError::ValidationError(
                    "streaming.content_path or streaming.tool_call_path must be set when streaming is enabled"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }

    /// 验证base_url_template变量匹配
    fn validate_base_url_template(provider: &ProviderDefinition) -> ManifestResult<()> {
        if let Some(tpl) = &provider.base_url_template {
            let re = Regex::new(r"\{([A-Za-z0-9_]+)\}").unwrap();
            for caps in re.captures_iter(tpl) {
                let var = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let vars = provider.connection_vars.as_ref().ok_or_else(|| {
                    ManifestError::ValidationError(format!(
                        "Provider with base_url_template requires connection_vars for '{}'",
                        var
                    ))
                })?;
                if !vars.contains_key(var) {
                    return Err(ManifestError::ValidationError(format!(
                        "Missing connection_vars entry '{}' for base_url_template",
                        var
                    )));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_deserialization() {
        let yaml = r#"
version: "1.0"
standard_schema:
  parameters:
    temperature:
      type: float
      range: [0.0, 2.0]
      default: 1.0
  tools:
    schema: "standard_tool_definition"
    choice_policy: ["auto", "none"]
    strict_mode: false
    parallel_calls: false
  response_format:
    types: ["text", "json"]
    schema_validation: false
providers:
  openai:
    version: "v1"
    base_url: "https://api.openai.com/v1"
    auth:
      type: bearer
      token_env: "OPENAI_API_KEY"
    payload_format: "openai_style"
    parameter_mappings:
      temperature: "temperature"
    response_format: "openai_style"
    response_paths:
      content: "choices[0].message.content"
models:
  gpt-4:
    provider: "openai"
    model_id: "gpt-4"
    context_window: 8192
    capabilities: ["chat", "tools"]
"#;

        let manifest: Manifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(manifest.version, "1.0");
        assert!(manifest.providers.contains_key("openai"));
        assert!(manifest.models.contains_key("gpt-4"));
    }

    #[test]
    fn test_manifest_validation() {
        let mut manifest = Manifest {
            version: "1.0".to_string(),
            metadata: ManifestMetadata::default(),
            standard_schema: StandardSchema {
                parameters: HashMap::new(),
                tools: ToolSchema {
                    schema: "test".to_string(),
                    choice_policy: vec![],
                    strict_mode: false,
                    parallel_calls: false,
                },
                response_format: ResponseFormatSchema {
                    types: vec![],
                    schema_validation: false,
                },
                multimodal: MultimodalSchema::default(),
                agentic_loop: None,
                streaming_events: None,
            },
            providers: HashMap::new(),
            models: HashMap::new(),
        };

        // 空manifest应该通过基本验证
        assert!(ManifestValidator::validate_manifest(&manifest).is_ok());

        // 添加无效的模型引用
        manifest.models.insert(
            "invalid".to_string(),
            ModelDefinition {
                provider: "nonexistent".to_string(),
                model_id: "invalid".to_string(),
                display_name: None,
                context_window: 4096,
                capabilities: vec![],
                pricing: None,
                overrides: HashMap::new(),
                status: ModelStatus::Active,
                tags: vec![],
                agentic_capabilities: None,
            },
        );

        // 应该检测到无效引用
        assert!(ManifestValidator::validate_manifest(&manifest).is_err());
    }
}
