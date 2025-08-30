use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure COHERE_API_KEY env var is set if making real requests
    let client = AiClient::new(Provider::Cohere)?;

    let request = ChatCompletionRequest::new(
        "command-xlarge-nightly".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Write a haiku about rust programming".to_string()),
            function_call: None,
        }],
    )
    .with_temperature(0.7)
    .with_max_tokens(60);

    // List models
    match client.list_models().await {
        Ok(models) => println!("Models: {:?}", models),
        Err(e) => eprintln!("Failed to list models: {}", e),
    }

    // Streaming
    let mut stream = client.chat_completion_stream(request).await?;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                for choice in c.choices {
                    if let Some(delta) = choice.delta.content {
                        print!("{}", delta);
                    }
                }
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
