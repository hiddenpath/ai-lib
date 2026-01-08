//! Agentic Loop状态管理
//!
//! 跟踪推理循环的状态、步骤和上下文

use crate::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 循环状态枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopState {
    /// 正在运行
    Running,

    /// 成功完成
    Completed,

    /// 达到最大迭代次数
    MaxIterationsReached,

    /// 被停止
    Stopped,

    /// 出错
    Error(String),
}

/// 循环步骤枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopStep {
    /// 推理步骤
    Reasoning(String),

    /// 工具执行步骤
    ToolExecution(Vec<ToolResult>),
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 工具名称
    pub tool_name: String,

    /// 输入参数
    pub input: serde_json::Value,

    /// 输出结果
    pub output: String,

    /// 执行状态
    pub success: bool,

    /// 执行时间（毫秒）
    pub execution_time_ms: u64,

    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 循环上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopContext {
    /// 当前状态
    pub state: LoopState,

    /// 执行的步骤
    pub steps: Vec<LoopStep>,

    /// 消息历史
    pub messages: Vec<Message>,

    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,

    /// 开始时间
    pub start_time: chrono::DateTime<chrono::Utc>,

    /// 结束时间
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl LoopContext {
    /// 创建新的循环上下文
    pub fn new(initial_query: String) -> Self {
        let mut messages = Vec::new();
        messages.push(Message {
            role: crate::types::Role::User,
            content: crate::Content::Text(initial_query),
            function_call: None,
        });

        Self {
            state: LoopState::Running,
            steps: Vec::new(),
            messages,
            metadata: HashMap::new(),
            start_time: chrono::Utc::now(),
            end_time: None,
        }
    }

    /// 添加步骤
    pub fn add_step(&mut self, step: LoopStep) {
        self.steps.push(step);
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// 完成循环
    pub fn complete(&mut self) {
        self.state = LoopState::Completed;
        self.end_time = Some(chrono::Utc::now());
    }

    /// 获取执行时间
    pub fn execution_time(&self) -> Option<std::time::Duration> {
        self.end_time
            .map(|end| (end - self.start_time).to_std().unwrap_or_default())
    }

    /// 获取最终答案
    pub fn final_answer(&self) -> Option<String> {
        for step in self.steps.iter().rev() {
            if let LoopStep::Reasoning(content) = step {
                if content.contains("FINAL_ANSWER:") {
                    return Some(content.split("FINAL_ANSWER:").nth(1)?.trim().to_string());
                }
            }
        }
        None
    }

    /// 获取使用的工具列表
    pub fn used_tools(&self) -> Vec<String> {
        let mut tools = Vec::new();
        for step in &self.steps {
            if let LoopStep::ToolExecution(results) = step {
                for result in results {
                    if !tools.contains(&result.tool_name) {
                        tools.push(result.tool_name.clone());
                    }
                }
            }
        }
        tools
    }
}

impl Default for LoopState {
    fn default() -> Self {
        LoopState::Running
    }
}

impl ToolResult {
    /// 创建成功的工具结果
    pub fn success(
        tool_name: String,
        input: serde_json::Value,
        output: String,
        execution_time_ms: u64,
    ) -> Self {
        Self {
            tool_name,
            input,
            output,
            success: true,
            execution_time_ms,
            error: None,
        }
    }

    /// 创建失败的工具结果
    pub fn failure(
        tool_name: String,
        input: serde_json::Value,
        error: String,
        execution_time_ms: u64,
    ) -> Self {
        Self {
            tool_name,
            input,
            output: String::new(),
            success: false,
            execution_time_ms,
            error: Some(error),
        }
    }
}
