use ai_lib::{AiClient, ChatCompletionRequest, Content, Message, Provider, Role};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optional: MISTRAL_API_KEY for live API calls
    // std::env::var("MISTRAL_API_KEY").expect("MISTRAL_API_KEY not set");

    let client = AiClient::new(Provider::Mistral)?;
    let req = ChatCompletionRequest::new(
        client.default_chat_model(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("Stream one short tip about Rust lifetimes."),
            function_call: None,
        }],
    );

    let mut stream = client.chat_completion_stream(req).await?;
    while let Some(chunk) = stream.next().await {
        let c = chunk?;
        if let Some(delta) = c.choices.get(0).and_then(|d| d.delta.content.clone()) {
            print!("{}", delta);
        }
    }
    println!();
    Ok(())
}
