use ai_lib::types::common::Content;
use ai_lib::types::function_call::{FunctionCall, FunctionCallPolicy, Tool};
use ai_lib::{ChatCompletionRequest, Message, Role};
use serde_json::json;

fn ascii_horse(size: &str) -> String {
    match size {
        "small" => String::from(
            r#"  \\
 (\_/)
 ( •_•)
 / >🐎"#,
        ),
        _ => String::from(
            r#"          ,--._______,-.
  ,-----'            _,'
 (      ASCII HORSE  / )
  `--.__________.--'"#,
        ),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example: Function Calling with a local tool 'ascii_horse'");

    // Define the tool
    let tool = Tool {
        name: "ascii_horse".to_string(),
        description: Some("Return an ASCII art horse. Accepts size: 'small'|'large'.".to_string()),
        parameters: Some(json!({
            "type": "object",
            "properties": {
                "size": { "type": "string", "enum": ["small", "large"] }
            },
            "required": ["size"]
        })),
    };

    // Build a request that offers the tool to the model
    let mut request = ChatCompletionRequest::new(
        "example-model".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("Please draw an ASCII horse for me."),
            function_call: None,
        }],
    );
    request.functions = Some(vec![tool.clone()]);
    request.function_call = Some(FunctionCallPolicy::Auto("ascii_horse".to_string()));

    println!(
        "Prepared request with functions: {}",
        serde_json::to_string_pretty(&request.functions).unwrap()
    );

    // Simulate model returning a function call (in a real run this would come from the provider)
    let simulated_call = FunctionCall {
        name: "ascii_horse".to_string(),
        arguments: Some(json!({ "size": "small" })),
    };
    println!(
        "Model requested function call: {}",
        serde_json::to_string_pretty(&simulated_call).unwrap()
    );

    // Execute the local tool
    let size_arg = simulated_call
        .arguments
        .as_ref()
        .and_then(|v| v.get("size"))
        .and_then(|s| s.as_str())
        .unwrap_or("small");

    let tool_output = ascii_horse(size_arg);

    // Convert tool output to a Message and append to conversation
    let tool_message = Message {
        role: Role::Assistant,
        content: Content::new_text(tool_output.clone()),
        function_call: None,
    };

    // In a normal flow you'd send the updated messages back to the model to continue the conversation.
    println!(
        "Tool output (appended as assistant message):\n\n{}\n",
        tool_message.content.as_text()
    );

    Ok(())
}
