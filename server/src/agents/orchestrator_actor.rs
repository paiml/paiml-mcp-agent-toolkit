use super::messaging::request_response::{AnalyzeRequest, TransformRequest, ValidateRequest};
use super::{AgentError, AgentResponse, AgentState};
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OrchestratorActor {
    _state: OrchestratorState,
}

#[derive(Clone, Serialize, Deserialize)]
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
    pub fn new() -> Self {
        Self {
            _state: OrchestratorState::default(),
        }
    }
}

impl Actor for OrchestratorActor {
    type Context = Context<Self>;
}

// Make request types implement Message trait
impl Message for AnalyzeRequest {
    type Result = Result<AgentResponse, AgentError>;
}

impl Message for TransformRequest {
    type Result = Result<AgentResponse, AgentError>;
}

impl Message for ValidateRequest {
    type Result = Result<AgentResponse, AgentError>;
}

impl Handler<AnalyzeRequest> for OrchestratorActor {
    type Result = Result<AgentResponse, AgentError>;

    fn handle(&mut self, _msg: AnalyzeRequest, _ctx: &mut Context<Self>) -> Self::Result {
        // Forward to analyzer actor
        Err(AgentError::ProcessingFailed("Not implemented".to_string()))
    }
}

impl Handler<TransformRequest> for OrchestratorActor {
    type Result = Result<AgentResponse, AgentError>;

    fn handle(&mut self, _msg: TransformRequest, _ctx: &mut Context<Self>) -> Self::Result {
        // Forward to transformer actor
        Err(AgentError::ProcessingFailed("Not implemented".to_string()))
    }
}

impl Handler<ValidateRequest> for OrchestratorActor {
    type Result = Result<AgentResponse, AgentError>;

    fn handle(&mut self, _msg: ValidateRequest, _ctx: &mut Context<Self>) -> Self::Result {
        // Forward to validator actor
        Err(AgentError::ProcessingFailed("Not implemented".to_string()))
    }
}

// We don't need to implement PmatAgent for OrchestratorActor for now
// since the trait requires AgentMessage which our request types don't implement