#[cfg(test)]
mod tests {
    use ai_lib::types::common::Content;
    use ai_lib::types::function_call::FunctionCall;
    use ai_lib::{Message, Role};

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

    #[test]
    fn tool_flow_appends_tool_response() {
        // initial messages
        let mut msgs = vec![Message {
            role: Role::User,
            content: Content::new_text("Please draw an ASCII horse"),
            function_call: None,
        }];

        // simulate model calling the tool
        let call = FunctionCall {
            name: "ascii_horse".into(),
            arguments: Some(serde_json::json!({"size":"small"})),
        };

        // execute tool locally
        let size_arg = call
            .arguments
            .as_ref()
            .and_then(|v| v.get("size"))
            .and_then(|s| s.as_str())
            .unwrap();
        let out = ascii_horse(size_arg);

        // append tool output
        let tool_msg = Message {
            role: Role::Assistant,
            content: Content::new_text(out.clone()),
            function_call: None,
        };
        msgs.push(tool_msg);

        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[1].content.as_text(), out);
    }
}
