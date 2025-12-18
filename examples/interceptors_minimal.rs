use ai_lib::types::common::Content;
use ai_lib::{AiClientBuilder, ChatCompletionRequest, Message, Provider, Role};

#[cfg(feature = "interceptors")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AiClientBuilder::new(Provider::OpenAI)
        .enable_minimal_interceptors()
        .build()?;

    let req = ChatCompletionRequest::new(
        "gpt-4o".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Ping".to_string()),
            function_call: None,
        }],
    );
    let _ = client.chat_completion(req).await;
    Ok(())
}

#[cfg(not(feature = "interceptors"))]
fn main() {
    println!("This example requires --features interceptors. Skipping.");
}
