#[cfg(feature = "interceptors")]
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};
#[cfg(feature = "interceptors")]
use ai_lib::types::common::Content;

#[cfg(feature = "interceptors")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Interceptor pipeline example: Timeout policy");

    struct Logger;
    #[async_trait::async_trait]
    impl ai_lib::interceptors::Interceptor for Logger {
        async fn on_request(&self, ctx: &ai_lib::interceptors::RequestContext, req: &ChatCompletionRequest) {
            println!("on_request provider={} model={} msgs={}", ctx.provider, ctx.model, req.messages.len());
        }
        async fn on_error(&self, ctx: &ai_lib::interceptors::RequestContext, _req: &ChatCompletionRequest, err: &ai_lib::AiLibError) {
            eprintln!("timeout? provider={} model={} err={:?}", ctx.provider, ctx.model, err);
        }
    }

    let client = AiClient::new(Provider::Groq)?;
    let model = "llama-3.3-70b-versatile";

    let pipeline = ai_lib::interceptors::InterceptorPipeline::new().with(Logger);
    let ctx = ai_lib::interceptors::RequestContext { provider: format!("{:?}", client.current_provider()), model: model.to_string() };

    // Wrap an artificial per-call timeout
    let req = ChatCompletionRequest::new(
        model.to_string(),
        vec![Message { role: Role::User, content: Content::new_text("Please respond within 1s"), function_call: None }]
    );

    let fut = pipeline.execute(&ctx, &req, || async {
        client.chat_completion(req.clone()).await
    });

    let res = tokio::time::timeout(std::time::Duration::from_secs(1), fut).await;
    let out: Result<(), ai_lib::AiLibError> = match res {
        Ok(Ok(_resp)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Ok(()),
    };
    out.map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })
}

#[cfg(not(feature = "interceptors"))]
fn main() {
    eprintln!("Enable feature: --features interceptors");
}


