//! 增强的错误处理和恢复机制模块
//!
//! Enhanced error handling and recovery mechanisms module.
//!
//! This module provides advanced error handling, context tracking,
//! and recovery strategies for AI API calls.
//!
//! Key features:
//! - `ErrorContext`: Rich error context for debugging
//! - `ErrorMonitor`: Error rate monitoring and alerting
//! - `ErrorRecoveryManager`: Automatic error recovery strategies

pub mod context;
pub mod monitoring;
pub mod recovery;

pub use context::{ErrorContext, SuggestedAction};
pub use monitoring::{ErrorMonitor, ErrorThresholds};
pub use recovery::{ErrorRecoveryManager, RecoveryStrategy};
