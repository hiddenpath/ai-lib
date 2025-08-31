/// AI-lib 配置检查工具 - AI-lib configuration check tool
use ai_lib::{AiClient, Provider};

fn main() {
    println!("🔧 AI-lib Configuration Check Tool");
    println!("==================================");

    // Check API keys
    let api_keys = vec![
        ("GROQ_API_KEY", "Groq"),
        ("OPENAI_API_KEY", "OpenAI"),
        ("DEEPSEEK_API_KEY", "DeepSeek"),
    ];

    println!("\n🔑 API Key Check:");
    let mut configured_count = 0;

    for (env_var, provider) in &api_keys {
        match std::env::var(env_var) {
            Ok(key) => {
                let masked = if key.len() > 8 {
                    format!("{}...{}", &key[..4], &key[key.len() - 4..])
                } else {
                    "*".repeat(key.len())
                };
                println!("   ✅ {}: {} ({})", provider, env_var, masked);
                configured_count += 1;
            }
            Err(_) => {
                println!("   ❌ {}: {} not set", provider, env_var);
            }
        }
    }

    // Check proxy configuration
    println!("\n🌐 Proxy Configuration Check:");
    match std::env::var("AI_PROXY_URL") {
        Ok(proxy_url) => {
            println!("   ✅ AI_PROXY_URL: {}", proxy_url);
        }
        Err(_) => {
            println!("   ℹ️  AI_PROXY_URL: not set (optional)");
        }
    }

    // Test client creation
    println!("\n🚀 Client Creation Test:");
    let providers = vec![
        (Provider::Groq, "Groq"),
        (Provider::OpenAI, "OpenAI"),
        (Provider::DeepSeek, "DeepSeek"),
    ];

    for (provider, name) in providers {
        match AiClient::new(provider) {
            Ok(_) => println!("   ✅ {}: client created successfully", name),
            Err(e) => println!("   ❌ {}: {}", name, e),
        }
    }

    // Configuration suggestions
    println!("\n💡 Configuration Suggestions:");
    if configured_count == 0 {
        println!("   • Set at least one API key to get started");
        println!("   • Recommended to set GROQ_API_KEY first (free and fast)");
    } else if configured_count < api_keys.len() {
        println!(
            "   • Configured {}/{} providers",
            configured_count,
            api_keys.len()
        );
        println!("   • Can set more API keys to test different providers");
    } else {
        println!("   • 🎉 All providers configured!");
        println!("   • Can run: cargo run --example test_all_providers");
    }

    println!("\n📝 Setup Examples (Windows):");
    println!("   set GROQ_API_KEY=your_groq_key");
    println!("   set OPENAI_API_KEY=your_openai_key");
    println!("   set DEEPSEEK_API_KEY=your_deepseek_key");
    println!("   set AI_PROXY_URL=http://proxy.example.com:8080");

    println!("\n📝 Setup Examples (Linux/Mac):");
    println!("   export GROQ_API_KEY=your_groq_key");
    println!("   export OPENAI_API_KEY=your_openai_key");
    println!("   export DEEPSEEK_API_KEY=your_deepseek_key");
    println!("   export AI_PROXY_URL=http://proxy.example.com:8080");
}
