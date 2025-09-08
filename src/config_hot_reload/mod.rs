use async_trait::async_trait;

/// A source of configuration snapshots (e.g., local file, env map).
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
    pub async fn next(&mut self) -> Option<String> { None }
}

/// No-op implementations
pub struct NoopConfigProvider;
#[async_trait]
impl ConfigProvider for NoopConfigProvider { async fn load(&self) -> Result<String, String> { Ok(String::new()) } }

pub struct NoopConfigWatcher;
#[async_trait]
impl ConfigWatcher for NoopConfigWatcher { async fn subscribe(&self) -> Result<ConfigStream, String> { Ok(ConfigStream) } }


