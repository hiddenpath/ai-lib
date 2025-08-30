This file has been moved to `docs/PR_0.2.0.md` to keep the repository root focused.
See `docs/PR_0.2.0.md` for the full PR notes.

## Migration notes

- Default behavior unchanged: `HttpTransport::new()` still creates a sensible default client. Consumers who need tuning can either instantiate a custom `reqwest::Client` or use `HttpTransportConfig`.
- Suggested changelog entry: "Add transport tuning options: pool_max_idle_per_host, pool_idle_timeout; add convenience constructor for injecting external reqwest::Client."

## Tests & Validation

- All crate tests pass locally (`cargo test`).
- Lint: `cargo clippy --all-targets -- -D warnings` passes locally.

## Follow-ups

- Consider exposing additional reqwest tuning knobs (e.g., connection keep-alive, connect timeout) in `HttpTransportConfig` if users request them.
- Add an example showing how to profile connection reuse under load (benchmarks).
