/// 网络连接诊断示例 - Network connection diagnosis example
use ai_lib::{AiClient, Provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Network Connection Diagnosis");
    println!("===============================");

    // Check proxy settings
    match std::env::var("AI_PROXY_URL") {
        Ok(proxy_url) => {
            println!("🌐 Current proxy setting: {}", proxy_url);
        }
        Err(_) => {
            println!("ℹ️  No proxy set");
        }
    }

    // Check API keys
    let providers = vec![
        ("GROQ_API_KEY", Provider::Groq, "Groq"),
        ("OPENAI_API_KEY", Provider::OpenAI, "OpenAI"),
        ("DEEPSEEK_API_KEY", Provider::DeepSeek, "DeepSeek"),
    ];

    println!("\n🔑 API Key Check:");
    for (env_var, provider, name) in &providers {
        match std::env::var(env_var) {
            Ok(_) => {
                println!("   ✅ {}: Set", name);

                // Test client creation
                match AiClient::new(*provider) {
                    Ok(_) => println!("      ✅ Client creation successful"),
                    Err(e) => println!("      ❌ Client creation failed: {}", e),
                }
            }
            Err(_) => {
                println!("   ❌ {}: Not set", name);
            }
        }
    }

    // Test basic network connection
    println!("\n🌐 Network Connection Test:");

    // Use reqwest directly for testing
    let client = reqwest::Client::new();

    // Test DeepSeek (domestic)
    println!("   Testing DeepSeek connection...");
    match client
        .get("https://api.deepseek.com/v1/models")
        .send()
        .await
    {
        Ok(response) => {
            println!(
                "      ✅ DeepSeek connection successful (status: {})",
                response.status()
            );
        }
        Err(e) => {
            println!("      ❌ DeepSeek connection failed: {}", e);
        }
    }

    println!("\n💡 Diagnosis Suggestions:");
    println!("   1. Ensure network connection is normal");
    println!("   2. Check firewall settings");
    println!("   3. Verify proxy server configuration");
    println!("   4. Confirm API key validity");

    Ok(())
}
