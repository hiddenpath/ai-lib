//! Streaming Parser - 统一处理各种streaming事件
//!
//! 支持：
//! - OpenAI streaming format
//! - Anthropic streaming format
//! - Gemini streaming format
//! - Cohere streaming format
//! - 统一事件模型与算子驱动管线

pub mod decoder;
pub mod events;
pub mod parser;
pub mod pipeline;

pub use decoder::{DecodedFrame, DecoderFormat, SseEventDecoder, StreamingFrameDecoder};
pub use events::{StreamingEvent, StreamingEventType};
pub use parser::{StreamingParser, StreamingResult};
pub use pipeline::StreamProcessor;
