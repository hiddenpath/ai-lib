//! Backpressure control for managing concurrent requests

use std::sync::Arc;
use tokio::sync::Semaphore;

/// Backpressure controller for managing concurrent requests
pub struct BackpressureController {
    max_concurrent_requests: usize,
    semaphore: Arc<Semaphore>,
}

/// Permit for executing a request under backpressure control
pub struct BackpressurePermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl BackpressureController {
    /// Create a new backpressure controller
    pub fn new(max_concurrent_requests: usize) -> Self {
        Self {
            max_concurrent_requests,
            semaphore: Arc::new(Semaphore::new(max_concurrent_requests)),
        }
    }

    /// Acquire a permit for executing a request
    pub async fn acquire_permit(&self) -> Result<BackpressurePermit, BackpressureError> {
        let permit = self.semaphore.clone().acquire_owned().await
            .map_err(|_| BackpressureError::SemaphoreClosed)?;
        
        Ok(BackpressurePermit {
            _permit: permit,
        })
    }

    /// Get the maximum number of concurrent requests
    pub fn max_concurrent_requests(&self) -> usize {
        self.max_concurrent_requests
    }

    /// Get the number of available permits
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// Backpressure error types
#[derive(Debug, thiserror::Error)]
pub enum BackpressureError {
    #[error("Semaphore is closed")]
    SemaphoreClosed,
    #[error("No permits available")]
    NoPermitsAvailable,
}
