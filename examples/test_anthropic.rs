use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Anthropic Claude 测试");
    println!("=======================");
    
    // 检查API密钥
    match std::env::var("ANTHROPIC_API_KEY") {
        Ok(_) => println!("✅ 检测到 ANTHROPIC_API_KEY"),
        Err(_) => {
            println!("❌ 未设置 ANTHROPIC_API_KEY 环境变量");
            println!("   请设置: export ANTHROPIC_API_KEY=your_api_key");
            println!("   获取API密钥: https://console.anthropic.com/");
            return Ok(());
        }
    }
    
    // 创建Anthropic客户端
    let client = AiClient::new(Provider::Anthropic)?;
    println!("✅ Anthropic客户端创建成功 (使用GenericAdapter)");
    
    // 测试聊天完成
    println!("\n💬 测试Claude聊天...");
    let request = ChatCompletionRequest::new(
        "claude-3-5-sonnet-20241022".to_string(),
        vec![Message {
            role: Role::User,
            content: "Hello Claude! Please respond with 'Hello from Anthropic Claude via ai-lib!' to confirm the connection works.".to_string(),
        }],
    ).with_max_tokens(50);
    
    match client.chat_completion(request).await {
        Ok(response) => {
            println!("✅ Claude聊天成功!");
            println!("   模型: {}", response.model);
            println!("   响应: '{}'", response.choices[0].message.content);
            println!("   Token使用: {} (prompt: {}, completion: {})", 
                response.usage.total_tokens,
                response.usage.prompt_tokens,
                response.usage.completion_tokens
            );
        }
        Err(e) => {
            println!("❌ Claude聊天失败: {}", e);
            
            // 分析错误类型
            let error_str = e.to_string();
            if error_str.contains("401") {
                println!("   → 认证错误，请检查ANTHROPIC_API_KEY");
            } else if error_str.contains("400") {
                println!("   → 请求格式错误，可能需要调整配置");
            } else if error_str.contains("429") {
                println!("   → 速率限制，请稍后重试");
            }
        }
    }
    
    // 测试模型列表（Claude可能不支持）
    println!("\n📋 测试模型列表...");
    match client.list_models().await {
        Ok(models) => {
            println!("✅ 模型列表获取成功: {:?}", models);
        }
        Err(e) => {
            println!("⚠️  模型列表获取失败: {}", e);
            println!("   (Claude可能不提供公开的模型列表端点)");
        }
    }
    
    println!("\n🎯 Anthropic配置驱动测试总结:");
    println!("   • 使用GenericAdapter + ProviderConfig");
    println!("   • 自定义端点: /messages (而不是/chat/completions)");
    println!("   • 自定义请求头: anthropic-version");
    println!("   • 证明配置驱动架构的灵活性");
    
    Ok(())
}