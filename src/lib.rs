#![cfg_attr(docsrs, feature(doc_cfg))]
//! AI-lib: A Unified AI SDK for Rust
//!
//! This library provides a single, consistent interface for interacting with multiple AI model providers.
//!
//! # Quick Start
//!
//! ```rust
//! use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role, Content};
//! // Or in applications, prefer the minimal prelude:
//! // use ai_lib::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Switch provider by changing Provider:: value
//!     let client = AiClient::new(Provider::Groq)?;
//!
//!     let request = ChatCompletionRequest::new(
//!         "llama3-8b-8192".to_string(),
//!         vec![Message {
//!             role: Role::User,
//!             content: Content::Text("Hello, how are you?".to_string()),
//!             function_call: None,
//!         }],
//!     );
//!
//!     println!("Client created successfully with provider: {:?}", client.current_provider());
//!     println!("Request prepared for model: {}", request.model);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Import guidance
//! - For application code, prefer `use ai_lib::prelude::*;` to get the minimal common set.
//! - Library authors can import explicitly from domain modules for fine-grained control.
//!
//! See the module tree and import patterns guide for details: [docs/MODULE_TREE_AND_IMPORTS.md](docs/MODULE_TREE_AND_IMPORTS.md)
//!
//! # Proxy Support
//!
//! AI-lib supports proxy configuration via environment variables:
//!
//! ```bash
//! # Set proxy server
//! export AI_PROXY_URL=http://proxy.example.com:8080
//!
//! # Proxy with authentication
//! export AI_PROXY_URL=http://username:password@proxy.example.com:8080
//!
//! # HTTPS proxy
//! export AI_PROXY_URL=https://proxy.example.com:8080
//! ```
//!
//! All AI provider requests will automatically use the specified proxy server.
//!
//! Optional environment variables for feature-gated capabilities:
//! - cost_metrics feature:
//!   - `COST_INPUT_PER_1K`: USD per 1000 input tokens
//!   - `COST_OUTPUT_PER_1K`: USD per 1000 output tokens
//!
//! These cost metrics are read from environment variables for simplicity.
//! 
//! Note: In ai-lib-pro, these can be centrally configured and hot-reloaded
//! via external configuration providers for enterprise deployments.

pub mod api;
pub mod client;
pub mod config;
pub mod metrics;
pub mod provider;
pub mod transport;
pub mod types;
pub mod utils; // minimal explicit configuration entrypoint

// Feature-gated modules (OSS progressive complexity)
#[cfg(feature = "interceptors")]
pub mod interceptors;

#[cfg(feature = "unified_sse")]
pub mod sse;

#[cfg(feature = "unified_transport")]
pub mod net { pub use crate::transport::client_factory; }

#[cfg(feature = "observability")]
pub mod observability;

#[cfg(feature = "config_hot_reload")]
pub mod config_hot_reload;

// Resilience modules
pub mod circuit_breaker;
pub mod rate_limiter;
pub mod error_handling;

// Re-export main types for user convenience
pub use api::ChatApi;
pub use client::{AiClient, AiClientBuilder, ModelOptions, Provider};
pub use types::{
    AiLibError, ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Role,
    FunctionCall, FunctionCallPolicy, Tool,
};
// Convenience re-exports: make the most-used types available from the crate root so
// users don't need deep imports for common flows.
pub use api::ChatCompletionChunk;
pub use client::CancelHandle;
pub use metrics::{Metrics, MetricsExt, NoopMetrics, NoopTimer, Timer};
pub use transport::{
    DynHttpTransport, DynHttpTransportRef, HttpClient, HttpTransport, TransportError,
};
// Re-export minimal configuration type
pub use config::ConnectionOptions;
// Export response metadata (Usage/UsageStatus) and common Content from canonical modules
pub use types::response::{Usage, UsageStatus};
pub use types::common::Content;

// Re-export configuration types
pub use provider::config::{FieldMapping, ProviderConfig};
pub use provider::configs::ProviderConfigs;

// Re-export model management tools (feature-gated)
#[cfg_attr(docsrs, doc(cfg(feature = "routing_mvp")))]
#[cfg(feature = "routing_mvp")]
pub use provider::models::{
    CustomModelManager, LoadBalancingStrategy, ModelArray, ModelCapabilities, ModelEndpoint,
    ModelInfo, ModelSelectionStrategy, PerformanceMetrics, PricingInfo, QualityTier, SpeedTier,
};

// Re-export batch processing functionality
pub use api::chat::{batch_utils, BatchResult};

// Re-export enhanced file utilities
pub use utils::file::{
    create_temp_dir, get_file_extension, get_file_size, guess_mime_from_path, is_audio_file,
    is_file_size_acceptable, is_image_file, is_text_file, is_video_file, read_file, remove_file,
    save_temp_file, validate_file,
};


/// Prelude with the minimal commonly used items for applications.
///
/// Prefer `use ai_lib::prelude::*;` in application code for better ergonomics.
/// Library authors may prefer importing explicit items from their modules.
pub mod prelude {
    pub use crate::{AiClient, AiClientBuilder, Provider};
    pub use crate::types::{ChatCompletionRequest, ChatCompletionResponse, Choice};
    pub use crate::types::common::{Content, Message, Role};
    pub use crate::types::response::{Usage, UsageStatus};
    pub use crate::types::error::AiLibError;
}

// Module tree and import guidance:
// - Prefer the `prelude` for the minimal usable set in apps
// - Top-level re-exports expose the most common types
// - Provider-specific modules are internal in OSS; use `Provider` enum and `AiClient` to select providers