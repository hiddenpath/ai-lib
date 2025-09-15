use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 OpenAI Basic Usage Example");
    println!("=============================");

    // Check OPENAI_API_KEY environment variable
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("❌ Please set OPENAI_API_KEY environment variable");
        println!("   Example: export OPENAI_API_KEY=your_api_key_here");
        return Ok(());
    }

    println!("🔧 Creating OpenAI client using independent adapter...");
    let client = AiClient::new(Provider::OpenAI)?;
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
        client.default_chat_model(),
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

    // Show OpenAI-specific features
    println!("\n🔍 OpenAI Provider Information:");
    println!("   • Provider Type: Independent (selected via Provider::OpenAI)");
    println!("   • Base URL: https://api.openai.com");
    println!("   • API Key: OPENAI_API_KEY environment variable");
    println!("   • Supported Models: gpt-3.5-turbo, gpt-4, gpt-4o");
    println!("   • Features: Chat completion, function calling, vision");

    Ok(())
}
