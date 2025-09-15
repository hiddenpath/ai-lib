use ai_lib::transport::{HttpTransport, HttpTransportConfig};
use ai_lib::{AiClient, Provider, ConnectionOptions};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = HttpTransportConfig {
        timeout: Duration::from_secs(30),
        proxy: None,
        pool_max_idle_per_host: Some(16),
        pool_idle_timeout: Some(Duration::from_secs(60)),
    };

    let transport = HttpTransport::new_with_config(cfg)?;
    // Use transport via ConnectionOptions (unified path) instead of constructing adapter directly
    let _client = AiClient::with_options(
        Provider::Groq,
        ConnectionOptions {
            base_url: Some("https://api.groq.com".into()),
            proxy: None,
            api_key: None,
            timeout: Some(std::time::Duration::from_secs(30)),
            disable_proxy: false,
        },
    )?;

    println!("Created client using HttpTransportConfig via ConnectionOptions");
    Ok(())
}
