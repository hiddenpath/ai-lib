use ai_lib::{AiClient, Provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 网络连接诊断");
    println!("================");

    // 检查代理设置
    match std::env::var("AI_PROXY_URL") {
        Ok(proxy_url) => {
            println!("🌐 当前代理设置: {}", proxy_url);
        }
        Err(_) => {
            println!("ℹ️  未设置代理");
        }
    }

    // 检查API密钥
    let providers = vec![
        ("GROQ_API_KEY", Provider::Groq, "Groq"),
        ("OPENAI_API_KEY", Provider::OpenAI, "OpenAI"),
        ("DEEPSEEK_API_KEY", Provider::DeepSeek, "DeepSeek"),
    ];

    println!("\n🔑 API密钥检查:");
    for (env_var, provider, name) in &providers {
        match std::env::var(env_var) {
            Ok(_) => {
                println!("   ✅ {}: 已设置", name);

                // 测试客户端创建
                match AiClient::new(*provider) {
                    Ok(_) => println!("      ✅ 客户端创建成功"),
                    Err(e) => println!("      ❌ 客户端创建失败: {}", e),
                }
            }
            Err(_) => {
                println!("   ❌ {}: 未设置", name);
            }
        }
    }

    // 测试基本网络连接
    println!("\n🌐 网络连接测试:");

    // 使用reqwest直接测试
    let client = reqwest::Client::new();

    // 测试DeepSeek (国内)
    println!("   测试DeepSeek连接...");
    match client
        .get("https://api.deepseek.com/v1/models")
        .send()
        .await
    {
        Ok(response) => {
            println!("      ✅ DeepSeek连接成功 (状态: {})", response.status());
        }
        Err(e) => {
            println!("      ❌ DeepSeek连接失败: {}", e);
        }
    }

    println!("\n💡 诊断建议:");
    println!("   1. 确保网络连接正常");
    println!("   2. 检查防火墙设置");
    println!("   3. 验证代理服务器配置");
    println!("   4. 确认API密钥有效性");

    Ok(())
}
