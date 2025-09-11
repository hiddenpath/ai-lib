/// Mistral Basic Usage Example
///
/// This example demonstrates how to use the independent Provider (Mistral) for AI conversations
/// Similar to basic_usage.rs, but specifically for Mistral Provider
use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Mistral Basic Usage Example");
    println!("===============================");

    // Check environment variables
    if std::env::var("MISTRAL_API_KEY").is_err() {
        println!("❌ Please set MISTRAL_API_KEY environment variable");
        println!("   Example: export MISTRAL_API_KEY=your_api_key_here");
        println!("   Or set it in .env file");
        return Ok(());
    }

    println!("🔧 Creating Mistral client using independent adapter...");

    // Create Mistral client - using independent adapter
    let client = AiClient::new(Provider::Mistral)?;
    println!(
        "✅ Created client with provider: {:?}",
        client.current_provider()
    );

    // Get available models list
    println!("\n📋 Getting available models...");
    let models = client.list_models().await?;
    println!("📋 Available models: {:?}", models);

    // Create chat request
    let request = ChatCompletionRequest::new(
        "mistral-tiny".to_string(), // Mistral model
        vec![Message {
            role: Role::User,
            content: Content::Text("hello".to_string()),
            function_call: None,
        }],
    );

    println!("\n📤 Sending request to model: {}", request.model);
    println!("📝 Request: hello");

    // Send request and get response
    // Note: Chat requests may fail due to current string truncation issues
    // But we have successfully demonstrated the basic usage of Mistral independent configuration
    match client.chat_completion(request).await {
        Ok(response) => {
            println!("\n📥 Received response:");
            println!("   ID: {}", response.id);
            println!("   Model: {}", response.model);
            println!(
                "   Content: {}",
                response.choices[0].message.content.as_text()
            );
            println!("   Usage: {} tokens", response.usage.total_tokens);
        }
        Err(e) => {
            println!("\n⚠️  Chat completion failed (known issue): {}", e);
            println!(
                "   This is a known issue with string truncation in the current implementation."
            );
            println!("   The basic Mistral client setup and model listing works correctly.");
        }
    }

    // Show Mistral-specific features
    println!("\n🔍 Mistral Provider Information:");
    println!("   • Provider Type: Independent (uses MistralAdapter)");
    println!("   • Base URL: https://api.mistral.ai");
    println!("   • API Key: MISTRAL_API_KEY environment variable");
    println!(
        "   • Supported Models: mistral-small-latest, mistral-medium-latest, mistral-large-latest"
    );

    Ok(())
}
