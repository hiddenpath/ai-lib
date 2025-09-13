# Performance Methodology and Interpretation

> INTERNAL DRAFT — NOT FOR PUBLICATION

This document outlines how we measure performance and how to interpret results. It is intended to keep benchmarks repeatable and transparent. Figures are indicative, not guarantees.

## Principles

- Reproducibility: pin toolchains, dependencies, datasets, and environment.
- Representativeness: test realistic workloads (chat, streaming, concurrency, retries).
- Transparency: publish scripts and configs; annotate caveats and sources of variance.
- Safety: do not log secrets; redact sensitive payloads.

## Environment

- Hardware: CPU model, cores, memory, storage type, network topology.
- OS/Kernel: version, scheduler settings if relevant.
- Rust: toolchain version, target, optimization flags.
- Network: provider region, latency/bandwidth baselines.

Record all of the above with the results.

## Workloads

- Single-request latency (cold/warm) for chat completion
- Streaming time-to-first-token (TTFT) and steady-state token rate
- Concurrency scaling (p50/p95/p99 latency vs requests-per-second)
- Retry/backoff impact under induced transient errors
- Proxy/base_url overhead (direct vs gateway)

## Metrics

- Latency: p50/p95/p99 (end-to-end, client-observed)
- Throughput: requests/sec at bounded error rates
- Resource: memory, CPU utilization
- Error rates: transport errors, timeouts, rate limits

## Procedure

1. Warm-up: discard N initial requests to stabilize caches and pools
2. Fixed-time or fixed-iterations runs with randomized message sets
3. Repeat runs (>=3) and report median and IQR; include min/max for context
4. Isolate variables: change one factor at a time (e.g., concurrency level)
5. Record environment snapshot and commit hash of the code under test

## Interpretation

- Treat numbers as indicative under the stated conditions, not universally applicable.
- Compare deltas, not absolutes, when the environment differs.
- Highlight trade-offs (e.g., gateway adds a hop but centralizes policy/keys).

## Reporting format (example)

- Scenario: Streaming TTFT, Provider=X, Model=Y, Concurrency=Z
- Env: CPU/Memory/OS/Net/Region, Rust toolchain, ai-lib commit
- Results: TTFT p50/p95/p99, steady token rate, error rate
- Notes: variance sources, retries, induced failures

## References

- Helicone AI Gateway README (benchmark methodology positioning) `https://github.com/Helicone/ai-gateway`
