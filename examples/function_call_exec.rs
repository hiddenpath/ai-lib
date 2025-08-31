use ai_lib::api::ChatApi;
use ai_lib::types::{ChatCompletionRequest, FunctionCallPolicy, Message, Role, Tool};
use ai_lib::AiClient;
use ai_lib::Provider;
use serde_json::json;

// This example demonstrates a simple function-calling loop using a mock transport or adapter.
// For documentation purposes this is synchronous-ish; in real apps run inside tokio runtime.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("See README for a documented example. This file is a placeholder.");
    Ok(())
}
