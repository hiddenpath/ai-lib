# ai-lib 0.3.0-rc.1 (Pre-release)

This RC focuses on optional, feature-gated infrastructure: unified SSE parsing, interceptor pipeline, shared reqwest client factory, minimal cost metrics, basic routing MVP, and observability/config hot-reload facades.

## Compatibility
- Default behavior remains unchanged. Existing code and examples work without enabling any new features.
- Suggested tweak in examples: prefer `client.default_chat_model()` instead of hard-coded model IDs.

## What’s New (feature-gated)
- `interceptors`: Interceptor trait + pipeline; examples: `interceptors_pipeline`, `deepseek_features`, `mistral_features`.
- `unified_sse`: Common SSE parser; wired in GenericAdapter, Cohere, Mistral streaming.
- `unified_transport`: Shared reqwest client factory (proxy/timeout/pool env overrides).
- `cost_metrics`: Minimal cost accounting (env: `COST_INPUT_PER_1K`, `COST_OUTPUT_PER_1K`).
- `routing_mvp`: Strategy-based routing (`with_round_robin_chain`, `with_failover_chain`, or `RoutingStrategyBuilder`)—no sentinel models required.
- `observability`: Tracer/AuditSink traits (Noop implementations).
- `config_hot_reload`: ConfigProvider/ConfigWatcher traits (Noop implementations).

## Quick Validation
- Baseline (no features):
  - `cargo build --examples`
  - `cargo test`
- SSE:
  - `cargo test --features unified_sse --test sse_parser_tests`
- Cost & Routing (non-network):
  - `cargo test --features "cost_metrics routing_mvp" --test cost_and_routing`
- Interceptors + unified_sse examples (require API keys):
  - `cargo run --features "interceptors unified_sse" --example deepseek_features`
  - `cargo run --features "interceptors unified_sse" --example mistral_features`

## Notes
- Some adapter-local SSE helpers remain; unified parser is preferred going forward.
- Observability and config hot-reload are facades. Concrete backends and hot-reload implementations may arrive in a later minor.

## Next
- Complete bench/CI regression harness.
- Finalize upgrade guide and publish 0.3.0 stable.
