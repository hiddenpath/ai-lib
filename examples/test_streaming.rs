use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 流式响应测试");
    println!("================");
    
    // 检查Groq API密钥
    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ 未设置GROQ_API_KEY");
        return Ok(());
    }
    
    // 创建Groq客户端
    let client = AiClient::new(Provider::Groq)?;
    println!("✅ Groq客户端创建成功");
    
    // 创建流式请求
    let request = ChatCompletionRequest::new(
        "llama3-8b-8192".to_string(),
        vec![Message {
            role: Role::User,
            content: "Please write a short poem about AI in exactly 4 lines.".to_string(),
        }],
    ).with_max_tokens(100)
     .with_temperature(0.7);
    
    println!("\n📤 发送流式请求...");
    println!("   模型: {}", request.model);
    println!("   消息: {}", request.messages[0].content);
    
    // 获取流式响应
    match client.chat_completion_stream(request).await {
        Ok(mut stream) => {
            println!("\n🌊 开始接收流式响应:");
            println!("{}", "─".repeat(50));
            
            let mut full_content = String::new();
            let mut chunk_count = 0;
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        chunk_count += 1;
                        
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                print!("{}", content);
                                full_content.push_str(content);
                                
                                // 刷新输出
                                use std::io::{self, Write};
                                io::stdout().flush().unwrap();
                            }
                            
                            // 检查是否完成
                            if choice.finish_reason.is_some() {
                                println!("\n{}", "─".repeat(50));
                                println!("✅ 流式响应完成!");
                                println!("   完成原因: {:?}", choice.finish_reason);
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
            
            println!("\n📊 流式响应统计:");
            println!("   数据块数量: {}", chunk_count);
            println!("   总内容长度: {} 字符", full_content.len());
            println!("   完整内容: \"{}\"", full_content.trim());
        }
        Err(e) => {
            println!("❌ 流式请求失败: {}", e);
        }
    }
    
    println!("\n💡 流式响应的优势:");
    println!("   • 实时显示生成内容");
    println!("   • 更好的用户体验");
    println!("   • 可以提前停止生成");
    println!("   • 适合长文本生成");
    
    Ok(())
}