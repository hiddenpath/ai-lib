# ai-lib Release Draft — v0.1.0 (2025-08-26)

This is a release draft for ai-lib v0.1.0. It summarizes the important changes since v0.0.2 and includes migration notes, quick upgrade steps, and suggested GitHub release copy.

---

## TL;DR

- Release: v0.1.0 (2025-08-26)
- Highlights: object-safe transport abstraction, Cohere & Mistral adapters, GenericAdapter improvements, model-listing example, various build & API stabilizations.
- Breaking / notable changes: Bedrock adapter deferred and removed from public exports. Adapters now accept an object-safe transport reference for DI/testing.

---

## Notable changes

### Added
- Object-safe transport trait: `DynHttpTransport` and boxed shim for `HttpTransport` to allow runtime injection and easier testing.
- Cohere adapter with SSE streaming + fallback simulation.
- Mistral HTTP adapter (conservative, non-SDK) with streaming support.
- `examples/list_models_smoke.rs` to quickly validate `list_models()` across providers.
- `GenericAdapter` improvements: optional API-key support, additional provider configs (Ollama base URL override, HuggingFace models endpoint, Azure OpenAI config).

### Changed
- Multiple adapters (OpenAI, Gemini, Generic, Cohere, Mistral) migrated to use `DynHttpTransportRef`.
- Bedrock: implementation deferred — removed from public exports and deleted from repo to avoid unused warnings (re-introduce when SigV4/AWS SDK integration is implemented).

### Fixed
- Resolved trait object-safety issues and several compile-time errors surfaced during migration.
- Added `bytes` dependency for stream handling and fixed missing imports.
- Fixed non-exhaustive match in `AiClient::switch_provider` (added AzureOpenAI mapping).

---

## Migration notes (for integrators)

- If you inject custom transports for testing, use adapter constructors that accept `DynHttpTransportRef`, for example:

```rust
let transport: DynHttpTransportRef = my_test_transport.into();
let adapter = GenericAdapter::with_transport_ref(config, transport)?;
```

- Bedrock support is intentionally deferred. If you previously expected a public `BedrockAdapter`, it has been removed; re-add when you implement SigV4 signing or integrate the AWS SDK.

---

## Quick upgrade guide

1. Update your Cargo dependency:

```toml
ai-lib = "0.1.0"
```

2. If you used transports directly, update calls to use `DynHttpTransportRef` where applicable and prefer the `with_transport_ref` constructors for tests.

3. Run the examples and smoke tests:

```powershell
cargo check -q --all-targets
cargo run --example list_models_smoke
```

4. Verify the environment variables for providers you use (API keys and optional base URLs):

- `AI_PROXY_URL` (optional) — sets HTTP/HTTPS proxy for all requests
- `COHERE_API_KEY`, `COHERE_BASE_URL` (optional)
- `MISTRAL_API_KEY`, `MISTRAL_BASE_URL` (optional)
- `OPENAI_API_KEY`, `AZURE_OPENAI_API_KEY`, `GROQ_API_KEY`, etc.

---

## Suggested GitHub release notes (short)

v0.1.0 — 2025-08-26

- New: object-safe transport abstraction for DI/testing (DynHttpTransport)
- New: Cohere adapter with streaming and fallback
- New: Mistral HTTP adapter with streaming
- Improved: GenericAdapter config expansion and optional API key support
- Fixed: multiple compile and import issues discovered during migration
- Note: AWS Bedrock deferred and removed from public API

---

## Suggested GitHub release notes (long)

This release advances the ai-lib architecture toward a stable, testable 0.1.0 by introducing an object-safe transport abstraction and adding new adapters. The `DynHttpTransport` trait and boxed shim allow adapters to receive runtime transport implementations (handy for tests and SDK integrations).

New adapters include a Cohere implementation with SSE streaming and a conservative Mistral HTTP adapter. The `GenericAdapter` gained several provider configurations and improved handling of optional API keys.

As part of cleanup and to focus scope for 0.1.0, the Bedrock adapter integration (which requires SigV4 signing or the AWS SDK) was deferred and removed from the public API. This avoids exposing a half-implemented provider while keeping the rest of the API stable.

---

## QA checklist before publishing

- [ ] Run `cargo check --all-targets` (should pass)
- [ ] Run `cargo test` (where applicable)
- [ ] Run smoke example: `cargo run --example list_models_smoke` (configure API keys as needed)
- [ ] Update README / docs if you want the release notes visible there
- [ ] Tag `v0.1.0` and push a changelog + release

---

## Post-release follow-ups

- Add unit/integration tests for the `DynHttpTransport` injection patterns
- Add explicit examples showing `with_transport_ref(...)` usage for testing
- Re-evaluate Bedrock integration approach (AWS SDK vs manual SigV4) and plan a separate PR

---

If you'd like, I can:
- Prepare a PR that updates `Cargo.toml` and `readme.md` to reference `0.1.0` and opens the GitHub release draft, or
- Create a ready-to-paste GitHub Release body formatted from the "Suggested GitHub release notes" above.

Tell me which next step you prefer and I'll do it.
