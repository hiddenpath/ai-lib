/// 测试所有AI提供商示例 - Test all AI providers example
use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Test All AI Providers");
    println!("=======================");

    // Check proxy configuration
    if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
        println!("🌐 Using proxy: {}", proxy_url);
    }

    let providers = vec![
        (Provider::Groq, "Groq", "llama3-8b-8192"),
        (Provider::OpenAI, "OpenAI", "gpt-3.5-turbo"),
        (Provider::DeepSeek, "DeepSeek", "deepseek-chat"),
    ];

    for (provider, name, model) in providers {
        println!("\n🔍 Testing Provider: {}", name);
        println!("{}", "─".repeat(30));

        match AiClient::new(provider) {
            Ok(client) => {
                println!("✅ Client created successfully");

                // Test model list
                match client.list_models().await {
                    Ok(models) => {
                        println!("📋 Available models count: {}", models.len());
                        if !models.is_empty() {
                            println!("   First 3 models: {:?}", &models[..models.len().min(3)]);
                        }
                    }
                    Err(e) => println!("⚠️  Failed to get model list: {}", e),
                }

                // Test chat completion
                let request = ChatCompletionRequest::new(
                    model.to_string(),
                    vec![Message {
                        role: Role::User,
                        content: Content::Text(
                            "Hello! Please respond with just 'Hi' to test the API.".to_string(),
                        ),
                        function_call: None,
                    }],
                )
                .with_max_tokens(10);

                println!("📤 Sending test request to model: {}", model);
                match client.chat_completion(request).await {
                    Ok(response) => {
                        println!("✅ Request successful!");
                        println!("   Response ID: {}", response.id);
                        println!(
                            "   Content: {}",
                            response.choices[0].message.content.as_text()
                        );
                        println!("   Tokens used: {}", response.usage.total_tokens);
                    }
                    Err(e) => println!("❌ Request failed: {}", e),
                }
            }
            Err(e) => {
                println!("❌ Client creation failed: {}", e);
            }
        }
    }

    println!("\n💡 Tips:");
    println!("   • Make sure to set corresponding API key environment variables");
    println!("   • GROQ_API_KEY, OPENAI_API_KEY, DEEPSEEK_API_KEY");
    println!("   • Optionally set AI_PROXY_URL to use proxy server");

    Ok(())
}
