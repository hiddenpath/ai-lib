//! Concurrency Best Practices with ai-lib (OSS)
//!
//! - Use one long-lived client per provider (reuses HTTP pool)
//! - Set a global max concurrency gate for overload protection
//! - Prefer bounded concurrency in batch APIs
//! - Tune pool via env: AI_HTTP_POOL_MAX_IDLE_PER_HOST, AI_HTTP_POOL_IDLE_TIMEOUT_MS

use ai_lib::{AiClientBuilder, ChatCompletionRequest, Message, Provider, Role};
use ai_lib::types::common::Content;
use futures::stream::{FuturesUnordered, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Global knobs (optional): set env externally
    // std::env::set_var("AI_HTTP_POOL_MAX_IDLE_PER_HOST", "64");
    // std::env::set_var("AI_HTTP_POOL_IDLE_TIMEOUT_MS", "120000");

    // 1) One long-lived client per provider
    let client = AiClientBuilder::new(Provider::Groq)
        .with_max_concurrency(64)
        .for_production()
        .build()?;

    // 2) Prepare a batch of requests
    let mut requests = Vec::new();
    for i in 0..50 {
        requests.push(ChatCompletionRequest::new(
            client.default_chat_model(),
            vec![Message {
                role: Role::User,
                content: Content::new_text(format!("Hello #{i}")),
                function_call: None,
            }],
        ));
    }

    // 3) Use bounded concurrency (e.g., 16)
    let results = client
        .chat_completion_batch(requests.clone(), Some(16))
        .await?;
    println!("bounded results: {}", results.len());

    // 4) Or build your own bounded stream (FuturesUnordered)
    let mut futs = FuturesUnordered::new();
    for req in requests.into_iter() {
        let c = &client;
        futs.push(async move { c.chat_completion(req).await });
        if futs.len() >= 16 {
            let _ = futs.next().await;
        }
    }
    while let Some(_r) = futs.next().await {}

    println!("done");
    Ok(())
}


