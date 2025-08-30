use ai_lib::types::common::Content;
use ai_lib::types::function_call::{FunctionCallPolicy, Tool};
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 OpenAI Function Calling example (ai-lib)");

    // Ensure OPENAI_API_KEY is set in env before running
    let client = AiClient::new(Provider::OpenAI)?;

    // Build a simple user message
    let user_msg = Message {
        role: Role::User,
        content: Content::Text("Please call the ascii_horse tool with size=3".to_string()),
        function_call: None,
    };

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

    let mut req = ChatCompletionRequest::new("gpt-4o-mini".to_string(), vec![user_msg]);
    req.functions = Some(vec![ascii_horse_tool]);
    req.function_call = Some(FunctionCallPolicy::Auto("auto".to_string()));
    req = req.with_max_tokens(200).with_temperature(0.0);

    println!("📤 Sending request to OpenAI (model={})", req.model);

    let resp = client.chat_completion(req).await?;

    // Handle a possible function call from the model: execute locally and send the result back
    for choice in resp.choices {
        let msg = choice.message;
        if let Some(fc) = msg.function_call {
            println!("🛠️  Model invoked function: {}", fc.name);
            let args = fc.arguments.unwrap_or(serde_json::json!(null));
            println!("   arguments: {}", args);

            // Simple local tool: ascii_horse
            if fc.name == "ascii_horse" {
                // Parse size param
                let size = args.get("size").and_then(|v| v.as_i64()).unwrap_or(3) as usize;
                let horse = generate_ascii_horse(size);
                println!("⚙️ Executed ascii_horse locally, output:\n{}", horse);

                // Send follow-up message with tool result as assistant message
                let tool_msg = Message {
                    role: Role::Assistant,
                    content: Content::Text(horse.clone()),
                    function_call: None,
                };

                let mut followup =
                    ChatCompletionRequest::new("gpt-4o-mini".to_string(), vec![tool_msg]);
                followup = followup.with_max_tokens(200).with_temperature(0.0);
                let follow_resp = client.chat_completion(followup).await?;
                for fc_choice in follow_resp.choices {
                    println!(
                        "🗨️ Final model response: {}",
                        fc_choice.message.content.as_text()
                    );
                }
            }
        } else {
            println!("💬 Model message: {}", msg.content.as_text());
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
