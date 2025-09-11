//! Model override feature demonstration
//! Model Override Feature Demo

use ai_lib::{AiClient, Content, Provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check environment variables
    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ Please set GROQ_API_KEY environment variable");
        println!("   Example: export GROQ_API_KEY=your_api_key_here");
        return Ok(());
    }

    println!("🚀 Model Override Feature Demo");
    println!("==============================");
    println!();

    // 1. Basic usage - maintain original simplicity
    println!("📋 1. Basic Usage - Using Default Model");
    let reply = AiClient::quick_chat_text(Provider::Groq, "Hello!").await?;
    println!("   ✅ Response: {}", reply);
    println!();

    // 2. Explicitly specify model
    println!("📋 2. Explicitly Specify Model");
    let reply =
        AiClient::quick_chat_text_with_model(Provider::Groq, "Hello!", "llama-3.1-8b-instant")
            .await?;
    println!("   ✅ Response: {}", reply);
    println!();

    // 3. Using ModelOptions
    println!("📋 3. Using ModelOptions");
    let client = AiClient::new(Provider::Groq)?;
    let mut request = client.build_simple_request("Hello!");
    request.model = "llama-3.1-70b-versatile".to_string();

    let response = client.chat_completion(request).await?;

    let reply = response.choices[0].message.content.as_text();
    println!("   ✅ Response: {}", reply);
    println!();

    // 4. AiClientBuilder custom default model
    println!("📋 4. AiClientBuilder Custom Default Model");
    let client = AiClient::builder(Provider::Groq)
        .with_default_chat_model("llama-3.1-8b-instant")
        .build()?;

    let request = client.build_simple_request("Hello!");
    println!("   Using model: {}", request.model);

    let response = client.chat_completion(request).await?;
    match &response.choices[0].message.content {
        Content::Text(text) => {
            println!("   ✅ Response: {}", text);
        }
        _ => println!("   ✅ Response: {:?}", response.choices[0].message.content),
    }
    println!();

    // 5. Explicitly specify model in build_simple_request
    println!("📋 5. Explicitly Specify Model in build_simple_request");
    let client = AiClient::new(Provider::Groq)?;
    let request = client.build_simple_request_with_model("Hello!", "llama-3.1-70b-versatile");

    println!("   Using model: {}", request.model);

    let response = client.chat_completion(request).await?;
    match &response.choices[0].message.content {
        Content::Text(text) => {
            println!("   ✅ Response: {}", text);
        }
        _ => println!("   ✅ Response: {:?}", response.choices[0].message.content),
    }
    println!();

    println!("🎉 Demo completed!");
    println!("==================");
    println!("✅ All model override features are working correctly");
    println!("✅ Backward compatibility is guaranteed");
    println!("✅ Flexible model specification methods are provided");

    Ok(())
}
