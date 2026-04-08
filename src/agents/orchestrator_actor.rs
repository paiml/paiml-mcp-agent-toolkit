#![cfg_attr(coverage_nightly, coverage(off))]
use super::messaging::request_response::{AnalyzeRequest, TransformRequest, ValidateRequest};
use super::{AgentError, AgentResponse, AgentState};
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Orchestrator actor.
pub struct OrchestratorActor {
    _state: OrchestratorState,
}

#[derive(Clone, Serialize, Deserialize)]
/// State representation for orchestrator.
pub struct OrchestratorState {
    last_event_id: u64,
    events_since_snapshot: usize,
    time_since_snapshot: Duration,
}

impl Default for OrchestratorState {
    fn default() -> Self {
        Self {
            last_event_id: 0,
            events_since_snapshot: 0,
            time_since_snapshot: Duration::ZERO,
        }
    }
}

impl AgentState for OrchestratorState {
    fn last_event_id(&self) -> u64 {
        self.last_event_id
    }

    fn events_since_snapshot(&self) -> usize {
        self.events_since_snapshot
    }

    fn time_since_snapshot(&self) -> Duration {
        self.time_since_snapshot
    }
}

impl Default for OrchestratorActor {
    fn default() -> Self {
        Self::new()
    }
}

impl OrchestratorActor {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            _state: OrchestratorState::default(),
        }
    }
}

impl Actor for OrchestratorActor {
    type Context = Context<Self>;
}

// --- Message trait impls and Handler impls ---
include!("orchestrator_actor_handlers.rs");

// --- Tests ---
include!("orchestrator_actor_tests.rs");
