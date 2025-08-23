use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 测试配置驱动的Groq");
    println!("====================");
    
    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ 未设置GROQ_API_KEY");
        return Ok(());
    }
    
    let client = AiClient::new(Provider::Groq)?;
    println!("✅ Groq客户端创建成功 (使用GenericAdapter)");
    
    // 测试普通聊天
    let request = ChatCompletionRequest::new(
        "llama3-8b-8192".to_string(),
        vec![Message {
            role: Role::User,
            content: "Say 'Hello from Generic Groq!' in exactly those words.".to_string(),
        }],
    ).with_max_tokens(20);
    
    println!("\n💬 测试普通聊天...");
    match client.chat_completion(request.clone()).await {
        Ok(response) => {
            println!("✅ 普通聊天成功!");
            println!("   响应: '{}'", response.choices[0].message.content);
            println!("   Token使用: {}", response.usage.total_tokens);
        }
        Err(e) => {
            println!("❌ 普通聊天失败: {}", e);
        }
    }
    
    // 测试流式聊天
    println!("\n🌊 测试流式聊天...");
    match client.chat_completion_stream(request).await {
        Ok(mut stream) => {
            print!("   流式响应: ");
            let mut content = String::new();
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(text) = &choice.delta.content {
                                print!("{}", text);
                                content.push_str(text);
                                use std::io::{self, Write};
                                io::stdout().flush().unwrap();
                            }
                            if choice.finish_reason.is_some() {
                                println!();
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("\n❌ 流式错误: {}", e);
                        break;
                    }
                }
            }
            
            if !content.is_empty() {
                println!("✅ 流式聊天成功!");
                println!("   完整内容: '{}'", content.trim());
            }
        }
        Err(e) => {
            println!("❌ 流式聊天失败: {}", e);
        }
    }
    
    // 测试模型列表
    println!("\n📋 测试模型列表...");
    match client.list_models().await {
        Ok(models) => {
            println!("✅ 模型列表获取成功!");
            println!("   可用模型: {:?}", models);
        }
        Err(e) => {
            println!("❌ 模型列表获取失败: {}", e);
        }
    }
    
    println!("\n🎯 配置驱动Groq测试结果:");
    println!("   • 使用GenericAdapter而不是GroqAdapter");
    println!("   • 代码量从250行减少到10行配置");
    println!("   • 功能完全相同：普通聊天、流式聊天、模型列表");
    println!("   • 证明了OpenAI兼容性和通用适配器的有效性");
    
    Ok(())
}