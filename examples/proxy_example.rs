use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 AI-lib 代理服务器支持示例");
    println!("============================");

    // 检查代理配置
    match std::env::var("AI_PROXY_URL") {
        Ok(proxy_url) => {
            println!("✅ 检测到代理配置: {}", proxy_url);
            println!("   所有HTTP请求将通过此代理服务器");
        }
        Err(_) => {
            println!("ℹ️  未设置AI_PROXY_URL环境变量");
            println!("   如需使用代理，请设置: export AI_PROXY_URL=http://proxy.example.com:8080");
        }
    }

    println!("\n🚀 创建AI客户端...");
    let client = AiClient::new(Provider::Groq)?;
    println!("✅ 客户端创建成功，提供商: {:?}", client.current_provider());

    // 创建测试请求
    let request = ChatCompletionRequest::new(
        "llama3-8b-8192".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello! This request may go through a proxy.".to_string()),
            function_call: None,
        }],
    );

    println!("\n📤 准备发送请求...");
    println!("   模型: {}", request.model);
    println!("   消息: {}", request.messages[0].content.as_text());

    // 获取模型列表（这个请求也会通过代理）
    match client.list_models().await {
        Ok(models) => {
            println!("\n📋 通过代理获取到的模型列表:");
            for model in models {
                println!("   • {}", model);
            }
        }
        Err(e) => {
            println!("\n⚠️  获取模型列表失败: {}", e);
            println!("   这可能是由于:");
            println!("   • 未设置GROQ_API_KEY环境变量");
            println!("   • 代理服务器配置错误");
            println!("   • 网络连接问题");
        }
    }

    println!("\n💡 代理配置说明:");
    println!("   • 设置环境变量: AI_PROXY_URL=http://your-proxy:port");
    println!("   • 支持HTTP和HTTPS代理");
    println!("   • 支持带认证的代理: http://user:pass@proxy:port");
    println!("   • 所有AI提供商都会自动使用此代理配置");

    Ok(())
}
