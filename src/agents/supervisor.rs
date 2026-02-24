#![cfg_attr(coverage_nightly, coverage(off))]
use super::analyzer_actor::AnalyzerActor;
use super::messages::{AnalyzeMessage, ValidateMessage};
use super::transformer_actor::TransformerActor;
use super::validator_actor::ValidatorActor;
use super::{AgentError, AgentResponse};
use actix::prelude::*;

// --- Type definitions, struct declarations, and trait impls ---
include!("supervisor_types.rs");

// --- Handler<ValidateCode> impl ---
include!("supervisor_handler.rs");

// --- Tests ---
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    include!("supervisor_tests_unit.rs");
    include!("supervisor_tests_integration.rs");
}
