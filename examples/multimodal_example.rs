use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Multimodal example: image + audio content in a message");

    let _client = AiClient::new(Provider::Groq)?;

    let request = ChatCompletionRequest::new(
        "multimodal-model".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::new_image(
                Some("https://example.com/dog.jpg".into()),
                Some("image/jpeg".into()),
                Some("dog.jpg".into()),
            ),
            function_call: None,
        }],
    );

    println!(
        "Prepared multimodal request; image URL: {}",
        request.messages[0].content.as_text()
    );

    // Note: this example demonstrates the type usage only and does not call the API.
    Ok(())
}
