#![cfg(not(feature = "unified_sse"))]

#[test]
fn legacy_event_sequence_non_ascii_doc() {
    // This test documents that legacy SSE helpers are now internal and not exposed.
    // The real event-sequence coverage lives in unified_sse golden tests.
    assert!(true);
}
