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
//! # 代理服务器支持
//!
//! AI-lib 支持通过环境变量配置代理服务器：
//!
//! ```bash
//! # 设置代理服务器
//! export AI_PROXY_URL=http://proxy.example.com:8080
//!
//! # 带认证的代理
//! export AI_PROXY_URL=http://username:password@proxy.example.com:8080
//!
//! # HTTPS代理
//! export AI_PROXY_URL=https://proxy.example.com:8080
//! ```
//!
//! 设置后，所有AI提供商的请求都会自动通过指定的代理服务器。

pub mod api;
pub mod client;
pub mod metrics;
pub mod provider;
pub mod transport;
pub mod types;
pub mod utils;

// 重新导出主要类型，方便用户使用
pub use api::ChatApi;
pub use client::{AiClient, Provider};
pub use types::{
    AiLibError, ChatCompletionRequest, ChatCompletionResponse, Choice, Message, Role, Usage,
};
// Convenience re-exports: make the most-used types available from the crate root so
// users don't need deep imports for common flows.
pub use api::ChatCompletionChunk;
pub use types::common::Content;
pub use client::CancelHandle;
pub use transport::{DynHttpTransport, DynHttpTransportRef, HttpTransport, HttpClient, TransportError};
pub use metrics::{Metrics, NoopMetrics, Timer, NoopTimer, MetricsExt};

// 重新导出配置相关类型
pub use provider::config::{ProviderConfig, FieldMapping};
pub use provider::configs::ProviderConfigs;

    // 重新导出批处理功能
    pub use api::chat::{BatchResult, batch_utils};

// 重新导出增强的文件工具
pub use utils::file::{
    save_temp_file, read_file, remove_file, guess_mime_from_path,
    validate_file, get_file_size, create_temp_dir,
    is_image_file, is_audio_file, is_video_file, is_text_file,
    get_file_extension, is_file_size_acceptable
};
