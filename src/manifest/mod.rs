//! YAML Manifest系统
//!
//! 这个模块实现了基于YAML的配置驱动AI适配器系统，
//! 完全废弃了原有的JSON配置，支持零代码扩展新AI提供商。

pub mod cli;
pub mod schema;
pub mod types;

pub use schema::{
    AgenticCapabilities,
    // 🆕 2025年扩展
    AgenticLoopSchema,
    AuthConfig,
    Capability,
    ConditionalMapping,
    ErrorMapping,
    JsonPath,
    Manifest,
    ManifestError,
    ManifestMetadata,
    ManifestResult,
    ManifestValidator,
    MappingRule,
    MediaTypeConfig,
    ModelDefinition,
    ModelStatus,
    MultiCandidateFeature,
    MultiCandidateSupport,
    MultimodalSchema,
    ParameterConstraints,
    ParameterDefinition,
    ParameterTransform,
    ParameterType,
    PayloadFormat,
    PricingInfo,
    PromptCachingConfig,
    ProviderDefinition,
    ProviderFeatures,
    ReasoningEffort,
    ReasoningTokensConfig,
    ResponseFormat,
    ResponseFormatSchema,
    ResponseMapping,
    ServicePriority,
    ServiceTierConfig,
    SpecialHandling,
    StandardSchema,
    StreamingConfig,
    StreamingEventsSchema,
    ToolCallFields,
    ToolCallsMapping,
    ToolInvokeStyle,
    ToolMappingConfig,
    ToolSchema,
    TransformType,
};

// 🆕 2025年核心类型 (Phase 1简化版本)
pub use types::{
    AgenticResponse, AudioContent, Choice, Citation, CitationChunk, CitationType, ContentPart,
    DocumentContent, FinalCandidate, ImageContent, InferenceParams, MessageRole,
    PartialContentDelta, PartialToolCall, ReasoningUsage, ResponseMetadata, StandardMessage,
    StandardRequest, StreamingEvent, ThinkingDelta, ToolCall, ToolCallEnded, ToolCallStarted,
    ToolCallStatus, ToolDefinition, ToolExecutionStatus, ToolInvocationStyle, ToolResult,
    UnifiedResponse, UploadStrategy, Usage, VideoContent,
};

use std::path::Path;
use std::sync::Arc;

/// Manifest加载器
pub struct ManifestLoader;

impl ManifestLoader {
    /// 从文件加载Manifest
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> ManifestResult<Manifest> {
        let content = std::fs::read_to_string(path)?;
        Self::load_from_string(&content)
    }

    /// 从字符串加载Manifest
    pub fn load_from_string(content: &str) -> ManifestResult<Manifest> {
        // 1. 结构验证 (Structural Validation)
        // 如果 YAML 缺字段、类型不对，Serde 这里直接报错
        let manifest: Manifest = serde_yaml::from_str(content)?;

        // 2. 逻辑验证 (Logical Validation)
        // 检查数值范围、URL格式等业务逻辑
        use validator::Validate;
        manifest.validate()?;

        // 3. 额外的Manifest特定验证
        ManifestValidator::validate_manifest(&manifest)?;

        Ok(manifest)
    }

    /// 加载并缓存Manifest（用于热重载）
    pub fn load_cached<P: AsRef<Path>>(path: P) -> ManifestResult<Arc<Manifest>> {
        let manifest = Self::load_from_file(path)?;
        Ok(Arc::new(manifest))
    }
}

/// 便捷的Manifest创建函数
pub fn load_manifest<P: AsRef<Path>>(path: P) -> ManifestResult<Manifest> {
    ManifestLoader::load_from_file(path)
}

/// 导出JSON Schema用于编辑器支持
/// 这实现了"Code-First"验证方式，通过Rust struct自动生成JSON Schema
/// 注意：目前只导出基本结构，完整的嵌套Schema将在Phase 3完善
pub fn export_json_schema() -> String {
    let schema = schemars::schema_for!(Manifest);
    serde_json::to_string_pretty(&schema).unwrap_or_else(|_| "{}".to_string())
}

/// 导出JSON Schema到文件
pub fn export_json_schema_to_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    use std::fs;
    let schema_json = export_json_schema();
    fs::write(path, schema_json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_manifest_from_file() {
        let yaml_content = r#"
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
  test_provider:
    version: "v1"
    base_url: "https://api.test.com/v1"
    auth:
      type: bearer
      token_env: "TEST_API_KEY"
    payload_format: "openai_style"
    parameter_mappings:
      temperature: "temperature"
    response_format: "openai_style"
    response_paths:
      content: "choices[0].message.content"
models:
  test_model:
    provider: "test_provider"
    model_id: "test-model"
    context_window: 4096
    capabilities: ["chat"]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(yaml_content.as_bytes()).unwrap();

        let manifest = ManifestLoader::load_from_file(temp_file.path()).unwrap();

        assert_eq!(manifest.version, "1.0");
        assert!(manifest.providers.contains_key("test_provider"));
        assert!(manifest.models.contains_key("test_model"));
    }

    #[test]
    fn test_load_invalid_manifest() {
        let invalid_yaml = r#"
version: "1.0"
# 缺少必要的standard_schema字段
providers: {}
models: {}
"#;

        let result = ManifestLoader::load_from_string(invalid_yaml);
        assert!(result.is_err());
    }
}
