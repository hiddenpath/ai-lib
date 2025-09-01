//! AI-lib: A Unified AI SDK for Rust
//!
//! This library provides a single, consistent interface for interacting with multiple AI model providers.
//!
//! # Quick Start
//!
//! ```rust
//! use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
//! use ai_lib::types::common::Content;
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

pub mod api;
pub mod client;
pub mod config;
pub mod metrics;
pub mod provider;
pub mod transport;
pub mod types;
pub mod utils; // minimal explicit configuration entrypoint

// Re-export main types for user convenience
pub use api::ChatApi;
pub use client::{AiClient, AiClientBuilder, Provider};
pub use types::{
    AiLibError, ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Role, Usage,
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
pub use types::common::Content;

// Re-export configuration types
pub use provider::config::{FieldMapping, ProviderConfig};
pub use provider::configs::ProviderConfigs;

// Re-export model management tools
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
