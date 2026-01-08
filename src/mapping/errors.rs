//! Mapping引擎错误类型

use thiserror::Error;

pub type MappingResult<T> = Result<T, MappingError>;

impl From<MappingError> for crate::AiLibError {
    fn from(err: MappingError) -> Self {
        match err {
            MappingError::PathNotFound { path } => {
                crate::AiLibError::ConfigurationError(format!("Mapping path not found: {}", path))
            }
            MappingError::InvalidPathSyntax { path } => {
                crate::AiLibError::ConfigurationError(format!("Invalid path syntax: {}", path))
            }
            MappingError::TypeConversionFailed { from, to } => {
                crate::AiLibError::ConfigurationError(format!(
                    "Type conversion failed: {} -> {}",
                    from, to
                ))
            }
            MappingError::TemplateRenderingFailed { template } => {
                crate::AiLibError::ConfigurationError(format!(
                    "Template rendering failed: {}",
                    template
                ))
            }
            MappingError::ConditionEvaluationFailed { condition } => {
                crate::AiLibError::ConfigurationError(format!(
                    "Condition evaluation failed: {}",
                    condition
                ))
            }
            MappingError::MissingRequiredParameter { parameter } => {
                crate::AiLibError::ConfigurationError(format!(
                    "Missing required parameter: {}",
                    parameter
                ))
            }
            MappingError::InvalidParameterValue { parameter, value } => {
                crate::AiLibError::ConfigurationError(format!(
                    "Invalid parameter value: {} = {}",
                    parameter, value
                ))
            }
            MappingError::ConfigurationError { message } => {
                crate::AiLibError::ConfigurationError(message)
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum MappingError {
    #[error("Path not found: {path}")]
    PathNotFound { path: String },

    #[error("Invalid path syntax: {path}")]
    InvalidPathSyntax { path: String },

    #[error("Type conversion failed: {from} -> {to}")]
    TypeConversionFailed { from: String, to: String },

    #[error("Template rendering failed: {template}")]
    TemplateRenderingFailed { template: String },

    #[error("Condition evaluation failed: {condition}")]
    ConditionEvaluationFailed { condition: String },

    #[error("Missing required parameter: {parameter}")]
    MissingRequiredParameter { parameter: String },

    #[error("Invalid parameter value: {parameter} = {value}")]
    InvalidParameterValue { parameter: String, value: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}
