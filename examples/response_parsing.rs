//! Response Parsing Example (generic)
//!
//! Demonstrates how to use ai-lib's response parsing utilities to extract
//! JSON, Markdown sections, and code blocks from model outputs.

#[cfg(feature = "response_parser")]
use ai_lib::prelude::*;
#[cfg(feature = "response_parser")]
use ai_lib::response_parser::{
    extract_code_blocks, extract_json_from_text, AutoParser, JsonResponseParser,
    MarkdownSectionParser, ParsedResponse, ResponseParser,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MathSolution {
    steps: Vec<String>,
    answer: String,
}

#[cfg(feature = "response_parser")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 ai-lib Response Parsing (generic)");

    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ Please set GROQ_API_KEY");
        return Ok(());
    }

    let client = AiClient::new(Provider::Groq)?;

    // Ask model to return both Markdown sections and JSON code block
    let request = ChatCompletionRequest::new(
        "qwen-qwq-32b".to_string(),
        vec![
            Message::system(
                "Return analysis with ## sections and include a JSON code block named solution.",
            ),
            Message::user("Solve 2x + 5 = 17 and explain briefly."),
        ],
    );

    let resp = client.chat_completion(request).await?;
    let content = resp.choices[0].message.content.as_text();
    println!("\n🤖 Raw response (first 300 chars):");
    println!("{}", &content[..content.len().min(300)]);

    // 1) Auto parse
    let auto = AutoParser::default();
    match auto.parse(&content).await? {
        ParsedResponse::Json(v) => println!("\n📦 Parsed JSON (auto): {}", v),
        ParsedResponse::Sections(map) => {
            println!("\n📑 Sections found:");
            for (k, v) in map.iter() {
                println!("- {} ({} chars)", k, v.len());
            }
        }
        ParsedResponse::CodeBlocks(blocks) => {
            println!("\n💻 Code blocks:");
            for (lang, code) in blocks {
                println!("- {}: {} chars", lang, code.len());
            }
        }
        ParsedResponse::Text(_) => println!("\n(plain text)"),
        ParsedResponse::JsonLines(lines) => {
            println!("\n📋 JSON Lines: {} items", lines.len());
        }
    }

    // 2) Extract JSON code block and deserialize
    if let Some(json_str) = extract_json_from_text(&content) {
        let parser = JsonResponseParser::<MathSolution>::new();
        if let Ok(sol) = parser.parse(&json_str).await {
            println!(
                "\n✅ Parsed MathSolution: answer={}, steps={}",
                sol.answer,
                sol.steps.len()
            );
        } else {
            println!("\n⚠️ Found JSON block but failed to parse as MathSolution");
        }
    }

    // 3) Show code blocks
    let blocks = extract_code_blocks(&content);
    if !blocks.is_empty() {
        println!("\n💻 Code blocks detected: {}", blocks.len());
    }

    Ok(())
}

#[cfg(not(feature = "response_parser"))]
fn main() {
    println!("This example requires the 'response_parser' feature to be enabled.");
    println!("Run with: cargo run --example response_parsing --features response_parser");
}
