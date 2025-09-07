//! Reasoning Models Best Practices Example
//!
//! This example demonstrates how to interact with reasoning models using ai-lib, including:
//! 1. Structured reasoning - using function calls for step-by-step reasoning
//! 2. Streaming reasoning - observing real-time output of reasoning process
//! 3. JSON format reasoning - getting structured reasoning results
//! 4. Reasoning configuration - using escape hatch to pass provider-specific parameters

use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
use ai_lib::types::common::Content;
use ai_lib::types::function_call::{Tool, FunctionCallPolicy};
use serde_json::json;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 AI-lib Reasoning Models Best Practices Example");
    println!("==================================================\n");

    // Check environment variables
    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ Please set GROQ_API_KEY environment variable");
        println!("   Example: export GROQ_API_KEY=your_api_key_here");
        return Ok(());
    }

    let client = AiClient::new(Provider::Groq)?;
    println!("✅ Groq client created successfully\n");

    // 1. Structured reasoning example
    println!("1️⃣ Structured Reasoning - Using Function Calls");
    println!("-----------------------------------------------");
    demonstrate_structured_reasoning(&client).await?;
    println!();

    // 2. Streaming reasoning example
    println!("2️⃣ Streaming Reasoning - Observing Reasoning Process");
    println!("----------------------------------------------------");
    demonstrate_streaming_reasoning(&client).await?;
    println!();

    // 3. JSON format reasoning example
    println!("3️⃣ JSON Format Reasoning - Structured Results");
    println!("---------------------------------------------");
    demonstrate_json_reasoning(&client).await?;
    println!();

    // 4. Reasoning configuration example
    println!("4️⃣ Reasoning Configuration - Provider-Specific Parameters");
    println!("--------------------------------------------------------");
    demonstrate_reasoning_config(&client).await?;
    println!();

    // 5. Math problem reasoning example
    println!("5️⃣ Math Problem Reasoning");
    println!("-------------------------");
    demonstrate_math_reasoning(&client).await?;
    println!();

    // 6. Logic reasoning example
    println!("6️⃣ Logic Reasoning");
    println!("------------------");
    demonstrate_logic_reasoning(&client).await?;

    println!("\n✅ All reasoning examples completed successfully!");
    Ok(())
}

/// Method 1: Structured reasoning - using function calls for step-by-step reasoning
async fn demonstrate_structured_reasoning(client: &AiClient) -> Result<(), Box<dyn std::error::Error>> {
    // Define reasoning tool
    let reasoning_tool = Tool {
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
                "verification": {"type": "string", "description": "Verification process"}
            },
            "required": ["problem", "steps", "final_answer"]
        })),
    };

    let request = ChatCompletionRequest::new(
        "qwen-qwq-32b".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Solve this math problem: A class has 30 students, 60% are girls and 40% are boys. If 25% of girls wear glasses and 20% of boys wear glasses, how many students in total wear glasses?".to_string()),
            function_call: None,
        }],
    )
    .with_functions(vec![reasoning_tool])
    .with_function_call(FunctionCallPolicy::Auto("auto".to_string()));

    let response = client.chat_completion(request).await?;

    // Process reasoning results
    for choice in response.choices {
        if let Some(function_call) = choice.message.function_call {
            if function_call.name == "step_by_step_reasoning" {
                if let Some(args) = function_call.arguments {
                    println!("📊 Structured reasoning result:");
                    println!("{}", serde_json::to_string_pretty(&args)?);
                }
            }
        } else {
            println!("💬 Model response: {}", choice.message.content.as_text());
        }
    }

    Ok(())
}

/// Method 2: Streaming reasoning - observing real-time output of reasoning process
async fn demonstrate_streaming_reasoning(client: &AiClient) -> Result<(), Box<dyn std::error::Error>> {
    let request = ChatCompletionRequest::new(
        "qwen-qwq-32b".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Please explain how photosynthesis works and show your reasoning process".to_string()),
            function_call: None,
        }],
    );

    let mut stream = client.chat_completion_stream(request).await?;
    let mut full_reasoning = String::new();

    println!("🌊 Reasoning process (streaming output):");
    println!("{}", "─".repeat(50));

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                if let Some(choice) = chunk.choices.first() {
                    if let Some(content) = &choice.delta.content {
                        print!("{}", content);
                        full_reasoning.push_str(content);
                        use std::io::Write;
                        std::io::stdout().flush().unwrap();
                    }
                }
            }
            Err(e) => {
                println!("\n❌ Streaming error: {}", e);
                break;
            }
        }
    }

    println!("\n{}", "─".repeat(50));
    println!("✅ Complete reasoning process saved ({} characters)", full_reasoning.len());
    Ok(())
}

/// Method 3: JSON format reasoning - getting structured reasoning results
async fn demonstrate_json_reasoning(client: &AiClient) -> Result<(), Box<dyn std::error::Error>> {
    let request = ChatCompletionRequest::new(
        "qwen-qwq-32b".to_string(),
        vec![Message {
            role: Role::System,
            content: Content::Text("You are a reasoning assistant. Always respond with valid JSON format containing your reasoning process and final answer.".to_string()),
            function_call: None,
        }, Message {
            role: Role::User,
            content: Content::Text("What is the capital of France and why is it important? Please provide your reasoning process.".to_string()),
            function_call: None,
        }],
    );

    let response = client.chat_completion(request).await?;

    // Parse JSON reasoning results
    for choice in response.choices {
        let content = choice.message.content.as_text();
        println!("📋 JSON format reasoning result:");
        
        if let Ok(reasoning_json) = serde_json::from_str::<serde_json::Value>(&content) {
            println!("{}", serde_json::to_string_pretty(&reasoning_json)?);
        } else {
            println!("⚠️  Unable to parse as JSON, raw content:");
            println!("{}", content);
        }
    }

    Ok(())
}

/// Method 4: Reasoning configuration - using escape hatch to pass provider-specific parameters
async fn demonstrate_reasoning_config(client: &AiClient) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = ChatCompletionRequest::new(
        "qwen-qwq-32b".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Solve this complex math problem: x² + 5x + 6 = 0, please show the complete solution process".to_string()),
            function_call: None,
        }],
    );

    // Add Groq-specific reasoning parameters
    request = request
        .with_provider_specific("reasoning_format", serde_json::Value::String("parsed".to_string()))
        .with_provider_specific("reasoning_effort", serde_json::Value::String("high".to_string()))
        .with_provider_specific("parallel_tool_calls", serde_json::Value::Bool(true))
        .with_provider_specific("service_tier", serde_json::Value::String("flex".to_string()));

    println!("⚙️  Using reasoning configuration:");
    println!("   - reasoning_format: parsed");
    println!("   - reasoning_effort: high");
    println!("   - parallel_tool_calls: true");
    println!("   - service_tier: flex");
    println!();

    let response = client.chat_completion(request).await?;

    println!("🧮 Reasoning result:");
    for choice in response.choices {
        println!("{}", choice.message.content.as_text());
    }

    Ok(())
}

/// Method 5: Math problem reasoning
async fn demonstrate_math_reasoning(client: &AiClient) -> Result<(), Box<dyn std::error::Error>> {
    let math_problems = vec![
        "Calculate the value of 2^10 + 3^5",
        "If a circle has radius 5cm, find its area and circumference",
        "Solve the equation: 2x + 3 = 11",
    ];

    for (i, problem) in math_problems.iter().enumerate() {
        println!("📐 Math problem {}: {}", i + 1, problem);
        
        let request = ChatCompletionRequest::new(
            "qwen-qwq-32b".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text(format!("Please solve this math problem and show your reasoning process: {}", problem)),
                function_call: None,
            }],
        );

        let response = client.chat_completion(request).await?;
        
        for choice in response.choices {
            println!("💡 Solution:");
            println!("{}", choice.message.content.as_text());
        }
        println!();
    }

    Ok(())
}

/// Method 6: Logic reasoning
async fn demonstrate_logic_reasoning(client: &AiClient) -> Result<(), Box<dyn std::error::Error>> {
    let logic_problems = vec![
        "If all birds can fly, and penguins are birds, can penguins fly? Please analyze this logical reasoning.",
        "There are three people: A, B, C. A says: 'B is lying', B says: 'C is lying', C says: 'A and B are both lying'. Who is telling the truth?",
        "In a room, there are 3 switches controlling 3 light bulbs. You can only enter the room once. How do you determine which switch controls which bulb?",
    ];

    for (i, problem) in logic_problems.iter().enumerate() {
        println!("🧩 Logic problem {}: {}", i + 1, problem);
        
        let request = ChatCompletionRequest::new(
            "qwen-qwq-32b".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text(format!("Please analyze this logic problem and show your reasoning process: {}", problem)),
                function_call: None,
            }],
        );

        let response = client.chat_completion(request).await?;
        
        for choice in response.choices {
            println!("🔍 Analysis:");
            println!("{}", choice.message.content.as_text());
        }
        println!();
    }

    Ok(())
}
