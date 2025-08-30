#[cfg(test)]
mod tests {
    use ai_lib::types::common::Content;
    use ai_lib::{ChatCompletionRequest, Message, Role};

    #[test]
    fn ignore_previous_keeps_system_and_last() {
        let req = ChatCompletionRequest::new(
            "model".to_string(),
            vec![
                Message {
                    role: Role::System,
                    content: Content::Text("system".into()),
                    function_call: None,
                },
                Message {
                    role: Role::User,
                    content: Content::Text("hi".into()),
                    function_call: None,
                },
                Message {
                    role: Role::Assistant,
                    content: Content::Text("reply".into()),
                    function_call: None,
                },
                Message {
                    role: Role::User,
                    content: Content::Text("new".into()),
                    function_call: None,
                },
            ],
        );

        let new = req.ignore_previous();
        assert_eq!(new.messages.len(), 2);
        assert!(matches!(new.messages[0].role, Role::System));
        assert!(matches!(new.messages[1].role, Role::User));
        assert_eq!(new.messages[1].content.as_text(), "new");
    }
}
