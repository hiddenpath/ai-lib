//! 配置热重载模块，支持运行时配置更新
//!
//! Configuration hot-reload module supporting runtime configuration updates.
//!
//! This module provides traits for loading configuration from external sources
//! and watching for changes, enabling dynamic reconfiguration without restarts.
//!
//! Key components:
//! - `ConfigProvider`: Configuration source interface
//! - `ConfigWatcher`: Change notification interface
//! - Stream-based configuration updates

use async_trait::async_trait;

/// 配置快照的来源（例如本地文件、环境映射）。
#[async_trait]
pub trait ConfigProvider: Send + Sync {
    async fn load(&self) -> Result<String, String>; // opaque JSON/YAML string
}

/// Watch for changes and notify subscribers. Default: no-op.
#[async_trait]
pub trait ConfigWatcher: Send + Sync {
    async fn subscribe(&self) -> Result<ConfigStream, String>;
}

/// Simple stream of config snapshots (opaque bytes or string); placeholder for OSS.
pub struct ConfigStream;

impl ConfigStream {
    pub async fn next(&mut self) -> Option<String> {
        None
    }
}

/// No-op implementations
pub struct NoopConfigProvider;
#[async_trait]
impl ConfigProvider for NoopConfigProvider {
    async fn load(&self) -> Result<String, String> {
        Ok(String::new())
    }
}

pub struct NoopConfigWatcher;
#[async_trait]
impl ConfigWatcher for NoopConfigWatcher {
    async fn subscribe(&self) -> Result<ConfigStream, String> {
        Ok(ConfigStream)
    }
}
