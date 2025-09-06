//! 推理工具库 - Reasoning Utils Library
//!
//! Provides convenient tools and helper functions for interacting with reasoning models

use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
use ai_lib::types::common::Content;
use serde_json::{Value, json};
// use std::collections::HashMap;

/// Reasoning utilities library
pub struct ReasoningUtils;

impl ReasoningUtils {
    /// Create reasoning prompt
    pub fn create_reasoning_prompt(problem: &str, format: ReasoningFormat) -> String {
        match format {
            ReasoningFormat::Structured => {
                format!("Please solve this problem and show your reasoning process, provide a structured answer:\n\nProblem: {}\n\nPlease provide:\n1. Problem understanding\n2. Step-by-step solution\n3. Final answer\n4. Verification process", problem)
            },
            ReasoningFormat::JSON => {
                format!("Please solve this problem and respond in JSON format, including your reasoning process:\n\nProblem: {}\n\nPlease respond in the following JSON format: {{\"problem_understanding\": \"...\", \"steps\": [{{\"step\": 1, \"description\": \"...\", \"reasoning\": \"...\"}}], \"final_answer\": \"...\", \"verification\": \"...\"}}", problem)
            },
            ReasoningFormat::Streaming => {
                format!("Please solve this problem and show your reasoning process step by step:\n\nProblem: {}", problem)
            },
            ReasoningFormat::StepByStep => {
                format!("Please solve this problem, showing your reasoning process step by step so readers can follow your thinking:\n\nProblem: {}", problem)
            }
        }
    }
    
    /// Parse reasoning result
    pub fn parse_reasoning_result(content: &str) -> Result<ReasoningResult, Box<dyn std::error::Error>> {
        // Try to parse JSON format
        if let Ok(json) = serde_json::from_str::<Value>(content) {
            return Ok(ReasoningResult::Structured(json));
        }
        
        // Parse text format
        Ok(ReasoningResult::Text(content.to_string()))
    }
    
    /// Create math reasoning request
    pub fn create_math_reasoning_request(problem: &str, model: &str) -> ChatCompletionRequest {
        ChatCompletionRequest::new(
            model.to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text(Self::create_reasoning_prompt(problem, ReasoningFormat::StepByStep)),
                function_call: None,
            }],
        )
    }
    
    /// Create logic reasoning request
    pub fn create_logic_reasoning_request(problem: &str, model: &str) -> ChatCompletionRequest {
        ChatCompletionRequest::new(
            model.to_string(),
            vec![Message {
                role: Role::System,
                content: Content::Text("You are a logic reasoning expert. Please carefully analyze the problem, show your reasoning process step by step, ensuring clear logic and correct conclusions.".to_string()),
                function_call: None,
            }, Message {
                role: Role::User,
                content: Content::Text(problem.to_string()),
                function_call: None,
            }],
        )
    }
    
    /// Create science reasoning request
    pub fn create_science_reasoning_request(problem: &str, model: &str) -> ChatCompletionRequest {
        ChatCompletionRequest::new(
            model.to_string(),
            vec![Message {
                role: Role::System,
                content: Content::Text("You are a science reasoning expert. Please solve problems based on scientific principles and logical reasoning, providing accurate and detailed explanations.".to_string()),
                function_call: None,
            }, Message {
                role: Role::User,
                content: Content::Text(problem.to_string()),
                function_call: None,
            }],
        )
    }
    
    /// Create reasoning request with configuration
    pub fn create_reasoning_config_request(
        problem: &str, 
        model: &str, 
        config: ReasoningConfig
    ) -> ChatCompletionRequest {
        let mut request = ChatCompletionRequest::new(
            model.to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text(problem.to_string()),
                function_call: None,
            }],
        );
        
        // Add reasoning configuration
        request = request
            .with_provider_specific("reasoning_format", serde_json::Value::String(
                format!("{:?}", config.format).to_lowercase()
            ))
            .with_provider_specific("reasoning_effort", serde_json::Value::String(
                format!("{:?}", config.effort).to_lowercase()
            ));
            
        if let Some(parallel) = config.parallel_tool_calls {
            request = request.with_provider_specific(
                "parallel_tool_calls", 
                serde_json::Value::Bool(parallel)
            );
        }
        
        if let Some(tier) = config.service_tier {
            request = request.with_provider_specific(
                "service_tier", 
                serde_json::Value::String(format!("{:?}", tier).to_lowercase())
            );
        }
        
        request
    }
    
    /// Create reasoning tool definition
    pub fn create_reasoning_tool() -> ai_lib::types::function_call::Tool {
        ai_lib::types::function_call::Tool {
            name: "step_by_step_reasoning".to_string(),
            description: Some("Execute step-by-step reasoning to solve complex problems".to_string()),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "problem": {"type": "string", "description": "The problem to solve"},
                    "steps": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "step_number": {"type": "integer", "description": "Step number"},
                                "description": {"type": "string", "description": "Step description"},
                                "reasoning": {"type": "string", "description": "Reasoning process"},
                                "conclusion": {"type": "string", "description": "Step conclusion"}
                            },
                            "required": ["step_number", "description", "reasoning", "conclusion"]
                        }
                    },
                    "final_answer": {"type": "string", "description": "Final answer"},
                    "verification": {"type": "string", "description": "Verification process"},
                    "confidence": {"type": "number", "description": "Answer confidence (0-1)"}
                },
                "required": ["problem", "steps", "final_answer"]
            })),
        }
    }
    
    /// Extract reasoning steps
    pub fn extract_reasoning_steps(content: &str) -> Vec<ReasoningStep> {
        let mut steps = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            let line = line.trim();
            if line.starts_with("步骤") || line.starts_with("Step") || 
               line.starts_with(char::is_numeric) && line.contains('.') {
                steps.push(ReasoningStep {
                    step_number: i + 1,
                    description: line.to_string(),
                    reasoning: String::new(),
                    conclusion: String::new(),
                });
            }
        }
        
        steps
    }
    
    /// Validate reasoning result
    pub fn validate_reasoning_result(result: &ReasoningResult) -> ValidationResult {
        match result {
            ReasoningResult::Structured(json) => {
                if json.get("final_answer").is_some() && json.get("steps").is_some() {
                    ValidationResult::Valid
                } else {
                    ValidationResult::Invalid("Missing required fields".to_string())
                }
            },
            ReasoningResult::Text(text) => {
                if text.len() > 50 && (text.contains("答案") || text.contains("answer")) {
                    ValidationResult::Valid
                } else {
                    ValidationResult::Invalid("Incomplete reasoning result".to_string())
                }
            }
        }
    }
}

/// Reasoning format enumeration
#[derive(Debug, Clone)]
pub enum ReasoningFormat {
    Structured,  // Structured format
    JSON,        // JSON format
    Streaming,   // Streaming format
    StepByStep,  // Step-by-step format
}

/// Reasoning configuration
#[derive(Debug, Clone)]
pub struct ReasoningConfig {
    pub format: ReasoningFormat,
    pub effort: ReasoningEffort,
    pub parallel_tool_calls: Option<bool>,
    pub service_tier: Option<ServiceTier>,
}

/// Reasoning effort level
#[derive(Debug, Clone)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
    None,
    Default,
}

/// Service tier
#[derive(Debug, Clone)]
pub enum ServiceTier {
    OnDemand,
    Flex,
    Auto,
}

/// Reasoning result
#[derive(Debug)]
pub enum ReasoningResult {
    Structured(Value),
    Text(String),
}

/// Reasoning step
#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub step_number: usize,
    pub description: String,
    pub reasoning: String,
    pub conclusion: String,
}

/// Validation result
#[derive(Debug)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

/// Reasoning model assistant
pub struct ReasoningAssistant {
    client: AiClient,
    model: String,
}

impl ReasoningAssistant {
    /// Create reasoning assistant
    pub fn new(client: AiClient, model: String) -> Self {
        Self { client, model }
    }
    
    /// Execute math reasoning
    pub async fn solve_math_problem(&self, problem: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = ReasoningUtils::create_math_reasoning_request(problem, &self.model);
        let response = self.client.chat_completion(request).await?;
        
        Ok(response.choices[0].message.content.as_text())
    }
    
    /// Execute logic reasoning
    pub async fn solve_logic_problem(&self, problem: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = ReasoningUtils::create_logic_reasoning_request(problem, &self.model);
        let response = self.client.chat_completion(request).await?;
        
        Ok(response.choices[0].message.content.as_text())
    }
    
    /// Execute science reasoning
    pub async fn solve_science_problem(&self, problem: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = ReasoningUtils::create_science_reasoning_request(problem, &self.model);
        let response = self.client.chat_completion(request).await?;
        
        Ok(response.choices[0].message.content.as_text())
    }
    
    /// Execute reasoning with configuration
    pub async fn solve_with_config(
        &self, 
        problem: &str, 
        config: ReasoningConfig
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request = ReasoningUtils::create_reasoning_config_request(problem, &self.model, config);
        let response = self.client.chat_completion(request).await?;
        
        Ok(response.choices[0].message.content.as_text())
    }
    
    /// Streaming reasoning
    pub async fn solve_streaming(&self, problem: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = ChatCompletionRequest::new(
            self.model.clone(),
            vec![Message {
                role: Role::User,
                content: Content::Text(ReasoningUtils::create_reasoning_prompt(
                    problem, 
                    ReasoningFormat::Streaming
                )),
                function_call: None,
            }],
        );
        
        let mut stream = self.client.chat_completion_stream(request).await?;
        let mut result = String::new();
        
        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            result.push_str(content);
                        }
                    }
                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        
        Ok(result)
    }
}

fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        if let Err(e) = run_example().await {
            eprintln!("Error: {}", e);
        }
    });
}

async fn run_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Reasoning Utils Library Example");
    println!("===================================");
    
    // Check environment variables
    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ Please set GROQ_API_KEY environment variable");
        return Ok(());
    }
    
    let client = AiClient::new(Provider::Groq)?;
    let assistant = ReasoningAssistant::new(client, "qwen-qwq-32b".to_string());
    
    // Math reasoning example
    println!("📐 Math reasoning example:");
    let math_result = assistant.solve_math_problem("Calculate the value of 2^10 + 3^5").await?;
    println!("{}", math_result);
    println!();
    
    // Logic reasoning example
    println!("🧩 Logic reasoning example:");
    let logic_result = assistant.solve_logic_problem("If all birds can fly, and penguins are birds, can penguins fly?").await?;
    println!("{}", logic_result);
    println!();
    
    // Science reasoning example
    println!("🔬 Science reasoning example:");
    let science_result = assistant.solve_science_problem("Explain why the sky is blue").await?;
    println!("{}", science_result);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_reasoning_prompt() {
        let prompt = ReasoningUtils::create_reasoning_prompt("test problem", ReasoningFormat::Structured);
        assert!(prompt.contains("test problem"));
        assert!(prompt.contains("Problem understanding"));
    }

    #[test]
    fn test_parse_reasoning_result() {
        let json_result = r#"{"final_answer": "42", "steps": []}"#;
        let result = ReasoningUtils::parse_reasoning_result(json_result);
        assert!(matches!(result, Ok(ReasoningResult::Structured(_))));
        
        let text_result = "This is a text reasoning result";
        let result = ReasoningUtils::parse_reasoning_result(text_result);
        assert!(matches!(result, Ok(ReasoningResult::Text(_))));
    }
}
