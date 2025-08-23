use async_trait::async_trait;
use reqwest::{Client, Method, Response, Proxy};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::Duration;
use std::env;
use super::error::TransportError;
use backoff::{ExponentialBackoff, future::retry};

/// HTTP传输客户端抽象，定义通用HTTP操作接口
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

/// 基于reqwest的HTTP传输实现，封装所有HTTP细节
/// 
/// HTTP transport implementation based on reqwest, encapsulating all HTTP details
/// 
/// This is the concrete implementation of the HttpClient trait, encapsulating all HTTP details
pub struct HttpTransport {
    client: Client,
    timeout: Duration,
}

impl HttpTransport {
    /// Create new HTTP transport instance
    /// 
    /// Automatically detects AI_PROXY_URL environment variable for proxy configuration
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(30))
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
    
    /// Create HTTP transport instance with custom proxy
    pub fn with_proxy(timeout: Duration, proxy_url: Option<&str>) -> Result<Self, TransportError> {
        let mut client_builder = Client::builder().timeout(timeout);
        
        if let Some(url) = proxy_url {
            let proxy = Proxy::all(url)
                .map_err(|e| TransportError::InvalidUrl(format!("Invalid proxy URL: {}", e)))?;
            client_builder = client_builder.proxy(proxy);
        }
        
        let client = client_builder
            .build()
            .map_err(|e| TransportError::HttpError(e))?;

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

        // Add Content-Type
        request_builder = request_builder.header("Content-Type", "application/json");

        // Add body
        if let Some(body) = body {
            let json_body = serde_json::to_string(body)?;
            request_builder = request_builder.body(json_body);
        }

        // Send request
        let response = request_builder.send().await?;
        
        // Handle response
        Self::handle_response(response).await
    }
    
    /// Determine if error is retryable
    fn is_retryable_error(&self, error: &TransportError) -> bool {
        match error {
            TransportError::HttpError(reqwest_err) => {
                reqwest_err.is_timeout() || reqwest_err.is_connect()
            },
            TransportError::ClientError { status, .. } => {
                *status == 429 || *status == 502 || *status == 503 || *status == 504
            },
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
            match self.execute_request(method.clone(), &url_clone, headers_clone.clone(), body_clone.as_ref()).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    if self.is_retryable_error(&e) {
                        Err(backoff::Error::transient(e))
                    } else {
                        Err(backoff::Error::permanent(e))
                    }
                }
            }
        }).await
    }
}

impl Default for HttpTransport {
    fn default() -> Self {
        Self::new()
    }
}


