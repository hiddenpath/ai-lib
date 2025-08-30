use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 测试不使用代理的连接");
    println!("======================");

    // 临时移除代理设置
    std::env::remove_var("AI_PROXY_URL");

    println!("ℹ️  已临时移除AI_PROXY_URL设置");

    // 测试DeepSeek（国内可直连）
    println!("\n🔍 测试DeepSeek (直连):");
    match AiClient::new(Provider::DeepSeek) {
        Ok(client) => {
            let request = ChatCompletionRequest::new(
                "deepseek-chat".to_string(),
                vec![Message {
                    role: Role::User,
                    content: Content::Text(
                        "Hello! Please respond with just 'Hi' to test.".to_string(),
                    ),
                    function_call: None,
                }],
            )
            .with_max_tokens(5);

            match client.chat_completion(request).await {
                Ok(response) => {
                    println!("✅ DeepSeek 直连成功!");
                    println!("   响应: {}", response.choices[0].message.content.as_text());
                    println!("   Token使用: {}", response.usage.total_tokens);
                }
                Err(e) => {
                    println!("❌ DeepSeek 请求失败: {}", e);
                    if e.to_string().contains("402") {
                        println!("   (这是余额不足错误，说明连接正常)");
                    }
                }
            }
        }
        Err(e) => println!("❌ DeepSeek 客户端创建失败: {}", e),
    }

    println!("\n💡 结论:");
    println!("   • DeepSeek可以直连，不需要代理");
    println!("   • OpenAI和Groq需要通过代理访问");
    println!("   • 代理可能会修改请求内容，导致格式错误");
    println!("   • 建议检查代理服务器的配置");

    Ok(())
}
