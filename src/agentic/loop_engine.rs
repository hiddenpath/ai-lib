//! Agentic Loop Engine - 核心推理循环实现
//!
//! 管理完整的推理-工具-推理工作流

use crate::agentic::state::{LoopContext, LoopState, LoopStep};
use crate::agentic::tool_orchestrator::ToolOrchestrator;
use crate::api::ChatProvider;
use crate::manifest::schema::{AgenticLoopSchema, Capability, Manifest};
use crate::{ChatCompletionRequest, Content, Message, Result as AiLibResult, Role};
use std::sync::Arc;

/// Agentic Loop引擎
pub struct AgenticLoop {
    /// Manifest配置
    manifest: Arc<Manifest>,

    /// 循环配置
    config: AgenticLoopSchema,

    /// 工具编排器
    tool_orchestrator: ToolOrchestrator,

    /// 推理提供商
    reasoning_provider: Box<dyn ChatProvider + Send + Sync>,
}

impl AgenticLoop {
    /// 创建新的AgenticLoop
    pub fn new(
        manifest: Arc<Manifest>,
        mut config: AgenticLoopSchema,
        reasoning_provider: Box<dyn ChatProvider + Send + Sync>,
    ) -> Self {
        let mut tool_orchestrator = ToolOrchestrator::new(manifest.clone());
        // 注册内置工具（可扩展从manifest.tools_mapping加载）
        tool_orchestrator.register_tool(crate::agentic::tool_orchestrator::WebSearchTool);
        tool_orchestrator.register_tool(crate::agentic::tool_orchestrator::CalculatorTool);
        tool_orchestrator.register_tool(crate::agentic::tool_orchestrator::FileReadTool);

        // 如果manifest标准层有agentic_loop配置，优先合并
        if let Some(std_cfg) = manifest.standard_schema.agentic_loop.as_ref() {
            config.max_iterations = std_cfg.max_iterations;
            config.stop_conditions = std_cfg.stop_conditions.clone();
            config.reasoning_effort = std_cfg.reasoning_effort.clone();
        }

        Self {
            manifest,
            config,
            tool_orchestrator,
            reasoning_provider,
        }
    }

    /// 执行完整的agentic循环
    pub async fn execute(&self, initial_query: &str) -> AiLibResult<LoopContext> {
        let mut context = LoopContext::new(initial_query.to_string());
        let mut step_count = 0;

        loop {
            step_count += 1;

            // 检查迭代限制
            if step_count > self.config.max_iterations {
                context.state = LoopState::MaxIterationsReached;
                break;
            }

            // 执行推理步骤
            let reasoning_result = self.perform_reasoning_step(&context).await?;
            context.add_step(LoopStep::Reasoning(reasoning_result.clone()));

            // 检查是否需要工具调用
            if let Some(tool_calls) = self.extract_tool_calls(&reasoning_result) {
                // 执行工具调用
                let tool_results = self.tool_orchestrator.execute_tools(tool_calls).await?;
                context.add_step(LoopStep::ToolExecution(tool_results.clone()));

                // 将工具结果添加到上下文中
                for result in tool_results {
                    context.messages.push(Message {
                        role: Role::Tool,
                        content: Content::Text(result.output),
                        function_call: None,
                    });
                }
            } else {
                // 没有工具调用，检查是否完成
                if self.should_finish(&reasoning_result) {
                    context.state = LoopState::Completed;
                    break;
                }
            }

            // 检查停止条件
            if self.check_stop_conditions(&context) {
                context.state = LoopState::Stopped;
                break;
            }
        }

        Ok(context)
    }

    /// 执行推理步骤
    async fn perform_reasoning_step(&self, context: &LoopContext) -> AiLibResult<String> {
        let mut messages = context.messages.clone();

        // 添加系统提示（如果配置了）
        if let Some(system_prompt) = self.generate_system_prompt(context) {
            messages.insert(
                0,
                Message {
                    role: Role::System,
                    content: Content::Text(system_prompt),
                    function_call: None,
                },
            );
        }

        let reasoning_model = self.select_reasoning_model();

        let request = ChatCompletionRequest {
            model: reasoning_model,
            messages,
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: Some(false),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            top_k: None,
            stop_sequences: None,
            logprobs: None,
            top_logprobs: None,
            seed: None,
            response_format_mode: None,
            functions: Some(self.tool_orchestrator.tool_definitions()),
            function_call: None,
            extensions: Default::default(),
        };

        let response = self.reasoning_provider.chat(request).await?;
        let content = response
            .choices
            .first()
            .map(|c| c.message.content.as_text())
            .unwrap_or_default();

        Ok(content)
    }

    /// 提取工具调用
    fn extract_tool_calls(&self, reasoning_result: &str) -> Option<Vec<String>> {
        // 简化的工具调用提取逻辑
        // 实际应该解析结构化输出
        if reasoning_result.contains("TOOL_CALL:") {
            Some(vec![reasoning_result.to_string()])
        } else {
            None
        }
    }

    /// 检查是否应该结束
    fn should_finish(&self, reasoning_result: &str) -> bool {
        // 简化的完成检查
        reasoning_result.contains("FINAL_ANSWER:") || reasoning_result.contains("CONCLUSION:")
    }

    /// 检查停止条件
    fn check_stop_conditions(&self, context: &LoopContext) -> bool {
        for condition in &self.config.stop_conditions {
            match condition.as_str() {
                "tool_result" => {
                    if context
                        .steps
                        .iter()
                        .any(|step| matches!(step, LoopStep::ToolExecution(_)))
                    {
                        return true;
                    }
                }
                "final_answer" => {
                    if let Some(LoopStep::Reasoning(last_reasoning)) = context.steps.last() {
                        if self.should_finish(last_reasoning) {
                            return true;
                        }
                    }
                }
                "max_iterations" => {
                    if context.steps.len() >= self.config.max_iterations as usize {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// 生成系统提示
    fn generate_system_prompt(&self, context: &LoopContext) -> Option<String> {
        Some(format!(
            "You are an AI assistant with access to various tools. \
             Current step: {}. Available tools: {}. \
             Use tools when needed, provide final answers when ready. Reasoning effort: {:?}",
            context.steps.len(),
            self.tool_orchestrator.available_tools().len(),
            self.config.reasoning_effort
        ))
    }

    /// 选择一个支持agentic能力的模型作为推理模型
    fn select_reasoning_model(&self) -> String {
        if let Some((_, model)) = self.manifest.models.iter().find(|(_, m)| {
            m.capabilities.contains(&Capability::Agentic)
                || m.capabilities.contains(&Capability::Reasoning)
        }) {
            return model.model_id.clone();
        }
        // 回退：任选一个模型
        self.manifest
            .models
            .values()
            .next()
            .map(|m| m.model_id.clone())
            .unwrap_or_else(|| "reasoning-model".to_string())
    }
}
