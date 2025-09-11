#![cfg(feature = "unified_sse")]

use ai_lib::sse::parser::{find_event_boundary, parse_sse_event};

#[test]
fn sse_tail_without_newline_flushes_on_next_chunk() {
    // First buffer has no trailing newline; second buffer provides separator
    let mut buf = b"data: {\"a\":1}".to_vec();
    assert!(find_event_boundary(&buf).is_none());
    buf.extend_from_slice(b"\n\n");
    let idx = find_event_boundary(&buf).expect("boundary expected");
    let event = std::str::from_utf8(&buf[..idx]).unwrap();
    let parsed = parse_sse_event(event).expect("some").expect("ok");
    assert!(parsed.is_some());
}

#[test]
fn sse_multibyte_utf8_content() {
    let data = "{\"choices\":[{\"delta\":{\"content\":\"你好，世界\"}}]}";
    let frame = format!("data: {}\n\n", data);
    let parsed = parse_sse_event(&frame).expect("some").expect("ok");
    let chunk = parsed.expect("chunk");
    assert!(chunk
        .choices
        .first()
        .and_then(|c| c.delta.content.as_ref())
        .is_some());
}

#[test]
fn sse_done_signal() {
    let frame = "data: [DONE]\n\n";
    let parsed = parse_sse_event(frame).expect("some").expect("ok");
    assert!(parsed.is_none());
}
