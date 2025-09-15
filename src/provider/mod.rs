pub mod ai21;
pub mod classification;
pub mod cohere;
pub mod config;
pub mod configs;
pub mod gemini;
pub mod generic;
pub mod mistral;
pub mod perplexity;
#[cfg(feature = "routing_mvp")]
pub mod models;
#[cfg(feature = "routing_mvp")]
pub use models::*;
pub mod openai;
pub mod pricing;
pub(crate) mod utils;

#[cfg_attr(docsrs, doc(cfg(feature = "routing_mvp")))]
pub use configs::ProviderConfigs;
#[doc(hidden)]
pub use ai21::AI21Adapter;
#[doc(hidden)]
pub use cohere::CohereAdapter;
#[doc(hidden)]
pub use gemini::GeminiAdapter;
#[doc(hidden)]
pub use generic::GenericAdapter;
#[doc(hidden)]
pub use mistral::MistralAdapter;
#[doc(hidden)]
pub use openai::OpenAiAdapter;
#[doc(hidden)]
pub use perplexity::PerplexityAdapter;
// Keep adapters internal to reduce public surface in OSS. Selection happens via `Provider` + `AiClient`.
// Internal re-exports (if any) stay private; public API should avoid exposing provider modules directly.
