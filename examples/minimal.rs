//! Minimal example for ai-lib-pro
//!
//! Safe to run without API keys; no network calls are performed.

use ai_lib_pro::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ai-lib-pro minimal example");
    println!("ai-lib-pro version: {}", env!("CARGO_PKG_VERSION"));

    // Create a client without sending requests (no network)
    let client = AiClient::new(Provider::Groq)?;
    println!("provider = {:?}", client.current_provider());

    // Show which PRO feature sets are compiled in
    println!("features:");
    println!("  full: {}", cfg!(feature = "full"));
    println!("  pricing_catalog: {}", cfg!(feature = "pricing_catalog"));
    println!("  routing_advanced: {}", cfg!(feature = "routing_advanced"));
    println!("  observability_pro: {}", cfg!(feature = "observability_pro"));
    println!("  quota: {}", cfg!(feature = "quota"));
    println!("  audit: {}", cfg!(feature = "audit"));
    println!("  kms: {}", cfg!(feature = "kms"));
    println!("  config_hot_reload_pro: {}", cfg!(feature = "config_hot_reload_pro"));

    Ok(())
}
