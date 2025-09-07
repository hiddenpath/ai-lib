/// AI-lib proxy server support example
use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 AI-lib Proxy Server Support Example");
    println!("=====================================");

    // Check proxy configuration
    match std::env::var("AI_PROXY_URL") {
        Ok(proxy_url) => {
            println!("✅ Proxy configuration detected: {}", proxy_url);
            println!("   All HTTP requests will go through this proxy server");
        }
        Err(_) => {
            println!("ℹ️  AI_PROXY_URL environment variable not set");
            println!("   To use proxy, set: export AI_PROXY_URL=http://proxy.example.com:8080");
        }
    }

    println!("\n🚀 Creating AI client...");
    let client = AiClient::new(Provider::Groq)?;
    println!(
        "✅ Client created successfully, provider: {:?}",
        client.current_provider()
    );

    // Create test request
    let request = ChatCompletionRequest::new(
        "llama3-8b-8192".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello! This request may go through a proxy.".to_string()),
            function_call: None,
        }],
    );

    println!("\n📤 Preparing to send request...");
    println!("   Model: {}", request.model);
    println!("   Message: {}", request.messages[0].content.as_text());

    // Get model list (this request will also go through proxy)
    match client.list_models().await {
        Ok(models) => {
            println!("\n📋 Model list obtained through proxy:");
            for model in models {
                println!("   • {}", model);
            }
        }
        Err(e) => {
            println!("\n⚠️  Failed to get model list: {}", e);
            println!("   This may be due to:");
            println!("   • GROQ_API_KEY environment variable not set");
            println!("   • Proxy server configuration error");
            println!("   • Network connection issue");
        }
    }

    println!("\n💡 Proxy Configuration Instructions:");
    println!("   • Set environment variable: AI_PROXY_URL=http://your-proxy:port");
    println!("   • Supports HTTP and HTTPS proxies");
    println!("   • Supports authenticated proxies: http://user:pass@proxy:port");
    println!("   • All AI providers will automatically use this proxy configuration");

    Ok(())
}
