use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 OpenAI 专项测试");
    println!("==================");
    
    // 检查OpenAI API密钥
    match std::env::var("OPENAI_API_KEY") {
        Ok(key) => {
            let masked = format!("{}...{}", &key[..8], &key[key.len()-4..]);
            println!("🔑 OpenAI API Key: {}", masked);
        }
        Err(_) => {
            println!("❌ 未设置OPENAI_API_KEY");
            return Ok(());
        }
    }
    
    // 创建OpenAI客户端
    println!("\n📡 创建OpenAI客户端...");
    let client = match AiClient::new(Provider::OpenAI) {
        Ok(client) => {
            println!("✅ 客户端创建成功");
            client
        }
        Err(e) => {
            println!("❌ 客户端创建失败: {}", e);
            return Ok(());
        }
    };
    
    // 测试模型列表
    println!("\n📋 获取模型列表...");
    match client.list_models().await {
        Ok(models) => {
            println!("✅ 成功获取 {} 个模型", models.len());
            
            // 显示GPT模型
            let gpt_models: Vec<_> = models.iter()
                .filter(|m| m.contains("gpt"))
                .take(5)
                .collect();
            println!("   GPT模型: {:?}", gpt_models);
        }
        Err(e) => {
            println!("❌ 获取模型列表失败: {}", e);
            return Ok(());
        }
    }
    
    // 测试聊天完成
    println!("\n💬 测试聊天完成...");
    let request = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message {
            role: Role::User,
            content: "Say 'Hello from OpenAI!' in exactly those words.".to_string(),
        }],
    ).with_max_tokens(20)
     .with_temperature(0.0);  // 使用0温度确保一致性
    
    match client.chat_completion(request).await {
        Ok(response) => {
            println!("✅ 聊天完成成功!");
            println!("   模型: {}", response.model);
            println!("   响应: '{}'", response.choices[0].message.content);
            println!("   Token使用: {} (prompt: {}, completion: {})", 
                response.usage.total_tokens,
                response.usage.prompt_tokens,
                response.usage.completion_tokens
            );
            println!("   完成原因: {:?}", response.choices[0].finish_reason);
        }
        Err(e) => {
            println!("❌ 聊天完成失败: {}", e);
            
            // 分析错误类型
            let error_str = e.to_string();
            if error_str.contains("400") {
                println!("   → 这是请求格式错误");
            } else if error_str.contains("401") {
                println!("   → 这是认证错误，检查API密钥");
            } else if error_str.contains("429") {
                println!("   → 这是速率限制错误");
            } else if error_str.contains("500") {
                println!("   → 这是服务器错误");
            }
        }
    }
    
    println!("\n🎯 OpenAI测试完成!");
    
    Ok(())
}