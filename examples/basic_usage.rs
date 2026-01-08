/// AI-lib basic usage example
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI-lib v0.5.0 Basic Usage Example");
    println!("=====================================");

    // v0.5.0 Pattern: Build client by model ID.
    // The configuration is automatically loaded from aimanifest.yaml.
    let client = AiClientBuilder::new(Provider::OpenAI)
        .with_model("gpt-4o")
        .build()?;

    println!(
        "✅ Created client for model: {} (Provider: {:?})",
        client.default_chat_model(),
        client.provider_name()
    );

    // Create chat request using standard API
    let request = ChatCompletionRequest::new(
        "gpt-4o".to_string(),
        vec![Message::user("Hello! Please introduce yourself briefly.")],
    )
    .with_temperature(0.7);

    println!("📤 Sending request via Manifest-Driven Adapter...");

    // Send request
    match client.chat_completion(request).await {
        Ok(response) => {
            println!("📥 Received response:");
            println!("   ID: {}", response.id);
            println!(
                "   Content: {}",
                response.choices[0].message.content.as_text()
            );
            println!("   Usage: {} tokens", response.usage.total_tokens);
        }
        Err(e) => {
            println!("⚠️  Chat failed: {}", e);
            println!("   Note: This example requires OPENAI_API_KEY environment variable.");
        }
    }

    Ok(())
}
