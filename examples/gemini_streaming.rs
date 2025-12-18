use ai_lib::{AiClient, ChatCompletionRequest, Content, Message, Provider, Role};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure API key is present
    std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");

    let client = AiClient::new(Provider::Gemini)?;
    let req = ChatCompletionRequest::new(
        client.default_chat_model(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("Stream a short haiku about Rust."),
            function_call: None,
        }],
    );

    let mut stream = client.chat_completion_stream(req).await?;
    while let Some(chunk) = stream.next().await {
        let c = chunk?;
        if let Some(delta) = c.choices.first().and_then(|d| d.delta.content.clone()) {
            print!("{}", delta);
        }
    }
    println!();
    Ok(())
}
