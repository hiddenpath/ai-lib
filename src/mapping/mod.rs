//! Mapping引擎 - 将StandardRequest转换为Provider-specific格式
//!
//! 这个模块实现了复杂的数据映射引擎，支持：
//! - 路径映射 (path mapping)
//! - 模板替换 (template substitution)
//! - 条件映射 (conditional mapping)
//! - 类型转换 (type conversion)

pub mod engine;
pub mod errors;
pub mod rules;

pub use engine::MappingEngine;
pub use errors::{MappingError, MappingResult};
pub use rules::{ConditionalMapping, MappingRule, ParameterTransform, TransformType};
