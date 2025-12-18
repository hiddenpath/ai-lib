//! Golden test cases for streaming functionality
//!
//! These tests ensure consistent behavior across different streaming formats
//! and edge cases that have been encountered in production.

use ai_lib::AiLibError;

#[cfg(feature = "unified_sse")]
#[test]
fn golden_sse_multibyte_utf8_boundary() -> Result<(), AiLibError> {
    // Test UTF-8 multibyte characters split across SSE event boundaries
    let chinese_text = "你好世界";
    let event1 = format!(
        r#"data: {{"id":"1","object":"chat.completion.chunk","created":0,"model":"test","choices":[{{"delta":{{"role":"assistant","content":"{}"}}}}]}}"#,
        "你好" // First two characters
    ) + "\n\n";
    let event2 = format!(
        r#"data: {{"id":"2","object":"chat.completion.chunk","created":0,"model":"test","choices":[{{"delta":{{"content":"{}"}}}}]}}"#,
        "世界" // Last two characters
    ) + "\n\n";

    let mut buffer = [event1.as_bytes(), event2.as_bytes()].concat();
    let mut content = String::new();

    while let Some(boundary) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
            if let Some(parsed) = ai_lib::sse::parser::parse_sse_event(event_text) {
                if let Some(chunk) = parsed? {
                    if let Some(delta_content) = chunk.choices[0].delta.content.clone() {
                        content.push_str(&delta_content);
                    }
                }
            }
        }
    }

    assert_eq!(content, chinese_text);
    Ok(())
}

#[cfg(feature = "unified_sse")]
#[test]
fn golden_sse_mixed_line_endings() -> Result<(), AiLibError> {
    // Test SSE events with mixed line endings (LF and CRLF)
    let event_lf = "data: {\"id\":\"1\",\"choices\":[{\"delta\":{\"content\":\"LF event\"}}]}\n\n";
    let event_crlf =
        "data: {\"id\":\"2\",\"choices\":[{\"delta\":{\"content\":\"CRLF event\"}}]}\r\n\r\n";

    let mut buffer = [event_lf.as_bytes(), event_crlf.as_bytes()].concat();
    let mut events_parsed = 0;

    while let Some(boundary) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
            if let Some(parsed) = ai_lib::sse::parser::parse_sse_event(event_text) {
                if parsed?.is_some() {
                    events_parsed += 1;
                }
            }
        }
    }

    assert_eq!(events_parsed, 2);
    Ok(())
}

#[cfg(feature = "unified_sse")]
#[test]
fn golden_sse_partial_json_recovery() -> Result<(), AiLibError> {
    // Test recovery from partial JSON in SSE events
    // First, test that partial JSON is ignored (no boundary found)
    let partial_event = "data: {\"id\":\"1\",\"choices\":[{\"delta\":{\"content\":\"partial";
    let mut buffer = partial_event.as_bytes().to_vec();

    // Should not find a boundary for partial JSON
    assert!(ai_lib::sse::parser::find_event_boundary(&buffer).is_none());

    // Now test complete JSON after partial - clear buffer first
    buffer.clear();
    let complete_event =
        "data: {\"id\":\"2\",\"choices\":[{\"delta\":{\"content\":\"complete\"}}]}\n\n";
    buffer.extend_from_slice(complete_event.as_bytes());

    let mut content = String::new();
    while let Some(boundary) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
            if let Some(parsed) = ai_lib::sse::parser::parse_sse_event(event_text) {
                if let Some(chunk) = parsed? {
                    if let Some(delta_content) = chunk.choices[0].delta.content.clone() {
                        content.push_str(&delta_content);
                    }
                }
            }
        }
    }

    assert_eq!(content, "complete");
    Ok(())
}

#[cfg(feature = "unified_sse")]
#[test]
fn golden_sse_whitespace_handling() -> Result<(), AiLibError> {
    // Test various whitespace scenarios in SSE events
    let events = vec![
        "data: {\"id\":\"1\",\"choices\":[{\"delta\":{\"content\":\"no_ws\"}}]}\n\n",
        "data: {\"id\":\"2\",\"choices\":[{\"delta\":{\"content\":\"with_ws\"}}]} \n\n",
        "data: {\"id\":\"3\",\"choices\":[{\"delta\":{\"content\":\"tab_ws\"}}]}\t\n\n",
        "data: {\"id\":\"4\",\"choices\":[{\"delta\":{\"content\":\"mixed_ws\"}}]} \t \n\n",
    ];

    let mut buffer = Vec::new();
    for event in events {
        buffer.extend_from_slice(event.as_bytes());
    }

    let mut parsed_count = 0;
    while let Some(boundary) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
            if let Some(parsed) = ai_lib::sse::parser::parse_sse_event(event_text) {
                if parsed?.is_some() {
                    parsed_count += 1;
                }
            }
        }
    }

    assert_eq!(parsed_count, 4);
    Ok(())
}

#[cfg(feature = "unified_sse")]
#[test]
fn golden_sse_nested_json_escaping() -> Result<(), AiLibError> {
    // Test SSE events with nested JSON and escaped characters
    let complex_json =
        r#"{"id":"1","choices":[{"delta":{"content":"He said: \"Hello, world!\""}}]}"#;
    let event = format!("data: {}\n\n", complex_json);

    let mut buffer = event.as_bytes().to_vec();
    let mut content = String::new();

    while let Some(boundary) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
            if let Some(parsed) = ai_lib::sse::parser::parse_sse_event(event_text) {
                if let Some(chunk) = parsed? {
                    if let Some(delta_content) = chunk.choices[0].delta.content.clone() {
                        content.push_str(&delta_content);
                    }
                }
            }
        }
    }

    assert_eq!(content, "He said: \"Hello, world!\"");
    Ok(())
}

#[cfg(feature = "unified_sse")]
#[test]
fn golden_sse_empty_content_chunks() -> Result<(), AiLibError> {
    // Test SSE events with empty content (should be handled gracefully)
    let events = vec![
        r#"data: {"id":"1","choices":[{"delta":{"role":"assistant"}}]}"#,
        r#"data: {"id":"2","choices":[{"delta":{"content":""}}]}"#,
        r#"data: {"id":"3","choices":[{"delta":{"content":"actual content"}}]}"#,
        r#"data: {"id":"4","choices":[{"delta":{}}]}"#,
    ];

    let mut buffer = Vec::new();
    for event in events {
        buffer.extend_from_slice(format!("{}\n\n", event).as_bytes());
    }

    let mut chunk_count = 0;
    let mut content_chunks = 0;

    while let Some(boundary) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
            if let Some(parsed) = ai_lib::sse::parser::parse_sse_event(event_text) {
                if let Some(chunk) = parsed? {
                    chunk_count += 1;
                    if chunk.choices[0].delta.content.is_some() {
                        content_chunks += 1;
                    }
                }
            }
        }
    }

    assert_eq!(chunk_count, 4);
    assert_eq!(content_chunks, 2); // Only chunks 2 and 3 have content
    Ok(())
}

#[cfg(feature = "unified_sse")]
#[test]
fn golden_sse_finish_reason_variants() -> Result<(), AiLibError> {
    // Test different finish_reason values
    let events = vec![
        r#"data: {"id":"1","choices":[{"delta":{"content":"content"},"finish_reason":null}]}"#,
        r#"data: {"id":"2","choices":[{"delta":{"content":"more"},"finish_reason":null}]}"#,
        r#"data: {"id":"3","choices":[{"delta":{},"finish_reason":"stop"}]}"#,
        r#"data: {"id":"4","choices":[{"delta":{},"finish_reason":"length"}]}"#,
    ];

    let mut buffer = Vec::new();
    for event in events {
        buffer.extend_from_slice(format!("{}\n\n", event).as_bytes());
    }

    let mut chunk_count = 0;
    let mut finish_reasons = Vec::new();

    while let Some(boundary) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
            if let Some(parsed) = ai_lib::sse::parser::parse_sse_event(event_text) {
                if let Some(chunk) = parsed? {
                    chunk_count += 1;
                    if let Some(finish_reason) = chunk.choices[0].finish_reason.clone() {
                        finish_reasons.push(finish_reason);
                    }
                }
            }
        }
    }

    assert_eq!(chunk_count, 4);
    assert_eq!(finish_reasons.len(), 2);
    assert!(finish_reasons.contains(&"stop".to_string()));
    assert!(finish_reasons.contains(&"length".to_string()));
    Ok(())
}

#[cfg(feature = "unified_sse")]
#[test]
fn golden_sse_very_large_single_event() -> Result<(), AiLibError> {
    // Test a single SSE event with very large content
    let large_content = "A".repeat(100_000); // 100KB
    let large_json = format!(
        r#"{{"id":"1","choices":[{{"delta":{{"content":"{}"}}}}]}}"#,
        large_content
    );
    let event = format!("data: {}\n\n", large_json);

    let mut buffer = event.as_bytes().to_vec();
    let mut content = String::new();

    while let Some(boundary) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
            if let Some(parsed) = ai_lib::sse::parser::parse_sse_event(event_text) {
                if let Some(chunk) = parsed? {
                    if let Some(delta_content) = chunk.choices[0].delta.content.clone() {
                        content.push_str(&delta_content);
                    }
                }
            }
        }
    }

    assert_eq!(content, large_content);
    assert_eq!(content.len(), 100_000);
    Ok(())
}
