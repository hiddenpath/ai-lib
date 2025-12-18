use ai_lib::types::common::Content;
use ai_lib::{
    client::AiClientBuilder, types::AiLibError, ChatCompletionRequest, Message, Provider, Role,
};

/// Demonstrates composing routing strategies before runtime by wiring providers
/// into a round-robin chain.
#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    // Requires GROQ_API_KEY and OPENROUTER_API_KEY to be set in the environment.
    let client = AiClientBuilder::new(Provider::Groq)
        .with_round_robin_chain(vec![Provider::Groq, Provider::OpenRouter])?
        .build()?;

    let req = ChatCompletionRequest::new(
        client.default_chat_model(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("Say hi and mention which provider you are."),
            function_call: None,
        }],
    );

    let resp = client.chat_completion(req).await?;
    println!("selected provider: {}", client.provider_name());
    println!("model: {}", resp.model);
    Ok(())
}
