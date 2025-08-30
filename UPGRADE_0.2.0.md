Upgrade notes for ai-lib 0.2.0

This release contains the following notable changes:


Migration guidance


```rust
let metrics = Arc::new(MyMetrics::new());
let adapter = OpenAiAdapter::with_transport_ref_and_metrics(transport, api_key, base_url, metrics)?;
```



Follow-ups (recommended)


This file has been moved to `docs/UPGRADE_0.2.0.md` to keep the repository root focused.
See `docs/UPGRADE_0.2.0.md` for the full upgrade notes.

