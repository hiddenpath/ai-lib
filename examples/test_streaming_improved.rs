use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 改进的流式响应测试");
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
            content: Content::Text(
                "Write a creative story about a robot learning to paint. Keep it under 100 words."
                    .to_string(),
            ),
            function_call: None,
        }],
    )
    .with_max_tokens(150)
    .with_temperature(0.8);

    println!("\n📤 发送流式请求...");

    match client.chat_completion_stream(request).await {
        Ok(mut stream) => {
            println!("🎨 AI创作中:");
            print!("   ");

            let mut content = String::new();
            let mut chunk_count = 0;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        chunk_count += 1;

                        if let Some(choice) = chunk.choices.first() {
                            if let Some(text) = &choice.delta.content {
                                if !text.is_empty() {
                                    print!("{}", text);
                                    content.push_str(text);

                                    use std::io::{self, Write};
                                    io::stdout().flush().unwrap();
                                }
                            }

                            if choice.finish_reason.is_some() {
                                println!("\n");
                                println!("✅ 创作完成! (原因: {:?})", choice.finish_reason);
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

            println!("\n📊 统计信息:");
            println!("   数据块: {}", chunk_count);
            println!("   字符数: {}", content.len());
            println!("   单词数: {}", content.split_whitespace().count());
        }
        Err(e) => {
            println!("❌ 流式请求失败: {}", e);
        }
    }

    // 测试DeepSeek流式响应
    if std::env::var("DEEPSEEK_API_KEY").is_ok() {
        println!("\n{}", "=".repeat(50));
        println!("🧠 测试DeepSeek流式响应");

        let deepseek_client = AiClient::new(Provider::DeepSeek)?;
        let request = ChatCompletionRequest::new(
            "deepseek-chat".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text("Explain quantum computing in one sentence.".to_string()),
                function_call: None,
            }],
        )
        .with_max_tokens(50);

        match deepseek_client.chat_completion_stream(request).await {
            Ok(mut stream) => {
                println!("🔬 DeepSeek回复:");
                print!("   ");

                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => {
                            if let Some(choice) = chunk.choices.first() {
                                if let Some(text) = &choice.delta.content {
                                    print!("{}", text);
                                    use std::io::{self, Write};
                                    io::stdout().flush().unwrap();
                                }
                                if choice.finish_reason.is_some() {
                                    println!("\n✅ DeepSeek流式响应成功!");
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            println!("\n❌ DeepSeek流式错误: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ DeepSeek流式请求失败: {}", e);
            }
        }
    }

    Ok(())
}
