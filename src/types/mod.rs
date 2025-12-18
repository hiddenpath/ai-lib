//! 类型定义模块，包含AI库的核心数据结构和类型
//!
//! Types module containing core data structures and type definitions for the AI library.
//!
//! This module defines the fundamental types used throughout ai-lib:
//! - `ChatCompletionRequest` and `ChatCompletionResponse` for API communication
//! - `Message`, `Role`, and `Content` for conversation modeling
//! - `AiLibError` for comprehensive error handling
//! - `Tool` and `FunctionCall` for function calling capabilities

pub mod common;
pub mod error;
pub mod request;
pub mod response;

pub use request::ChatCompletionRequest;
pub use response::ChatCompletionResponse;
pub mod function_call;
pub use common::{Choice, Message, Role};
pub use error::AiLibError;
pub use function_call::{FunctionCall, FunctionCallPolicy, Tool};
/// Usage and UsageStatus are response-level metadata; prefer importing from
/// `ai_lib::types::response::{Usage, UsageStatus}` or the crate root re-exports.
pub use response::{Usage, UsageStatus};

// Additional code may follow
