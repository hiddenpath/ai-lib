#[cfg(feature = "interceptors")]
use ai_lib::prelude::*;

#[cfg(feature = "interceptors")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI-lib v0.5.0 Interceptor Example: Retry Policy");
    println!("==================================================");

    // Simple logger interceptor
    struct Logger;
    #[async_trait::async_trait]
    impl ai_lib::interceptors::Interceptor for Logger {
        async fn on_request(
            &self,
            ctx: &ai_lib::interceptors::RequestContext,
            req: &ChatCompletionRequest,
        ) {
            println!(
                "📡 [Request] provider={} model={} messages={}",
                ctx.provider,
                ctx.model,
                req.messages.len()
            );
        }
        async fn on_response(
            &self,
            ctx: &ai_lib::interceptors::RequestContext,
            _req: &ChatCompletionRequest,
            _resp: &ai_lib::ChatCompletionResponse,
        ) {
            println!(
                "✅ [Response] provider={} model={}",
                ctx.provider, ctx.model
            );
        }
        async fn on_error(
            &self,
            ctx: &ai_lib::interceptors::RequestContext,
            _req: &ChatCompletionRequest,
            err: &ai_lib::AiLibError,
        ) {
            eprintln!(
                "❌ [Error] provider={} model={} err={:?}",
                ctx.provider, ctx.model, err
            );
        }
    }

    // v0.5.0 Pattern: Build client via builder
    let client = AiClientBuilder::new(Provider::OpenAI)
        .with_model("gpt-4o")
        .build()?;

    // Compose interceptor pipeline
    let pipeline = ai_lib::interceptors::InterceptorPipeline::new().with(Logger);
    let ctx = ai_lib::interceptors::RequestContext {
        provider: client.provider_name().to_string(),
        model: client.default_chat_model().to_string(),
    };

    let req = ChatCompletionRequest::new(
        client.default_chat_model().to_string(),
        vec![Message::user("Please say hello in a creative way.")],
    );

    // Retry policy with exponential backoff (max 3 attempts)
    println!("🔄 Starting execution with retry policy...");
    let mut last_err: Option<ai_lib::AiLibError> = None;

    for attempt in 1..=3 {
        let res = pipeline
            .execute(&ctx, &req, || async {
                client.chat_completion(req.clone()).await
            })
            .await;

        match res {
            Ok(resp) => {
                println!(
                    "\n✨ Success on attempt {}: {}",
                    attempt,
                    resp.choices[0].message.content.as_text()
                );
                return Ok(());
            }
            Err(err) => {
                last_err = Some(err.clone());
                let retryable = matches!(
                    err,
                    ai_lib::AiLibError::NetworkError(_)
                        | ai_lib::AiLibError::ProviderError(_)
                        | ai_lib::AiLibError::RateLimitExceeded(_)
                );

                if attempt == 3 || !retryable {
                    eprintln!(
                        "\n🚫 Giving up after {} attempts. Error: {:?}",
                        attempt, err
                    );
                    break;
                }

                let backoff_ms = 200u64 * (1 << (attempt - 1));
                println!(
                    "⚠️  Attempt {} failed, retrying in {} ms...",
                    attempt, backoff_ms
                );
                tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
            }
        }
    }

    let err = last_err.unwrap_or(ai_lib::AiLibError::ProviderError(
        "retry failed".to_string(),
    ));
    Err(Box::<dyn std::error::Error>::from(err))
}

#[cfg(not(feature = "interceptors"))]
fn main() {
    eprintln!("Enable feature: --features interceptors");
}
