use ai_lib::metrics::Metrics;
use ai_lib::types::AiLibError;
use std::sync::Arc;

use serde_json::json;

// Include test utility mock transport into this test crate's module namespace
mod utils {
    include!("utils/mock_transport.rs");
}

use ai_lib::ChatApi;
use std::sync::{Mutex, OnceLock};

static TIMER_STORE: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn take_timers_global() -> Vec<String> {
    let m = TIMER_STORE.get_or_init(|| Mutex::new(Vec::new()));
    let mut lock = m.lock().unwrap();
    std::mem::take(&mut *lock)
}

struct MockMetrics {
    calls: tokio::sync::Mutex<Vec<(String, u64)>>,
}

impl MockMetrics {
    fn new() -> Self {
        Self {
            calls: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    async fn take_calls(&self) -> Vec<(String, u64)> {
        let mut lock = self.calls.lock().await;
        std::mem::take(&mut *lock)
    }
}

#[async_trait::async_trait]
impl Metrics for MockMetrics {
    async fn incr_counter(&self, name: &str, value: u64) {
        let mut lock = self.calls.lock().await;
        lock.push((name.to_string(), value));
    }

    async fn record_gauge(&self, _name: &str, _value: f64) {}

    async fn start_timer(&self, name: &str) -> Option<Box<dyn ai_lib::metrics::Timer + Send>> {
        Some(Box::new(MockTimer {
            name: name.to_string(),
        }))
    }

    async fn record_histogram(&self, _name: &str, _value: f64) {}

    async fn record_histogram_with_tags(&self, _name: &str, _value: f64, _tags: &[(&str, &str)]) {}

    async fn incr_counter_with_tags(&self, _name: &str, _value: u64, _tags: &[(&str, &str)]) {}

    async fn record_gauge_with_tags(&self, _name: &str, _value: f64, _tags: &[(&str, &str)]) {}

    async fn record_error(&self, _name: &str, _error_type: &str) {}

    async fn record_success(&self, _name: &str, _success: bool) {}
}

struct MockTimer {
    name: String,
}

impl ai_lib::metrics::Timer for MockTimer {
    fn stop(self: Box<Self>) {
        let name = self.name.clone();
        let m = TIMER_STORE.get_or_init(|| Mutex::new(Vec::new()));
        let mut lock = m.lock().unwrap();
        lock.push(name);
    }
}

#[tokio::test]
async fn generic_adapter_calls_metrics() -> Result<(), AiLibError> {
    use ai_lib::provider::config::ProviderConfig;
    use ai_lib::provider::generic::GenericAdapter;
    use ai_lib::types::{ChatCompletionRequest, Message, Role};

    // use existing MockTransport from tests/utils to avoid network
    let post_resp = json!({
        "id": "r1",
        "object": "chat.completion",
        "created": 0,
        "model": "m",
        "choices": [{ "message": { "role": "assistant", "content": "ok" }, "finish_reason": null }],
        "usage": { "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 }
    });

    let transport = Arc::new(utils::MockTransport::new(post_resp));
    let cfg = ProviderConfig::openai_compatible("http://example", "API_KEY", "gpt-3.5-turbo", None);
    let metrics_arc = Arc::new(MockMetrics::new());
    let metrics: Arc<dyn Metrics> = metrics_arc.clone();

    let adapter =
        GenericAdapter::with_transport_ref_and_metrics(cfg, transport.clone(), metrics.clone())?;

    let msg = Message {
        role: Role::User,
        content: ai_lib::types::common::Content::Text("hi".to_string()),
        function_call: None,
    };
    let req = ChatCompletionRequest::new("m".to_string(), vec![msg]);

    let _ = adapter.chat_completion(req).await?;

    let calls = metrics_arc.take_calls().await;
    assert!(calls
        .iter()
        .any(|(n, _)| n.contains("generic.requests") || n.contains("requests")));

    // timers should have been started and stopped, or at least the adapter recorded a timer event
    let timers = take_timers_global();
    let timer_ok = timers
        .iter()
        .any(|t| t.contains("generic.request_duration_ms"));
    let fallback_ok = calls
        .iter()
        .any(|(n, _)| n == "generic.request_timer_recorded");
    assert!(
        timer_ok || fallback_ok,
        "no timer recorded and no fallback counter present"
    );
    Ok(())
}

#[tokio::test]
async fn openai_adapter_calls_metrics() -> Result<(), AiLibError> {
    use ai_lib::provider::openai::OpenAiAdapter;
    use ai_lib::types::{ChatCompletionRequest, Message, Role};

    let post_resp = json!({
        "id": "r2",
        "object": "chat.completion",
        "created": 0,
        "model": "m",
        "choices": [{ "message": { "role": "assistant", "content": "ok" }, "finish_reason": null }],
        "usage": { "prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0 }
    });

    let transport = Arc::new(utils::MockTransport::new(post_resp));
    let metrics_arc = Arc::new(MockMetrics::new());
    let metrics: Arc<dyn Metrics> = metrics_arc.clone();

    let adapter = OpenAiAdapter::with_transport_ref_and_metrics(
        transport.clone(),
        "API_KEY".to_string(),
        "http://example.com".to_string(),
        metrics.clone(),
    )?;

    let msg = Message {
        role: Role::User,
        content: ai_lib::types::common::Content::Text("hi".to_string()),
        function_call: None,
    };
    let req = ChatCompletionRequest::new("m".to_string(), vec![msg]);

    let _ = adapter.chat_completion(req).await?;

    let calls = metrics_arc.take_calls().await;
    assert!(calls
        .iter()
        .any(|(n, _)| n.contains("openai.requests") || n.contains("requests")));

    let timers = take_timers_global();
    assert!(timers
        .iter()
        .any(|t| t.contains("openai.request_duration_ms")));
    Ok(())
}
