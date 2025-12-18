//! Bring-your-own provider example.
//!
//! This example shows how to inject an OpenAI-compatible backend at runtime
//! without touching the `Provider` enum. Configure the required API key via
//! `LABS_GATEWAY_TOKEN` (or whatever you set with `with_api_key_env`).

use ai_lib::{
    client::{AiClientBuilder, Provider},
    provider::builders::CustomProviderBuilder,
    types::{AiLibError, ChatCompletionRequest, Message, Role},
};

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    // Compose a provider for an OpenAI-compatible gateway.
    let labs_gateway = CustomProviderBuilder::new("labs-gateway")
        .with_base_url("https://labs.example.com/v1")
        .with_api_key_env("LABS_GATEWAY_TOKEN")
        .with_default_chat_model("labs-gpt-35")
        .with_headers([("x-labs-region", "iad")])
        .build_provider()?;

    // Inject the custom provider into the client builder.
    let client = AiClientBuilder::new(Provider::OpenAI)
        .with_strategy(labs_gateway)
        .build()?;

    let request = ChatCompletionRequest::new(
        "labs-gpt-35".to_string(),
        vec![Message {
            role: Role::User,
            content: ai_lib::types::common::Content::Text(
                "Summarize ai-lib's routing features in one sentence.".to_string(),
            ),
            function_call: None,
        }],
    );

    let response = client.chat_completion(request).await?;
    println!(
        "[labs-gateway] {}",
        response.first_text().unwrap_or("no content")
    );

    Ok(())
}
