use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example: upload a local image and reference it in a chat request
    let client = AiClient::new(Provider::OpenAI)?;
    // Provide a real image path
    let remote = client.upload_file("./examples/assets/sample.jpg").await?;

    let msg = Message {
        role: Role::User,
        content: ai_lib::types::common::Content::Image {
            url: Some(remote),
            mime: Some("image/jpeg".into()),
            name: None,
        },
        function_call: None,
    };
    let req = ChatCompletionRequest::new(client.default_chat_model(), vec![msg]);
    let resp = client.chat_completion(req).await?;
    println!("{}", resp.first_text()?);
    Ok(())
}
