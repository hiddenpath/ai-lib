use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试所有AI提供商");
    println!("==================");
    
    // 检查代理配置
    if let Ok(proxy_url) = std::env::var("AI_PROXY_URL") {
        println!("🌐 使用代理: {}", proxy_url);
    }
    
    let providers = vec![
        (Provider::Groq, "Groq", "llama3-8b-8192"),
        (Provider::OpenAI, "OpenAI", "gpt-3.5-turbo"),
        (Provider::DeepSeek, "DeepSeek", "deepseek-chat"),
    ];
    
    for (provider, name, model) in providers {
        println!("\n🔍 测试提供商: {}", name);
        println!("{}", "─".repeat(30));
        
        match AiClient::new(provider) {
            Ok(client) => {
                println!("✅ 客户端创建成功");
                
                // 测试模型列表
                match client.list_models().await {
                    Ok(models) => {
                        println!("📋 可用模型数量: {}", models.len());
                        if !models.is_empty() {
                            println!("   前3个模型: {:?}", &models[..models.len().min(3)]);
                        }
                    }
                    Err(e) => println!("⚠️  获取模型列表失败: {}", e),
                }
                
                // 测试聊天完成
                let request = ChatCompletionRequest::new(
                    model.to_string(),
                    vec![Message {
                        role: Role::User,
                        content: "Hello! Please respond with just 'Hi' to test the API.".to_string(),
                    }],
                ).with_max_tokens(10);
                
                println!("📤 发送测试请求到模型: {}", model);
                match client.chat_completion(request).await {
                    Ok(response) => {
                        println!("✅ 请求成功!");
                        println!("   响应ID: {}", response.id);
                        println!("   内容: {}", response.choices[0].message.content);
                        println!("   使用tokens: {}", response.usage.total_tokens);
                    }
                    Err(e) => println!("❌ 请求失败: {}", e),
                }
            }
            Err(e) => {
                println!("❌ 客户端创建失败: {}", e);
            }
        }
    }
    
    println!("\n💡 提示:");
    println!("   • 确保设置了对应的API密钥环境变量");
    println!("   • GROQ_API_KEY, OPENAI_API_KEY, DEEPSEEK_API_KEY");
    println!("   • 可选设置AI_PROXY_URL使用代理服务器");
    
    Ok(())
}