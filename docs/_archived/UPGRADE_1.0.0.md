## ai-lib 1.0 Upgrade Guide

This guide helps you migrate projects from the 0.3.x line to the new 1.0 architecture. The changes focus on three pillars: the `ChatProvider` trait, strategy-based routing, and a clarified request surface.

---

## Summary of Changes

| Area | Before | After |
|------|--------|-------|
| Core abstraction | `AiClient` stored concrete adapters | `AiClient` owns `Box<dyn ChatProvider>` |
| Failover API | `AiClient::with_failover(Vec<Provider>)` | `AiClientBuilder::with_failover_chain` / `with_round_robin_chain` |
| Routing trigger | Magic model `"__route__"` | Explicit strategy builders |
| Provider escape hatch | `with_provider_specific()` placeholder | `ChatCompletionRequest.extensions` + `with_extension()` |
| Builder ergonomics | `AiClientBuilder` only | Per-provider builders (`GroqBuilder`, `OpenAiBuilder`, …) |

---

## 1. Replace `with_failover` with Builder Chains

```rust
// ⛔️ Old (0.3.x)
let client = AiClient::new(Provider::OpenAI)?
    .with_failover(vec![Provider::Anthropic, Provider::Groq]);

// ✅ New (1.0)
use ai_lib::client::AiClientBuilder;

let client = AiClientBuilder::new(Provider::OpenAI)
    .with_failover_chain(vec![Provider::Anthropic, Provider::Groq])?
    .build()?;
```

`RoutingStrategyBuilder::build_round_robin()` is available when you prefer load distribution instead of sequential failover.

---

## 2. Remove `"__route__"` Sentinels

Routing no longer relies on `"__route__"` placeholders. Instead, construct explicit strategies (above) or implement your own `ChatProvider`:

```rust
struct MyRouter {
    providers: Vec<Box<dyn ChatProvider>>,
}

#[async_trait::async_trait]
impl ChatProvider for MyRouter {
    fn name(&self) -> &str { "my_router" }
    async fn chat(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse, AiLibError> {
        // custom logic…
    }
    // implement stream/batch/list_models as needed
}
```

Inject the router via `AiClientBuilder::with_strategy(Box::new(MyRouter { … }))`.

---

## 3. Migrate Provider-Specific Overrides

`ChatCompletionRequest` now exposes `extensions` plus helper methods:

```rust
let request = ChatCompletionRequest::new(model, messages)
    .with_extension("reasoning_format", serde_json::json!("parsed"))
    .with_extension("parallel_tool_calls", serde_json::json!(true));
```

Adapters automatically merge `extensions` into the outgoing JSON payload.

> **Note**: The deprecated `with_provider_specific()` method has been removed in 1.0. Use `with_extension()` instead.

---

## 4. Adopt Provider Builders (Optional but Recommended)

Provider-specific builders wrap `AiClientBuilder` with sensible defaults:

```rust
use ai_lib::provider::GroqBuilder;

let client = GroqBuilder::new()
    .with_base_url("https://custom.groq.com")
    .with_proxy(Some("http://proxy.internal:8080"))
    .build()?;                // returns AiClient

let groq_provider = GroqBuilder::new().build_provider()?; // returns Box<dyn ChatProvider>
```

Builders exist for every variant in the `Provider` enum (e.g., `OpenAiBuilder`, `DeepSeekBuilder`, `QwenBuilder`, `Ai21Builder`).

---

## 5. Miscellaneous Changes

1. `AiLibError::is_retryable()` now inspects HTTP status codes directly (5xx and 429 are retryable).
2. `types::common::{Usage, UsageStatus}` aliases were removed—import them from `ai_lib::types::response`.
3. `AiClientBuilder::build_provider()` exposes boxed providers for composing custom strategies.

---

- [ ] Replace any `with_failover` usage with `AiClientBuilder::with_failover_chain` (or a custom `RoutingStrategyBuilder`).
- [ ] Remove `"__route__"` sentinel models.
- [ ] Update custom adapters to honor `request.extensions` if they inspect raw JSON.
- [ ] Import `Usage`/`UsageStatus` from `ai_lib::types::response`.
- [ ] (Optional) Adopt provider builders for clearer configuration.

Need help? Open a GitHub discussion or ping `@ai-lib` in the community Slack. Happy upgrading! 🎉

