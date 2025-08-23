// This is the start of the mod.rs file
// It includes various modules for the AI library

pub mod request;
pub mod response;
pub mod common;
pub mod error;

pub use request::ChatCompletionRequest;
pub use response::ChatCompletionResponse;
pub use common::{Message, Role, Choice, Usage};
pub use error::AiLibError;

// Additional code may follow
// ...existing code...
