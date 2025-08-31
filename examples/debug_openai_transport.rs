/// OpenAI传输层调试示例 - OpenAI transport layer debugging example
use ai_lib::transport::{HttpClient, HttpTransport};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 OpenAI Transport Layer Debugging");
    println!("==================================");

    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("❌ OPENAI_API_KEY not set");
            return Ok(());
        }
    };

    // Use our HttpTransport
    let transport = HttpTransport::new();

    // Test GET request (model list) - we know this works
    println!("\n📋 Test GET request (model list):");
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));

    match transport
        .get::<serde_json::Value>("https://api.openai.com/v1/models", Some(headers))
        .await
    {
        Ok(response) => {
            let model_count = response["data"]
                .as_array()
                .map(|arr| arr.len())
                .unwrap_or(0);
            println!("✅ GET request successful, got {} models", model_count);
        }
        Err(e) => {
            println!("❌ GET request failed: {}", e);
        }
    }

    // Test POST request (chat completion) - this has issues
    println!("\n💬 Test POST request (chat completion):");
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));

    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Say 'test' in one word."
            }
        ],
        "max_tokens": 5
    });

    println!(
        "Request body: {}",
        serde_json::to_string_pretty(&request_body)?
    );

    match transport
        .post::<serde_json::Value, serde_json::Value>(
            "https://api.openai.com/v1/chat/completions",
            Some(headers),
            &request_body,
        )
        .await
    {
        Ok(response) => {
            println!("✅ POST request successful!");
            println!("Response: {}", serde_json::to_string_pretty(&response)?);
        }
        Err(e) => {
            println!("❌ POST request failed: {}", e);

            // Analyze error type
            let error_str = e.to_string();
            if error_str.contains("you must provide a model parameter") {
                println!("🔍 This error is strange because we did provide the model parameter");
                println!("   Possible reasons:");
                println!("   1. Proxy server modified the request body");
                println!("   2. Content-Type header issue");
                println!("   3. JSON serialization issue");
            }
        }
    }

    println!("\n💡 Debug Conclusion:");
    println!("   • GET request works → authentication and network connection OK");
    println!("   • POST request fails → may be proxy or request format issue");
    println!("   • Recommend checking proxy server's POST request handling");

    Ok(())
}
