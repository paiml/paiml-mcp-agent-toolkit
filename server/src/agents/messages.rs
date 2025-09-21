use actix::prelude::*;
use serde::{Deserialize, Serialize};
use super::{Priority, AgentResponse, AgentError};

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<AgentResponse, AgentError>")]
pub struct AnalyzeMessage {
    pub code: String,
    pub priority: Priority,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<AgentResponse, AgentError>")]
pub struct TransformMessage {
    pub code: String,
    pub rules: Vec<String>,
    pub priority: Priority,
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<AgentResponse, AgentError>")]
pub struct ValidateMessage {
    pub metrics: crate::modules::analyzer::Metrics,
    pub thresholds: crate::modules::validator::Thresholds,
    pub priority: Priority,
}