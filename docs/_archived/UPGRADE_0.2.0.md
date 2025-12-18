Upgrade notes for ai-lib 0.2.0

This release contains the following notable changes:

- Added a minimal `Metrics` trait and `NoopMetrics` implementation. Adapters now accept an `Arc<dyn Metrics>` and increment request counters and use timers for request durations.
- Standardized metric key names across adapters: `<provider>.requests` and `<provider>.request_duration_ms`.
- Added tests that assert adapters call metrics (`tests/metrics_tests.rs`).
- Added upload adapter tests and various clippy-driven fixes.

Migration guidance

- If you instantiate adapters via their no-arg constructors (e.g. `OpenAiAdapter::new()`), behavior is unchanged: adapters default to `NoopMetrics`.
- If you create adapters in tests or production with injected transports, you can now pass a metrics implementation using the existing `with_transport_ref_and_metrics` or `with_metrics` constructors. Example:

```rust
let metrics = Arc::new(MyMetrics::new());
let adapter = OpenAiAdapter::with_transport_ref_and_metrics(transport, api_key, base_url, metrics)?;
```

Compatibility notes

- The public API surface is minimally changed: only additional constructors that accept `Arc<dyn Metrics>` were added. Existing code that used earlier constructors continues to work.
- If you implement custom `Metrics` for tests, make sure the implementation is `Send + Sync + 'static`.

Follow-ups (recommended)

- Standardize metric keys in a central module (future work).
- Add more metrics (upload success/failure counters, timers for uploads).
- Add CI enforcement for clippy + tests.
