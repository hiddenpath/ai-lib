use crate::api::{ChatCompletionChunk, ChoiceDelta, MessageDelta};
use crate::types::AiLibError;
use crate::types::Role;

/// Find SSE event boundary supporting both LF+LF and CRLF+CRLF
pub fn find_event_boundary(buffer: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 1 < buffer.len() {
        // LF LF
        if buffer[i] == b'\n' && buffer[i + 1] == b'\n' {
            return Some(i + 2);
        }
        // CR LF CR LF
        if i + 3 < buffer.len()
            && buffer[i] == b'\r'
            && buffer[i + 1] == b'\n'
            && buffer[i + 2] == b'\r'
            && buffer[i + 3] == b'\n'
        {
            return Some(i + 4);
        }
        i += 1;
    }
    None
}

/// Parse a raw SSE event text into an optional chunk.
/// Returns Ok(None) for [DONE] signals.
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
