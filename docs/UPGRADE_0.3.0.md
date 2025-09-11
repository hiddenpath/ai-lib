# Upgrade Guide: 0.2.x → 0.3.0

This guide documents behavioral notes and recommended changes when upgrading to 0.3.0.

## TL;DR
- Default behavior remains the same. Existing code should compile unchanged.
- New capabilities are behind feature flags. Opt in when needed; otherwise, they have zero overhead.
- Prefer `client.default_chat_model()` in examples instead of hard-coded model IDs.

## What Changed
- Optional features (all default-off):
  - `interceptors`: Interceptor trait + pipeline for cross-cutting concerns.
  - `unified_sse`: Common streaming parser; adapters (Generic/Cohere/Mistral) can opt into unified parsing.
  - `unified_transport`: Shared reqwest client factory (proxy/timeout/pool via env vars).
  - `cost_metrics`: Minimal cost accounting hooks driven by env (`COST_INPUT_PER_1K`, `COST_OUTPUT_PER_1K`).
  - `routing_mvp`: Basic `ModelArray` routing (use sentinel model "__route__").
  - `observability`: `Tracer`/`AuditSink` facades (Noop by default).
  - `config_hot_reload`: ConfigProvider/ConfigWatcher facades (Noop by default).

## Source Compatibility
- `AiClient::new(Provider::X)` and `ChatCompletionRequest::new(model, messages)` remain unchanged.
- Streaming methods unchanged. Unified SSE parsing is only used if the `unified_sse` feature is enabled.
- No breaking renames in public APIs. Some adapter-local SSE helpers are now candidates for deprecation but still present.

## Recommended Adjustments
- Replace model literals in examples with `client.default_chat_model()` to decouple from vendor changes.
- If you previously wrote custom streaming parsing per provider, consider enabling `unified_sse` and removing custom code.
- If you need basic cost reporting, enable `cost_metrics` and set:
  - `COST_INPUT_PER_1K` (USD per 1K input tokens)
  - `COST_OUTPUT_PER_1K` (USD per 1K output tokens)

## Feature Usage Snippets
- Interceptors + unified SSE (requires API keys):
```bash
cargo run --features "interceptors unified_sse" --example deepseek_features
cargo run --features "interceptors unified_sse" --example mistral_features
```

- SSE tests:
```bash
cargo test --features unified_sse --test sse_parser_tests
```

- Cost & routing tests (non-network):
```bash
cargo test --features "cost_metrics routing_mvp" --test cost_and_routing
```

## Environment & Transport Notes
- Proxy: `AI_PROXY_URL`
- Timeout (shared client factory): `AI_HTTP_TIMEOUT_SECS` (seconds)
- Pool tuning (shared client factory):
  - `AI_HTTP_POOL_MAX_IDLE_PER_HOST`
  - `AI_HTTP_POOL_IDLE_TIMEOUT_MS`

## Deprecation Path (Informational)
- Adapter-local SSE helpers will be kept for a transition period. The unified parser is the preferred path.

## Troubleshooting
- “Tests compile but do nothing”: ensure you run file-based tests via `--test <name>` not name filters.
- “Models not found / 401 / 429”: verify vendor API keys and quotas; examples do not include backoff by default.

---
If issues arise during upgrade, please open an issue with repro steps and feature flags used.
