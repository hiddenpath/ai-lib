use ai_lib::provider::GenericAdapter;
use ai_lib::transport::HttpTransport;
use ai_lib::ProviderConfigs;
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
    let transport = HttpTransport::with_client(reqwest_client, Duration::from_secs(30));

    let config = ProviderConfigs::groq();
    let _adapter = GenericAdapter::with_transport(config, transport)?;

    println!("Created generic adapter with custom transport");
    Ok(())
}
