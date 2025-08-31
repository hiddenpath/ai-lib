/// OpenAI Provider 测试示例 - OpenAI Provider test example
use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 OpenAI Provider Test");
    println!("======================");

    // Check API key
    match std::env::var("OPENAI_API_KEY") {
        Ok(_) => println!("✅ OPENAI_API_KEY detected"),
        Err(_) => {
            println!("❌ OPENAI_API_KEY environment variable not set");
            println!("   Please set: export OPENAI_API_KEY=your_api_key");
            return Ok(());
        }
    }

    // Create OpenAI client
    let client = AiClient::new(Provider::OpenAI)?;
    println!("✅ OpenAI client created successfully");

    // Get model list
    println!("\n📋 Getting OpenAI model list...");
    match client.list_models().await {
        Ok(models) => {
            println!("✅ Successfully got {} models", models.len());
            println!("   Common models:");
            for model in models.iter().filter(|m| m.contains("gpt")) {
                println!("   • {}", model);
            }
        }
        Err(e) => println!("❌ Failed to get model list: {}", e),
    }

    // Test chat completion
    println!("\n💬 Testing chat completion...");
    let request = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text(
                "Hello! Please respond with 'Hello from OpenAI!' to confirm the connection."
                    .to_string(),
            ),
            function_call: None,
        }],
    )
    .with_max_tokens(20)
    .with_temperature(0.7);

    match client.chat_completion(request).await {
        Ok(response) => {
            println!("✅ Chat completion successful!");
            println!("   Model: {}", response.model);
            println!(
                "   Response: {}",
                response.choices[0].message.content.as_text()
            );
            println!(
                "   Token usage: {} (prompt: {}, completion: {})",
                response.usage.total_tokens,
                response.usage.prompt_tokens,
                response.usage.completion_tokens
            );
        }
        Err(e) => println!("❌ Chat completion failed: {}", e),
    }

    println!("\n🎯 OpenAI config-driven test completed!");
    println!("   This demonstrates the power of config-driven architecture:");
    println!("   • No need to write OpenAI-specific code");
    println!("   • Just add configuration in ProviderConfigs");
    println!("   • Automatically supports all OpenAI-compatible features");

    Ok(())
}
