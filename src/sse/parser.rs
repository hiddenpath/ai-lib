use crate::api::{ChatCompletionChunk, ChoiceDelta, MessageDelta};
use crate::manifest::schema::StreamingDecoder as DecoderConfig;
use crate::types::AiLibError;
use crate::types::Role;

/// Find SSE event boundary supporting both LF+LF and CRLF+CRLF
pub fn find_event_boundary(buffer: &[u8]) -> Option<usize> {
    find_event_boundary_with_delim(buffer, None)
}

/// Find SSE event boundary with configurable delimiter
pub fn find_event_boundary_with_delim(buffer: &[u8], delimiter: Option<&str>) -> Option<usize> {
    let delim = delimiter.unwrap_or("\n\n");

    // Check for exact delimiter match
    if let Some(pos) = find_bytes(buffer, delim.as_bytes()) {
        return Some(pos + delim.len());
    }

    // Fallback: CR LF CR LF for SSE
    if delimiter.is_none() {
        let mut i = 0;
        while i + 3 < buffer.len() {
            if buffer[i] == b'\r'
                && buffer[i + 1] == b'\n'
                && buffer[i + 2] == b'\r'
                && buffer[i + 3] == b'\n'
            {
                return Some(i + 4);
            }
            i += 1;
        }
    }
    None
}

/// Find byte pattern in buffer
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
}

/// Decoder format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecoderFormat {
    /// Standard SSE with data: prefix (OpenAI, etc.)
    Sse,
    /// Anthropic SSE with event: and data: pairs
    AnthropicSse,
    /// Gemini JSON streaming (array elements or NDJSON)
    GeminiJson,
    /// Cohere NDJSON
    CohereNdjson,
    /// OpenAI Responses API SSE
    ResponsesApi,
}

impl DecoderFormat {
    pub fn from_str(s: &str) -> Self {
        match s {
            "anthropic_sse" => DecoderFormat::AnthropicSse,
            "gemini_json" => DecoderFormat::GeminiJson,
            "cohere_native" | "cohere_ndjson" => DecoderFormat::CohereNdjson,
            "responses_api" => DecoderFormat::ResponsesApi,
            _ => DecoderFormat::Sse,
        }
    }
}

/// Parse a raw SSE event text into an optional chunk.
/// Returns Ok(None) for \[DONE\] signals.
pub fn parse_sse_event(
    event_text: &str,
) -> Option<Result<Option<ChatCompletionChunk>, AiLibError>> {
    for line in event_text.lines() {
        let line = line.trim();
        if let Some(stripped) = line.strip_prefix("data: ") {
            let data = stripped;
            if data == "[DONE]" {
                return Some(Ok(None));
            }
            return Some(parse_chunk_data(data));
        }
    }
    None
}

/// Parse OpenAI-like chunk JSON into a chunk struct
pub fn parse_chunk_data(data: &str) -> Result<Option<ChatCompletionChunk>, AiLibError> {
    let json: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| AiLibError::ProviderError(format!("JSON parse error: {}", e)))?;

    let mut choices_vec: Vec<ChoiceDelta> = Vec::new();
    if let Some(arr) = json["choices"].as_array() {
        for (index, choice) in arr.iter().enumerate() {
            let delta = &choice["delta"];
            let role = delta.get("role").and_then(|v| v.as_str()).map(|r| match r {
                "assistant" => Role::Assistant,
                "user" => Role::User,
                "system" => Role::System,
                _ => Role::Assistant,
            });
            let content = delta
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let md = MessageDelta { role, content };
            let cd = ChoiceDelta {
                index: index as u32,
                delta: md,
                finish_reason: choice
                    .get("finish_reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            };
            choices_vec.push(cd);
        }
    }

    Ok(Some(ChatCompletionChunk {
        id: json["id"].as_str().unwrap_or_default().to_string(),
        object: json["object"]
            .as_str()
            .unwrap_or("chat.completion.chunk")
            .to_string(),
        created: json["created"].as_u64().unwrap_or(0),
        model: json["model"].as_str().unwrap_or_default().to_string(),
        choices: choices_vec,
    }))
}

/// Configuration-driven SSE parser
pub struct ConfigDrivenParser {
    format: DecoderFormat,
    delimiter: String,
    prefix: String,
    done_signal: String,
}

impl ConfigDrivenParser {
    /// Create parser from manifest decoder config
    pub fn from_config(cfg: Option<&DecoderConfig>, event_format: Option<&str>) -> Self {
        let format = DecoderFormat::from_str(event_format.unwrap_or("sse"));

        let (delimiter, prefix, done_signal) = if let Some(c) = cfg {
            (
                c.delimiter.clone().unwrap_or_else(|| "\n\n".to_string()),
                c.prefix.clone().unwrap_or_else(|| "data: ".to_string()),
                c.done_signal.clone().unwrap_or_else(|| "[DONE]".to_string()),
            )
        } else {
            ("\n\n".to_string(), "data: ".to_string(), "[DONE]".to_string())
        };

        Self {
            format,
            delimiter,
            prefix,
            done_signal,
        }
    }

    /// Get the delimiter for this parser
    pub fn delimiter(&self) -> &str {
        &self.delimiter
    }

    /// Check if a line is a done signal
    pub fn is_done(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed == self.done_signal
            || trimmed == format!("data: {}", self.done_signal)
            || trimmed == format!("data:{}", self.done_signal)
    }

    /// Parse a raw event text into a JSON Value (format-aware)
    pub fn parse_to_json(&self, event_text: &str) -> Option<serde_json::Value> {
        let trimmed = event_text.trim();
        if trimmed.is_empty() || self.is_done(trimmed) {
            return None;
        }

        match self.format {
            DecoderFormat::Sse | DecoderFormat::ResponsesApi => {
                self.parse_sse_lines(event_text)
            }
            DecoderFormat::AnthropicSse => {
                self.parse_anthropic_sse(event_text)
            }
            DecoderFormat::GeminiJson => {
                self.parse_gemini_json(event_text)
            }
            DecoderFormat::CohereNdjson => {
                self.parse_ndjson_line(event_text)
            }
        }
    }

    /// Parse standard SSE lines
    fn parse_sse_lines(&self, text: &str) -> Option<serde_json::Value> {
        for line in text.lines() {
            let line = line.trim();
            // Try configured prefix first
            if let Some(data) = line.strip_prefix(&self.prefix) {
                if self.is_done(data) {
                    return None;
                }
                return serde_json::from_str(data).ok();
            }
            // Fallback to data: prefix
            if let Some(data) = line.strip_prefix("data:") {
                let data = data.trim_start();
                if self.is_done(data) {
                    return None;
                }
                return serde_json::from_str(data).ok();
            }
        }
        None
    }

    /// Parse Anthropic SSE with event: and data: pairs
    fn parse_anthropic_sse(&self, text: &str) -> Option<serde_json::Value> {
        let mut _event_type: Option<String> = None;
        let mut data_content: Option<String> = None;

        for line in text.lines() {
            let line = line.trim();
            if let Some(evt) = line.strip_prefix("event:") {
                _event_type = Some(evt.trim().to_string());
            } else if let Some(data) = line.strip_prefix("data:") {
                let data = data.trim();
                if self.is_done(data) {
                    return None;
                }
                data_content = Some(data.to_string());
            }
        }

        data_content.and_then(|d| serde_json::from_str(&d).ok())
    }

    /// Parse Gemini JSON streaming (handles both array chunks and NDJSON)
    fn parse_gemini_json(&self, text: &str) -> Option<serde_json::Value> {
        let trimmed = text.trim();

        // Handle streaming array format: data: {...} or just {...}
        if let Some(data) = trimmed.strip_prefix("data:") {
            return serde_json::from_str(data.trim()).ok();
        }

        // Direct JSON object
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            return serde_json::from_str(trimmed).ok();
        }

        None
    }

    /// Parse NDJSON line (Cohere style)
    fn parse_ndjson_line(&self, text: &str) -> Option<serde_json::Value> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return None;
        }
        serde_json::from_str(trimmed).ok()
    }

    /// Parse event and convert to ChatCompletionChunk (OpenAI-compatible)
    pub fn parse_to_chunk(&self, event_text: &str) -> Option<Result<Option<ChatCompletionChunk>, AiLibError>> {
        if self.is_done(event_text) {
            return Some(Ok(None));
        }

        let json = self.parse_to_json(event_text)?;

        // Convert to OpenAI-compatible chunk format
        Some(self.json_to_chunk(&json))
    }

    /// Convert parsed JSON to ChatCompletionChunk (normalizes different formats)
    fn json_to_chunk(&self, json: &serde_json::Value) -> Result<Option<ChatCompletionChunk>, AiLibError> {
        match self.format {
            DecoderFormat::Sse | DecoderFormat::ResponsesApi => {
                // Already OpenAI format
                parse_chunk_data(&json.to_string())
            }
            DecoderFormat::AnthropicSse => {
                self.anthropic_to_chunk(json)
            }
            DecoderFormat::GeminiJson => {
                self.gemini_to_chunk(json)
            }
            DecoderFormat::CohereNdjson => {
                self.cohere_to_chunk(json)
            }
        }
    }

    /// Convert Anthropic event JSON to ChatCompletionChunk
    fn anthropic_to_chunk(&self, json: &serde_json::Value) -> Result<Option<ChatCompletionChunk>, AiLibError> {
        let event_type = json["type"].as_str().unwrap_or("");

        match event_type {
            "content_block_delta" => {
                let delta_type = json["delta"]["type"].as_str().unwrap_or("");
                let content = match delta_type {
                    "text_delta" => json["delta"]["text"].as_str().map(|s| s.to_string()),
                    _ => None,
                };
                let index = json["index"].as_u64().unwrap_or(0) as u32;

                Ok(Some(ChatCompletionChunk {
                    id: String::new(),
                    object: "chat.completion.chunk".to_string(),
                    created: 0,
                    model: String::new(),
                    choices: vec![ChoiceDelta {
                        index,
                        delta: MessageDelta {
                            role: None,
                            content,
                        },
                        finish_reason: None,
                    }],
                }))
            }
            "message_stop" | "message_delta" => {
                let stop_reason = json["delta"]["stop_reason"].as_str().map(|s| s.to_string());
                Ok(Some(ChatCompletionChunk {
                    id: String::new(),
                    object: "chat.completion.chunk".to_string(),
                    created: 0,
                    model: String::new(),
                    choices: vec![ChoiceDelta {
                        index: 0,
                        delta: MessageDelta {
                            role: None,
                            content: None,
                        },
                        finish_reason: stop_reason,
                    }],
                }))
            }
            _ => Ok(None), // Skip other event types
        }
    }

    /// Convert Gemini JSON to ChatCompletionChunk
    fn gemini_to_chunk(&self, json: &serde_json::Value) -> Result<Option<ChatCompletionChunk>, AiLibError> {
        let candidates = json["candidates"].as_array();

        let mut choices = Vec::new();
        if let Some(cands) = candidates {
            for (idx, cand) in cands.iter().enumerate() {
                let parts = cand["content"]["parts"].as_array();
                let content = parts
                    .and_then(|p| p.first())
                    .and_then(|part| part["text"].as_str())
                    .map(|s| s.to_string());

                let finish_reason = cand["finishReason"].as_str().map(|s| s.to_string());

                choices.push(ChoiceDelta {
                    index: idx as u32,
                    delta: MessageDelta {
                        role: Some(Role::Assistant),
                        content,
                    },
                    finish_reason,
                });
            }
        }

        if choices.is_empty() {
            return Ok(None);
        }

        Ok(Some(ChatCompletionChunk {
            id: String::new(),
            object: "chat.completion.chunk".to_string(),
            created: 0,
            model: String::new(),
            choices,
        }))
    }

    /// Convert Cohere NDJSON to ChatCompletionChunk
    fn cohere_to_chunk(&self, json: &serde_json::Value) -> Result<Option<ChatCompletionChunk>, AiLibError> {
        let event_type = json["event_type"].as_str().unwrap_or("");

        match event_type {
            "text-generation" => {
                let text = json["text"].as_str().map(|s| s.to_string());
                Ok(Some(ChatCompletionChunk {
                    id: String::new(),
                    object: "chat.completion.chunk".to_string(),
                    created: 0,
                    model: String::new(),
                    choices: vec![ChoiceDelta {
                        index: 0,
                        delta: MessageDelta {
                            role: None,
                            content: text,
                        },
                        finish_reason: None,
                    }],
                }))
            }
            "stream-end" => {
                let reason = json["finish_reason"].as_str().map(|s| s.to_string());
                Ok(Some(ChatCompletionChunk {
                    id: String::new(),
                    object: "chat.completion.chunk".to_string(),
                    created: 0,
                    model: String::new(),
                    choices: vec![ChoiceDelta {
                        index: 0,
                        delta: MessageDelta {
                            role: None,
                            content: None,
                        },
                        finish_reason: reason,
                    }],
                }))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_openai_sse() {
        let parser = ConfigDrivenParser::from_config(None, Some("sse"));
        let event = r#"data: {"id":"chatcmpl-123","choices":[{"index":0,"delta":{"content":"Hello"}}]}"#;
        let json = parser.parse_to_json(event);
        assert!(json.is_some());
        assert_eq!(json.unwrap()["choices"][0]["delta"]["content"], "Hello");
    }

    #[test]
    fn test_parse_anthropic_sse() {
        let parser = ConfigDrivenParser::from_config(None, Some("anthropic_sse"));
        let event = "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}";
        let json = parser.parse_to_json(event);
        assert!(json.is_some());
        assert_eq!(json.unwrap()["delta"]["text"], "Hi");
    }

    #[test]
    fn test_parse_gemini_json() {
        let parser = ConfigDrivenParser::from_config(None, Some("gemini_json"));
        let event = r#"{"candidates":[{"content":{"parts":[{"text":"Hello"}]}}]}"#;
        let json = parser.parse_to_json(event);
        assert!(json.is_some());
        assert_eq!(json.unwrap()["candidates"][0]["content"]["parts"][0]["text"], "Hello");
    }

    #[test]
    fn test_parse_cohere_ndjson() {
        let parser = ConfigDrivenParser::from_config(None, Some("cohere_ndjson"));
        let event = r#"{"event_type":"text-generation","text":"Hello"}"#;
        let json = parser.parse_to_json(event);
        assert!(json.is_some());
        assert_eq!(json.unwrap()["text"], "Hello");
    }

    #[test]
    fn test_done_signal() {
        let parser = ConfigDrivenParser::from_config(None, None);
        assert!(parser.is_done("[DONE]"));
        assert!(parser.is_done("data: [DONE]"));
        assert!(!parser.is_done("data: hello"));
    }
}
