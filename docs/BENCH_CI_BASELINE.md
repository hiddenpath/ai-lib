# Bench & CI Baseline (Draft)

## Scope
- Micro: request build/serialize overhead, SSE parse cost.
- Macro (mock): high-concurrency throughput with mock transport.

## Initial Commands (to be implemented)
- Micro (to be added in `/bench/micro`):
  - `cargo bench --bench micro_overhead`
  - `cargo bench --bench stream_parse`
- Macro (to be added in `/bench/macro`):
  - `cargo run --example bench_mock_throughput -- --concurrency 512 --duration 15s`

## Reporting
- Capture JSON summaries and attach to CI artifacts for trend tracking.
- Maintain indicative ranges in README with methodology disclaimer.

## Next
- Add Criterion-based micro benches; add a mock throughput example.
- Extend CI: nightly workflow to compare regressions.
