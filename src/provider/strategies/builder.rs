use crate::{
    api::ChatProvider,
    provider::strategies::{FailoverProvider, RoundRobinProvider},
    types::AiLibError,
};

/// Helper to compose provider strategies declaratively before handing them to `AiClientBuilder`.
pub struct RoutingStrategyBuilder {
    providers: Vec<Box<dyn ChatProvider>>,
}

impl Default for RoutingStrategyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RoutingStrategyBuilder {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn with_provider(mut self, provider: Box<dyn ChatProvider>) -> Self {
        self.providers.push(provider);
        self
    }

    pub fn extend<I>(mut self, providers: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn ChatProvider>>,
    {
        self.providers.extend(providers);
        self
    }

    pub fn build_failover(self) -> Result<FailoverProvider, AiLibError> {
        FailoverProvider::new(self.providers)
    }

    pub fn build_round_robin(self) -> Result<RoundRobinProvider, AiLibError> {
        RoundRobinProvider::new(self.providers)
    }
}
