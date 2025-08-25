use crate::types::AiLibError;
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;
use std::collections::HashMap;
use std::sync::Arc;

/// Object-safe transport abstraction for dynamic usage in adapters.
/// Methods use serde_json::Value and bytes stream to remain object-safe.
pub trait DynHttpTransport: Send + Sync {
    fn get_json<'a>(&'a self, url: &'a str, headers: Option<HashMap<String, String>>) -> Pin<Box<dyn futures::Future<Output = Result<serde_json::Value, AiLibError>> + Send + 'a>>;
    fn post_json<'a>(&'a self, url: &'a str, headers: Option<HashMap<String, String>>, body: serde_json::Value) -> Pin<Box<dyn futures::Future<Output = Result<serde_json::Value, AiLibError>> + Send + 'a>>;
    fn post_stream<'a>(&'a self, url: &'a str, headers: Option<HashMap<String, String>>, body: serde_json::Value) -> Pin<Box<dyn futures::Future<Output = Result<Pin<Box<dyn Stream<Item = Result<Bytes, AiLibError>> + Send>>, AiLibError>> + Send + 'a>>;
}

pub type DynHttpTransportRef = Arc<dyn DynHttpTransport + Send + Sync>;
