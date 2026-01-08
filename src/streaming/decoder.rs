//! Streaming frame decoders using tokio_util codec
//!
//! Provides SSE, data_lines, and JSON streaming decoders driven by Manifest configuration.

use bytes::BytesMut;
use serde_json::Value;
use std::io;
use tokio_util::codec::Decoder;

use crate::manifest::schema::StreamingDecoder as DecoderConfig;

/// Unified streaming frame decoder that wraps format-specific decoders
pub struct StreamingFrameDecoder {
    format: DecoderFormat,
    delimiter: String,
    prefix: String,
    done_signal: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecoderFormat {
    DataLines,
    AnthropicSse,
    GeminiJson,
    CohereNative,
    ResponsesApi,
}

impl Default for DecoderFormat {
    fn default() -> Self {
        DecoderFormat::DataLines
    }
}

impl StreamingFrameDecoder {
    /// Create decoder from manifest configuration
    pub fn from_config(cfg: Option<&DecoderConfig>, event_format: Option<&str>) -> Self {
        let format = match event_format.unwrap_or("data_lines") {
            "anthropic_sse" => DecoderFormat::AnthropicSse,
            "gemini_json" => DecoderFormat::GeminiJson,
            "cohere_native" => DecoderFormat::CohereNative,
            "responses_api" => DecoderFormat::ResponsesApi,
            _ => DecoderFormat::DataLines,
        };

        let (delimiter, prefix, done_signal) = if let Some(c) = cfg {
            (
                c.delimiter.clone().unwrap_or_else(|| "\n".to_string()),
                c.prefix.clone().unwrap_or_else(|| "data: ".to_string()),
                c.done_signal.clone().unwrap_or_else(|| "[DONE]".to_string()),
            )
        } else {
            ("\n".to_string(), "data: ".to_string(), "[DONE]".to_string())
        };

        Self {
            format,
            delimiter,
            prefix,
            done_signal,
        }
    }

    /// Check if a line indicates stream termination
    pub fn is_done_signal(&self, line: &str) -> bool {
        line.trim() == self.done_signal || line.trim() == format!("data: {}", self.done_signal)
    }

    /// Parse a raw line into a JSON Value (if valid)
    pub fn parse_line(&self, line: &str) -> Option<Value> {
        let trimmed = line.trim();
        if trimmed.is_empty() || self.is_done_signal(trimmed) {
            return None;
        }

        match self.format {
            DecoderFormat::DataLines | DecoderFormat::ResponsesApi => {
                // SSE data: prefix
                let data = if trimmed.starts_with(&self.prefix) {
                    &trimmed[self.prefix.len()..]
                } else if trimmed.starts_with("data:") {
                    trimmed[5..].trim_start()
                } else {
                    trimmed
                };
                serde_json::from_str(data).ok()
            }
            DecoderFormat::AnthropicSse => {
                // Anthropic uses event: / data: pairs
                if trimmed.starts_with("event:") {
                    // Skip event lines, wait for data
                    return None;
                }
                let data = if trimmed.starts_with("data:") {
                    trimmed[5..].trim_start()
                } else {
                    trimmed
                };
                serde_json::from_str(data).ok()
            }
            DecoderFormat::GeminiJson => {
                // Gemini returns raw JSON objects (possibly in array)
                serde_json::from_str(trimmed).ok()
            }
            DecoderFormat::CohereNative => {
                // Cohere returns NDJSON
                serde_json::from_str(trimmed).ok()
            }
        }
    }
}

/// Frame output from decoder
#[derive(Debug, Clone)]
pub enum DecodedFrame {
    /// Valid JSON frame
    Json(Value),
    /// Stream termination signal
    Done,
    /// Empty/skip frame
    Skip,
}

impl Decoder for StreamingFrameDecoder {
    type Item = DecodedFrame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Find delimiter position
        let delim_bytes = self.delimiter.as_bytes();
        let delim_len = delim_bytes.len();

        // Simple line-based decoding
        if let Some(pos) = src.windows(delim_len).position(|w| w == delim_bytes) {
            let line_bytes = src.split_to(pos + delim_len);
            let line = String::from_utf8_lossy(&line_bytes[..pos]).to_string();

            if self.is_done_signal(&line) {
                return Ok(Some(DecodedFrame::Done));
            }

            if let Some(json) = self.parse_line(&line) {
                return Ok(Some(DecodedFrame::Json(json)));
            }

            // Empty or unparseable line
            return Ok(Some(DecodedFrame::Skip));
        }

        // Not enough data yet
        Ok(None)
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Process remaining buffer
        if buf.is_empty() {
            return Ok(None);
        }

        let remaining = String::from_utf8_lossy(buf).to_string();
        buf.clear();

        if self.is_done_signal(&remaining) {
            return Ok(Some(DecodedFrame::Done));
        }

        if let Some(json) = self.parse_line(&remaining) {
            return Ok(Some(DecodedFrame::Json(json)));
        }

        Ok(None)
    }
}

/// Helper to decode a complete SSE event block (for Anthropic's multi-line events)
pub struct SseEventDecoder {
    event_type: Option<String>,
    data_lines: Vec<String>,
}

impl SseEventDecoder {
    pub fn new() -> Self {
        Self {
            event_type: None,
            data_lines: Vec::new(),
        }
    }

    /// Feed a line to the decoder, returns parsed event if complete
    pub fn feed_line(&mut self, line: &str) -> Option<(Option<String>, Value)> {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            // Empty line marks end of event block
            if !self.data_lines.is_empty() {
                let data = self.data_lines.join("\n");
                let event_type = self.event_type.take();
                self.data_lines.clear();

                if let Ok(json) = serde_json::from_str(&data) {
                    return Some((event_type, json));
                }
            }
            return None;
        }

        if let Some(evt) = trimmed.strip_prefix("event:") {
            self.event_type = Some(evt.trim().to_string());
        } else if let Some(data) = trimmed.strip_prefix("data:") {
            self.data_lines.push(data.trim().to_string());
        }

        None
    }

    /// Reset decoder state
    pub fn reset(&mut self) {
        self.event_type = None;
        self.data_lines.clear();
    }
}

impl Default for SseEventDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_lines_decoder() {
        let decoder = StreamingFrameDecoder::from_config(None, Some("data_lines"));

        let json = decoder.parse_line(r#"data: {"id": "1", "content": "hello"}"#);
        assert!(json.is_some());
        let v = json.unwrap();
        assert_eq!(v["id"], "1");
    }

    #[test]
    fn test_done_signal() {
        let decoder = StreamingFrameDecoder::from_config(None, Some("data_lines"));
        assert!(decoder.is_done_signal("[DONE]"));
        assert!(decoder.is_done_signal("data: [DONE]"));
        assert!(!decoder.is_done_signal("data: hello"));
    }

    #[test]
    fn test_sse_event_decoder() {
        let mut decoder = SseEventDecoder::new();

        // Feed event type
        assert!(decoder.feed_line("event: content_block_delta").is_none());
        // Feed data
        assert!(decoder
            .feed_line(r#"data: {"delta": {"text": "hi"}}"#)
            .is_none());
        // Empty line triggers event
        let result = decoder.feed_line("");
        assert!(result.is_some());

        let (event_type, data) = result.unwrap();
        assert_eq!(event_type, Some("content_block_delta".to_string()));
        assert_eq!(data["delta"]["text"], "hi");
    }
}

