use ai_lib::ProviderConfigs;
use ai_lib::transport::{HttpTransport, HttpTransportConfig};
use ai_lib::provider::GenericAdapter;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = HttpTransportConfig {
        timeout: Duration::from_secs(30),
        proxy: None,
        pool_max_idle_per_host: Some(16),
        pool_idle_timeout: Some(Duration::from_secs(60)),
    };

    let transport = HttpTransport::new_with_config(cfg)?;
    let provider_cfg = ProviderConfigs::groq();
    let _adapter = GenericAdapter::with_transport(provider_cfg, transport)?;

    println!("Created adapter using HttpTransportConfig");
    Ok(())
}
