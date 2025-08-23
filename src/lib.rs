//! AI-lib: Rust统一AI SDK库，提供多个AI提供商的统一接口
//! 
//! AI-lib: A Unified AI SDK for Rust
//! 
//! This library provides a single, consistent interface for interacting with multiple AI model providers.
//! 
//! # Quick Start
//! 
//! ```rust
//! use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Switch model providers by just changing the Provider value
//!     let client = AiClient::new(Provider::Groq)?;
//!     
//!     let request = ChatCompletionRequest::new(
//!         "llama3-8b-8192".to_string(),
//!         vec![Message {
//!             role: Role::User,
//!             content: "Hello, how are you?".to_string(),
//!         }],
//!     );
//!     
//!     // Note: GROQ_API_KEY environment variable must be set to actually call the API
//!     // let response = client.chat_completion(request).await?;
//!     // println!("Response: {}", response.choices[0].message.content);
//!     
//!     println!("Client created successfully with provider: {:?}", client.current_provider());
//!     println!("Request prepared for model: {}", request.model);
//!     
//!     Ok(())
//! }
//! ```
//! 
//! # Proxy Server Support
//! 
//! AI-lib supports proxy server configuration via environment variables:
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
//! Once set, all AI provider requests will automatically go through the specified proxy server.

pub mod api;
pub mod types;
pub mod provider;
pub mod client;
pub mod transport;

// 重新导出主要类型，方便用户使用
pub use api::ChatApi;
pub use types::{ChatCompletionRequest, ChatCompletionResponse, Message, Role, Choice, Usage, AiLibError};
pub use client::{AiClient, Provider};

