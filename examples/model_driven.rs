//! Custom Model Configuration Example
//!
//! Demonstrates how to use the v0.5.0 data-driven model registry
//! to configure clients with specific models.

use ai_lib::{AiClientBuilder, ChatCompletionRequest, Message, Provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 ai-lib v0.5.0 Model-Driven Example\n");

    // Check for API key
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("❌ Please set OPENAI_API_KEY environment variable");
        return Ok(());
    }

    // v0.5.0 Model-Driven Approach
    // The `with_model()` method looks up the model in the registry
    // and automatically configures the correct provider and protocol.
    let client = AiClientBuilder::new(Provider::OpenAI)
        .with_model("gpt-4o") // Model ID from registry
        .build()?;

    println!(
        "✅ Created client for model: {}",
        client.default_chat_model()
    );
    println!("   Provider: {}", client.provider_name());

    // Create a simple request
    let request = ChatCompletionRequest::new(
        "gpt-4o".to_string(),
        vec![
            Message::system("You are a helpful assistant."),
            Message::user("What is 2 + 2? Answer in one word."),
        ],
    );

    // Execute the request
    let response = client.chat_completion(request).await?;

    if let Some(choice) = response.choices.first() {
        println!("\n🤖 Response: {}", choice.message.content.as_text());
    }

    // Display usage info
    let usage = response.usage;
    println!(
        "\n📊 Usage: {} prompt + {} completion = {} total tokens",
        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
    );

    Ok(())
}
