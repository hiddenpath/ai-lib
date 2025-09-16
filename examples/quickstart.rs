/// AI-lib quickstart example
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI-lib Quickstart Example");
    println!("============================");

    // 🎯 Simplest usage - create client with one line of code
    println!("\n📋 Simplest usage:");
    let client = AiClient::new(Provider::Groq)?;
    println!("✅ Client created successfully!");

    // 🔧 If you need custom configuration, use builder pattern
    println!("\n📋 Custom configuration:");
    let client = AiClientBuilder::new(Provider::Groq)
        .with_base_url("https://custom.groq.com") // Optional: custom server
        .with_proxy(Some("http://proxy.example.com:8080")) // Optional: custom proxy
        .build()?;
    println!("✅ Custom client created successfully!");

    // 📝 Create chat request
    println!("\n📋 Create chat request:");
    let request = ChatCompletionRequest::new(
        "llama3-8b-8192".to_string(), // Model name
        vec![Message {
            role: Role::User,
            content: Content::new_text("Hello! How are you?"),
            function_call: None,
        }],
    );
    println!("✅ Request created successfully!");

    // 🌐 Send request (requires GROQ_API_KEY environment variable)
    println!("\n📋 Send request:");
    println!("   Note: Set GROQ_API_KEY environment variable for actual API calls");
    println!("   Usage: export GROQ_API_KEY=your_api_key_here");

    // Check if API key is available
    match std::env::var("GROQ_API_KEY") {
        Ok(_) => {
            println!("✅ GROQ_API_KEY detected, ready to send actual requests");
            // Uncomment the following code to send actual request
            // let response = client.chat_completion(request).await?;
            // println!("🤖 AI response: {}", response.choices[0].message.content.as_text());
        }
        Err(_) => {
            println!("ℹ️  GROQ_API_KEY not set, skipping actual request");
            println!("   This is a demo showing how to build requests");
        }
    }

    // 🎨 More customization options
    println!("\n📋 More customization options:");
    let advanced_client = AiClientBuilder::new(Provider::Groq)
        .with_timeout(std::time::Duration::from_secs(60)) // 60 second timeout
        .with_pool_config(16, std::time::Duration::from_secs(60)) // Connection pool config
        .build()?;
    println!("✅ Advanced configuration client created successfully!");

    // 🔄 Switch to other providers
    println!("\n📋 Switch to other providers:");
    let deepseek_client = AiClient::new(Provider::DeepSeek)?;
    println!("✅ DeepSeek client created successfully!");

    let ollama_client = AiClient::new(Provider::Ollama)?;
    println!("✅ Ollama client created successfully!");

    println!("\n🎉 Quickstart completed!");
    println!("\n💡 Key points:");
    println!("   1. AiClient::new() - Simplest usage with automatic environment detection");
    println!("   2. AiClientBuilder - Builder pattern with custom configuration support");
    println!(
        "   3. Environment variable priority: Explicit setting > Environment variable > Default"
    );
    println!("   4. Support for all mainstream AI providers");
    println!("   5. Backward compatible, existing code requires no changes");

    Ok(())
}
