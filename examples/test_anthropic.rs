/// Anthropic Claude 测试示例 - Anthropic Claude test example
use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Anthropic Claude Test");
    println!("========================");

    // Check API key
    match std::env::var("ANTHROPIC_API_KEY") {
        Ok(_) => println!("✅ ANTHROPIC_API_KEY detected"),
        Err(_) => {
            println!("❌ ANTHROPIC_API_KEY environment variable not set");
            println!("   Please set: export ANTHROPIC_API_KEY=your_api_key");
            println!("   Get API key: https://console.anthropic.com/");
            return Ok(());
        }
    }

    // Create Anthropic client
    let client = AiClient::new(Provider::Anthropic)?;
    println!("✅ Anthropic client created successfully (using GenericAdapter)");

    // Test chat completion
    println!("\n💬 Testing Claude chat...");
    let request = ChatCompletionRequest::new(
        "claude-3-5-sonnet-20241022".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello Claude! Please respond with 'Hello from Anthropic Claude via ai-lib!' to confirm the connection works.".to_string()),
            function_call: None,
        }],
    ).with_max_tokens(50);

    match client.chat_completion(request).await {
        Ok(response) => {
            println!("✅ Claude chat successful!");
            println!("   Model: {}", response.model);
            println!(
                "   Response: '{}'",
                response.choices[0].message.content.as_text()
            );
            println!(
                "   Token usage: {} (prompt: {}, completion: {})",
                response.usage.total_tokens,
                response.usage.prompt_tokens,
                response.usage.completion_tokens
            );
        }
        Err(e) => {
            println!("❌ Claude chat failed: {}", e);

            // Analyze error type
            let error_str = e.to_string();
            if error_str.contains("401") {
                println!("   → Authentication error, please check ANTHROPIC_API_KEY");
            } else if error_str.contains("400") {
                println!("   → Request format error, may need to adjust configuration");
            } else if error_str.contains("429") {
                println!("   → Rate limit, please try again later");
            }
        }
    }

    // Test model list (Claude may not support this)
    println!("\n📋 Testing model list...");
    match client.list_models().await {
        Ok(models) => {
            println!("✅ Model list retrieved successfully: {:?}", models);
        }
        Err(e) => {
            println!("⚠️  Failed to get model list: {}", e);
            println!("   (Claude may not provide public model list endpoint)");
        }
    }

    println!("\n🎯 Anthropic Config-driven Test Summary:");
    println!("   • Uses GenericAdapter + ProviderConfig");
    println!("   • Custom endpoint: /messages (instead of /chat/completions)");
    println!("   • Custom headers: anthropic-version");
    println!("   • Demonstrates the flexibility of config-driven architecture");

    Ok(())
}
