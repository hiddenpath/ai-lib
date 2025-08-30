use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  混合架构全面验证");
    println!("====================");

    let providers = vec![
        // 配置驱动适配器
        (
            Provider::Groq,
            "Groq",
            "配置驱动",
            "GenericAdapter",
            "llama3-8b-8192",
        ),
        (
            Provider::DeepSeek,
            "DeepSeek",
            "配置驱动",
            "GenericAdapter",
            "deepseek-chat",
        ),
        (
            Provider::Anthropic,
            "Anthropic",
            "配置驱动",
            "GenericAdapter + 特殊认证",
            "claude-3-5-sonnet-20241022",
        ),
        // 独立适配器
        (
            Provider::OpenAI,
            "OpenAI",
            "独立适配器",
            "OpenAiAdapter",
            "gpt-3.5-turbo",
        ),
        (
            Provider::Gemini,
            "Gemini",
            "独立适配器",
            "GeminiAdapter",
            "gemini-1.5-flash",
        ),
    ];

    println!("📊 架构对比:");
    println!(
        "   配置驱动: {} 个提供商",
        providers
            .iter()
            .filter(|(_, _, type_, _, _)| *type_ == "配置驱动")
            .count()
    );
    println!(
        "   独立适配器: {} 个提供商",
        providers
            .iter()
            .filter(|(_, _, type_, _, _)| *type_ == "独立适配器")
            .count()
    );

    for (provider, name, arch_type, impl_type, model) in providers {
        println!("\n{}", "=".repeat(50));
        println!("🔍 测试: {} ({})", name, arch_type);

        match AiClient::new(provider) {
            Ok(client) => {
                println!("✅ 客户端创建成功 ({})", impl_type);

                // 统一的API调用 - 证明接口一致性
                let request = ChatCompletionRequest::new(
                    model.to_string(),
                    vec![Message {
                        role: Role::User,
                        content: Content::Text(format!("Say 'Hello from {}!' exactly.", name)),
                        function_call: None,
                    }],
                )
                .with_max_tokens(20);

                match client.chat_completion(request).await {
                    Ok(response) => {
                        println!("✅ 统一API调用成功!");
                        println!(
                            "   响应: '{}'",
                            response.choices[0].message.content.as_text().trim()
                        );
                        println!("   Token: {}", response.usage.total_tokens);
                    }
                    Err(e) => {
                        println!("❌ API调用失败: {}", e);
                        if e.to_string().contains("402") || e.to_string().contains("Insufficient") {
                            println!("   (余额不足 - 连接正常)");
                        } else if e.to_string().contains("environment variable not set") {
                            println!("   (需要API密钥)");
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ 客户端创建失败: {}", e);
            }
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("🎯 混合架构优势总结:");

    println!("\n📈 配置驱动适配器 (GenericAdapter):");
    println!("   ✅ 极低扩展成本: ~15行配置添加新提供商");
    println!("   ✅ 代码复用: 共享HTTP、SSE、错误处理逻辑");
    println!("   ✅ 灵活配置: 支持不同端点、认证、字段映射");
    println!("   ✅ 适用场景: OpenAI兼容的API");

    println!("\n🔧 独立适配器:");
    println!("   ✅ 完全自定义: 处理任意API格式和协议");
    println!("   ✅ 特殊功能: 多模态、流式、复杂认证");
    println!("   ✅ 性能优化: 针对特定API的优化");
    println!("   ✅ 适用场景: 特殊API设计 (如Gemini)");

    println!("\n🏗️  架构价值:");
    println!("   🎯 统一接口: 用户无需关心底层实现差异");
    println!("   ⚡ 灵活扩展: 根据API特点选择最佳实现方式");
    println!("   🔄 渐进演进: 从配置驱动开始，需要时升级为独立适配器");
    println!("   📊 成本效益: 平衡开发效率与功能完整性");

    println!("\n🚀 渐进式架构演进完成:");
    println!("   ✅ 第一阶段: GenericAdapter + ProviderConfig 基础架构");
    println!("   ✅ 第二阶段: 多提供商配置驱动验证");
    println!("   ✅ 第三阶段: 混合架构 (配置驱动 + 独立适配器)");
    println!("   📋 第四阶段: 配置文件支持 (待实现)");

    Ok(())
}
