use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    // Initialize the AI client
    let client = AiClient::new(Provider::OpenAI)?;

    println!("🤖 AI-Lib Multimodal Example");
    println!("=============================");

    // Example 1: Text-only message
    println!("\n1. Text Message:");
    let text_message = Message {
        role: Role::User,
        content: Content::new_text("Hello, can you help me analyze this image?"),
        function_call: None,
    };

    // Example 2: Image content from file path (auto-processed)
    println!("\n2. Image from File Path:");
    let image_content = Content::from_image_file("examples/assets/sample.png");
    let image_message = Message {
        role: Role::User,
        content: image_content,
        function_call: None,
    };

    // Example 3: Image content with explicit URL
    println!("\n3. Image with URL:");
    let url_image_content = Content::new_image(
        Some("https://example.com/sample.jpg".to_string()),
        Some("image/jpeg".to_string()),
        Some("sample.jpg".to_string()),
    );
    let url_image_message = Message {
        role: Role::User,
        content: url_image_content,
        function_call: None,
    };

    // Example 4: Audio content from file path
    println!("\n4. Audio from File Path:");
    let audio_content = Content::from_audio_file("examples/assets/sample.mp3");
    let audio_message = Message {
        role: Role::User,
        content: audio_content,
        function_call: None,
    };

    // Example 5: Mixed content (text + image)
    println!("\n5. Mixed Content (Text + Image):");
    let mixed_messages = vec![
        Message {
            role: Role::User,
            content: Content::new_text("Please analyze this image and tell me what you see."),
            function_call: None,
        },
        Message {
            role: Role::User,
            content: Content::from_image_file("examples/assets/sample.png"),
            function_call: None,
        },
    ];

    // Example 6: Data URL content
    println!("\n6. Data URL Content:");
    let data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";
    let data_url_content = Content::from_data_url(
        data_url.to_string(),
        Some("image/png".to_string()),
        Some("tiny.png".to_string()),
    );
    let data_url_message = Message {
        role: Role::User,
        content: data_url_content,
        function_call: None,
    };

    println!("\n📝 Content Types Created:");
    println!("   • Text: {:?}", text_message.content);
    println!("   • Image from file: {:?}", image_message.content);
    println!("   • Image from URL: {:?}", url_image_message.content);
    println!("   • Audio from file: {:?}", audio_message.content);
    println!("   • Data URL: {:?}", data_url_message.content);

    println!("\n💡 Usage Notes:");
    println!("   • Content::from_image_file() and Content::from_audio_file() automatically detect MIME types");
    println!("   • The AI client handles file processing (upload or inline) based on provider capabilities");
    println!("   • File size limits are respected - large files are uploaded, small files are inlined as data URLs");
    println!("   • Mixed content messages are supported for complex multimodal interactions");

    // Note: In a real application, you would send these messages to the AI client:
    // let response = client.chat_completion(ChatCompletionRequest::new(
    //     client.default_chat_model(),
    //     mixed_messages,
    // )).await?;

    Ok(())
}
