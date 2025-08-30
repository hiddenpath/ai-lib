use serde::{Deserialize, Serialize};

/// Tool / Function definition used for Function Calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    /// JSON Schema for parameters — stored as raw JSON string or object
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FunctionCallPolicy {
    Auto(String), // e.g. "auto" or explicit name
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Option<serde_json::Value>,
}
