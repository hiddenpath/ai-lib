#![cfg(any(feature = "cost_metrics", feature = "routing_mvp"))]

use ai_lib::{AiClient, ChatCompletionRequest, Provider};

#[test]
fn cost_env_defaults_are_parsable() {
    #[cfg(feature = "cost_metrics")]
    {
        // Ensure helper works with defaults even if env not set
        let usd = ai_lib::metrics::cost::estimate_usd(1000, 2000);
        assert!(usd >= 0.0);
    }
}

#[tokio::test]
async fn can_build_client_and_request_without_network() {
    // Build a client and construct a request only; do not send network calls.
    let client = AiClient::new(Provider::Groq).expect("client");
    let req = ChatCompletionRequest::new(client.default_chat_model(), vec![]);
    assert!(!req.model.is_empty());
}
