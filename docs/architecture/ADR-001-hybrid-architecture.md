# ADR-001: Hybrid Architecture Design

## Status
**Accepted** - 2024-12

## Context
ai-lib needs to support 20+ AI providers with varying API formats. We need an architecture that:
- Minimizes code duplication
- Allows easy addition of new providers
- Maintains type safety and performance
- Supports both OpenAI-compatible and custom APIs

## Decision
Implement a **hybrid architecture** with two adapter types:

### 1. Config-Driven Adapters (GenericAdapter)
For OpenAI-compatible providers:
- Single `GenericAdapter` implementation
- Provider-specific behavior via `ProviderConfig`
- Covers: Groq, DeepSeek, Anthropic, Azure OpenAI, etc.

### 2. Independent Adapters
For providers with unique APIs:
- Dedicated adapter implementations
- Custom request/response handling
- Covers: Gemini, Mistral, Cohere, Perplexity, AI21

### Unified Interface
Both adapter types implement `ChatApi` trait, wrapped by `ChatProvider` for routing strategies.

## Consequences

### Positive
- **Reduced Code Duplication**: 15+ providers share GenericAdapter
- **Easy Provider Addition**: New OpenAI-compatible providers need only config
- **Type Safety**: Rust's type system enforces correctness
- **Flexibility**: Custom adapters for unique APIs

### Negative
- **Complexity**: Two code paths to maintain
- **Learning Curve**: Contributors must understand both patterns

### Trade-offs
- Chose flexibility over simplicity
- Prioritized maintainability over minimal abstraction

## Implementation
```rust
// Config-driven
let groq = GenericAdapter::new(ProviderConfigs::groq())?;

// Independent
let gemini = GeminiAdapter::new()?;

// Unified interface
let client = AiClient::new(Provider::Groq)?;
```

## Alternatives Considered
1. **Single Generic Adapter**: Too rigid for unique APIs
2. **All Independent Adapters**: Massive code duplication
3. **Plugin System**: Over-engineered for current needs

## References
- `src/provider/generic.rs` - GenericAdapter implementation
- `src/provider/gemini.rs` - Independent adapter example
- `src/provider/config.rs` - ProviderConfig structure
