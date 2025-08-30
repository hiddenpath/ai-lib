use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 配置驱动架构全面测试");
    println!("========================");

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

    println!("📊 配置驱动提供商统计:");
    println!("   • 总数: {} 个提供商", providers.len());
    println!("   • 代码量: 每个仅需 ~10行配置");
    println!("   • 复用: 共享GenericAdapter + SSE解析");

    for (provider, name, model, api_key_env) in providers {
        println!("\n{}", "=".repeat(50));
        println!("🔍 测试: {}", name);

        // 检查API密钥
        match std::env::var(api_key_env) {
            Ok(_) => {
                println!("✅ API密钥已设置");

                // 创建客户端
                match AiClient::new(provider) {
                    Ok(client) => {
                        println!("✅ 客户端创建成功 (GenericAdapter)");

                        // 测试聊天
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
                                println!("✅ 聊天测试成功!");
                                println!(
                                    "   响应: '{}'",
                                    response.choices[0].message.content.as_text()
                                );
                                println!("   Token: {}", response.usage.total_tokens);
                            }
                            Err(e) => {
                                println!("❌ 聊天测试失败: {}", e);
                                if e.to_string().contains("402")
                                    || e.to_string().contains("Insufficient")
                                {
                                    println!("   (余额不足 - 连接正常)");
                                }
                            }
                        }

                        // 测试模型列表
                        match client.list_models().await {
                            Ok(models) => {
                                println!("✅ 模型列表: {} 个模型", models.len());
                            }
                            Err(_) => {
                                println!("⚠️  模型列表不可用 (某些提供商不支持)");
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ 客户端创建失败: {}", e);
                    }
                }
            }
            Err(_) => {
                println!("⚠️  {} 未设置，跳过测试", api_key_env);
            }
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("🎯 配置驱动架构优势总结:");
    println!("   ✅ 代码复用: 3个提供商共享1套SSE解析逻辑");
    println!("   ✅ 零代码扩展: 新增提供商只需配置");
    println!("   ✅ 灵活配置: 支持不同端点、请求头、字段映射");
    println!("   ✅ 统一接口: 所有提供商使用相同API");
    println!("   ✅ 流式支持: 自动获得流式响应能力");

    println!("\n📈 架构演进进度:");
    println!("   ✅ 第一阶段: GenericAdapter + ProviderConfig");
    println!("   ✅ 第二阶段: 预定义配置 (Groq, DeepSeek, Anthropic)");
    println!("   🔄 第三阶段: 混合架构 (配置驱动 + 独立适配器)");
    println!("   📋 第四阶段: 配置文件支持 (待实现)");

    Ok(())
}
