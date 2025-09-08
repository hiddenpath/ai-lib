# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.21] - 2025-09-07 - Major Architecture Improvements & Performance Enhancement
- Added feature-gated modules and docs:
  - `interceptors`: Interceptor trait + pipeline (example provided)
  - `unified_sse`: Common SSE parser and tests; `GenericAdapter` wired under flag
  - `unified_transport`: Shared reqwest client factory
  - `cost_metrics`: Env-driven minimal cost accounting (COST_INPUT_PER_1K, COST_OUTPUT_PER_1K)
  - `routing_mvp`: Basic `ModelArray` routing via special model "__route__"
  - `observability`: Tracer/AuditSink traits (Noop implementations)
  - `config_hot_reload`: ConfigProvider/Watcher traits (Noop implementations)
- Standardized metric keys via `metrics::keys`
- Function calling mapping extended in `GenericAdapter` and `MistralAdapter`
- Docs updated (README/README_CN/rustdoc) with env vars, features, and PRO notes
  - Added Feature Flags section; noted deprecation path for adapter-local SSE helpers in favor of unified parser

### 🚀 Major Changes
- **Direct HTTP Client Implementation**: All adapters now use direct `reqwest::Client` instead of intermediate abstraction layers
- **Performance Boost**: Eliminated double serialization overhead and simplified HTTP call paths
- **Enhanced Reliability**: Unified error handling and better HTTP status code processing across all adapters

### ✅ Improved Adapters
- **Independent Adapters**: OpenAI, Mistral, Cohere, Gemini - all refactored to direct reqwest implementation
- **Config-driven Adapters**: GenericAdapter improved, benefiting Groq, DeepSeek, and all OpenAI-compatible providers
- **Unified Proxy Support**: Consistent `AI_PROXY_URL` environment variable support across all adapters

### 🔧 Technical Improvements
- **Architecture Simplification**: `Adapter → HttpTransport → execute_request → reqwest` → `Adapter → reqwest`
- **Better Serialization**: Using `reqwest::Client::json()` method for reliable JSON serialization
- **Enterprise Features Preserved**: All metrics, monitoring, auditing, and traffic management features maintained
- **Backward Compatibility**: No breaking changes to public APIs

### 🧹 Code Quality
- **Warning Cleanup**: Resolved all compiler warnings with proper `#[allow(dead_code)]` annotations
- **Documentation**: Updated all Chinese comments and rustdoc to English
- **Testing**: Core functionality verified across all major providers

### 📊 Impact
- **Performance**: Faster HTTP request processing through reduced abstraction layers
- **Reliability**: More robust error handling and HTTP response processing
- **Maintainability**: Cleaner codebase with simplified architecture
- **Compatibility**: Drop-in replacement with no migration required

## [0.2.20] - 2025-09-05 - Resilience & Error Handling Enhancement + Reasoning Models Support

### Added
- **Circuit Breaker Implementation**: Complete circuit breaker pattern with state management (Closed, Open, HalfOpen)
- **Rate Limiting with Token Bucket**: Advanced rate limiting with adaptive capabilities and burst handling
- **Enhanced Error Handling**: Intelligent error classification, pattern analysis, and recovery suggestions
- **Resilience Configuration**: Comprehensive configuration system with production, development, and conservative presets
- **Metrics Integration**: Full metrics collection for circuit breaker, rate limiter, and error handling
- **Backpressure Control**: Semaphore-based backpressure management for concurrent request control
- **Error Pattern Analysis**: Automatic error pattern detection and frequency analysis
- **Smart Recovery Strategies**: Context-aware error recovery with suggested actions
- **Builder Pattern Enhancements**: New builder methods for resilience configuration (`with_smart_defaults`, `for_production`, `for_development`)
- **System-Level Provider Classification**: New `ProviderClassification` trait for unified provider behavior management
- **Provider Classification Constants**: System-level constants (`CONFIG_DRIVEN_PROVIDERS`, `INDEPENDENT_PROVIDERS`, `ALL_PROVIDERS`) for single source of truth
- **Unified Configuration Management**: Centralized provider configuration mapping through trait methods
- **Provider Classification Module**: New `src/provider/classification.rs` module with comprehensive provider behavior definitions
- **Reasoning Models Support**: Comprehensive support for reasoning models through existing API capabilities
  - **Best Practices Examples**: Complete examples demonstrating structured, streaming, and JSON format reasoning
  - **Reasoning Utils Library**: Helper functions and assistant classes for reasoning model interactions
  - **Provider-Specific Configuration**: Escape hatch mechanism for passing vendor-specific parameters
  - **Multi-Format Support**: Support for structured, streaming, JSON, and step-by-step reasoning formats
  - **Documentation**: Detailed guide for reasoning model integration and best practices

### Enhanced
- **AiClientBuilder**: Added resilience configuration support with progressive complexity API
- **Error Classification**: Extended error types with detailed categorization (RateLimit, Network, Authentication, Provider, Timeout, Configuration, Validation, Serialization, Deserialization, FileOperation, ModelNotFound, ContextLengthExceeded, UnsupportedFeature)
- **Transport Error Handling**: Improved error serialization and Clone trait support
- **Configuration Management**: Added `ResilienceConfig`, `BackpressureConfig`, and `ErrorHandlingConfig` structures
- **Provider Enum**: Added `PartialEq` trait to `Provider` enum for classification support
- **Code Organization**: Eliminated duplicate provider classification logic across multiple modules
- **Type Safety**: Compile-time provider classification with automatic adapter type detection
- **ChatCompletionRequest**: Added `with_functions()`, `with_function_call()`, and `with_provider_specific()` methods for enhanced functionality

### Changed
- **Provider Classification Logic**: Replaced hardcoded `matches!` statements with trait-based classification
- **Configuration Retrieval**: Simplified provider configuration access through unified trait methods
- **Adapter Creation**: Streamlined adapter creation logic using provider classification

### Technical Details
- **Circuit Breaker**: Configurable failure thresholds, recovery timeouts, and success thresholds
- **Rate Limiter**: Token bucket algorithm with adaptive rate adjustment and burst capacity
- **Error Recovery**: Pattern-based error analysis with intelligent suggestion generation
- **Metrics**: Comprehensive monitoring with success rates, failure rates, and operational metrics
- **Thread Safety**: All components use atomic operations and proper synchronization
- **Async Support**: Full async/await support throughout all resilience features
- **Reasoning Models**: Support for Groq's qwen-qwq-32b, deepseek-r1-distill-llama-70b, and other reasoning models

### API Changes
- **New Modules**: `circuit_breaker`, `rate_limiter`, `error_handling`
- **New Types**: `CircuitBreaker`, `TokenBucket`, `ErrorRecoveryManager`, `ErrorContext`, `SuggestedAction`
- **New Configuration**: `ResilienceConfig`, `CircuitBreakerConfig`, `RateLimiterConfig`, `ErrorThresholds`
- **New Builder Methods**: `with_smart_defaults()`, `for_production()`, `for_development()`, `with_resilience_config()`
- **New Request Methods**: `with_functions()`, `with_function_call()`, `with_provider_specific()`

### Breaking Changes
- None - All new features are additive and backward compatible

### Dependencies
- Added `chrono` with `serde` feature for timestamp serialization
- Enhanced existing dependencies with no new external dependencies

### Examples
- **Resilience Example**: `cargo run --example resilience_example` - Complete resilience features demonstration
- **Reasoning Best Practices**: `cargo run --example reasoning_best_practices` - Reasoning models integration examples
- **Reasoning Utils**: `cargo run --example reasoning_utils` - Reasoning utilities and helper functions

### Reasoning Models Support

ai-lib now provides comprehensive support for reasoning models through existing API capabilities:

#### Supported Models
- **Groq**: qwen-qwq-32b, deepseek-r1-distill-llama-70b, openai/gpt-oss-20b, openai/gpt-oss-120b
- **Other Providers**: Any provider supporting reasoning capabilities through function calling

#### Usage Patterns
1. **Structured Reasoning**: Use function calls for step-by-step reasoning with structured output
2. **Streaming Reasoning**: Observe real-time reasoning process through streaming responses
3. **JSON Format Reasoning**: Get structured reasoning results in JSON format
4. **Provider-Specific Configuration**: Use escape hatch for vendor-specific parameters

#### Example Usage
```rust
// Structured reasoning with function calls
let reasoning_tool = Tool::new_json(
    "step_by_step_reasoning",
    Some("Execute step-by-step reasoning"),
    serde_json::json!({
        "type": "object",
        "properties": {
            "problem": {"type": "string"},
            "steps": {"type": "array", "items": {"type": "object"}},
            "final_answer": {"type": "string"}
        }
    })
);

let request = ChatCompletionRequest::new(model, messages)
    .with_functions(vec![reasoning_tool])
    .with_function_call(FunctionCallPolicy::Auto);

// Provider-specific reasoning configuration
let request = ChatCompletionRequest::new(model, messages)
    .with_provider_specific("reasoning_format", serde_json::Value::String("parsed".to_string()))
    .with_provider_specific("reasoning_effort", serde_json::Value::String("high".to_string()));
```

### Developer Experience
- **Simplified Provider Management**: Single location for adding new providers and their classifications
- **Reduced Code Duplication**: Eliminated ~50 lines of repetitive provider classification code
- **Better Maintainability**: Centralized provider behavior definitions reduce maintenance overhead

### How to Add New Providers

#### For Config-Driven Providers (using GenericAdapter):
1. Add provider to `Provider` enum in `src/client.rs`
2. Add provider to `CONFIG_DRIVEN_PROVIDERS` array in `src/provider/classification.rs`
3. Add configuration method to `ProviderConfigs` in `src/provider/configs.rs`
4. Add configuration mapping in `ProviderClassification::get_default_config()` implementation

#### For Independent Providers (using dedicated adapters):
1. Add provider to `Provider` enum in `src/client.rs`
2. Add provider to `INDEPENDENT_PROVIDERS` array in `src/provider/classification.rs`
3. Create dedicated adapter implementation
4. Add adapter creation logic in `AiClientBuilder::build()` method

#### Example - Adding a New Config-Driven Provider:
```rust
// 1. Add to Provider enum
pub enum Provider {
    // ... existing providers
    NewProvider,  // Add here
}

// 2. Add to CONFIG_DRIVEN_PROVIDERS array
pub const CONFIG_DRIVEN_PROVIDERS: &[Provider] = &[
    // ... existing providers
    Provider::NewProvider,  // Add here
];

// 3. Add configuration method
impl ProviderConfigs {
    pub fn new_provider() -> ProviderConfig {
        ProviderConfig::openai_compatible(
            "https://api.newprovider.com/v1",
            "NEW_PROVIDER_API_KEY",
            "new-model",
            None,
        )
    }
}

// 4. Add to trait implementation
fn get_default_config(&self) -> Result<ProviderConfig, AiLibError> {
    match self {
        // ... existing mappings
        Provider::NewProvider => Ok(ProviderConfigs::new_provider()),
        // ...
    }
}
```

### Benefits
- **Single Source of Truth**: All provider classifications defined in one place
- **Type Safety**: Compile-time validation of provider behavior
- **Extensibility**: Easy addition of new providers with automatic classification
- **Maintainability**: Reduced code duplication and centralized management

## [0.2.12] - 2025-04-15

### Added
- **System Configuration Management**: Comprehensive configuration system with environment variable support and explicit overrides
- **Configuration Validation Tools**: Built-in tools for validating configuration settings
- **Enhanced Documentation**: Complete rewrite of README with feature-focused organization
- **Context Control**: Advanced conversation management with context control features
- **File Upload & Multimodal Processing**: Automatic file handling with upload and inline support
- **Performance Optimizations**: Enterprise-grade performance with minimal overhead
- **Security & Privacy Features**: Built-in security features for enterprise environments

### Enhanced
- **README Documentation**: Restructured and enhanced both English and Chinese README files
- **Provider Support**: Updated provider information with detailed feature descriptions
- **Configuration Examples**: Added comprehensive configuration examples and best practices
- **Performance Documentation**: Added performance characteristics and optimization tips
- **Use Case Examples**: Added enterprise, research, production, and privacy-first use cases

### Changed
- **Documentation Structure**: Reorganized documentation to highlight key features and capabilities
- **Configuration Management**: Enhanced configuration management with validation tools
- **Performance Metrics**: Updated performance documentation with specific benchmarks

### Examples
- **Configuration Check**: `cargo run --example check_config` - Validate configuration settings
- **Network Diagnosis**: `cargo run --example network_diagnosis` - Troubleshoot connectivity
- **Proxy Testing**: `cargo run --example proxy_example` - Test proxy configuration
- **Explicit Config**: `cargo run --example explicit_config` - Runtime configuration

## [0.2.1] - 2025-01-20

### Added
- **Client Builder Pattern**: New `AiClientBuilder` for flexible client configuration with progressive customization levels
- **Automatic Environment Variable Detection**: Auto-detects `GROQ_BASE_URL`, `AI_PROXY_URL` and other provider-specific environment variables
- **Model Management Tools**: Comprehensive tools for building custom model managers and arrays
  - `CustomModelManager` with multiple selection strategies (Round-robin, Weighted, Performance-based, Cost-based)
  - `ModelArray` for load balancing and A/B testing with health checks
  - `ModelInfo`, `ModelCapabilities`, `PricingInfo`, and `PerformanceMetrics` structures
- **Enhanced Provider Configuration**: Each provider now specifies default chat and optional multimodal models
- **Progressive Customization**: Four levels of client configuration from simple to advanced

### Enhanced
- **ProviderConfig**: Added `chat_model` and `multimodal_model` fields for better model management
- **ProviderConfigs**: Updated all provider configurations with sensible default models
- **Documentation**: Comprehensive English documentation and examples for all new features
- **Examples**: Added new examples demonstrating builder pattern and model management tools

### Changed
- **Backward Compatibility**: All existing code continues to work without modification
- **Method Signatures**: Updated `ProviderConfig::openai_compatible()` to include model parameters
- **Configuration Validation**: Enhanced validation to include new model fields

### Examples
- **Quickstart**: `cargo run --example quickstart` - Simple usage guide
- **Builder Pattern**: `cargo run --example builder_pattern` - Complete builder pattern demonstration
- **Model Management**: `cargo run --example model_management` - Custom model managers and arrays

### Key Benefits
- **Developer Experience**: Faster time-to-first-AI-app with sensible defaults
- **Flexibility**: Easy customization for advanced use cases
- **Production Ready**: Load balancing, health checks, and sophisticated model selection
- **Extensible**: Build custom model managers for any provider or use case

## [0.2.0] - 2024-12-15

### Added
- Hybrid architecture with universal streaming support
- Enterprise-grade error handling, retry, and proxy support
- Multimodal primitives, function-calling, and metrics scaffold
- Transport injection and upload tests
- Support for multiple AI providers including Groq, OpenAI, Anthropic, and more

### Changed
- Improved error handling and retry mechanisms
- Enhanced streaming capabilities
- Better proxy support and configuration

## [0.1.0] - 2024-12-10

### Added
- Initial release of ai-lib
- Basic AI provider support
- Core chat completion functionality
- Streaming support
- Basic error handling

---

## Versioning

- **Major version (0.x.0)**: Breaking changes or major new features
- **Minor version (0.2.x)**: New features, enhancements, or improvements
- **Patch version (0.2.x)**: Bug fixes and minor improvements

## Migration Guide

### From 0.2.0 to 0.2.1

No breaking changes. All existing code continues to work:

```rust
// This still works exactly the same
let client = AiClient::new(Provider::Groq)?;

// New optional features available
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy("http://proxy.example.com:8080")
    .build()?;
```

### New Features Usage

```rust
// Custom model manager
let mut manager = CustomModelManager::new("groq")
    .with_strategy(ModelSelectionStrategy::PerformanceBased);

// Model array for load balancing
let mut array = ModelArray::new("production")
    .with_strategy(LoadBalancingStrategy::RoundRobin);
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE-MIT](LICENSE-MIT) file for details.