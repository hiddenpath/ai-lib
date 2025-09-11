# 0.3.0-rc.1 Release Steps (GitHub Pre-release)

## 1) Ensure green status locally
- `cargo clippy --all-features -- -D warnings`
- `cargo test`
- `cargo build --examples`
- Feature smoke tests:
  - `cargo test --features unified_sse --test sse_parser_tests`
  - `cargo test --features "cost_metrics routing_mvp" --test cost_and_routing`
  - `cargo run --features "interceptors unified_sse" --example deepseek_features`
  - `cargo run --features "interceptors unified_sse" --example mistral_features`

## 2) Tag & Pre-release
- Tag: `v0.3.0-rc.1`
- Title: `0.3.0-rc.1`
- Body: paste `docs/RELEASE_NOTES_0.3.0-rc.1.md`
- Mark as "Pre-release"

## 3) Communicate
- Share feature flags and quick validation matrix with early adopters.
- Clarify no crates.io publish for RC; final `0.3.0` will be published after full regression.

## 4) Next iteration
- Add bench/CI harness and upgrade guide.
- Close RC issues and cut `0.3.0` stable.
