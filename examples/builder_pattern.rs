/// AI client builder pattern example
use ai_lib::types::common::Content;
use ai_lib::{AiClient, AiClientBuilder, ChatCompletionRequest, Message, Provider, Role};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI Client Builder Pattern Example");
    println!("===================================");

    // Example 1: Simplest usage - automatic environment variable detection
    println!("\n📋 Example 1: Simplest usage");
    println!("   Automatically detect GROQ_BASE_URL and AI_PROXY_URL from environment variables");

    let client = AiClientBuilder::new(Provider::Groq).build()?;
    println!(
        "✅ Client created successfully, provider: {:?}",
        client.current_provider()
    );

    // Example 2: Custom base_url
    println!("\n📋 Example 2: Custom base_url");
    println!("   Use custom Groq server address");

    let client = AiClientBuilder::new(Provider::Groq)
        .with_base_url("https://custom.groq.com")
        .build()?;
    println!("✅ Client created successfully with custom base_url");

    // Example 3: Custom base_url and proxy
    println!("\n📋 Example 3: Custom base_url and proxy");
    println!("   Use custom server and proxy");

    let client = AiClientBuilder::new(Provider::Groq)
        .with_base_url("https://custom.groq.com")
        .with_proxy(Some("http://proxy.example.com:8080"))
        .build()?;
    println!("✅ Client created successfully with custom base_url and proxy");

    // Example 4: Full custom configuration
    println!("\n📋 Example 4: Full custom configuration");
    println!("   Custom timeout, connection pool and other advanced configurations");

    let client = AiClientBuilder::new(Provider::Groq)
        .with_base_url("https://custom.groq.com")
        .with_proxy(Some("http://proxy.example.com:8080"))
        .with_timeout(Duration::from_secs(60))
        .with_pool_config(32, Duration::from_secs(90))
        .build()?;
    println!("✅ Client created successfully with full custom configuration");

    // Example 5: Use convenient builder method
    println!("\n📋 Example 5: Use convenient builder method");
    println!("   Create builder through AiClient::builder()");

    let client = AiClient::builder(Provider::Groq)
        .with_base_url("https://custom.groq.com")
        .with_proxy(Some("http://proxy.example.com:8080"))
        .build()?;
    println!("✅ Client created successfully using convenient builder method");

    // Example 6: Environment variable priority demonstration
    println!("\n📋 Example 6: Environment variable priority demonstration");
    println!("   Set environment variables, then use builder");

    // Set environment variables
    std::env::set_var("GROQ_BASE_URL", "https://env.groq.com");
    std::env::set_var("AI_PROXY_URL", "http://env.proxy.com:8080");

    // Don't set any custom configuration, should use environment variables
    let client = AiClientBuilder::new(Provider::Groq).build()?;
    println!("✅ Client created successfully using environment variable configuration");

    // Explicit settings override environment variables
    let client = AiClientBuilder::new(Provider::Groq)
        .with_base_url("https://explicit.groq.com")
        .with_proxy(Some("http://explicit.proxy.com:8080"))
        .build()?;
    println!(
        "✅ Client created successfully, explicit configuration overrides environment variables"
    );

    // Example 7: Different provider configurations
    println!("\n📋 Example 7: Different provider configurations");

    // Groq
    let groq_client = AiClientBuilder::new(Provider::Groq)
        .with_base_url("https://custom.groq.com")
        .build()?;
    println!("✅ Groq client created successfully");

    // DeepSeek
    let deepseek_client = AiClientBuilder::new(Provider::DeepSeek)
        .with_base_url("https://custom.deepseek.com")
        .with_proxy(Some("http://proxy.example.com:8080"))
        .build()?;
    println!("✅ DeepSeek client created successfully");

    // Ollama (local deployment)
    let ollama_client = AiClientBuilder::new(Provider::Ollama)
        .with_base_url("http://localhost:11434")
        .build()?;
    println!("✅ Ollama client created successfully");

    // Example 8: Error handling
    println!("\n📋 Example 8: Error handling");
    println!("   Try to set custom configuration for unsupported provider");

    match AiClientBuilder::new(Provider::OpenAI)
        .with_base_url("https://custom.openai.com")
        .build()
    {
        Ok(_) => println!("❌ This should not succeed"),
        Err(e) => println!("✅ Correctly caught error: {}", e),
    }

    println!("\n🎉 All examples completed!");
    println!("\n💡 Advantages of builder pattern:");
    println!("   1. Automatic environment variable detection, reducing configuration code");
    println!("   2. Support for progressive custom configuration");
    println!("   3. Method chaining for cleaner code");
    println!("   4. Backward compatible, existing code requires no changes");
    println!("   5. Support for advanced configuration (timeout, connection pool, etc.)");

    Ok(())
}
