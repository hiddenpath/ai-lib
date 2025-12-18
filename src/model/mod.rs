//! 模型管理模块，提供模型选择和解析功能
//!
//! Model management module providing model selection and resolution functionality.
//!
//! This module handles model name resolution, fallback logic, and provider-specific
//! model catalog management.
//!
//! Key components:
//! - `ModelResolver`: Resolves model names and handles fallbacks
//! - `catalog`: Provider model catalogs and metadata
//! - `ModelResolution`: Model resolution results with context

pub mod catalog;
pub mod resolver;

pub use resolver::{ModelResolution, ModelResolutionSource, ModelResolver};
