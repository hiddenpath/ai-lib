use super::error::TransportError;
use async_trait::async_trait;
use backoff::{future::retry, ExponentialBackoff};
use reqwest::{Client, Method, Proxy, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

/// HTTP transport client abstraction, defining common HTTP operation interface
///
/// HTTP transport client abstraction defining generic HTTP operation interface
///
/// This trait defines the generic interface for all HTTP operations,
/// allowing the adapter layer to avoid direct interaction with reqwest
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Send HTTP request
    async fn request<T, R>(
        &self,
        method: Method,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<&T>,
    ) -> Result<R, TransportError>
    where
        T: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de>;

    /// Send HTTP request with retry
    async fn request_with_retry<T, R>(
        &self,
        method: Method,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<&T>,
        _max_retries: u32,
    ) -> Result<R, TransportError>
    where
        T: Serialize + Send + Sync + Clone,
        R: for<'de> Deserialize<'de>;

    /// Send GET request
    async fn get<R>(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<R, TransportError>
    where
        R: for<'de> Deserialize<'de>,
    {
        self.request(Method::GET, url, headers, None::<&()>).await
    }

    /// Send POST request
    async fn post<T, R>(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: &T,
    ) -> Result<R, TransportError>
    where
        T: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de>,
    {
        self.request(Method::POST, url, headers, Some(body)).await
    }

    /// Send PUT request
    async fn put<T, R>(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: &T,
    ) -> Result<R, TransportError>
    where
        T: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de>,
    {
        self.request(Method::PUT, url, headers, Some(body)).await
    }
}

/// Reqwest-based HTTP transport implementation, encapsulating all HTTP details
///
/// HTTP transport implementation based on reqwest, encapsulating all HTTP details
///
/// This is the concrete implementation of the HttpClient trait, encapsulating all HTTP details
pub struct HttpTransport {
    client: Client,
    timeout: Duration,
}

/// Transport configuration for constructing a reqwest Client
pub struct HttpTransportConfig {
    pub timeout: Duration,
    pub proxy: Option<String>,
    /// Maximum idle connections per host (maps to reqwest::ClientBuilder::pool_max_idle_per_host)
    pub pool_max_idle_per_host: Option<usize>,
    /// Idle timeout for pooled connections (maps to reqwest::ClientBuilder::pool_idle_timeout)
    pub pool_idle_timeout: Option<Duration>,
}

impl HttpTransport {
    /// Create new HTTP transport instance
    ///
    /// Automatically detects AI_PROXY_URL environment variable for proxy configuration
    ///
    /// Note: This method will always check for AI_PROXY_URL environment variable.
    /// If you want to avoid automatic proxy detection, use `new_without_proxy()` instead.
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(30))
    }

    /// Create new HTTP transport instance without automatic proxy detection
    ///
    /// This method creates a transport instance without checking AI_PROXY_URL environment variable.
    /// Use this when you want explicit control over proxy configuration.
    pub fn new_without_proxy() -> Self {
        Self::with_timeout_without_proxy(Duration::from_secs(30))
    }

    /// Create HTTP transport instance with timeout
    ///
    /// Automatically detects AI_PROXY_URL environment variable for proxy configuration
    pub fn with_timeout(timeout: Duration) -> Self {
        let mut client_builder = Client::builder().timeout(timeout);

        // Check proxy configuration
        if let Ok(proxy_url) = env::var("AI_PROXY_URL") {
            match Proxy::all(&proxy_url) {
                Ok(proxy) => {
                    client_builder = client_builder.proxy(proxy);
                }
                Err(_) => {
                    // Silently ignore invalid proxy URL in production
                }
            }
        }

        let client = client_builder
            .build()
            .expect("Failed to create HTTP client");

        Self { client, timeout }
    }

    /// Create HTTP transport instance with timeout without automatic proxy detection
    ///
    /// This method creates a transport instance with timeout but without checking AI_PROXY_URL environment variable.
    pub fn with_timeout_without_proxy(timeout: Duration) -> Self {
        let client_builder = Client::builder().timeout(timeout);

        let client = client_builder
            .build()
            .expect("Failed to create HTTP client");

        Self { client, timeout }
    }

    /// Create an instance from an existing reqwest::Client (injected)
    pub fn with_client(client: Client, timeout: Duration) -> Self {
        Self { client, timeout }
    }

    /// Convenience alias that makes the intent explicit: create from a pre-built reqwest::Client.
    ///
    /// This is a small, descriptive wrapper around `with_client` that callers may find more
    /// discoverable when constructing transports from an external `reqwest::Client`.
    pub fn with_reqwest_client(client: Client, timeout: Duration) -> Self {
        Self::with_client(client, timeout)
    }

    /// Create instance using HttpTransportConfig
    pub fn new_with_config(config: HttpTransportConfig) -> Result<Self, TransportError> {
        let mut client_builder = Client::builder().timeout(config.timeout);

        // Apply pool tuning if provided
        if let Some(max_idle) = config.pool_max_idle_per_host {
            client_builder = client_builder.pool_max_idle_per_host(max_idle);
        }
        if let Some(idle_timeout) = config.pool_idle_timeout {
            client_builder = client_builder.pool_idle_timeout(idle_timeout);
        }

        if let Some(proxy_url) = config.proxy {
            if let Ok(proxy) = Proxy::all(&proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }

        let client = client_builder.build().map_err(|e| TransportError::HttpError(e.to_string()))?;
        Ok(Self {
            client,
            timeout: config.timeout,
        })
    }

    /// Create HTTP transport instance with custom proxy
    pub fn with_proxy(timeout: Duration, proxy_url: Option<&str>) -> Result<Self, TransportError> {
        let mut client_builder = Client::builder().timeout(timeout);

        if let Some(url) = proxy_url {
            let proxy = Proxy::all(url)
                .map_err(|e| TransportError::InvalidUrl(format!("Invalid proxy URL: {}", e)))?;
            client_builder = client_builder.proxy(proxy);
        }

        let client = client_builder.build().map_err(|e| TransportError::HttpError(e.to_string()))?;

        Ok(Self { client, timeout })
    }

    /// Get current timeout setting
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Execute actual HTTP request
    async fn execute_request<T, R>(
        &self,
        method: Method,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<&T>,
    ) -> Result<R, TransportError>
    where
        T: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de>,
    {
        let mut request_builder = self.client.request(method, url);

        // Add headers
        if let Some(headers) = headers {
            for (key, value) in headers {
                request_builder = request_builder.header(key, value);
            }
        }

        // Add JSON body using reqwest's json() for correct serialization and headers
        if let Some(body) = body {
            request_builder = request_builder.json(body);
        }

        // Send request
        let response = request_builder.send().await?;

        // Handle response
        Self::handle_response(response).await
    }

    /// Determine if error is retryable
    fn is_retryable_error(&self, error: &TransportError) -> bool {
        match error {
            TransportError::HttpError(err_msg) => {
                err_msg.contains("timeout") || err_msg.contains("connection")
            }
            TransportError::ClientError { status, .. } => {
                *status == 429 || *status == 502 || *status == 503 || *status == 504
            }
            TransportError::ServerError { .. } => true,
            _ => false,
        }
    }

    /// Handle HTTP response with unified error handling
    async fn handle_response<R>(response: Response) -> Result<R, TransportError>
    where
        R: for<'de> Deserialize<'de>,
    {
        let status = response.status();

        if status.is_success() {
            let json_text = response.text().await?;
            let result: R = serde_json::from_str(&json_text)?;
            Ok(result)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(TransportError::from_status(status.as_u16(), error_text))
        }
    }
}

#[async_trait]
impl HttpClient for HttpTransport {
    async fn request<T, R>(
        &self,
        method: Method,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<&T>,
    ) -> Result<R, TransportError>
    where
        T: Serialize + Send + Sync,
        R: for<'de> Deserialize<'de>,
    {
        self.execute_request(method, url, headers, body).await
    }

    async fn request_with_retry<T, R>(
        &self,
        method: Method,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<&T>,
        _max_retries: u32,
    ) -> Result<R, TransportError>
    where
        T: Serialize + Send + Sync + Clone,
        R: for<'de> Deserialize<'de>,
    {
        let backoff = ExponentialBackoff {
            max_elapsed_time: Some(Duration::from_secs(60)),
            max_interval: Duration::from_secs(10),
            ..Default::default()
        };

        let headers_clone = headers.clone();
        let body_clone = body.cloned();
        let url_clone = url.to_string();

        retry(backoff, || async {
            match self
                .execute_request(
                    method.clone(),
                    &url_clone,
                    headers_clone.clone(),
                    body_clone.as_ref(),
                )
                .await
            {
                Ok(result) => Ok(result),
                Err(e) => {
                    if self.is_retryable_error(&e) {
                        Err(backoff::Error::transient(e))
                    } else {
                        Err(backoff::Error::permanent(e))
                    }
                }
            }
        })
        .await
    }
}

impl Default for HttpTransport {
    fn default() -> Self {
        Self::new()
    }
}

// Object-safe wrapper implementation to allow dynamic dispatch for transports
pub struct HttpTransportBoxed {
    inner: HttpTransport,
}

impl HttpTransportBoxed {
    pub fn new(inner: HttpTransport) -> Self {
        Self { inner }
    }
}

use crate::transport::dyn_transport::{DynHttpTransport, DynHttpTransportRef};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;

impl DynHttpTransport for HttpTransportBoxed {
    fn get_json<'a>(
        &'a self,
        url: &'a str,
        headers: Option<HashMap<String, String>>,
    ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, crate::types::AiLibError>> {
        Box::pin(async move {
            let res: Result<serde_json::Value, TransportError> = self.inner.get(url, headers).await;
            match res {
                Ok(v) => Ok(v),
                Err(e) => Err(crate::types::AiLibError::ProviderError(format!(
                    "Transport error: {}",
                    e
                ))),
            }
        })
    }

    fn post_json<'a>(
        &'a self,
        url: &'a str,
        headers: Option<HashMap<String, String>>,
        body: serde_json::Value,
    ) -> futures::future::BoxFuture<'a, Result<serde_json::Value, crate::types::AiLibError>> {
        Box::pin(async move {
            let res: Result<serde_json::Value, TransportError> =
                self.inner.post(url, headers, &body).await;
            match res {
                Ok(v) => Ok(v),
                Err(e) => Err(crate::types::AiLibError::ProviderError(format!(
                    "Transport error: {}",
                    e
                ))),
            }
        })
    }

    fn post_stream<'a>(
        &'a self,
        _url: &'a str,
        _headers: Option<HashMap<String, String>>,
        _body: serde_json::Value,
    ) -> futures::future::BoxFuture<
        'a,
        Result<
            Pin<Box<dyn Stream<Item = Result<Bytes, crate::types::AiLibError>> + Send>>,
            crate::types::AiLibError,
        >,
    > {
        Box::pin(async move {
            // Build request
            let mut req = self.inner.client.post(_url).json(&_body);
            // Apply headers
            if let Some(h) = _headers {
                for (k, v) in h.into_iter() {
                    req = req.header(k, v);
                }
            }
            // Ensure Accept header for event-streams
            req = req.header("Accept", "text/event-stream");

            let resp = req.send().await.map_err(|e| {
                crate::types::AiLibError::ProviderError(format!("Stream request failed: {}", e))
            })?;
            if !resp.status().is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(crate::types::AiLibError::ProviderError(format!(
                    "Stream error: {}",
                    text
                )));
            }

            let byte_stream = resp.bytes_stream().map(|res| match res {
                Ok(b) => Ok(b),
                Err(e) => Err(crate::types::AiLibError::ProviderError(format!(
                    "Stream chunk error: {}",
                    e
                ))),
            });

            let boxed_stream: Pin<
                Box<dyn Stream<Item = Result<Bytes, crate::types::AiLibError>> + Send>,
            > = Box::pin(byte_stream);
            Ok(boxed_stream)
        })
    }

    fn upload_multipart<'a>(
        &'a self,
        url: &'a str,
        headers: Option<HashMap<String, String>>,
        field_name: &'a str,
        file_name: &'a str,
        bytes: Vec<u8>,
    ) -> Pin<
        Box<
            dyn futures::Future<Output = Result<serde_json::Value, crate::types::AiLibError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            // Build multipart form
            let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name.to_string());
            let form = reqwest::multipart::Form::new().part(field_name.to_string(), part);

            let mut req = self.inner.client.post(url).multipart(form);
            if let Some(h) = headers {
                for (k, v) in h.into_iter() {
                    req = req.header(k, v);
                }
            }

            let resp = req.send().await.map_err(|e| {
                crate::types::AiLibError::ProviderError(format!("upload request failed: {}", e))
            })?;
            if !resp.status().is_success() {
                let text = resp.text().await.unwrap_or_default();
                return Err(crate::types::AiLibError::ProviderError(format!(
                    "upload error: {}",
                    text
                )));
            }
            let j: serde_json::Value = resp.json().await.map_err(|e| {
                crate::types::AiLibError::ProviderError(format!("parse upload response: {}", e))
            })?;
            Ok(j)
        })
    }
}

impl HttpTransport {
    /// Produce an Arc-wrapped object-safe transport reference
    pub fn boxed(self) -> DynHttpTransportRef {
        Arc::new(HttpTransportBoxed::new(self))
    }
}
