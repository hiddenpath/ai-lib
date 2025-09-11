//! JSONL Streaming Protocol Example
//!
//! This example demonstrates the new JSONL streaming protocol that provides
//! both streaming experience and complete JSON data without conflicts.

#[cfg(feature = "unified_sse")]
use ai_lib::sse::jsonl_parser::{FinalData, JsonlMessage, JsonlParser};
use ai_lib::types::AiLibError;

#[cfg(feature = "unified_sse")]
#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    println!("JSONL Streaming Protocol Demo");
    println!("=============================");

    // Simulate streaming data
    let jsonl_data = r#"{"type":"delta","data":"你"}
{"type":"delta","data":"好"}
{"type":"delta","data":"呀"}
{"type":"final","data":{"answer":"你好呀","confidence":0.98,"metadata":{"tokens":3}}}"#;

    let mut parser = JsonlParser::new();
    let mut accumulated = String::new();

    println!("\nProcessing JSONL stream:");
    println!("------------------------");

    for line in jsonl_data.lines() {
        if let Some(chunk) = parser.parse_line(line)? {
            if let Some(content) = &chunk.choices[0].delta.content {
                accumulated.push_str(content);
                println!("Delta: '{}' (accumulated: '{}')", content, accumulated);
            }

            if let Some(finish_reason) = &chunk.choices[0].finish_reason {
                println!("Final: {}", finish_reason);
            }
        }
    }

    println!(
        "\nFinal accumulated content: '{}'",
        parser.accumulated_content()
    );

    // Demonstrate the protocol structure
    println!("\nProtocol Structure:");
    println!("------------------");

    let delta_example = JsonlMessage::Delta {
        data: "Hello".to_string(),
    };
    println!(
        "Delta message: {}",
        serde_json::to_string_pretty(&delta_example)
            .map_err(|e| AiLibError::ProviderError(format!("JSON serialization error: {}", e)))?
    );

    let final_example = JsonlMessage::Final {
        data: FinalData {
            answer: "Hello World".to_string(),
            confidence: Some(0.95),
            metadata: Some(serde_json::json!({
                "tokens": 2,
                "model": "gpt-4"
            })),
        },
    };
    println!(
        "Final message: {}",
        serde_json::to_string_pretty(&final_example)
            .map_err(|e| AiLibError::ProviderError(format!("JSON serialization error: {}", e)))?
    );

    Ok(())
}

#[cfg(not(feature = "unified_sse"))]
fn main() {
    // This example requires the `unified_sse` feature to be enabled.
    println!(
        "jsonl_streaming example requires --features unified_sse to run. Skipping."
    );
}
