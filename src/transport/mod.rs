//! 传输层模块，提供统一的HTTP客户端和网络通信功能
//!
//! Transport layer module providing unified HTTP client and network communication.
//!
//! This module abstracts HTTP communication details from provider adapters,
//! providing configurable connection pooling, proxy support, and error handling.

#[cfg(feature = "unified_transport")]
pub mod client_factory;
pub mod dyn_transport;
pub mod error;
pub mod http;

pub use dyn_transport::{DynHttpTransport, DynHttpTransportRef};
pub use error::TransportError;
pub use http::HttpTransportConfig;
pub use http::{HttpClient, HttpTransport};
