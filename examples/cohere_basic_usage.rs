use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Cohere Basic Usage Example");
    println!("==============================");

    // Check COHERE_API_KEY environment variable
    if std::env::var("COHERE_API_KEY").is_err() {
        println!("❌ Please set COHERE_API_KEY environment variable");
        println!("   Example: export COHERE_API_KEY=your_api_key_here");
        return Ok(());
    }

    println!("🔧 Creating Cohere client using independent adapter...");
    let client = AiClient::new(Provider::Cohere)?;
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
        "command".to_string(), // Cohere model
        vec![Message {
            role: Role::User,
            content: Content::Text("hello".to_string()),
            function_call: None,
        }],
    );

    println!("\n📤 Sending request to model: {}", request.model);
    println!("📝 Request: hello");

    // Send request and get response
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
            println!("\n⚠️  Chat completion failed: {}", e);
            println!("   This might be due to API key issues or model availability.");
        }
    }

    // Show Cohere-specific features
    println!("\n🔍 Cohere Provider Information:");
    println!("   • Provider Type: Independent (uses CohereAdapter)");
    println!("   • Base URL: https://api.cohere.ai");
    println!("   • API Key: COHERE_API_KEY environment variable");
    println!("   • Supported Models: command, command-light, command-nightly");
    println!("   • Features: Chat completion, text generation, streaming");

    Ok(())
}
