//! Tests for explicit override precedence: explicit > env > default
use ai_lib::{AiClient, ConnectionOptions, Provider};
use std::env;

#[tokio::test]
async fn explicit_api_key_overrides_env_openai() {
    // Setup an env var with a dummy value
    env::set_var("OPENAI_API_KEY", "env-key");
    let opts = ConnectionOptions {
        api_key: Some("explicit-key".into()),
        ..Default::default()
    };
    let client = AiClient::with_options(Provider::OpenAI, opts).expect("client");
    // We can't directly read the key (private), so rely on making a minimal request conversion path.
    // For now we just assert construction success; deeper inspection would require exposing adapter state.
    assert!(client.connection_options().unwrap().api_key.as_deref() == Some("explicit-key"));
}

#[tokio::test]
async fn disable_proxy_flag_applied() {
    env::set_var("AI_PROXY_URL", "http://should-not-be-used:8080");
    let opts = ConnectionOptions {
        disable_proxy: true,
        ..Default::default()
    };
    let client = AiClient::with_options(Provider::Groq, opts).expect("client");
    assert!(client.connection_options().unwrap().disable_proxy);
}

#[tokio::test]
async fn base_url_override_openai() {
    env::set_var("OPENAI_API_KEY", "env-key");
    let opts = ConnectionOptions {
        api_key: Some("explicit".into()),
        base_url: Some("https://example.com/v1".into()),
        ..Default::default()
    };
    let client = AiClient::with_options(Provider::OpenAI, opts).expect("client");
    assert_eq!(
        client.connection_options().unwrap().base_url.as_deref(),
        Some("https://example.com/v1")
    );
}
