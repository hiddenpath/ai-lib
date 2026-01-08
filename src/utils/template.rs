//! Template Engine - 简单的字符串模板替换
//!
//! 支持Azure OpenAI等需要URL模板的场景
//! 格式: {variable_name} 或 {{variable_name}}

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Template Engine错误
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Missing variable: {0}")]
    MissingVariable(String),

    #[error("Invalid template syntax: {0}")]
    InvalidSyntax(String),
}

/// Template Engine - 简单的字符串替换
pub struct TemplateEngine;

impl TemplateEngine {
    /// 替换模板中的变量
    ///
    /// # 参数
    /// - `template`: 模板字符串，支持 `{variable}` 格式
    /// - `vars`: 变量映射表
    ///
    /// # 示例
    /// ```
    /// use ai_lib::utils::template::TemplateEngine;
    /// use std::collections::HashMap;
    ///
    /// let mut vars = HashMap::new();
    /// vars.insert("resource_name".to_string(), "my-resource".to_string());
    /// vars.insert("deployment".to_string(), "gpt-4".to_string());
    ///
    /// let template = "https://{resource_name}.openai.azure.com/openai/deployments/{deployment}";
    /// let result = TemplateEngine::replace(template, &vars).unwrap();
    /// assert_eq!(result, "https://my-resource.openai.azure.com/openai/deployments/gpt-4");
    /// ```
    pub fn replace(
        template: &str,
        vars: &HashMap<String, String>,
    ) -> Result<String, TemplateError> {
        // 支持两种占位格式：{var} 与 ${VAR}
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$\{([A-Za-z0-9_]+)\}").unwrap());
        let normalized = RE.replace_all(template, "{$1}");
        let template = normalized.as_ref();

        let mut result = String::with_capacity(template.len());
        let mut chars = template.chars().peekable();
        let mut in_brace = false;
        let mut var_name = String::new();

        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    if in_brace {
                        // 转义的 {{ 或错误的嵌套
                        return Err(TemplateError::InvalidSyntax(
                            "Nested braces not allowed".to_string(),
                        ));
                    }
                    in_brace = true;
                    var_name.clear();
                }
                '}' => {
                    if !in_brace {
                        result.push(ch);
                        continue;
                    }

                    // 查找变量值
                    if var_name.is_empty() {
                        return Err(TemplateError::InvalidSyntax(
                            "Empty variable name".to_string(),
                        ));
                    }

                    let value = vars
                        .get(&var_name)
                        .ok_or_else(|| TemplateError::MissingVariable(var_name.clone()))?;

                    result.push_str(value);
                    in_brace = false;
                    var_name.clear();
                }
                _ => {
                    if in_brace {
                        var_name.push(ch);
                    } else {
                        result.push(ch);
                    }
                }
            }
        }

        if in_brace {
            return Err(TemplateError::InvalidSyntax(
                "Unclosed brace in template".to_string(),
            ));
        }

        Ok(result)
    }

    /// 替换模板中的变量（允许缺失变量，使用空字符串）
    pub fn replace_optional(template: &str, vars: &HashMap<String, String>) -> String {
        Self::replace(template, vars).unwrap_or_else(|_| template.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_replacement() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "test".to_string());
        vars.insert("value".to_string(), "123".to_string());

        let template = "Hello {name}, value is {value}";
        let result = TemplateEngine::replace(template, &vars).unwrap();
        assert_eq!(result, "Hello test, value is 123");
    }

    #[test]
    fn test_azure_url_template() {
        let mut vars = HashMap::new();
        vars.insert("resource_name".to_string(), "my-resource".to_string());
        vars.insert("deployment".to_string(), "gpt-4".to_string());

        let template = "https://{resource_name}.openai.azure.com/openai/deployments/{deployment}/chat/completions";
        let result = TemplateEngine::replace(template, &vars).unwrap();
        assert_eq!(
            result,
            "https://my-resource.openai.azure.com/openai/deployments/gpt-4/chat/completions"
        );
    }

    #[test]
    fn test_dollar_brace_template() {
        let mut vars = HashMap::new();
        vars.insert("RESOURCE".to_string(), "my-resource".to_string());
        vars.insert("DEPLOYMENT".to_string(), "gpt-4".to_string());

        let template = "https://${RESOURCE}.openai.azure.com/openai/deployments/${DEPLOYMENT}";
        let result = TemplateEngine::replace(template, &vars).unwrap();
        assert_eq!(
            result,
            "https://my-resource.openai.azure.com/openai/deployments/gpt-4"
        );
    }

    #[test]
    fn test_missing_variable() {
        let vars = HashMap::new();
        let template = "Hello {name}";
        let result = TemplateEngine::replace(template, &vars);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_variables() {
        let vars = HashMap::new();
        let template = "Hello World";
        let result = TemplateEngine::replace(template, &vars).unwrap();
        assert_eq!(result, "Hello World");
    }
}
