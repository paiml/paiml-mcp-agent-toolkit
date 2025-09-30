// Claude Agent SDK Integration Module
// Implements EXTREME TDD methodology per docs/specifications/claude-agent-integration.md

pub mod bridge;
pub mod cache;
pub mod error;
pub mod feature_flags;
pub mod observability;
pub mod pool;
pub mod quality_gates;
pub mod sandbox;
pub mod transport;

pub use bridge::{BridgeConfig, BridgeRequest, BridgeResponse, ClaudeBridge};
pub use cache::{AnalysisResult, CacheMetrics, TwoTierCache};
pub use error::{BridgeError, BridgeResult, ErrorCode};
pub use feature_flags::{FeatureFlags, RolloutStrategy};
pub use observability::{BridgeMetrics, MetricsCollector};
pub use pool::ResilientConnectionPool;
pub use sandbox::BridgeSandbox;
pub use transport::StdioTransport;

#[cfg(test)]
mod tests;
