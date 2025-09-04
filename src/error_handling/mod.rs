//! Enhanced error handling and recovery mechanisms
//! 
//! This module provides advanced error handling, context tracking,
//! and recovery strategies for AI API calls.

pub mod context;
pub mod recovery;
pub mod monitoring;

pub use context::{ErrorContext, SuggestedAction};
pub use recovery::{ErrorRecoveryManager, RecoveryStrategy};
pub use monitoring::{ErrorMonitor, ErrorThresholds};
