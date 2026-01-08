use ai_lib::prelude::*;
use ai_lib::types::function_call::{FunctionCallPolicy, Tool};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 AI-lib v0.5.0 Function Calling Example");
    println!("========================================");

    // v0.5.0 Pattern: Model-driven client
    let client = AiClientBuilder::new(Provider::OpenAI)
        .with_model("gpt-4o")
        .build()?;

    // Define a Tool (JSON Schema for parameters)
    let ascii_horse_tool = Tool {
        name: "ascii_horse".to_string(),
        description: Some("Draws an ASCII horse of given size".to_string()),
        parameters: Some(json!({
            "type": "object",
            "properties": {
                "size": { "type": "integer", "description": "Size of the horse" }
            },
            "required": ["size"]
        })),
    };

    // Build request with tools
    let mut req = ChatCompletionRequest::new(
        "gpt-4o".to_string(),
        vec![Message::user(
            "Please call the ascii_horse tool with size=3",
        )],
    );
    req.functions = Some(vec![ascii_horse_tool]);
    req.function_call = Some(FunctionCallPolicy::Auto("auto".to_string()));
    req = req.with_max_tokens(500).with_temperature(0.0);

    println!("📤 Sending request for function call (model={})", req.model);

    match client.chat_completion(req).await {
        Ok(resp) => {
            // Handle possible function call
            for choice in resp.choices {
                let msg = choice.message;
                if let Some(fc) = msg.function_call {
                    println!("🛠️  Model invoked function: {}", fc.name);
                    let args = fc.arguments.unwrap_or(json!(null));
                    println!("   Arguments: {}", args);

                    if fc.name == "ascii_horse" {
                        let size = args.get("size").and_then(|v| v.as_i64()).unwrap_or(3) as usize;
                        let horse = generate_ascii_horse(size);
                        println!("⚙️ Executed locally, output:\n{}", horse);

                        println!("✅ Function call successfully demonstrated!");
                    }
                } else {
                    println!("💬 Model message: {}", msg.content.as_text());
                }
            }
        }
        Err(e) => {
            println!("⚠️  Chat failed: {}", e);
            println!("   Note: This example requires appropriate API keys.");
        }
    }

    Ok(())
}

fn generate_ascii_horse(size: usize) -> String {
    let mut out = String::new();
    let s = std::cmp::max(1, size);
    for _ in 0..s {
        out.push_str("  \\ \\__\\\n");
    }
    out.push_str(" (horse)\n");
    out
}
