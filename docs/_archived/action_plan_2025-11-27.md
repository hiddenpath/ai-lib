---
title: ai-lib Phase 4 Action Plan (2025-11-27)
author: Project Lead
---

# Action Plan Overview

Goal: complete the Trait Shift and provider extensibility roadmap so ai-lib truly delivers “one unified interface for any AI model,” whether prebuilt or user-defined.

## 1. Trait Shift Execution

- Introduce a `ChatProvider` alias (or rename) that supersedes `ChatApi`. 
- Refactor `AiClient` to hold `Box<dyn ChatProvider>` rather than a `Provider` enum + adapter; `Provider` becomes a factory helper only.
- Ensure all request/stream/batch paths interact with trait objects; drop enum-based branching outside the factory.

## 2. Custom Provider Injection UX

- Document and showcase `AdapterProvider::new` + `AiClientBuilder::with_strategy` for user-supplied providers.
- Consider a `CustomProviderBuilder` helper that accepts base URL, API key, headers, etc., so consumers can plug in OpenAI-compatible endpoints without editing the codebase.

## 3. Routing & Failover Rework

- Remove `__route__` sentinel logic from request/stream modules; delegate routing decisions to explicit strategy providers.
- Wire `RoutingStrategyBuilder`, `FailoverProvider`, and `RoundRobinProvider` into `AiClientBuilder`, so strategy composition happens before runtime.
- Relocate `health_check` utilities into the strategies module (or a dedicated health submodule) and ensure they are feature-gated and unit tested.

## 4. Feature Completeness & Dead Code Cleanup

- Audit unused adapters (e.g., `bedrock.rs`) and either remove them or move to the pro codebase.
- Resolve outstanding implementation-plan items: `extensions`/`provider_specific`, removal of deprecated aliases, etc.
- Add focused tests for `provider::utils` (upload helpers) and `ProviderFactory` to prevent regressions during the Trait Shift.

## 5. Observability & Documentation

- Emit structured metrics/logs using `error_code_with_severity()` (e.g., `failover.error_code`).
- Update README and `docs/ADDING_PROVIDERS.md` to highlight trait-based extensibility and per-provider builders (`GroqBuilder`, `OpenAiBuilder`, etc.).
- Provide end-to-end examples showing strategy composition plus custom provider injection, so contributors can follow the Trait Shift workflow.

---

Next session should begin by tackling Section 1 to unblock later steps. Subsequent updates can reference this document to keep context lightweight.

