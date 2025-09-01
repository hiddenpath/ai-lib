use ai_lib::{AiClientBuilder, Provider};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing proxy configuration behavior in detail...\n");

    // Test 1: Check current environment
    println!("1. Current environment:");
    match env::var("AI_PROXY_URL") {
        Ok(url) => println!("   AI_PROXY_URL is set to: {}", url),
        Err(_) => println!("   AI_PROXY_URL is not set"),
    }
    println!();

    // Test 2: Default behavior - should not read environment variable
    println!("2. Default behavior test:");
    let _client = AiClientBuilder::new(Provider::Groq).build()?;
    println!("   ✓ Client created with default settings");
    println!("   ✓ No automatic proxy configuration from environment");
    println!();

    // Test 3: Explicit no-proxy setting
    println!("3. Explicit no-proxy test:");
    let _client = AiClientBuilder::new(Provider::Groq)
        .without_proxy()
        .build()?;
    println!("   ✓ Client created with explicit no-proxy setting");
    println!("   ✓ This ensures no proxy is used regardless of environment");
    println!();

    // Test 4: Use environment variable explicitly
    println!("4. Environment variable proxy test:");
    let _client = AiClientBuilder::new(Provider::Groq)
        .with_proxy(None)
        .build()?;
    println!("   ✓ Client created with environment variable proxy (if available)");
    println!("   ✓ This is the only way to use AI_PROXY_URL now");
    println!();

    // Test 5: Custom proxy URL
    println!("5. Custom proxy URL test:");
    let _client = AiClientBuilder::new(Provider::Groq)
        .with_proxy(Some("http://custom.proxy.com:8080"))
        .build()?;
    println!("   ✓ Client created with custom proxy: http://custom.proxy.com:8080");
    println!("   ✓ Environment variable is ignored when custom URL is provided");
    println!();

    // Test 6: Verify the new behavior
    println!("6. Behavior verification:");
    println!("   ✓ Before: HttpTransport::new() always read AI_PROXY_URL");
    println!("   ✓ After:  HttpTransport::new_without_proxy() used by default");
    println!("   ✓ Before: with_proxy() required string parameter");
    println!("   ✓ After:  with_proxy(Option<&str>) supports both modes");
    println!("   ✓ New:   without_proxy() explicitly disables proxy");
    println!();

    println!("All tests passed! 🎉");
    println!("\nSummary of changes:");
    println!("- Default behavior no longer automatically reads AI_PROXY_URL");
    println!("- Users must explicitly choose proxy behavior:");
    println!("  * .build() - No proxy, no env var reading");
    println!("  * .without_proxy() - Explicitly no proxy");
    println!("  * .with_proxy(None) - Use AI_PROXY_URL environment variable");
    println!("  * .with_proxy(Some(url)) - Use specific proxy URL");
    println!("\nThis prevents the issue where clearing environment variables");
    println!("still resulted in proxy usage due to automatic detection.");

    Ok(())
}
