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
//!     // 切换模型提供商，只需更改 Provider 的值
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
//!     // 注意：这里需要设置GROQ_API_KEY环境变量才能实际调用API
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
pub mod types;
pub mod provider;
pub mod client;
pub mod transport;

// 重新导出主要类型，方便用户使用
pub use api::ChatApi;
pub use types::{ChatCompletionRequest, ChatCompletionResponse, Message, Role, Choice, Usage, AiLibError};
pub use client::{AiClient, Provider};
