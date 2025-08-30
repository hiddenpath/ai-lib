use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 调试请求格式");
    println!("===============");

    // 创建测试请求
    let request = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello!".to_string()),
            function_call: None,
        }],
    )
    .with_max_tokens(10);

    println!("📤 原始请求:");
    println!("   模型: {}", request.model);
    println!("   消息数量: {}", request.messages.len());
    println!(
        "   消息[0]: {:?} - {}",
        request.messages[0].role,
        request.messages[0].content.as_text()
    );
    println!("   max_tokens: {:?}", request.max_tokens);

    // 测试OpenAI
    println!("\n🤖 测试OpenAI...");
    match AiClient::new(Provider::OpenAI) {
        Ok(client) => {
            match client.chat_completion(request.clone()).await {
                Ok(response) => {
                    println!("✅ 成功!");
                    println!("   响应: {}", response.choices[0].message.content.as_text());
                }
                Err(e) => {
                    println!("❌ 失败: {}", e);

                    // 如果是400错误，说明请求格式有问题
                    if e.to_string().contains("400") {
                        println!("   这通常表示请求格式不正确");
                        println!("   让我们检查请求是否包含必要字段...");
                    }
                }
            }
        }
        Err(e) => println!("❌ 客户端创建失败: {}", e),
    }

    Ok(())
}
