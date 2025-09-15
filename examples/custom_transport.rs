use ai_lib::transport::HttpTransport;
use ai_lib::{AiClient, Provider, ConnectionOptions};
use reqwest::Client;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a reqwest client with custom pool settings
    let reqwest_client = Client::builder()
        .pool_max_idle_per_host(32)
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .build()?;

    // Wrap into library transport and inject
    let _transport = HttpTransport::with_client(reqwest_client, Duration::from_secs(30));
    // Prefer unified client path; transport can be injected via feature-gated factory in real apps
    let _client = AiClient::with_options(
        Provider::Groq,
        ConnectionOptions {
            base_url: Some("https://api.groq.com".into()),
            proxy: None,
            api_key: None,
            timeout: Some(Duration::from_secs(30)),
            disable_proxy: false,
        },
    )?;

    println!("Created client with custom transport settings");
    Ok(())
}
