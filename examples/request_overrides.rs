use ai_lib::{
    AiClient, ChatCompletionRequest, ConnectionOptions, Content, Message, Provider, Role,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example: per-client/request-level overrides without changing global env
    let opts = ConnectionOptions {
        base_url: None, // use provider default
        proxy: Some("http://localhost:8080".to_string()),
        api_key: None, // or Some("override_key".into())
        timeout: Some(std::time::Duration::from_secs(45)),
        disable_proxy: false,
    };

    let client = AiClient::with_options(Provider::Groq, opts)?;
    let req = ChatCompletionRequest::new(
        client.default_chat_model(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("Say hi."),
            function_call: None,
        }],
    );
    let resp = client.chat_completion(req).await?;
    println!("{}", resp.first_text()?);
    Ok(())
}
