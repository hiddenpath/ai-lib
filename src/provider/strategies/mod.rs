mod builder;
mod failover;
mod round_robin;

pub use builder::RoutingStrategyBuilder;
pub use failover::FailoverProvider;
pub use round_robin::RoundRobinProvider;

#[cfg(feature = "routing_mvp")]
pub mod health;
#[cfg(feature = "routing_mvp")]
pub use health::health_check;
