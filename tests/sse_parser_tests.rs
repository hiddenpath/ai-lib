#![cfg(feature = "unified_sse")]

use ai_lib::sse::parser::{find_event_boundary, parse_chunk_data, parse_sse_event};

#[test]
fn test_find_event_boundary_lf_lf() {
    let data = b"data: {\"x\":1}\n\nrest";
    let idx = find_event_boundary(data).expect("boundary expected");
    // boundary should include the two newlines
    assert_eq!(&data[..idx], b"data: {\"x\":1}\n\n");
}

#[test]
fn test_find_event_boundary_crlf_crlf() {
    let data = b"data: {\"x\":1}\r\n\r\nrest";
    let idx = find_event_boundary(data).expect("boundary expected");
    assert_eq!(&data[..idx], b"data: {\"x\":1}\r\n\r\n");
}

#[test]
fn test_parse_sse_event_done() {
    let text = "data: [DONE]\n\n";
    let parsed = parse_sse_event(text).expect("some").expect("ok option");
    assert!(parsed.is_none(), "[DONE] should produce None chunk");
}

#[test]
fn test_parse_chunk_data_simple() {
    let payload = r#"{
        "id":"x",
        "object":"chat.completion.chunk",
        "created":0,
        "model":"m",
        "choices":[{"delta":{"role":"assistant","content":"Hello"},"finish_reason":null}]
    }"#;

    let chunk = parse_chunk_data(payload)
        .expect("parse ok")
        .expect("some chunk");
    assert_eq!(chunk.model, "m");
    assert_eq!(chunk.choices.len(), 1);
    assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("Hello"));
}
