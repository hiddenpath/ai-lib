use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI-lib Batch Processing Example");
    println!("==================================");

    // 创建客户端
    let client = AiClient::new(Provider::Groq)?;
    println!(
        "✅ Created client with provider: {:?}",
        client.current_provider()
    );

    // 准备多个请求
    let requests = vec![
        ChatCompletionRequest::new(
            "llama3-8b-8192".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text("What is the capital of France?".to_string()),
                function_call: None,
            }],
        )
        .with_temperature(0.7)
        .with_max_tokens(50),

        ChatCompletionRequest::new(
            "llama3-8b-8192".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text("What is 2 + 2?".to_string()),
                function_call: None,
            }],
        )
        .with_temperature(0.1)
        .with_max_tokens(20),

        ChatCompletionRequest::new(
            "llama3-8b-8192".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text("Tell me a short joke.".to_string()),
                function_call: None,
            }],
        )
        .with_temperature(0.9)
        .with_max_tokens(100),

        ChatCompletionRequest::new(
            "llama3-8b-8192".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text("What is the largest planet in our solar system?".to_string()),
                function_call: None,
            }],
        )
        .with_temperature(0.5)
        .with_max_tokens(60),
    ];

    println!("📤 Prepared {} requests for batch processing", requests.len());

    // 方法1: 使用并发限制的批处理
    println!("\n🔄 Method 1: Batch processing with concurrency limit (2)");
    let start_time = std::time::Instant::now();
    
    let responses = client.chat_completion_batch(requests.clone(), Some(2)).await?;
    
    let duration = start_time.elapsed();
    println!("⏱️  Batch processing completed in {:?}", duration);

    // 处理响应
    for (i, response) in responses.iter().enumerate() {
        match response {
            Ok(resp) => {
                println!(
                    "✅ Request {}: {}",
                    i + 1,
                    resp.choices[0].message.content.as_text()
                );
            }
            Err(e) => {
                println!("❌ Request {} failed: {}", i + 1, e);
            }
        }
    }

    // 方法2: 使用智能批处理（自动选择策略）
    println!("\n🧠 Method 2: Smart batch processing");
    let start_time = std::time::Instant::now();
    
    let responses = client.chat_completion_batch_smart(requests.clone()).await?;
    
    let duration = start_time.elapsed();
    println!("⏱️  Smart batch processing completed in {:?}", duration);

    // 统计成功和失败
    let successful: Vec<_> = responses.iter().filter_map(|r| r.as_ref().ok()).collect();
    let failed: Vec<_> = responses.iter().enumerate().filter_map(|(i, r)| {
        r.as_ref().err().map(|e| (i, e))
    }).collect();

    println!("📊 Results:");
    println!("   ✅ Successful: {}/{}", successful.len(), responses.len());
    println!("   ❌ Failed: {}/{}", failed.len(), responses.len());
    println!("   📈 Success rate: {:.1}%", (successful.len() as f64 / responses.len() as f64) * 100.0);

    // 方法3: 无限制并发批处理
    println!("\n🚀 Method 3: Unlimited concurrent batch processing");
    let start_time = std::time::Instant::now();
    
    let responses = client.chat_completion_batch(requests, None).await?;
    
    let duration = start_time.elapsed();
    println!("⏱️  Unlimited concurrent processing completed in {:?}", duration);

    // 显示所有响应
    for (i, response) in responses.iter().enumerate() {
        match response {
            Ok(resp) => {
                println!(
                    "✅ Request {}: {}",
                    i + 1,
                    resp.choices[0].message.content.as_text()
                );
            }
            Err(e) => {
                println!("❌ Request {} failed: {}", i + 1, e);
            }
        }
    }

    println!("\n🎉 Batch processing example completed successfully!");
    Ok(())
}
