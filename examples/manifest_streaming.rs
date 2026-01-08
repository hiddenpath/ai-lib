//! Manifest-driven streaming example
//!
//! Demonstrates the operator-based streaming pipeline driven by aimanifest.yaml.
//! This example shows how StreamProcessor interprets event_map rules from the manifest.

use ai_lib::manifest::schema::{StreamingConfig, StreamingEventRule};
use ai_lib::streaming::events::StreamingEvent;
use ai_lib::streaming::pipeline::StreamProcessor;
use serde_json::json;
use std::collections::HashMap;

fn main() {
    println!("🚀 Manifest-Driven Streaming Example");
    println!("=====================================\n");

    // Test 1: OpenAI-style streaming
    test_openai_streaming();

    // Test 2: Anthropic-style streaming
    test_anthropic_streaming();

    // Test 3: Gemini-style streaming
    test_gemini_streaming();

    // Test 4: Cohere-style streaming
    test_cohere_streaming();

    println!("\n✅ All streaming format tests completed!");
}

fn make_fields(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn test_openai_streaming() {
    println!("📡 Testing OpenAI-style streaming...");

    let config = StreamingConfig {
        event_format: Some("data_lines".to_string()),
        decoder: None,
        frame_selector: Some("exists($.choices)".to_string()),
        accumulator: None,
        candidate: None,
        event_map: vec![
            StreamingEventRule {
                matcher: "exists($.choices[0].delta.content)".to_string(),
                emit: "PartialContentDelta".to_string(),
                fields: make_fields(&[("content", "$.choices[0].delta.content")]),
            },
            StreamingEventRule {
                matcher: "exists($.choices[0].finish_reason)".to_string(),
                emit: "FinalCandidate".to_string(),
                fields: make_fields(&[("finish_reason", "$.choices[0].finish_reason")]),
            },
        ],
        stop_condition: Some("$.choices[0].finish_reason != null".to_string()),
        extra_metadata_path: Some("$.usage".to_string()),
        content_path: Some("choices[0].delta.content".to_string()),
        tool_call_path: None,
        finish_reason_path: None,
    };

    let mut processor = StreamProcessor::new(config);

    // Simulate OpenAI streaming chunks
    let chunks = vec![
        json!({"id": "chatcmpl-123", "choices": [{"index": 0, "delta": {"role": "assistant"}}]}),
        json!({"id": "chatcmpl-123", "choices": [{"index": 0, "delta": {"content": "Hello"}}]}),
        json!({"id": "chatcmpl-123", "choices": [{"index": 0, "delta": {"content": " world"}}]}),
        json!({"id": "chatcmpl-123", "choices": [{"index": 0, "delta": {"content": "!"}}]}),
        json!({"id": "chatcmpl-123", "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}], "usage": {"prompt_tokens": 10, "completion_tokens": 5}}),
    ];

    let mut events = Vec::new();
    for chunk in chunks {
        if let Some(event) = processor.process(&chunk) {
            events.push(event);
        }
    }

    println!("   Processed {} events", events.len());
    for event in &events {
        print_event(event);
    }
    println!();
}

fn test_anthropic_streaming() {
    println!("📡 Testing Anthropic-style streaming...");

    let config = StreamingConfig {
        event_format: Some("anthropic_sse".to_string()),
        decoder: None,
        frame_selector: Some("$.type in ['content_block_delta', 'message_stop']".to_string()),
        accumulator: None,
        candidate: None,
        event_map: vec![
            StreamingEventRule {
                matcher: "$.type == 'content_block_delta' && $.delta.type == 'text_delta'".to_string(),
                emit: "PartialContentDelta".to_string(),
                fields: make_fields(&[("content", "$.delta.text")]),
            },
            StreamingEventRule {
                matcher: "$.type == 'message_stop'".to_string(),
                emit: "StreamEnd".to_string(),
                fields: make_fields(&[("finish_reason", "end_turn")]),
            },
        ],
        stop_condition: Some("$.type == 'message_stop'".to_string()),
        extra_metadata_path: None,
        content_path: None,
        tool_call_path: None,
        finish_reason_path: None,
    };

    let mut processor = StreamProcessor::new(config);

    // Simulate Anthropic streaming chunks
    let chunks = vec![
        json!({"type": "content_block_start", "index": 0, "content_block": {"type": "text", "text": ""}}),
        json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": "Hello"}}),
        json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": " from"}}),
        json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": " Claude!"}}),
        json!({"type": "message_stop"}),
    ];

    let mut events = Vec::new();
    for chunk in chunks {
        if let Some(event) = processor.process(&chunk) {
            events.push(event);
        }
    }

    println!("   Processed {} events", events.len());
    for event in &events {
        print_event(event);
    }
    println!();
}

fn test_gemini_streaming() {
    println!("📡 Testing Gemini-style streaming...");

    let config = StreamingConfig {
        event_format: Some("gemini_json".to_string()),
        decoder: None,
        frame_selector: Some("exists($.candidates)".to_string()),
        accumulator: None,
        candidate: None,
        event_map: vec![
            StreamingEventRule {
                matcher: "exists($.candidates[0].content.parts[0].text)".to_string(),
                emit: "PartialContentDelta".to_string(),
                fields: make_fields(&[("content", "$.candidates[0].content.parts[0].text")]),
            },
            StreamingEventRule {
                matcher: "exists($.candidates[0].finishReason)".to_string(),
                emit: "FinalCandidate".to_string(),
                fields: make_fields(&[("finish_reason", "$.candidates[0].finishReason")]),
            },
        ],
        stop_condition: Some("exists($.candidates[0].finishReason)".to_string()),
        extra_metadata_path: Some("$.usageMetadata".to_string()),
        content_path: None,
        tool_call_path: None,
        finish_reason_path: None,
    };

    let mut processor = StreamProcessor::new(config);

    // Simulate Gemini streaming chunks
    let chunks = vec![
        json!({"candidates": [{"content": {"parts": [{"text": "Hello"}], "role": "model"}, "index": 0}]}),
        json!({"candidates": [{"content": {"parts": [{"text": " from"}], "role": "model"}, "index": 0}]}),
        json!({"candidates": [{"content": {"parts": [{"text": " Gemini!"}], "role": "model"}, "index": 0, "finishReason": "STOP"}], "usageMetadata": {"promptTokenCount": 5, "candidatesTokenCount": 3}}),
    ];

    let mut events = Vec::new();
    for chunk in chunks {
        if let Some(event) = processor.process(&chunk) {
            events.push(event);
        }
    }

    println!("   Processed {} events", events.len());
    for event in &events {
        print_event(event);
    }
    println!();
}

fn test_cohere_streaming() {
    println!("📡 Testing Cohere-style streaming...");

    let config = StreamingConfig {
        event_format: Some("cohere_native".to_string()),
        decoder: None,
        frame_selector: Some("exists($.event_type)".to_string()),
        accumulator: None,
        candidate: None,
        event_map: vec![
            StreamingEventRule {
                matcher: "$.event_type == 'text-generation'".to_string(),
                emit: "PartialContentDelta".to_string(),
                fields: make_fields(&[("content", "$.text")]),
            },
            StreamingEventRule {
                matcher: "$.event_type == 'stream-end'".to_string(),
                emit: "StreamEnd".to_string(),
                fields: make_fields(&[("finish_reason", "$.finish_reason")]),
            },
        ],
        stop_condition: Some("$.event_type == 'stream-end'".to_string()),
        extra_metadata_path: None,
        content_path: None,
        tool_call_path: None,
        finish_reason_path: None,
    };

    let mut processor = StreamProcessor::new(config);

    // Simulate Cohere streaming chunks
    let chunks = vec![
        json!({"event_type": "stream-start", "generation_id": "abc123"}),
        json!({"event_type": "text-generation", "text": "Hello"}),
        json!({"event_type": "text-generation", "text": " from"}),
        json!({"event_type": "text-generation", "text": " Cohere!"}),
        json!({"event_type": "stream-end", "finish_reason": "COMPLETE", "response": {"meta": {"tokens": {"input_tokens": 5, "output_tokens": 3}}}}),
    ];

    let mut events = Vec::new();
    for chunk in chunks {
        if let Some(event) = processor.process(&chunk) {
            events.push(event);
        }
    }

    println!("   Processed {} events", events.len());
    for event in &events {
        print_event(event);
    }
    println!();
}

fn print_event(event: &StreamingEvent) {
    match event {
        StreamingEvent::PartialContentDelta(delta) => {
            println!("   📝 ContentDelta: {:?}", delta.delta);
        }
        StreamingEvent::PartialToolCall(tc) => {
            println!("   🔧 ToolCall: {:?} - {:?}", tc.tool_name, tc.arguments_delta);
        }
        StreamingEvent::FinalCandidate(fc) => {
            println!("   🏁 FinalCandidate: finish_reason={:?}", fc.finish_reason);
        }
        StreamingEvent::StreamEnd => {
            println!("   ⏹️  StreamEnd");
        }
        StreamingEvent::ThinkingDelta(td) => {
            println!("   💭 Thinking: {:?}", td.thinking);
        }
        StreamingEvent::Metadata(md) => {
            println!("   📊 Metadata: {:?}", md.data);
        }
        StreamingEvent::ToolCallStarted(tcs) => {
            println!("   🔧▶ ToolCallStarted: {:?}", tcs.tool_name);
        }
        StreamingEvent::ToolCallEnded(tce) => {
            println!("   🔧⏹ ToolCallEnded: {:?}", tce.tool_call_id);
        }
        _ => {
            println!("   ❓ Other event: {:?}", event);
        }
    }
}

