use crate::types::AiLibError;
use bytes::Bytes;
use futures::future::BoxFuture;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

// Type aliases to reduce complexity in trait signatures
pub type BytesStream = Pin<Box<dyn Stream<Item = Result<Bytes, AiLibError>> + Send>>;
pub type StreamFuture<'a> = BoxFuture<'a, Result<BytesStream, AiLibError>>;

/// Object-safe transport abstraction for dynamic usage in adapters.
/// Methods use serde_json::Value and bytes stream to remain object-safe.
pub trait DynHttpTransport: Send + Sync {
    fn get_json<'a>(
        &'a self,
        url: &'a str,
        headers: Option<HashMap<String, String>>,
    ) -> BoxFuture<'a, Result<serde_json::Value, AiLibError>>;
    fn post_json<'a>(
        &'a self,
        url: &'a str,
        headers: Option<HashMap<String, String>>,
        body: serde_json::Value,
    ) -> BoxFuture<'a, Result<serde_json::Value, AiLibError>>;
    fn post_stream<'a>(
        &'a self,
        url: &'a str,
        headers: Option<HashMap<String, String>>,
        body: serde_json::Value,
    ) -> StreamFuture<'a>;
    /// Upload a single file via multipart/form-data. Implementations should post a multipart
    /// form with a single file field and return the parsed JSON body as serde_json::Value.
    fn upload_multipart<'a>(
        &'a self,
        url: &'a str,
        headers: Option<HashMap<String, String>>,
        field_name: &'a str,
        file_name: &'a str,
        bytes: Vec<u8>,
    ) -> BoxFuture<'a, Result<serde_json::Value, AiLibError>>;
}

pub type DynHttpTransportRef = Arc<dyn DynHttpTransport + Send + Sync>;
