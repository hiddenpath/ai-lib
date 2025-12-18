//! API定义模块，包含核心接口和数据结构
//!
//! API definition module containing core interfaces and data structures.
//!
//! This module defines the fundamental traits and types used by all AI provider
//! adapters, ensuring consistent behavior across different vendor implementations.

pub mod chat;

pub use chat::{
    ChatApi, ChatCompletionChunk, ChatProvider, ChoiceDelta, MessageDelta, ModelInfo,
    ModelPermission,
};
