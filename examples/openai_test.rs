use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 OpenAI Provider 测试");
    println!("=====================");
    
    // 检查API密钥
    match std::env::var("OPENAI_API_KEY") {
        Ok(_) => println!("✅ 检测到 OPENAI_API_KEY"),
        Err(_) => {
            println!("❌ 未设置 OPENAI_API_KEY 环境变量");
            println!("   请设置: export OPENAI_API_KEY=your_api_key");
            return Ok(());
        }
    }
    
    // 创建OpenAI客户端
    let client = AiClient::new(Provider::OpenAI)?;
    println!("✅ OpenAI客户端创建成功");
    
    // 获取模型列表
    println!("\n📋 获取OpenAI模型列表...");
    match client.list_models().await {
        Ok(models) => {
            println!("✅ 成功获取 {} 个模型", models.len());
            println!("   常用模型:");
            for model in models.iter().filter(|m| m.contains("gpt")) {
                println!("   • {}", model);
            }
        }
        Err(e) => println!("❌ 获取模型列表失败: {}", e),
    }
    
    // 测试聊天完成
    println!("\n💬 测试聊天完成...");
    let request = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message {
            role: Role::User,
            content: "Hello! Please respond with 'Hello from OpenAI!' to confirm the connection.".to_string(),
        }],
    ).with_max_tokens(20)
     .with_temperature(0.7);
    
    match client.chat_completion(request).await {
        Ok(response) => {
            println!("✅ 聊天完成成功!");
            println!("   模型: {}", response.model);
            println!("   响应: {}", response.choices[0].message.content);
            println!("   Token使用: {} (prompt: {}, completion: {})", 
                response.usage.total_tokens,
                response.usage.prompt_tokens,
                response.usage.completion_tokens
            );
        }
        Err(e) => println!("❌ 聊天完成失败: {}", e),
    }
    
    println!("\n🎯 OpenAI配置驱动测试完成!");
    println!("   这证明了配置驱动架构的强大之处：");
    println!("   • 无需编写OpenAI特定代码");
    println!("   • 只需在ProviderConfigs中添加配置");
    println!("   • 自动支持所有OpenAI兼容的功能");
    
    Ok(())
}