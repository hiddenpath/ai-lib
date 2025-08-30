use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 代理功能测试");
    println!("================");

    // 检查代理设置
    let proxy_url = match std::env::var("AI_PROXY_URL") {
        Ok(url) => {
            println!("🌐 使用代理: {}", url);
            url
        }
        Err(_) => {
            println!("❌ 未设置AI_PROXY_URL，无法测试");
            return Ok(());
        }
    };

    // 测试代理是否能正常工作
    println!("\n📡 测试代理连接...");

    // 使用reqwest测试代理
    let proxy = reqwest::Proxy::all(&proxy_url)?;
    let client = reqwest::Client::builder().proxy(proxy).build()?;

    // 测试一个简单的HTTP请求
    match client.get("https://httpbin.org/ip").send().await {
        Ok(response) => {
            println!("✅ 代理连接成功!");
            if let Ok(text) = response.text().await {
                println!("   响应: {}", text);
            }
        }
        Err(e) => {
            println!("❌ 代理连接失败: {}", e);
            println!("   请检查代理服务器是否正常运行");
            return Ok(());
        }
    }

    // 测试AI服务
    println!("\n🤖 测试AI服务连接...");

    // 测试Groq (通过代理)
    if std::env::var("GROQ_API_KEY").is_ok() {
        println!("   测试Groq...");
        match AiClient::new(Provider::Groq) {
            Ok(client) => {
                match client.list_models().await {
                    Ok(models) => {
                        println!("      ✅ Groq模型列表获取成功 ({} 个模型)", models.len());

                        // 尝试简单的聊天
                        let request = ChatCompletionRequest::new(
                            "llama3-8b-8192".to_string(),
                            vec![Message {
                                role: Role::User,
                                content: Content::Text("Say 'Hello' in one word.".to_string()),
                                function_call: None,
                            }],
                        )
                        .with_max_tokens(5);

                        match client.chat_completion(request).await {
                            Ok(response) => {
                                println!("      ✅ Groq聊天测试成功!");
                                println!(
                                    "         响应: {}",
                                    response.choices[0].message.content.as_text()
                                );
                            }
                            Err(e) => {
                                println!("      ❌ Groq聊天测试失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("      ❌ Groq模型列表获取失败: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("      ❌ Groq客户端创建失败: {}", e);
            }
        }
    }

    println!("\n💡 测试结果分析:");
    println!("   • 如果代理连接成功但AI服务失败，可能是:");
    println!("     - 代理服务器不支持HTTPS");
    println!("     - 代理服务器有访问限制");
    println!("     - API密钥无效");
    println!("   • 如果代理连接失败，请检查:");
    println!("     - 代理服务器地址和端口");
    println!("     - 代理服务器是否需要认证");
    println!("     - 网络防火墙设置");

    Ok(())
}
