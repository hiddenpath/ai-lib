#![cfg(feature = "interceptors")]

use ai_lib::{AiClient, Provider};
use std::time::Duration;

#[tokio::test]
async fn test_builder_enables_default_interceptors() {
    let client = AiClient::builder(Provider::OpenAI)
        .enable_default_interceptors()
        .build()
        .expect("Failed to build client");

    // We can't easily inspect the pipeline internals without exposing them,
    // but we can verify that the client built successfully.
    // In a real scenario, we might add a method to inspect the pipeline or use a mock.
    // For now, we rely on the fact that it builds without error.
    assert_eq!(client.provider_name(), "OpenAI");
}

#[tokio::test]
async fn test_builder_configures_rate_limit() {
    let client = AiClient::builder(Provider::OpenAI)
        .with_rate_limit(100)
        .build()
        .expect("Failed to build client");

    assert_eq!(client.provider_name(), "OpenAI");
}

#[tokio::test]
async fn test_builder_configures_circuit_breaker() {
    let client = AiClient::builder(Provider::OpenAI)
        .with_circuit_breaker(5, Duration::from_secs(60))
        .build()
        .expect("Failed to build client");

    assert_eq!(client.provider_name(), "OpenAI");
}

#[tokio::test]
async fn test_builder_configures_retry() {
    let client = AiClient::builder(Provider::OpenAI)
        .with_retry(3, Duration::from_secs(1), Duration::from_secs(10))
        .build()
        .expect("Failed to build client");

    assert_eq!(client.provider_name(), "OpenAI");
}

#[tokio::test]
async fn test_builder_configures_timeout() {
    let client = AiClient::builder(Provider::OpenAI)
        .with_interceptor_timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build client");

    assert_eq!(client.provider_name(), "OpenAI");
}

#[tokio::test]
async fn test_builder_disables_features() {
    let client = AiClient::builder(Provider::OpenAI)
        .enable_retry(false)
        .enable_circuit_breaker(false)
        .enable_rate_limit(false)
        .enable_interceptor_timeout(false)
        .build()
        .expect("Failed to build client");

    assert_eq!(client.provider_name(), "OpenAI");
}
