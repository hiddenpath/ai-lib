#[cfg(all(feature = "interceptors"))]
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};
#[cfg(all(feature = "interceptors"))]
use ai_lib::types::common::Content;

#[cfg(all(feature = "interceptors"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Interceptor pipeline example: CircuitBreaker policy");

    // Logger
    struct Logger;
    #[async_trait::async_trait]
    impl ai_lib::interceptors::Interceptor for Logger {
        async fn on_request(&self, ctx: &ai_lib::interceptors::RequestContext, _req: &ChatCompletionRequest) {
            println!("on_request provider={} model={}", ctx.provider, ctx.model);
        }
        async fn on_error(&self, ctx: &ai_lib::interceptors::RequestContext, _req: &ChatCompletionRequest, err: &ai_lib::AiLibError) {
            eprintln!("on_error provider={} model={} err={:?}", ctx.provider, ctx.model, err);
        }
    }

    // Build breaker
    use ai_lib::circuit_breaker::config::CircuitBreakerConfig;
    use ai_lib::circuit_breaker::breaker::CircuitBreaker;
    let breaker = CircuitBreaker::new(CircuitBreakerConfig::development());

    let client = AiClient::new(Provider::Groq)?;
    let model = "llama-3.3-70b-versatile";
    let pipeline = ai_lib::interceptors::InterceptorPipeline::new().with(Logger);
    let ctx = ai_lib::interceptors::RequestContext { provider: format!("{:?}", client.current_provider()), model: model.to_string() };
    let req = ChatCompletionRequest::new(
        model.to_string(),
        vec![Message { role: Role::User, content: Content::new_text("Breaker demo"), function_call: None }]
    );

    // Protect the call using circuit breaker
    let result = breaker.call(async {
        pipeline.execute(&ctx, &req, || async { client.chat_completion(req.clone()).await }).await
    }).await;

    match result {
        Ok(resp) => { println!("ok: {}", resp.model); Ok(()) }
        Err(e) => { eprintln!("circuit error: {:?}", e); Ok(()) }
    }
}

#[cfg(not(all(feature = "interceptors")))]
fn main() {
    eprintln!("Enable feature: --features interceptors");
}


