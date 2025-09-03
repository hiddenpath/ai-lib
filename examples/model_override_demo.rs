use ai_lib::{AiClient, AiClientBuilder, Provider, ModelOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 检查环境变量
    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ 请设置 GROQ_API_KEY 环境变量");
        println!("   例如: export GROQ_API_KEY=your_api_key_here");
        return Ok(());
    }
    
    println!("🚀 模型覆盖功能演示");
    println!("==================");
    println!();
    
    // 1. 基础用法 - 保持原有简洁性
    println!("📋 1. 基础用法 - 使用默认模型");
    let reply = AiClient::quick_chat_text(Provider::Groq, "Hello!").await?;
    println!("   ✅ 响应: {}", reply);
    println!();
    
    // 2. 显式指定模型
    println!("📋 2. 显式指定模型");
    let reply = AiClient::quick_chat_text_with_model(
        Provider::Groq, 
        "Hello!", 
        "llama-3.1-8b-instant"
    ).await?;
    println!("   ✅ 响应: {}", reply);
    println!();
    
    // 3. 使用 ModelOptions
    println!("📋 3. 使用 ModelOptions");
    let options = ModelOptions::default()
        .with_chat_model("llama-3.1-8b-instant");
    
    let reply = AiClient::quick_chat_text_with_options(
        Provider::Groq, 
        "Hello!", 
        options
    ).await?;
    println!("   ✅ 响应: {}", reply);
    println!();
    
    // 4. AiClientBuilder 自定义默认模型
    println!("📋 4. AiClientBuilder 自定义默认模型");
    let client = AiClientBuilder::new(Provider::Groq)
        .with_default_chat_model("llama-3.1-8b-instant")
        .build()?;
    
    let request = client.build_simple_request("Hello!");
    println!("   使用模型: {}", request.model);
    let response = client.chat_completion(request).await?;
    match &response.choices[0].message.content {
        ai_lib::types::common::Content::Text(text) => {
            println!("   ✅ 响应: {}", text);
        }
        _ => println!("   ✅ 响应: {:?}", response.choices[0].message.content),
    }
    println!();
    
    // 5. 显式指定模型的 build_simple_request
    println!("📋 5. 显式指定模型的 build_simple_request");
    let client = AiClient::new(Provider::Groq)?;
    let request = client.build_simple_request_with_model(
        "Hello!",
        "llama-3.1-8b-instant"
    );
    println!("   使用模型: {}", request.model);
    let response = client.chat_completion(request).await?;
    match &response.choices[0].message.content {
        ai_lib::types::common::Content::Text(text) => {
            println!("   ✅ 响应: {}", text);
        }
        _ => println!("   ✅ 响应: {:?}", response.choices[0].message.content),
    }
    println!();
    
    println!("🎉 演示完成！");
    println!("=============");
    println!("✅ 所有模型覆盖功能都正常工作");
    println!("✅ 向后兼容性得到保证");
    println!("✅ 提供了灵活的模型指定方式");
    
    Ok(())
}
