use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    println!("🤖 AI-Lib v0.5.0 Multimodal Example");
    println!("====================================");

    // v0.5.0 Pattern: Initialize with a multimodal-capable model
    let client = AiClientBuilder::new(Provider::OpenAI)
        .with_model("gpt-4o")
        .build()?;

    // Example 1: Simple text message
    println!("\n1. Text Message:");
    let text_message = Message::user("Hello, can you help me analyze this image?");

    // Example 2: Multimodal content (Text + Image URL)
    println!("\n2. Multimodal Content (Text + Image URL):");
    let multimodal_messages = vec![
        Message::user("Describe the following image in detail:"),
        Message::user_with_content(Content::Image {
            url: Some("https://example.com/sample.jpg".to_string()),
            mime: Some("image/jpeg".to_string()),
            name: None,
        }),
    ];

    // Example 3: Data URL (Inlined Image)
    println!("\n3. Inlined Image (Data URL):");
    let data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";
    let data_url_message = Message::user_with_content(Content::from_data_url(
        data_url.to_string(),
        Some("image/png".to_string()),
        Some("tiny.png".to_string()),
    ));

    println!("\n📝 Multimodal Message Structures Created:");
    println!("   • Simple User Message: {:?}", text_message.role);
    println!(
        "   • Mixed Content Elements: {}",
        match &mixed_message.content {
            Content::Mixed(v) => v.len(),
            _ => 1,
        }
    );
    println!("   • Data URL Message: Created successfully");

    println!("\n💡 Usage Notes for v0.5.0:");
    println!("   • Use Message::user_with_content() for complex multimodal inputs.");
    println!(
        "   • The Manifest defines which models support 'vision' or 'multimodal' capabilities."
    );
    println!("   • The SDK handles the mapping to provider-specific multimodal formats (OpenAI, Gemini, Anthropic).");

    // Note: To send these:
    // let request = ChatCompletionRequest::new("gpt-4o".to_string(), vec![mixed_message]);
    // let response = client.chat_completion(request).await?;

    Ok(())
}
