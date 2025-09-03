use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
use ai_lib::types::common::Content;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 检查环境变量
    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ 请设置 GROQ_API_KEY 环境变量");
        println!("   例如: export GROQ_API_KEY=your_api_key_here");
        println!("   或者在 .env 文件中设置");
        return Ok(());
    }
    
    println!("🔧 使用新的Provider分类系统创建Groq客户端...");
    
    // 创建Groq客户端 - 使用新的provider分类系统
    let client = AiClient::new(Provider::Groq)?;
    
    // 创建聊天请求
    let request = ChatCompletionRequest::new(
        "llama-3.1-8b-instant".to_string(), // Groq的可用模型
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello! Please respond with a simple greeting.".to_string()),
            function_call: None,
        }],
    );
    
    println!("🚀 Sending request to Groq...");
    println!("📝 Request: Hello! Please respond with a simple greeting.");
    println!();
    
    // 发送请求并获取响应
    let response = client.chat_completion(request).await?;
    
    println!("✅ Groq Response:");
    match &response.choices[0].message.content {
        Content::Text(text) => println!("{}", text),
        Content::Json(json) => println!("JSON: {:?}", json),
        Content::Image { url, mime, name } => println!("Image: url={:?}, mime={:?}, name={:?}", url, mime, name),
        Content::Audio { url, mime } => println!("Audio: url={:?}, mime={:?}", url, mime),
    }
    
    Ok(())
}
