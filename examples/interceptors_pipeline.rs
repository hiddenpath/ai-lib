#[cfg(feature = "interceptors")]
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};
#[cfg(feature = "interceptors")]
use ai_lib::types::common::Content;

#[cfg(feature = "interceptors")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Interceptor pipeline example (no-op interceptor)");

    // No-op interceptor just prints request/response lifecycle
    struct Logger;
    #[async_trait::async_trait]
    impl ai_lib::interceptors::Interceptor for Logger {
        async fn on_request(&self, ctx: &ai_lib::interceptors::RequestContext, req: &ChatCompletionRequest) {
            println!("on_request provider={} model={} msgs={} ", ctx.provider, ctx.model, req.messages.len());
        }
        async fn on_response(&self, ctx: &ai_lib::interceptors::RequestContext, _req: &ChatCompletionRequest, _resp: &ai_lib::ChatCompletionResponse) {
            println!("on_response provider={} model={}", ctx.provider, ctx.model);
        }
        async fn on_error(&self, ctx: &ai_lib::interceptors::RequestContext, _req: &ChatCompletionRequest, err: &ai_lib::AiLibError) {
            eprintln!("on_error provider={} model={} err={:?}", ctx.provider, ctx.model, err);
        }
    }

    // Build client from env selection: PROVIDER={groq|deepseek}, default groq
    let provider = std::env::var("PROVIDER").unwrap_or_else(|_| "groq".to_string());
    let (prov, default_model) = match provider.as_str() {
        "deepseek" => (Provider::DeepSeek, "deepseek-chat"),
        _ => (Provider::Groq, "llama-3.3-70b-versatile"),
    };
    let client = AiClient::new(prov)?;

    // Compose pipeline (not wired to AiClient yet; just show how to execute around a call)
    let pipeline = ai_lib::interceptors::InterceptorPipeline::new().with(Logger);
    let ctx = ai_lib::interceptors::RequestContext { provider: format!("{:?}", client.current_provider()), model: default_model.to_string() };

    let req = ChatCompletionRequest::new(
        default_model.to_string(),
        vec![Message { role: Role::User, content: Content::new_text("Hello"), function_call: None }]
    );

    // Wrap the actual call using the pipeline (for demo)
    let resp = pipeline.execute(&ctx, &req, || async {
        client.chat_completion(req.clone()).await
    }).await?;

    println!("resp.model={}", resp.model);
    Ok(())
}


#[cfg(not(feature = "interceptors"))]
fn main() {
    eprintln!("This example requires the 'interceptors' feature. Try: cargo run --features interceptors --example interceptors_pipeline");
}


