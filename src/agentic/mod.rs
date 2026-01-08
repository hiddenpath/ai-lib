//! Agentic Loop - 推理-工具-推理循环
//!
//! 实现完整的agentic workflow，支持：
//! - 推理步骤管理
//! - 工具调用编排
//! - 状态追踪
//! - 停止条件判断

pub mod loop_engine;
pub mod state;
pub mod tool_orchestrator;

pub use loop_engine::AgenticLoop;
pub use state::{LoopContext, LoopState, LoopStep};
pub use tool_orchestrator::ToolOrchestrator;
