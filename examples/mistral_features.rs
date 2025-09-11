use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[cfg(all(feature = "interceptors", feature = "unified_sse"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mistral feature demo: interceptors + unified_sse");

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
                "on_request provider={} model={} msgs={}",
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
            println!("on_response provider={} model={}", ctx.provider, ctx.model);
        }
        async fn on_error(
            &self,
            ctx: &ai_lib::interceptors::RequestContext,
            _req: &ChatCompletionRequest,
            err: &ai_lib::AiLibError,
        ) {
            eprintln!(
                "on_error provider={} model={} err={:?}",
                ctx.provider, ctx.model, err
            );
        }
    }

    // Independent provider (MistralAdapter)
    let client = AiClient::new(Provider::Mistral)?;
    let model = client.default_chat_model();

    let req = ChatCompletionRequest::new(
        model.clone(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("Explain what is Rust borrow checker in one line."),
            function_call: None,
        }],
    );

    let pipeline = ai_lib::interceptors::InterceptorPipeline::new().with(Logger);
    let ctx = ai_lib::interceptors::RequestContext {
        provider: format!("{:?}", client.current_provider()),
        model: model.clone(),
    };
    let _resp = pipeline
        .execute(&ctx, &req, || async {
            client.chat_completion(req.clone()).await
        })
        .await?;

    // Streaming (unified_sse)
    let mut stream = client
        .chat_completion_stream(ChatCompletionRequest::new(
            model,
            vec![Message {
                role: Role::User,
                content: Content::new_text("Stream 3-5 words."),
                function_call: None,
            }],
        ))
        .await?;
    use futures::StreamExt;
    while let Some(chunk) = stream.next().await {
        let c = chunk?;
        if let Some(delta) = c.choices.get(0).and_then(|d| d.delta.content.clone()) {
            print!("{}", delta);
        }
    }
    println!();
    Ok(())
}

#[cfg(not(all(feature = "interceptors", feature = "unified_sse")))]
fn main() {
    eprintln!("Enable features: --features \"interceptors unified_sse\"");
}
