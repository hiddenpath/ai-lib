//! Mapping规则定义
//!
//! 定义了各种映射规则的枚举和结构体

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 参数映射规则枚举
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum MappingRule {
    /// 直接映射到目标路径
    Direct(String),

    /// 条件映射：基于条件选择不同的映射规则
    Conditional(Vec<ConditionalMapping>),

    /// 转换映射：对值进行转换后再映射
    Transform(ParameterTransform),
}

/// 条件映射规则
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConditionalMapping {
    /// 条件表达式
    pub condition: String,

    /// 目标路径
    pub target_path: String,

    /// 可选的转换规则
    #[serde(default)]
    pub transform: Option<ParameterTransform>,
}

/// 参数转换规则
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParameterTransform {
    /// 转换类型
    #[serde(rename = "type")]
    pub transform_type: TransformType,

    /// 目标路径（如果与原始路径不同）
    #[serde(default)]
    pub target_path: Option<String>,

    /// 转换参数
    #[serde(default)]
    pub params: serde_json::Map<String, serde_json::Value>,
}

/// 转换类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransformType {
    /// 数值缩放 (e.g., temperature * factor)
    Scale,

    /// 字符串格式化 (mustache-style)
    Format,

    /// 枚举值映射
    EnumMap,

    /// 路径重构 (e.g., "a.b" -> "x.y.z")
    PathRewrite,

    /// 类型转换 (e.g., number -> string)
    TypeCast,

    /// 自定义转换 (预留)
    Custom,
}

/// 路径重构规则
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PathRewriteRule {
    /// 源路径模式 (支持通配符)
    pub source_pattern: String,

    /// 目标路径模板
    pub target_template: String,
}

/// 枚举映射规则
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnumMappingRule {
    /// 映射表: source_value -> target_value
    pub mappings: serde_json::Map<String, serde_json::Value>,

    /// 默认值（如果找不到映射）
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
}

impl MappingRule {
    /// 创建直接映射规则
    pub fn direct(path: impl Into<String>) -> Self {
        MappingRule::Direct(path.into())
    }

    /// 创建条件映射规则
    pub fn conditional(conditions: Vec<ConditionalMapping>) -> Self {
        MappingRule::Conditional(conditions)
    }

    /// 创建转换映射规则
    pub fn transform(transform: ParameterTransform) -> Self {
        MappingRule::Transform(transform)
    }
}

impl ParameterTransform {
    /// 创建缩放转换
    pub fn scale(factor: f64) -> Self {
        let mut params = serde_json::Map::new();
        params.insert("factor".to_string(), serde_json::json!(factor));

        Self {
            transform_type: TransformType::Scale,
            target_path: None,
            params,
        }
    }

    /// 创建格式化转换
    pub fn format(template: impl Into<String>) -> Self {
        let mut params = serde_json::Map::new();
        params.insert("template".to_string(), serde_json::json!(template.into()));

        Self {
            transform_type: TransformType::Format,
            target_path: None,
            params,
        }
    }

    /// 创建枚举映射转换
    pub fn enum_map(mappings: serde_json::Map<String, serde_json::Value>) -> Self {
        let mut params = serde_json::Map::new();
        params.insert("mappings".to_string(), serde_json::json!(mappings));

        Self {
            transform_type: TransformType::EnumMap,
            target_path: None,
            params,
        }
    }

    /// 创建路径重写转换
    pub fn path_rewrite(
        source_pattern: impl Into<String>,
        target_template: impl Into<String>,
    ) -> Self {
        let mut params = serde_json::Map::new();
        params.insert(
            "source_pattern".to_string(),
            serde_json::json!(source_pattern.into()),
        );
        params.insert(
            "target_template".to_string(),
            serde_json::json!(target_template.into()),
        );

        Self {
            transform_type: TransformType::PathRewrite,
            target_path: None,
            params,
        }
    }
}
