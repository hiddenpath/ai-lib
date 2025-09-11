use ai_lib::types::AiLibError;

#[cfg(feature = "unified_sse")]
#[test]
fn golden_non_ascii_concatenated_frames() -> Result<(), AiLibError> {
    // Two SSE events back-to-back with multibyte UTF-8
    let event1 = "data: {\"id\":\"1\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"m\",\"choices\":[{\"delta\":{\"role\":\"assistant\",\"content\":\"你好，\"}}]}\n\n";
    let event2 = "data: {\"id\":\"2\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"m\",\"choices\":[{\"delta\":{\"content\":\"世界！\"}}]}\n\n";
    let mut buffer = [event1.as_bytes(), event2.as_bytes()].concat();

    // First boundary
    while let Some(boundary) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
            if let Some(parsed) = ai_lib::sse::parser::parse_sse_event(event_text) {
                let chunk = parsed?.expect("should produce a chunk");
                let delta = &chunk.choices[0].delta;
                if let Some(c) = &delta.content {
                    assert!(c == "你好，" || c == "世界！");
                }
            }
        }
    }
    Ok(())
}

#[cfg(feature = "unified_sse")]
#[test]
fn golden_large_payload_split_boundaries() -> Result<(), AiLibError> {
    // Build a large content and split across artificial chunks with CRLF boundaries
    let large = "a".repeat(8192) + "结束";
    let json = format!(
        "data: {{\"id\":\"1\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"m\",\"choices\":[{{\"delta\":{{\"role\":\"assistant\",\"content\":\"{}\"}}}}]}}\r\n\r\n",
        large
    );
    let mut buffer = json.as_bytes().to_vec();
    let mut seen = false;
    while let Some(boundary) = ai_lib::sse::parser::find_event_boundary(&buffer) {
        let event_bytes = buffer.drain(..boundary).collect::<Vec<_>>();
        if let Ok(event_text) = std::str::from_utf8(&event_bytes) {
            if let Some(parsed) = ai_lib::sse::parser::parse_sse_event(event_text) {
                let chunk = parsed?.expect("chunk");
                assert!(matches!(
                    chunk.choices[0].delta.role,
                    Some(ai_lib::types::Role::Assistant)
                ));
                assert!(chunk.choices[0]
                    .delta
                    .content
                    .as_ref()
                    .unwrap()
                    .ends_with("结束"));
                seen = true;
            }
        }
    }
    assert!(seen);
    Ok(())
}
