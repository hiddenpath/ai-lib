//! Server-Sent Events (SSE) 解析模块，支持实时流式响应
//!
//! Server-Sent Events (SSE) parsing module for real-time streaming responses.
//!
//! This module handles parsing of streaming responses from AI providers,
//! supporting both standard SSE format and JSONL streaming protocols.
//!
//! Key components:
//! - `parser`: Standard SSE event parsing
//! - `jsonl_parser`: JSONL streaming protocol parsing

pub mod jsonl_parser;
pub mod parser;
