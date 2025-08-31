/// 配置驱动的AI-lib示例 - Config-driven AI-lib example
use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Config-driven AI-lib Example");
    println!("================================");

    // Demonstrate the advantages of config-driven approach: easy provider switching
    let providers = vec![
        (Provider::Groq, "Groq"),
        (Provider::OpenAI, "OpenAI"),
        (Provider::DeepSeek, "DeepSeek"),
    ];

    for (provider, name) in providers {
        println!("\n📡 Testing Provider: {}", name);

        // Create client - just change the enum value
        let client = AiClient::new(provider)?;
        println!(
            "✅ Client created successfully: {:?}",
            client.current_provider()
        );

        // Get model list
        match client.list_models().await {
            Ok(models) => println!("📋 Available models: {:?}", models),
            Err(e) => println!("⚠️  Failed to get model list: {}", e),
        }

        // Create test request
        let request = ChatCompletionRequest::new(
            "test-model".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text("Hello from ai-lib!".to_string()),
                function_call: None,
            }],
        );

        println!("📤 Request prepared, model: {}", request.model);
        println!("   (Need to set corresponding API_KEY environment variable for actual calls)");
    }

    println!("\n🎯 Core advantages of config-driven approach:");
    println!("   • Zero-code switching: just change Provider enum value");
    println!("   • Unified interface: all providers use the same API");
    println!("   • Rapid expansion: add new compatible providers with just configuration");

    Ok(())
}
