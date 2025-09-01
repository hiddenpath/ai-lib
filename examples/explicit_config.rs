//! Minimal explicit configuration example using ConnectionOptions.
use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, ConnectionOptions, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Explicit configuration example");
    let opts = ConnectionOptions {
        base_url: None,                                      // fallback to provider default
        proxy: Some("http://proxy.example.com:8080".into()), // or None to use AI_PROXY_URL
        api_key: None,                                       // rely on environment for now
        timeout: Some(std::time::Duration::from_secs(40)),
        disable_proxy: false,
    };

    let client = AiClient::with_options(Provider::Groq, opts)?;

    let req = ChatCompletionRequest::new(
        "llama3-8b-8192".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Ping from explicit config".into()),
            function_call: None,
        }],
    );

    // This may fail if GROQ_API_KEY not set; we only show structure.
    match client.chat_completion(req).await {
        Ok(resp) => println!("Response model: {}", resp.model),
        Err(e) => println!(
            "Request failed (expected in example without API key): {}",
            e
        ),
    }
    Ok(())
}
