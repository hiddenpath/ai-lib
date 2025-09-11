//! ai-lib-pro: Enterprise-grade extensions for ai-lib
//!
//! This crate provides advanced capabilities that layer on top of the open-source `ai-lib`:
//! - Advanced routing and load balancing
//! - Enterprise observability and monitoring
//! - Pricing catalogs and cost management
//! - Quota and rate limiting
//! - Audit logging and compliance
//! - Key management and encryption
//! - Hot-reload configuration management
//!
//! # Status
//! 
//! This is currently a skeleton implementation. Individual PRO features are being
//! developed incrementally and will be released as they become stable.
//!
//! # Versioning
//! 
//! Follows SemVer independently from `ai-lib`. PRO features are additive and
//! maintain backward compatibility within major versions.
//!
//! # Usage
//!
//! ```toml
//! [dependencies]
//! ai-lib-pro = { version = "0.1", features = ["routing_advanced", "observability_pro"] }
//! ```
//!
//! ```rust
//! use ai_lib_pro::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Use ai-lib as usual; PRO features are additive and optional
//!     let client = AiClient::new(Provider::OpenAI)?;
//!     
//!     // PRO features are available when enabled
//!     #[cfg(feature = "routing_advanced")]
//!     {
//!         // Advanced routing logic here
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Feature Flags
//!
//! - `pricing_catalog`: Centralized pricing tables and cost calculation
//! - `routing_advanced`: Policy-driven routing, health checks, fallbacks
//! - `observability_pro`: Structured logging, metrics, tracing, exporters
//! - `quota`: Tenant/organization quotas and rate policies
//! - `audit`: Comprehensive audit trails with redaction strategies
//! - `kms`: Envelope encryption and key rotation
//! - `config_hot_reload_pro`: Dynamic configuration providers and watchers
//! - `full`: Enables all PRO features
//!
//! # Architecture
//!
//! PRO features are designed to be:
//! - **Additive**: Layer on top of OSS `ai-lib` without breaking changes
//! - **Optional**: Each feature can be enabled independently
//! - **Composable**: Features work together when multiple are enabled
//! - **Stable**: Follow SemVer and maintain backward compatibility

#[cfg(feature = "pricing_catalog")]
pub mod pricing_catalog {
    //! Centralized pricing tables and cost calculation
    //! 
    //! Provides dynamic pricing information, cost calculation,
    //! and budget management capabilities.
}

#[cfg(feature = "routing_advanced")]
pub mod routing_advanced {
    //! Advanced routing and load balancing
    //! 
    //! Policy-driven routing, health monitoring, automatic failover,
    //! and intelligent load distribution across multiple providers.
}

#[cfg(feature = "observability_pro")]
pub mod observability_pro {
    //! Enterprise observability and monitoring
    //! 
    //! Structured logging, metrics collection, distributed tracing,
    //! and integration with monitoring platforms.
}

#[cfg(feature = "quota")]
pub mod quota {
    //! Quota and rate limiting
    //! 
    //! Tenant/organization-level quotas, rate limiting policies,
    //! and usage tracking.
}

#[cfg(feature = "audit")]
pub mod audit {
    //! Audit logging and compliance
    //! 
    //! Comprehensive audit trails, compliance reporting,
    //! and data redaction strategies.
}

#[cfg(feature = "kms")]
pub mod kms {
    //! Key management and encryption
    //! 
    //! Envelope encryption, key rotation, and integration
    //! with enterprise key management systems.
}

#[cfg(feature = "config_hot_reload_pro")]
pub mod config_hot_reload_pro {
    //! Dynamic configuration management
    //! 
    //! Hot-reload configuration providers, configuration watchers,
    //! and dynamic policy updates.
}

// Re-export ai-lib types for convenience
pub mod prelude {
    //! Re-exports commonly used types from ai-lib and ai-lib-pro
    
    pub use ai_lib::*;
    
    // Add PRO specific re-exports here as features are implemented
    #[cfg(feature = "pricing_catalog")]
    pub use crate::pricing_catalog::*;
    
    #[cfg(feature = "routing_advanced")]
    pub use crate::routing_advanced::*;
    
    #[cfg(feature = "observability_pro")]
    pub use crate::observability_pro::*;
    
    #[cfg(feature = "quota")]
    pub use crate::quota::*;
    
    #[cfg(feature = "audit")]
    pub use crate::audit::*;
    
    #[cfg(feature = "kms")]
    pub use crate::kms::*;
    
    #[cfg(feature = "config_hot_reload_pro")]
    pub use crate::config_hot_reload_pro::*;
}