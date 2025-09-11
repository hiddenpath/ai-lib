use ai_lib::{AiClientBuilder, Provider, ChatCompletionRequest, Message, Role};
use ai_lib::types::common::Content;

#[cfg(feature = "interceptors")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AiClientBuilder::new(Provider::OpenAI)
        .enable_default_interceptors()
        .build()?;

    let req = ChatCompletionRequest::new(
        "gpt-4o".to_string(),
        vec![Message { role: Role::User, content: Content::Text("Hello".to_string()), function_call: None }],
    );
    let _ = client.chat_completion(req).await;
    Ok(())
}

#[cfg(not(feature = "interceptors"))]
fn main() {
    println!("This example requires --features interceptors. Skipping.");
}


