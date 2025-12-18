/// AI-lib basic usage example
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI-lib Basic Usage Example");
    println!("================================");

    // Switch model provider by changing Provider value
    let client = AiClient::new(Provider::Groq)?;
    println!(
        "✅ Created client with provider: {:?}",
        client.provider_name()
    );

    // Get list of supported models
    let models = client.list_models().await?;
    println!("📋 Available models: {:?}", models);

    // Create chat request using convenient Message::user() constructor
    let request = ChatCompletionRequest::new(
        "llama-3.1-8b-instant".to_string(),
        vec![Message::user("Hello! Please introduce yourself briefly.")],
    )
    .with_temperature(0.7)
    .with_max_tokens(100);

    println!("📤 Sending request to model: {}", request.model);

    // Send request
    let response = client.chat_completion(request).await?;

    println!("📥 Received response:");
    println!("   ID: {}", response.id);
    println!("   Model: {}", response.model);
    println!(
        "   Content: {}",
        response.choices[0].message.content.as_text()
    );
    println!("   Usage: {} tokens", response.usage.total_tokens);

    Ok(())
}
