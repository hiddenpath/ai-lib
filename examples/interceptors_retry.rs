#[cfg(feature = "interceptors")]
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};
#[cfg(feature = "interceptors")]
use ai_lib::types::common::Content;

#[cfg(feature = "interceptors")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Interceptor pipeline example: Retry policy");

    // Simple logger interceptor
    struct Logger;
    #[async_trait::async_trait]
    impl ai_lib::interceptors::Interceptor for Logger {
        async fn on_request(&self, ctx: &ai_lib::interceptors::RequestContext, req: &ChatCompletionRequest) {
            println!("on_request provider={} model={} msgs={}", ctx.provider, ctx.model, req.messages.len());
        }
        async fn on_response(&self, ctx: &ai_lib::interceptors::RequestContext, _req: &ChatCompletionRequest, _resp: &ai_lib::ChatCompletionResponse) {
            println!("on_response provider={} model={}", ctx.provider, ctx.model);
        }
        async fn on_error(&self, ctx: &ai_lib::interceptors::RequestContext, _req: &ChatCompletionRequest, err: &ai_lib::AiLibError) {
            eprintln!("on_error provider={} model={} err={:?}", ctx.provider, ctx.model, err);
        }
    }

    // Provider selection by env
    let provider = std::env::var("PROVIDER").unwrap_or_else(|_| "groq".to_string());
    let (prov, default_model) = match provider.as_str() {
        "deepseek" => (Provider::DeepSeek, "deepseek-chat"),
        _ => (Provider::Groq, "llama-3.3-70b-versatile"),
    };
    let client = AiClient::new(prov)?;

    // Compose pipeline
    let pipeline = ai_lib::interceptors::InterceptorPipeline::new().with(Logger);
    let ctx = ai_lib::interceptors::RequestContext { provider: format!("{:?}", client.current_provider()), model: default_model.to_string() };

    let req = ChatCompletionRequest::new(
        default_model.to_string(),
        vec![Message { role: Role::User, content: Content::new_text("Say hello"), function_call: None }]
    );

    // Retry policy with exponential backoff (max 3 attempts)
    let mut last_err: Option<ai_lib::AiLibError> = None;
    for attempt in 1..=3 {
        let res = pipeline.execute(&ctx, &req, || async {
            client.chat_completion(req.clone()).await
        }).await;

        match res {
            Ok(resp) => {
                println!("ok on attempt {}: model={}", attempt, resp.model);
                return Ok(())
            }
            Err(err) => {
                last_err = Some(err.clone());
                // Naive retry conditions (network/provider/ratelimit)
                let retryable = matches!(
                    err,
                    ai_lib::AiLibError::NetworkError(_) |
                    ai_lib::AiLibError::ProviderError(_) |
                    ai_lib::AiLibError::RateLimitExceeded(_)
                );
                if attempt == 3 || !retryable {
                    eprintln!("give up after {} attempts: {:?}", attempt, err);
                    break;
                }
                let backoff_ms = 200u64 * (1 << (attempt - 1));
                println!("retrying in {} ms...", backoff_ms);
                tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
            }
        }
    }

    let err = last_err.unwrap_or(ai_lib::AiLibError::ProviderError("retry failed".to_string()));
    Err(Box::<dyn std::error::Error>::from(err))
}

#[cfg(not(feature = "interceptors"))]
fn main() {
    eprintln!("Enable feature: --features interceptors");
}


