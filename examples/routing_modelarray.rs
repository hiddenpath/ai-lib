#[cfg(feature = "routing_mvp")]
use ai_lib::{AiClientBuilder, ChatCompletionRequest, Message, Provider, Role};
#[cfg(feature = "routing_mvp")]
use ai_lib::types::common::Content;

#[cfg(feature = "routing_mvp")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ai_lib::provider::models::{ModelArray, ModelEndpoint, LoadBalancingStrategy};

    // Construct a simple ModelArray with two endpoints
    let mut array = ModelArray::new("demo")
        .with_strategy(LoadBalancingStrategy::RoundRobin);
    array.add_endpoint(ModelEndpoint {
        name: "groq-70b".to_string(),
        model_name: "llama-3.3-70b-versatile".to_string(),
        url: "https://api.groq.com".to_string(),
        weight: 1.0,
        healthy: true,
        connection_count: 0,
    });
    array.add_endpoint(ModelEndpoint {
        name: "groq-8b".to_string(),
        model_name: "llama-3.1-8b-instant".to_string(),
        url: "https://api.groq.com".to_string(),
        weight: 1.0,
        healthy: true,
        connection_count: 0,
    });

    // Build client with routing array
    let client = AiClientBuilder::new(Provider::Groq)
        .with_routing_array(array)
        .build()?;

    // Use sentinel model "__route__" to trigger routing selection
    let req = ChatCompletionRequest::new(
        "__route__".to_string(),
        vec![Message { role: Role::User, content: Content::new_text("Say hi"), function_call: None }]
    );
    let resp = client.chat_completion(req).await?;
    println!("selected model: {}", resp.model);
    Ok(())
}

#[cfg(not(feature = "routing_mvp"))]
fn main() {
    eprintln!("Enable feature: --features routing_mvp");
}


