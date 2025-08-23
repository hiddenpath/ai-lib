use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 清洁版流式响应测试");
    println!("======================");
    
    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ 未设置GROQ_API_KEY");
        return Ok(());
    }
    
    let client = AiClient::new(Provider::Groq)?;
    println!("✅ Groq客户端创建成功");
    
    let request = ChatCompletionRequest::new(
        "llama3-8b-8192".to_string(),
        vec![Message {
            role: Role::User,
            content: "Write a haiku about programming.".to_string(),
        }],
    ).with_max_tokens(50)
     .with_temperature(0.8);
    
    println!("\n📤 发送流式请求: {}", request.messages[0].content);
    
    match client.chat_completion_stream(request).await {
        Ok(mut stream) => {
            println!("\n🎭 AI回复:");
            print!("   ");
            
            let mut content_parts = Vec::new();
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                // 尝试解析JSON内容
                                if content.contains("\"content\":") {
                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
                                        if let Some(text) = json["content"].as_str() {
                                            if !text.is_empty() {
                                                print!("{}", text);
                                                content_parts.push(text.to_string());
                                                use std::io::{self, Write};
                                                io::stdout().flush().unwrap();
                                            }
                                        }
                                    }
                                } else if !content.trim().is_empty() && !content.contains("data:") {
                                    // 直接输出非JSON内容
                                    print!("{}", content);
                                    content_parts.push(content.clone());
                                    use std::io::{self, Write};
                                    io::stdout().flush().unwrap();
                                }
                            }
                            
                            if choice.finish_reason.is_some() {
                                println!("\n");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("\n❌ 流式响应错误: {}", e);
                        break;
                    }
                }
            }
            
            let full_content = content_parts.join("");
            if !full_content.is_empty() {
                println!("✅ 流式响应完成!");
                println!("📝 完整内容: \"{}\"", full_content.trim());
            } else {
                println!("⚠️  未提取到有效内容，可能需要改进SSE解析");
            }
        }
        Err(e) => {
            println!("❌ 流式请求失败: {}", e);
        }
    }
    
    Ok(())
}