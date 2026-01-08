//! Streaming Parser核心实现
//!
//! 解析各种provider的streaming响应，转换为统一的StreamingEvent

use crate::api::ChatCompletionChunk;
use crate::manifest::schema::{ResponseFormat, StreamingConfig};
use crate::streaming::events::{
    PartialContentDelta, PartialToolCall, StreamingEvent, StreamingEventType, ThinkingDelta,
    ToolCallStarted,
};
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::Result as AiLibResult;

/// Streaming解析结果
pub type StreamingResult<T> = AiLibResult<T>;

/// Streaming Parser - 解析不同格式的streaming响应
pub struct StreamingParser {
    /// 响应格式
    response_format: ResponseFormat,

    /// Streaming事件配置
    streaming_config: Option<StreamingConfig>,

    /// 解析状态
    parse_state: ParseState,

    /// 工具调用状态跟踪
    tool_call_states: HashMap<String, ToolCallState>,
}

/// 解析状态
#[derive(Debug, Clone)]
pub enum ParseState {
    /// 初始状态
    Initial,

    /// 正在解析内容
    ParsingContent,

    /// 正在解析工具调用
    ParsingToolCalls,

    /// 完成
    Finished,
}

/// 工具调用状态
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ToolCallState {
    /// 工具调用ID
    id: String,

    /// 函数名
    function_name: Option<String>,

    /// 参数累积
    arguments: String,

    /// 是否已开始
    started: bool,
}

impl StreamingParser {
    /// 创建新的Streaming Parser
    pub fn new(response_format: ResponseFormat, streaming_config: Option<StreamingConfig>) -> Self {
        Self {
            response_format,
            streaming_config,
            parse_state: ParseState::Initial,
            tool_call_states: HashMap::new(),
        }
    }

    /// 解析一行streaming数据
    pub fn parse_line(&mut self, line: &str) -> StreamingResult<Option<StreamingEvent>> {
        if line.trim().is_empty() {
            return Ok(None);
        }

        match self.response_format {
            ResponseFormat::OpenaiStyle => self.parse_openai_line(line),
            ResponseFormat::AnthropicStyle => self.parse_anthropic_line(line),
            ResponseFormat::GeminiStyle => self.parse_gemini_line(line),
            ResponseFormat::Custom(_) => self.parse_custom_line(line),
        }
    }

    /// 解析OpenAI格式的streaming行
    fn parse_openai_line(&mut self, line: &str) -> StreamingResult<Option<StreamingEvent>> {
        // OpenAI格式: data: {"choices": [...], ...}
        if !line.starts_with("data: ") {
            return Ok(None);
        }

        let data = &line[6..]; // 移除 "data: " 前缀

        if data == "[DONE]" {
            return Ok(Some(StreamingEvent::StreamEnd));
        }

        let chunk: ChatCompletionChunk = serde_json::from_str(data).map_err(|e| {
            crate::AiLibError::ParseError(format!("Failed to parse OpenAI chunk: {}", e))
        })?;

        // 转换为StreamingEvent
        let event = StreamingEvent::from(chunk);

        // 更新解析状态
        self.update_parse_state(&event);

        Ok(Some(event))
    }

    /// 解析Anthropic格式的streaming行
    fn parse_anthropic_line(&mut self, line: &str) -> StreamingResult<Option<StreamingEvent>> {
        // Anthropic格式: event: content_block_delta\ndata: {"delta": {"text": "..."}}
        if !line.starts_with("event: ") && !line.starts_with("data: ") {
            return Ok(None);
        }

        // 解析事件类型和数据
        let (event_type, data) = if line.starts_with("event: ") {
            // Event line detected - in real SSE we'd need to read next data: line
            // Simplified: skip event lines, wait for data
            return Ok(None);
        } else {
            ("data", &line[6..])
        };

        let value: Value = serde_json::from_str(data).map_err(|e| {
            crate::AiLibError::ParseError(format!("Failed to parse Anthropic data: {}", e))
        })?;

        let event = match event_type {
            "content_block_delta" => {
                if let Some(text) = value
                    .get("delta")
                    .and_then(|d| d.get("text"))
                    .and_then(|t| t.as_str())
                {
                    StreamingEvent::PartialContentDelta(PartialContentDelta {
                        delta: text.to_string(),
                        choice_index: 0,
                        finish_reason: None,
                        candidate_index: None,
                    })
                } else {
                    return Ok(None);
                }
            }
            "thinking" => {
                if let Some(thought) = value.get("thinking").and_then(|t| t.as_str()) {
                    StreamingEvent::ThinkingDelta(ThinkingDelta {
                        thinking: thought.to_string(),
                        signature: value
                            .get("signature")
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string()),
                    })
                } else {
                    return Ok(None);
                }
            }
            "tool_use" => self.parse_anthropic_tool_call(&value)?,
            _ => return Ok(None),
        };

        self.update_parse_state(&event);
        Ok(Some(event))
    }

    /// 解析Gemini格式的streaming行
    fn parse_gemini_line(&mut self, line: &str) -> StreamingResult<Option<StreamingEvent>> {
        // Gemini格式解析
        let value: Value = serde_json::from_str(line).map_err(|e| {
            crate::AiLibError::ParseError(format!("Failed to parse Gemini line: {}", e))
        })?;

        // 检查是否有候选结果
        if let Some(candidates) = value.get("candidates").and_then(|c| c.as_array()) {
            for candidate in candidates {
                if let Some(content) = candidate.get("content") {
                    if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                        for part in parts {
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                let event =
                                    StreamingEvent::PartialContentDelta(PartialContentDelta {
                                        delta: text.to_string(),
                                        choice_index: 0,
                                        finish_reason: candidate
                                            .get("finishReason")
                                            .and_then(|f| f.as_str())
                                            .map(|s| s.to_string()),
                                        candidate_index: None,
                                    });

                                self.update_parse_state(&event);
                                return Ok(Some(event));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// 解析自定义格式的streaming行
    fn parse_custom_line(&mut self, line: &str) -> StreamingResult<Option<StreamingEvent>> {
        // 自定义格式，根据配置解析
        if let Some(_config) = &self.streaming_config {
            // 使用配置的解析规则（TODO: 使用 StreamProcessor 替代）
            let value: Value = serde_json::from_str(line).map_err(|e| {
                crate::AiLibError::ParseError(format!("Failed to parse custom line: {}", e))
            })?;

            // 根据配置决定如何解析
            // 这里是简化的实现
            if let Some(text) = value.get("text").and_then(|t| t.as_str()) {
                let event = StreamingEvent::PartialContentDelta(PartialContentDelta {
                    delta: text.to_string(),
                    choice_index: 0,
                    finish_reason: None,
                    candidate_index: None,
                });

                self.update_parse_state(&event);
                return Ok(Some(event));
            }
        }

        Ok(None)
    }

    /// 解析Anthropic工具调用
    fn parse_anthropic_tool_call(&mut self, value: &Value) -> StreamingResult<StreamingEvent> {
        if let Some(tool_use) = value.get("tool_use") {
            let tool_call_id = tool_use
                .get("id")
                .and_then(|id| id.as_str())
                .unwrap_or("")
                .to_string();

            // 检查是否是新的工具调用
            if !self.tool_call_states.contains_key(&tool_call_id) {
                let state = ToolCallState {
                    id: tool_call_id.clone(),
                    function_name: tool_use
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string()),
                    arguments: String::new(),
                    started: false,
                };
                self.tool_call_states.insert(tool_call_id.clone(), state);

                // 发送工具调用开始事件
                return Ok(StreamingEvent::ToolCallStarted(ToolCallStarted {
                    tool_call_id: tool_call_id.clone(),
                    tool_name: tool_use
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string(),
                    initial_arguments: None,
                }));
            }

            // 更新工具调用状态
            if let Some(state) = self.tool_call_states.get_mut(&tool_call_id) {
                if let Some(partial_json) = tool_use.get("partial_json").and_then(|p| p.as_str()) {
                    state.arguments.push_str(partial_json);
                }

                return Ok(StreamingEvent::PartialToolCall(PartialToolCall {
                    tool_call_id: tool_call_id.clone(),
                    function_name_delta: None,
                    arguments_delta: Some(state.arguments.clone()),
                    tool_name: state.function_name.clone(),
                    candidate_index: None,
                }));
            }
        }

        Err(crate::AiLibError::ParseError(
            "Invalid Anthropic tool call format".to_string(),
        ))
    }

    /// 更新解析状态
    fn update_parse_state(&mut self, event: &StreamingEvent) {
        match event.event_type() {
            StreamingEventType::PartialContentDelta => {
                self.parse_state = ParseState::ParsingContent;
            }
            StreamingEventType::PartialToolCall | StreamingEventType::ToolCallStarted => {
                self.parse_state = ParseState::ParsingToolCalls;
            }
            StreamingEventType::FinalCandidate
            | StreamingEventType::StreamEnd
            | StreamingEventType::Error => {
                self.parse_state = ParseState::Finished;

                // 清理工具调用状态
                self.tool_call_states.clear();
            }
            _ => {}
        }
    }

    /// 获取当前解析状态
    pub fn parse_state(&self) -> &ParseState {
        &self.parse_state
    }

    /// 重置解析器状态
    pub fn reset(&mut self) {
        self.parse_state = ParseState::Initial;
        self.tool_call_states.clear();
    }

    /// 检查是否完成
    pub fn is_finished(&self) -> bool {
        matches!(self.parse_state, ParseState::Finished)
    }
}

/// 将原始字节流转换为StreamingEvent流的适配器
pub struct StreamingEventStream<S> {
    /// 底层字节流
    stream: S,

    /// Streaming解析器
    parser: StreamingParser,

    /// 缓冲区
    buffer: String,
}

impl<S> StreamingEventStream<S>
where
    S: Stream<Item = AiLibResult<Vec<u8>>> + Unpin,
{
    /// 创建新的StreamingEventStream
    pub fn new(stream: S, parser: StreamingParser) -> Self {
        Self {
            stream,
            parser,
            buffer: String::new(),
        }
    }
}

impl<S> Stream for StreamingEventStream<S>
where
    S: Stream<Item = AiLibResult<Vec<u8>>> + Unpin,
{
    type Item = AiLibResult<StreamingEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut();

        loop {
            // 首先尝试从缓冲区解析事件
            if let Some(newline_pos) = this.buffer.find('\n') {
                let line = this.buffer[..newline_pos].to_string();
                this.buffer = this.buffer[newline_pos + 1..].to_string();

                match this.parser.parse_line(&line) {
                    Ok(Some(event)) => return Poll::Ready(Some(Ok(event))),
                    Ok(None) => continue, // 继续处理下一行
                    Err(e) => return Poll::Ready(Some(Err(e))),
                }
            }

            // 如果缓冲区没有完整行，从流中读取更多数据
            match Pin::new(&mut this.stream).poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    let chunk = String::from_utf8_lossy(&bytes);
                    this.buffer.push_str(&chunk);

                    // 如果读到EOF但缓冲区还有数据，继续处理
                    if chunk.is_empty() && !this.buffer.is_empty() {
                        continue;
                    }
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => {
                    // 流结束，处理剩余缓冲区
                    if !this.buffer.is_empty() {
                        let line = this.buffer.clone();
                        this.buffer.clear();

                        match this.parser.parse_line(&line) {
                            Ok(Some(event)) => return Poll::Ready(Some(Ok(event))),
                            Ok(None) => {}
                            Err(e) => return Poll::Ready(Some(Err(e))),
                        }
                    }

                    // 发送结束事件
                    if !this.parser.is_finished() {
                        this.parser.update_parse_state(&StreamingEvent::StreamEnd);
                        return Poll::Ready(Some(Ok(StreamingEvent::StreamEnd)));
                    }

                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::schema::ResponseFormat;

    #[test]
    fn test_openai_done_parsing() {
        let mut parser = StreamingParser::new(ResponseFormat::OpenaiStyle, None);
        let result = parser.parse_line("data: [DONE]");
        assert!(matches!(result, Ok(Some(StreamingEvent::StreamEnd))));
    }

    #[test]
    fn test_empty_line_ignored() {
        let mut parser = StreamingParser::new(ResponseFormat::OpenaiStyle, None);
        let result = parser.parse_line("");
        assert_eq!(result, Ok(None));

        let result = parser.parse_line("   ");
        assert_eq!(result, Ok(None));
    }

    #[test]
    fn test_parser_state_tracking() {
        let mut parser = StreamingParser::new(ResponseFormat::OpenaiStyle, None);

        // 初始状态
        assert!(matches!(parser.parse_state(), ParseState::Initial));

        // 模拟内容增量
        let content_event = StreamingEvent::PartialContentDelta(PartialContentDelta {
            delta: "Hello".to_string(),
            choice_index: 0,
            finish_reason: None,
            candidate_index: None,
        });
        parser.update_parse_state(&content_event);
        assert!(matches!(parser.parse_state(), ParseState::ParsingContent));

        // 模拟完成
        let final_event =
            StreamingEvent::FinalCandidate(crate::streaming::events::FinalCandidate {
                choices: vec![],
                usage: None,
                finish_reason: Some("stop".to_string()),
                model: None,
            });
        parser.update_parse_state(&final_event);
        assert!(matches!(parser.parse_state(), ParseState::Finished));
        assert!(parser.is_finished());
    }
}
