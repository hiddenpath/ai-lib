/// 配置驱动架构全面测试示例 - Comprehensive config-driven architecture test example
use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Comprehensive Config-Driven Architecture Test");
    println!("=============================================");

    let providers = vec![
        (Provider::Groq, "Groq", "llama3-8b-8192", "GROQ_API_KEY"),
        (
            Provider::DeepSeek,
            "DeepSeek",
            "deepseek-chat",
            "DEEPSEEK_API_KEY",
        ),
        (
            Provider::Anthropic,
            "Anthropic Claude",
            "claude-3-5-sonnet-20241022",
            "ANTHROPIC_API_KEY",
        ),
    ];

    println!("📊 Config-Driven Provider Statistics:");
    println!("   • Total: {} providers", providers.len());
    println!("   • Code volume: only ~10 lines of configuration per provider");
    println!("   • Reuse: shared GenericAdapter + SSE parsing");

    for (provider, name, model, api_key_env) in providers {
        println!("\n{}", "=".repeat(50));
        println!("🔍 Testing: {}", name);

        // Check API key
        match std::env::var(api_key_env) {
            Ok(_) => {
                println!("✅ API key set");

                // Create client
                match AiClient::new(provider) {
                    Ok(client) => {
                        println!("✅ Client created successfully (GenericAdapter)");

                        // Test chat
                        let request = ChatCompletionRequest::new(
                            model.to_string(),
                            vec![Message {
                                role: Role::User,
                                content: Content::Text(format!(
                                    "Say 'Hello from {}!' exactly.",
                                    name
                                )),
                                function_call: None,
                            }],
                        )
                        .with_max_tokens(20);

                        match client.chat_completion(request).await {
                            Ok(response) => {
                                println!("✅ Chat test successful!");
                                println!(
                                    "   Response: '{}'",
                                    response.choices[0].message.content.as_text()
                                );
                                println!("   Tokens: {}", response.usage.total_tokens);
                            }
                            Err(e) => {
                                println!("❌ Chat test failed: {}", e);
                                if e.to_string().contains("402")
                                    || e.to_string().contains("Insufficient")
                                {
                                    println!("   (Insufficient balance - connection OK)");
                                }
                            }
                        }

                        // Test model list
                        match client.list_models().await {
                            Ok(models) => {
                                println!("✅ Model list: {} models", models.len());
                            }
                            Err(_) => {
                                println!(
                                    "⚠️  Model list unavailable (some providers don't support)"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Client creation failed: {}", e);
                    }
                }
            }
            Err(_) => {
                println!("⚠️  {} not set, skipping test", api_key_env);
            }
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("🎯 Config-Driven Architecture Advantages Summary:");
    println!("   ✅ Code reuse: 3 providers share 1 set of SSE parsing logic");
    println!("   ✅ Zero-code expansion: add new providers with just configuration");
    println!("   ✅ Flexible configuration: support different endpoints, headers, field mapping");
    println!("   ✅ Unified interface: all providers use the same API");
    println!("   ✅ Streaming support: automatically get streaming response capability");

    println!("\n📈 Architecture Evolution Progress:");
    println!("   ✅ Phase 1: GenericAdapter + ProviderConfig");
    println!("   ✅ Phase 2: Predefined configurations (Groq, DeepSeek, Anthropic)");
    println!("   🔄 Phase 3: Hybrid architecture (config-driven + independent adapters)");
    println!("   📋 Phase 4: Configuration file support (to be implemented)");

    Ok(())
}
