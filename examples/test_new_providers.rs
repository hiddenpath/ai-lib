use ai_lib::{client::AiClientBuilder, AiClient, Provider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing new providers integration...\n");

    // Test config-driven providers (OpenAI-compatible)
    let config_driven_providers = vec![
        Provider::OpenRouter,
        Provider::Replicate,
        Provider::ZhipuAI,
        Provider::MiniMax,
    ];

    for provider in config_driven_providers {
        println!("Testing {} (config-driven)...", format!("{:?}", provider));

        match AiClient::new(provider) {
            Ok(_client) => {
                println!("  ✓ Client created successfully");
                println!("  ✓ Default model: {}", provider.default_chat_model());
                if let Some(multimodal) = provider.default_multimodal_model() {
                    println!("  ✓ Multimodal model: {}", multimodal);
                }
                println!("  ✓ Is config-driven: {}", provider.is_config_driven());
            }
            Err(e) => {
                println!("  ✗ Failed to create client: {}", e);
            }
        }
        println!();
    }

    // Test independent providers
    let independent_providers = vec![Provider::Perplexity, Provider::AI21];

    for provider in independent_providers {
        println!("Testing {} (independent)...", format!("{:?}", provider));

        match AiClient::new(provider) {
            Ok(_client) => {
                println!("  ✓ Client created successfully");
                println!("  ✓ Default model: {}", provider.default_chat_model());
                if let Some(multimodal) = provider.default_multimodal_model() {
                    println!("  ✓ Multimodal model: {}", multimodal);
                }
                println!("  ✓ Is independent: {}", provider.is_independent());
            }
            Err(e) => {
                println!("  ✗ Failed to create client: {}", e);
            }
        }
        println!();
    }

    // Test failover with new providers
    println!("Testing failover with new providers...");
    let _client = AiClientBuilder::new(Provider::OpenAI)
        .with_failover_chain(vec![
            Provider::OpenRouter,
            Provider::Replicate,
            Provider::Perplexity,
        ])?
        .build()?;
    println!("  ✓ Failover chain configured via AiClientBuilder");

    println!("\nAll tests completed!");
    Ok(())
}
