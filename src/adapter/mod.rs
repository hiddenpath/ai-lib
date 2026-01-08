//! 动态适配器系统
//!
//! 这个模块实现了基于YAML配置的动态AI适配器，
//! 能够零代码扩展新AI提供商。

pub mod dynamic;

pub use dynamic::ConfigDrivenAdapter;
