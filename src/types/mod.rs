// This is the start of the mod.rs file
// It includes various modules for the AI library

pub mod common;
pub mod error;
pub mod request;
pub mod response;

pub use request::ChatCompletionRequest;
pub use response::ChatCompletionResponse;
pub mod function_call;
pub use common::{Choice, Message, Role, Usage, UsageStatus};
pub use error::AiLibError;
pub use function_call::{FunctionCall, FunctionCallPolicy, Tool};

// Additional code may follow
