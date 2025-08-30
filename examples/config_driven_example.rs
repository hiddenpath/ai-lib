use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 配置驱动的AI-lib示例");
    println!("========================");

    // 演示配置驱动的优势：轻松切换提供商
    let providers = vec![
        (Provider::Groq, "Groq"),
        (Provider::OpenAI, "OpenAI"),
        (Provider::DeepSeek, "DeepSeek"),
    ];

    for (provider, name) in providers {
        println!("\n📡 测试提供商: {}", name);

        // 创建客户端 - 只需改变枚举值
        let client = AiClient::new(provider)?;
        println!("✅ 客户端创建成功: {:?}", client.current_provider());

        // 获取模型列表
        match client.list_models().await {
            Ok(models) => println!("📋 可用模型: {:?}", models),
            Err(e) => println!("⚠️  获取模型列表失败: {}", e),
        }

        // 创建测试请求
        let request = ChatCompletionRequest::new(
            "test-model".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text("Hello from ai-lib!".to_string()),
                function_call: None,
            }],
        );

        println!("📤 请求已准备，模型: {}", request.model);
        println!("   (需要设置对应的API_KEY环境变量才能实际调用)");
    }

    println!("\n🎯 配置驱动的核心优势:");
    println!("   • 零代码切换: 只需改变Provider枚举值");
    println!("   • 统一接口: 所有提供商使用相同的API");
    println!("   • 快速扩展: 新增兼容提供商只需添加配置");

    Ok(())
}
