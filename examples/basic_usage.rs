use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI-lib Basic Usage Example");
    println!("================================");
    
    // 切换模型提供商，只需更改 Provider 的值
    let client = AiClient::new(Provider::Groq)?;
    println!("✅ Created client with provider: {:?}", client.current_provider());
    
    // 获取支持的模型列表
    let models = client.list_models().await?;
    println!("📋 Available models: {:?}", models);
    
    // 创建聊天请求
    let request = ChatCompletionRequest::new(
        "llama3-8b-8192".to_string(),
        vec![Message {
            role: Role::User,
            content: "Hello! Please introduce yourself briefly.".to_string(),
        }],
    ).with_temperature(0.7)
     .with_max_tokens(100);
    
    println!("📤 Sending request to model: {}", request.model);
    
    // 发送请求
    let response = client.chat_completion(request).await?;
    
    println!("📥 Received response:");
    println!("   ID: {}", response.id);
    println!("   Model: {}", response.model);
    println!("   Content: {}", response.choices[0].message.content);
    println!("   Usage: {} tokens", response.usage.total_tokens);
    
    Ok(())
}
