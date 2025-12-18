use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DeepSeek Basic Usage Example");
    println!("================================");

    // Check DEEPSEEK_API_KEY environment variable
    if std::env::var("DEEPSEEK_API_KEY").is_err() {
        println!("❌ Please set DEEPSEEK_API_KEY environment variable");
        println!("   Example: export DEEPSEEK_API_KEY=your_api_key_here");
        return Ok(());
    }

    println!("🔧 Creating DeepSeek client using config-driven adapter...");
    let client = AiClient::new(Provider::DeepSeek)?;
    println!(
        "✅ Created client with provider: {:?}",
        client.provider_name()
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

    // Show DeepSeek-specific features
    println!("\n🔍 DeepSeek Provider Information:");
    println!("   • Provider Type: Config-driven (selected via Provider::DeepSeek)");
    println!("   • Base URL: https://api.deepseek.com/v1");
    println!("   • API Key: DEEPSEEK_API_KEY environment variable");
    println!("   • Supported Models: deepseek-chat, deepseek-coder");
    println!("   • Features: Chat completion, code generation, reasoning");

    Ok(())
}
