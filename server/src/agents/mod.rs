// Agent system with Actix actors
pub mod analyzer_actor;
pub mod messages;
pub mod messaging;
pub mod registry;
pub mod supervisor;
pub mod transformer_actor;
pub mod validator_actor;

use actix::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type AgentId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub max_concurrent_tasks: usize,
}

#[async_trait]
pub trait PmatAgent: Send + Sync + 'static {
    type Config: Send + Sync;
    type State: AgentState;
    type Message: AgentMessage;

    fn capabilities(&self) -> AgentCapabilities;
    async fn initialize(config: Self::Config) -> Result<Self, AgentError>
    where
        Self: Sized;
    async fn process(&mut self, msg: Self::Message) -> Result<AgentResponse, AgentError>;
    async fn checkpoint(&self) -> Result<Self::State, AgentError>;
}

pub trait AgentState: Send + Sync + Clone + Serialize + for<'de> Deserialize<'de> {
    fn last_event_id(&self) -> u64;
    fn events_since_snapshot(&self) -> usize;
    fn time_since_snapshot(&self) -> std::time::Duration;
}

pub trait AgentMessage: Send + Sync + Message<Result = Result<AgentResponse, AgentError>> {
    fn priority(&self) -> Priority;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentResponse {
    Success(serde_json::Value),
    Analyzed(crate::modules::analyzer::Metrics),
    Transformed(crate::modules::transformer::TransformResult),
    Validated(crate::modules::validator::ValidationResult),
    Error(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    NotFound(AgentId),
    #[error("Agent initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Agent processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Agent communication failed: {0}")]
    CommunicationFailed(String),
    #[error("Agent timeout: {0}")]
    Timeout(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

pub enum AgentClass {
    Analyzer,
    Transformer,
    Validator,
    Orchestrator,
    Monitor,
}

pub struct AgentSpec {
    pub id: AgentId,
    pub class: AgentClass,
    pub config: serde_json::Value,
}

// Removed AgentHandle for now - will implement properly with specific actor types

// System initialization
// Note: actix::System::new() returns SystemRunner, not System
pub fn init_agent_system() {
    // TODO: Properly implement agent system initialization
    // actix::System::new() returns SystemRunner which auto-runs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_generation() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical < Priority::High);
        assert!(Priority::High < Priority::Normal);
        assert!(Priority::Normal < Priority::Low);
    }
}
