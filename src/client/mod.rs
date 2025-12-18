//! 客户端模块入口点，提供统一的AI客户端接口和构建器
//!
//! Client module entry point.
//!
//! This module wires together the refactored client implementation that
//! previously lived in the monolithic `src/client.rs`.

mod batch;
mod builder;
mod client_impl;
mod helpers;
mod metadata;
mod model_options;
mod provider;
mod provider_factory;
mod request;
mod stream;

pub use builder::AiClientBuilder;
pub use client_impl::AiClient;
pub use model_options::ModelOptions;
pub use provider::Provider;
pub use stream::CancelHandle;

pub(crate) use provider_factory::ProviderFactory;
