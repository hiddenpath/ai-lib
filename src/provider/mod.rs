//! 提供商适配器模块，实现对不同AI厂商API的统一适配
//!
//! Provider adapter module implementing unified adapters for different AI provider APIs.
//!
//! This module contains adapters for all supported AI providers, implementing the
//! `ChatProvider` trait for consistent interface across different vendor APIs.
//! Most providers use the `GenericAdapter` for OpenAI-compatible endpoints,
//! while unique APIs have dedicated adapter implementations.

pub mod ai21;
pub mod builders;
pub mod chat_provider;
pub mod classification;
pub mod cohere;
pub mod config;
pub mod configs;
pub mod gemini;
pub mod generic;
pub mod mistral;
#[cfg(feature = "routing_mvp")]
pub mod models;
pub mod perplexity;
pub mod strategies;
#[cfg(feature = "routing_mvp")]
pub use models::*;
pub mod openai;
pub mod pricing;
pub(crate) mod utils;

#[doc(hidden)]
pub use ai21::AI21Adapter;
#[doc(hidden)]
pub use cohere::CohereAdapter;
#[cfg_attr(docsrs, doc(cfg(feature = "routing_mvp")))]
pub use configs::ProviderConfigs;
#[doc(hidden)]
pub use gemini::GeminiAdapter;
#[doc(hidden)]
#[allow(deprecated)]
pub use generic::GenericAdapter;
#[doc(hidden)]
pub use mistral::MistralAdapter;
#[doc(hidden)]
pub use openai::OpenAiAdapter;
#[doc(hidden)]
pub use perplexity::PerplexityAdapter;

// Re-export common routing strategies at provider level for convenience
// Usage: `use ai_lib::provider::FailoverProvider;`
pub use strategies::FailoverProvider;
pub use strategies::RoundRobinProvider;
pub use strategies::RoutingStrategyBuilder;

// Keep adapters internal to reduce public surface in OSS. Selection happens via `Provider` + `AiClient`.
// Internal re-exports (if any) stay private; public API should avoid exposing provider modules directly.
