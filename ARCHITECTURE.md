# ai-lib Architecture

This document explains the layering, responsibilities, and extensibility model for ai-lib (OSS) and ai-lib-pro (PRO).

## 1. Goals and Non-goals

- Goals
  - Provide a single, ergonomic Rust API across heterogeneous AI providers.
  - Unify request models, streaming formats, function-calling semantics, and error shapes.
  - Offer reliability primitives (timeouts, retries, backpressure) with minimal ceremony.
  - Keep OSS small, composable, and dependency-light; enable PRO to layer advanced enterprise features.
- Non-goals
  - Vendor-specific full-surface mirroring (the SDK exposes a unified, not exhaustive surface).
  - Managing customer billing/accounts in OSS (belongs to PRO/enterprise systems).

## 2. High-level Modules (OSS)

- api: user-facing chat APIs and streaming interfaces.
- provider: provider adapters; two adapter styles:
  - Independent adapters (handwritten) for complex providers (e.g., OpenAI, Gemini).
  - Config-driven adapters for structurally similar providers (e.g., Groq, HuggingFace-like).
- sse: unified SSE/JSONL parser for normalized streaming deltas.
- transport: injectable HTTP client and streaming transport.
- types: request/response, function-calling, error categories, usage status.
- interceptors (feature): retry, timeout, circuit breaker, rate limiting (pipeline).
- circuit_breaker, rate_limiter: resilience primitives.
- metrics: trait-based hooks for observability integrations.
- config_hot_reload (feature): traits for config providers/watchers.
- observability (feature): pluggable tracing/audit interfaces.
- utils: file IO helpers and minimal shared utilities.

## 3. Provider Adapter Taxonomy

- Independent adapters
  - Use provider-specific payloads, headers, and error mapping when necessary.
  - Pros: exactness, full control; Cons: more code.
- Config-driven adapters
  - Declarative field mappings and conventions; unify similar HTTP shapes.
  - Pros: low maintenance; Cons: limited to compatible surfaces.

Adapters must conform to a common trait surface used by `AiClient`, enabling drop-in provider switching.

## 4. Reliability & Streaming

- Reliability
  - Timeouts, retries (with backoff), and error classification.
  - Rate limiting and backpressure (lightweight gates).
  - Circuit breaker to isolate failing upstreams.
- Streaming
  - Unified parser for SSE and JSONL; consistent `ChatCompletionChunk` deltas across providers.
  - Backpressure-aware streaming and cancellation via `CancelHandle`.

## 5. Configuration Model

- Progressive configuration:
  - Environment variables → Builder overrides → Explicit `ConnectionOptions` → Custom transport injection.
- Proxy and pool configuration are centralized with `unified_transport` feature.
- Hot-reload traits (feature) allow PRO or external systems to inject dynamic configuration.

## 6. OSS vs PRO Responsibilities

- OSS (ai-lib)
  - Unified API surface (chat/multimodal/functions).
  - Provider adapters (independent + config-driven).
  - Unified streaming (SSE/JSONL).
  - Reliability primitives (timeouts/retry/backpressure/circuit-breaker).
  - Minimal cost metrics (feature-gated), usage status unification.
  - Metrics trait and examples; no external dependencies required.
  - Security defaults (no sensitive content logging).
  - Packaging: clean crates.io publishing; excludes PRO modules by design.

- PRO (ai-lib-pro)
  - Advanced routing/orchestration (policy-driven, health, performance, cost).
  - Pricing catalog and budget guardrails; usage reconciliation/finalization.
  - Multi-tenant quotas and priority scheduling; adaptive concurrency.
  - Enterprise observability (structured logging, trace/exporters, audit & compliance).
  - Centralized configuration and hot reload with enterprise backends.
  - Enterprise security (key management, envelope encryption).
  - Seamless upgrade path with no breaking changes to OSS consumers.

## 7. Extension Points

- Transport: implement `HttpTransport` to customize HTTP stack/pooling/TLS.
- Metrics: implement `Metrics` to bridge Prometheus/Otel/StatsD.
- Interceptors: plug retry/timeout/rate-limit/breaker strategies.
- Provider adapters: add new providers via adapter traits (prefer config-driven first).
- Model management (feature-gated): `ModelArray`, custom selection strategies.

## 8. Versioning & MSRV

- Semantic versioning (SemVer). Public APIs are stable within major versions.
- MSRV policy:
  - OSS targets MSRV ≥ 1.70 today. MSRV bumps are not considered breaking changes.
  - MSRV increases will be announced in release notes; try to limit to ≤ 1 bump/year.
- Inspiration: adopt clear MSRV/semver posture similar to google-cloud-rust’s documented approach (see `google-cloud-rust`).

## 9. Publishing & Compliance

- ai-lib is published to crates.io with only OSS modules and feature flags.
- ai-lib-pro is not published to crates.io (private/proprietary); ensure `publish = false`.
- README clearly states provider-specific requirements are unified behind a consistent interface.


