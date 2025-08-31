/// 调试请求格式示例 - Debug request format example
use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debug Request Format");
    println!("======================");

    // Create test request
    let request = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello!".to_string()),
            function_call: None,
        }],
    )
    .with_max_tokens(10);

    println!("📤 Original Request:");
    println!("   Model: {}", request.model);
    println!("   Message count: {}", request.messages.len());
    println!(
        "   Message[0]: {:?} - {}",
        request.messages[0].role,
        request.messages[0].content.as_text()
    );
    println!("   max_tokens: {:?}", request.max_tokens);

    // Test OpenAI
    println!("\n🤖 Testing OpenAI...");
    match AiClient::new(Provider::OpenAI) {
        Ok(client) => {
            match client.chat_completion(request.clone()).await {
                Ok(response) => {
                    println!("✅ Success!");
                    println!(
                        "   Response: {}",
                        response.choices[0].message.content.as_text()
                    );
                }
                Err(e) => {
                    println!("❌ Failed: {}", e);

                    // If it's a 400 error, it indicates request format issues
                    if e.to_string().contains("400") {
                        println!("   This usually indicates incorrect request format");
                        println!("   Let's check if the request contains necessary fields...");
                    }
                }
            }
        }
        Err(e) => println!("❌ Client creation failed: {}", e),
    }

    Ok(())
}
