#![cfg(feature = "unified_sse")]

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeltaPayload {
    role: Option<String>,
    content: Option<String>,
}

fn reference_parse_events(stream: &str) -> Vec<DeltaPayload> {
    // Split by blank lines, for each data: JSON parse and extract delta.role/content
    let mut payloads = Vec::new();
    for block in stream.split("\n\n") {
        for line in block.lines() {
            let line = line.trim();
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" { continue; }
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    let choices = json.get("choices").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    if let Some(first) = choices.get(0) {
                        let delta = first.get("delta").cloned().unwrap_or(serde_json::json!({}));
                        let role = delta.get("role").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let content = delta.get("content").and_then(|v| v.as_str()).map(|s| s.to_string());
                        payloads.push(DeltaPayload { role, content });
                    }
                }
            }
        }
    }
    payloads
}

#[test]
fn md5_and_sequence_consistency_with_unified_parser() -> Result<(), Box<dyn std::error::Error>> {
    use md5;
    // Three concatenated events (non-ASCII and large payload included)
    let json1 = serde_json::json!({
        "id": "chatcmpl-1",
        "object": "chat.completion.chunk",
        "created": 1,
        "model": "m",
        "choices": [{
            "index": 0,
            "delta": {"role": "assistant", "content": "你好，"},
            "finish_reason": serde_json::Value::Null
        }]
    });
    let large = "a".repeat(1024);
    let json2 = serde_json::json!({
        "id": "chatcmpl-2",
        "object": "chat.completion.chunk",
        "created": 2,
        "model": "m",
        "choices": [{
            "index": 0,
            "delta": {"content": large},
            "finish_reason": serde_json::Value::Null
        }]
    });
    let json3 = serde_json::json!({
        "id": "chatcmpl-3",
        "object": "chat.completion.chunk",
        "created": 3,
        "model": "m",
        "choices": [{
            "index": 0,
            "delta": {"content": "世界！"},
            "finish_reason": serde_json::Value::Null
        }]
    });
    let event1 = format!("id: 1\ndata: {}\n\n", json1.to_string());
    let event2 = format!("id: 2\ndata: {}\n\n", json2.to_string());
    let event3 = format!("id: 3\ndata: {}\n\n", json3.to_string());
    let stream = format!("{}{}{}", event1, event2, event3);

    // Unified parser path
    let mut buffer = Vec::new();
    let mut unified_payloads: Vec<DeltaPayload> = Vec::new();
    buffer.extend_from_slice(stream.as_bytes());
    while let Some(end) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let bytes = buffer.drain(..end).collect::<Vec<_>>();
        let text = std::str::from_utf8(&bytes)?;
        if let Some(parsed_res) = ai_lib::sse::parser::parse_sse_event(text) {
            let chunk_opt = parsed_res?;
            if let Some(chunk) = chunk_opt {
                let role = chunk.choices.get(0).and_then(|ch| ch.delta.role.clone()).map(|r| match r {
                    ai_lib::types::Role::Assistant => "assistant".to_string(),
                    ai_lib::types::Role::User => "user".to_string(),
                    ai_lib::types::Role::System => "system".to_string(),
                });
                let content = chunk.choices.get(0).and_then(|ch| ch.delta.content.clone());
                unified_payloads.push(DeltaPayload { role, content });
            }
        }
    }

    // Reference path
    let ref_payloads = reference_parse_events(&stream);

    // Sequence equality: same number and same (role, content)
    assert_eq!(unified_payloads, ref_payloads);

    // Whole-stream MD5 (joined content only)
    let uni_joined = unified_payloads.iter().filter_map(|p| p.content.as_ref()).cloned().collect::<Vec<_>>().join("\n");
    let ref_joined = ref_payloads.iter().filter_map(|p| p.content.as_ref()).cloned().collect::<Vec<_>>().join("\n");
    let h_uni = format!("{:x}", md5::compute(uni_joined.as_bytes()));
    let h_ref = format!("{:x}", md5::compute(ref_joined.as_bytes()));
    assert_eq!(h_uni, h_ref);
    Ok(())
}


