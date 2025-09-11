use crate::api::{ChatCompletionChunk, ChoiceDelta, MessageDelta};
use crate::types::AiLibError;
use crate::types::Role;
use serde::{Deserialize, Serialize};

/// JSONL streaming protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JsonlMessage {
    /// Incremental content delta
    #[serde(rename = "delta")]
    Delta {
        /// The content chunk
        data: String,
    },
    /// Final result with complete data
    #[serde(rename = "final")]
    Final {
        /// Complete response data
        data: FinalData,
    },
}

/// Final response data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalData {
    /// Complete answer text
    pub answer: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: Option<f64>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// JSONL parser for the new streaming protocol
pub struct JsonlParser {
    accumulated_content: String,
}

impl JsonlParser {
    pub fn new() -> Self {
        Self {
            accumulated_content: String::new(),
        }
    }

    /// Parse a single JSONL line
    pub fn parse_line(&mut self, line: &str) -> Result<Option<ChatCompletionChunk>, AiLibError> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(None);
        }

        let message: JsonlMessage = serde_json::from_str(line)
            .map_err(|e| AiLibError::ProviderError(format!("JSONL parse error: {}", e)))?;

        match message {
            JsonlMessage::Delta { data } => {
                self.accumulated_content.push_str(&data);

                // Create a chunk for the delta
                let delta = ChoiceDelta {
                    index: 0,
                    delta: MessageDelta {
                        role: Some(Role::Assistant),
                        content: Some(data),
                    },
                    finish_reason: None,
                };

                Ok(Some(ChatCompletionChunk {
                    id: "jsonl_delta".to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: 0,
                    model: "jsonl_streaming".to_string(),
                    choices: vec![delta],
                }))
            }
            JsonlMessage::Final { data } => {
                // Create a final chunk with the complete answer
                let delta = ChoiceDelta {
                    index: 0,
                    delta: MessageDelta {
                        role: Some(Role::Assistant),
                        content: Some(data.answer),
                    },
                    finish_reason: Some("stop".to_string()),
                };

                Ok(Some(ChatCompletionChunk {
                    id: "jsonl_final".to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: 0,
                    model: "jsonl_streaming".to_string(),
                    choices: vec![delta],
                }))
            }
        }
    }

    /// Get accumulated content so far
    pub fn accumulated_content(&self) -> &str {
        &self.accumulated_content
    }

    /// Reset accumulated content
    pub fn reset(&mut self) {
        self.accumulated_content.clear();
    }
}

impl Default for JsonlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert traditional SSE format to JSONL format
pub fn convert_sse_to_jsonl(sse_data: &str) -> Result<String, AiLibError> {
    let mut jsonl_lines = Vec::new();

    // Parse SSE events
    for line in sse_data.lines() {
        let line = line.trim();
        if let Some(stripped) = line.strip_prefix("data: ") {
            if stripped == "[DONE]" {
                continue;
            }

            // Try to parse as JSON and extract content
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(stripped) {
                if let Some(choices) = json["choices"].as_array() {
                    for choice in choices {
                        if let Some(delta) = choice["delta"].as_object() {
                            if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                                if !content.is_empty() {
                                    let delta_msg = JsonlMessage::Delta {
                                        data: content.to_string(),
                                    };
                                    jsonl_lines.push(serde_json::to_string(&delta_msg).map_err(
                                        |e| {
                                            AiLibError::ProviderError(format!(
                                                "JSON serialization error: {}",
                                                e
                                            ))
                                        },
                                    )?);
                                }
                            }

                            // Check for finish reason
                            if let Some(finish_reason) =
                                choice.get("finish_reason").and_then(|v| v.as_str())
                            {
                                if finish_reason == "stop" {
                                    // Create final message with accumulated content
                                    let final_msg = JsonlMessage::Final {
                                        data: FinalData {
                                            answer: "".to_string(), // Would need to accumulate
                                            confidence: None,
                                            metadata: None,
                                        },
                                    };
                                    jsonl_lines.push(serde_json::to_string(&final_msg).map_err(
                                        |e| {
                                            AiLibError::ProviderError(format!(
                                                "JSON serialization error: {}",
                                                e
                                            ))
                                        },
                                    )?);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(jsonl_lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonl_delta_parsing() {
        let mut parser = JsonlParser::new();

        let delta_line = r#"{"type":"delta","data":"你"}"#;
        let chunk = parser.parse_line(delta_line).unwrap();

        assert!(chunk.is_some());
        let chunk = chunk.unwrap();
        assert_eq!(chunk.choices[0].delta.content, Some("你".to_string()));
        assert_eq!(parser.accumulated_content(), "你");
    }

    #[test]
    fn test_jsonl_final_parsing() {
        let mut parser = JsonlParser::new();

        // First add some deltas
        parser
            .parse_line(r#"{"type":"delta","data":"你"}"#)
            .unwrap();
        parser
            .parse_line(r#"{"type":"delta","data":"好"}"#)
            .unwrap();
        parser
            .parse_line(r#"{"type":"delta","data":"呀"}"#)
            .unwrap();

        // Then final
        let final_line = r#"{"type":"final","data":{"answer":"你好呀","confidence":0.98}}"#;
        let chunk = parser.parse_line(final_line).unwrap();

        assert!(chunk.is_some());
        let chunk = chunk.unwrap();
        assert_eq!(chunk.choices[0].delta.content, Some("你好呀".to_string()));
        assert_eq!(chunk.choices[0].finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_jsonl_multiple_deltas() {
        let mut parser = JsonlParser::new();

        let deltas = vec!["你", "好", "呀"];
        for delta in deltas {
            let line = format!(r#"{{"type":"delta","data":"{}"}}"#, delta);
            let chunk = parser.parse_line(&line).unwrap();
            assert!(chunk.is_some());
        }

        assert_eq!(parser.accumulated_content(), "你好呀");
    }

    #[test]
    fn test_sse_to_jsonl_conversion() {
        let sse_data = r#"data: {"id":"1","choices":[{"delta":{"content":"你"}}]}
data: {"id":"2","choices":[{"delta":{"content":"好"}}]}
data: {"id":"3","choices":[{"delta":{"content":"呀"}}]}
data: {"id":"4","choices":[{"delta":{},"finish_reason":"stop"}]}"#;

        let jsonl = convert_sse_to_jsonl(sse_data).unwrap();
        let lines: Vec<&str> = jsonl.lines().collect();

        assert_eq!(lines.len(), 4); // 3 deltas + 1 final
        assert!(lines[0].contains(r#""type":"delta""#));
        assert!(lines[0].contains(r#""data":"你""#));
    }
}
