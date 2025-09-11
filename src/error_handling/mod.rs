//! Enhanced error handling and recovery mechanisms
//!
//! This module provides advanced error handling, context tracking,
//! and recovery strategies for AI API calls.

pub mod context;
pub mod monitoring;
pub mod recovery;

pub use context::{ErrorContext, SuggestedAction};
pub use monitoring::{ErrorMonitor, ErrorThresholds};
pub use recovery::{ErrorRecoveryManager, RecoveryStrategy};
