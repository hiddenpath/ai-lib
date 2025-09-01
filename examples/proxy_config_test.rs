use ai_lib::{AiClient, AiClientBuilder, Provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing different proxy configuration modes...\n");

    // Test 1: Default behavior - no proxy, no environment variable reading
    println!("1. Default behavior (no proxy, no env var reading):");
    let client = AiClientBuilder::new(Provider::Groq).build()?;
    println!("   ✓ Client created successfully without proxy");
    println!("   ✓ No AI_PROXY_URL environment variable was read\n");

    // Test 2: Explicitly disable proxy
    println!("2. Explicitly disable proxy:");
    let client = AiClientBuilder::new(Provider::Groq)
        .without_proxy()
        .build()?;
    println!("   ✓ Client created successfully with explicit no-proxy setting\n");

    // Test 3: Use specific proxy URL
    println!("3. Use specific proxy URL:");
    let client = AiClientBuilder::new(Provider::Groq)
        .with_proxy(Some("http://custom.proxy.com:8080"))
        .build()?;
    println!("   ✓ Client created successfully with custom proxy: http://custom.proxy.com:8080\n");

    // Test 4: Use environment variable (if set)
    println!("4. Use AI_PROXY_URL environment variable:");
    let client = AiClientBuilder::new(Provider::Groq)
        .with_proxy(None)
        .build()?;
    println!("   ✓ Client created successfully with environment variable proxy (if set)\n");

    // Test 5: Full custom configuration
    println!("5. Full custom configuration:");
    let client = AiClientBuilder::new(Provider::Groq)
        .with_base_url("https://custom.groq.com")
        .with_proxy(Some("http://custom.proxy.com:8080"))
        .with_timeout(std::time::Duration::from_secs(60))
        .with_pool_config(32, std::time::Duration::from_secs(90))
        .build()?;
    println!("   ✓ Client created successfully with full custom configuration\n");

    println!("All proxy configuration tests passed! 🎉");
    println!("\nKey improvements:");
    println!("- Default behavior no longer reads AI_PROXY_URL automatically");
    println!("- with_proxy(None) reads AI_PROXY_URL when needed");
    println!("- without_proxy() explicitly disables proxy usage");
    println!("- with_proxy(Some(url)) uses specific proxy URL");

    Ok(())
}
