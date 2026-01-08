/// AI-lib quickstart example
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI-lib v0.5.0 Quickstart");
    println!("============================");

    // 🎯 Simplest v0.5.0 usage - create client with a model ID
    // Configurations are loaded from the embedded aimanifest.yaml
    println!("\n📋 Model-Driven simplified usage:");
    let client = AiClientBuilder::new(Provider::OpenAI)
        .with_model("gpt-4o")
        .build()?;
    println!(
        "✅ Client created via Manifest for model: {}",
        client.default_chat_model()
    );

    // 🔧 Customize behavior while remaining manifest-compatible
    println!("\n📋 Advanced Client Configuration:");
    let _custom_client = AiClientBuilder::new(Provider::OpenAI)
        .with_model("gpt-4o")
        .with_timeout(std::time::Duration::from_secs(30))
        .build()?;
    println!("✅ Custom client created successfully!");

    // 📝 Create a structured chat request
    println!("\n📋 Creating a chat request:");
    let request = ChatCompletionRequest::new(
        "gpt-4o".to_string(),
        vec![Message::user("Tell me one interesting fact about Rust.")],
    );
    println!("✅ Request built for model: {}", request.model);

    // 🌐 Multi-provider support (all driven by local or embedded YAML)
    println!("\n📋 Switching Providers (Zero code change needed):");

    // Switch to Groq (Llama 3) via manifest
    let groq_client = AiClientBuilder::new(Provider::Groq)
        .with_model("llama-3.3-70b-versatile")
        .build()?;
    println!(
        "✅ Groq client (Manifest-driven) created: {}",
        groq_client.default_chat_model()
    );

    // Switch to Mistral via manifest
    let mistral_client = AiClientBuilder::new(Provider::Mistral)
        .with_model("mistral-large-latest")
        .build()?;
    println!(
        "✅ Mistral client (Manifest-driven) created: {}",
        mistral_client.default_chat_model()
    );

    println!("\n🎉 Quickstart completed!");
    println!("\n💡 Key points for v0.5.0:");
    println!("   1. Manifest-First: No hardcoded provider logic; all details are in YAML.");
    println!("   2. Model-Centric: Use .with_model() to pick specific capabilities.");
    println!("   3. Unified SSE: Streaming is handled by operators, not provider branches.");
    println!("   4. Zero-Code: Add new providers to aimanifest.yaml without rebuilding the SDK.");

    Ok(())
}
