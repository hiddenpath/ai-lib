use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 HTTPS代理测试");
    println!("================");
    
    // 临时设置HTTPS代理
    std::env::set_var("AI_PROXY_URL", "https://192.168.2.13:8889");
    println!("🌐 使用HTTPS代理: https://192.168.2.13:8889");
    
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("❌ 未设置OPENAI_API_KEY");
        return Ok(());
    }
    
    // 测试OpenAI
    println!("\n🤖 测试OpenAI (HTTPS代理):");
    let client = AiClient::new(Provider::OpenAI)?;
    
    let request = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message {
            role: Role::User,
            content: "Say 'HTTPS proxy works!' exactly.".to_string(),
        }],
    ).with_max_tokens(10);
    
    match client.chat_completion(request).await {
        Ok(response) => {
            println!("✅ HTTPS代理测试成功!");
            println!("   响应: '{}'", response.choices[0].message.content);
            println!("   Token使用: {}", response.usage.total_tokens);
        }
        Err(e) => {
            println!("❌ HTTPS代理测试失败: {}", e);
            
            // 分析错误
            let error_str = e.to_string();
            if error_str.contains("you must provide a model parameter") {
                println!("   → 这可能是代理服务器的问题，而不是HTTPS协议问题");
            } else if error_str.contains("certificate") || error_str.contains("tls") {
                println!("   → HTTPS证书或TLS相关问题");
            } else if error_str.contains("connection") {
                println!("   → 连接问题，可能是代理服务器配置");
            }
        }
    }
    
    // 测试Groq (对比)
    println!("\n🚀 测试Groq (HTTPS代理对比):");
    if std::env::var("GROQ_API_KEY").is_ok() {
        let groq_client = AiClient::new(Provider::Groq)?;
        let groq_request = ChatCompletionRequest::new(
            "llama3-8b-8192".to_string(),
            vec![Message {
                role: Role::User,
                content: "Say 'Groq HTTPS proxy works!' exactly.".to_string(),
            }],
        ).with_max_tokens(10);
        
        match groq_client.chat_completion(groq_request).await {
            Ok(response) => {
                println!("✅ Groq HTTPS代理成功!");
                println!("   响应: '{}'", response.choices[0].message.content);
            }
            Err(e) => {
                println!("❌ Groq HTTPS代理失败: {}", e);
            }
        }
    }
    
    println!("\n💡 HTTPS代理测试结论:");
    println!("   • 如果Groq成功而OpenAI失败，说明是OpenAI特定问题");
    println!("   • 如果都失败，可能是HTTPS代理配置问题");
    println!("   • 如果都成功，说明HTTPS代理完全支持");
    
    Ok(())
}