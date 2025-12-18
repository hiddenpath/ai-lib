# ADR-002: Feature Flags Strategy

## Status
**Accepted** - 2024-12

## Context
ai-lib aims to be a lean OSS library while supporting advanced features. We need:
- Minimal default binary size
- Optional advanced capabilities
- Clear upgrade path to ai-lib-pro
- No breaking changes when enabling features

## Decision
Implement **progressive complexity** via Cargo feature flags:

### Core Features (Always Available)
- Basic chat completion
- Provider abstraction
- Error handling
- Streaming support

### Optional Features (Feature-Gated)
- `interceptors` - Retry, timeout, circuit breaker
- `unified_sse` - Common SSE parser
- `unified_transport` - Shared HTTP client
- `cost_metrics` - Basic cost tracking
- `routing_mvp` - Model selection strategies
- `observability` - Tracing interfaces
- `config_hot_reload` - Dynamic configuration

### Convenience Bundles
- `resilience` → `interceptors`
- `streaming` → `unified_sse`
- `all` → All OSS features

## Consequences

### Positive
- **Lean Defaults**: Minimal dependencies for simple use cases
- **Pay for What You Use**: Only compile needed features
- **Clear Boundaries**: OSS vs PRO feature separation
- **Backward Compatible**: Features are additive only

### Negative
- **Testing Complexity**: Must test feature combinations
- **Documentation Overhead**: Feature-specific docs needed
- **Compilation Time**: More feature combinations to build

### Metrics
- Default binary: ~2MB
- With `all` features: ~3.5MB
- Compilation time increase: ~15%

## Implementation
```toml
# Cargo.toml
[features]
default = []
interceptors = []
unified_sse = []
all = ["interceptors", "unified_sse", ...]

# Usage
[dependencies]
ai-lib = { version = "0.3", features = ["interceptors"] }
```

```rust
// Code gating
#[cfg(feature = "interceptors")]
pub mod interceptors;
```

## Guidelines

### When to Add a Feature Flag
- Feature adds significant dependencies
- Feature is optional for most users
- Feature has performance implications
- Feature is experimental

### When NOT to Use Feature Flags
- Core functionality
- Bug fixes
- Performance improvements
- API refinements

## Alternatives Considered
1. **No Feature Flags**: Bloated binary for simple use cases
2. **Separate Crates**: Fragmented ecosystem
3. **Runtime Configuration**: No compile-time optimization

## Migration Path
Users can gradually enable features:
```toml
# Start simple
ai-lib = "0.3"

# Add resilience
ai-lib = { version = "0.3", features = ["resilience"] }

# Full OSS
ai-lib = { version = "0.3", features = ["all"] }

# Upgrade to PRO
ai-lib-pro = "0.1"
```

## References
- `Cargo.toml` - Feature definitions
- `docs/UPGRADE_1.0.0.md` - Migration guide
- `README.md` - Feature documentation
