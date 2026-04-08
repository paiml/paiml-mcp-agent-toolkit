// State machine template generation: TemplateGenerator impl and all
// generation functions for Cargo.toml, main.rs, state definitions,
// transitions, invariants, and tests for state machine agents.

#[async_trait]
impl TemplateGenerator for StateMachineTemplate {
    fn generate(&self, ctx: &AgentContext) -> Result<GeneratedFiles> {
        let mut files = GeneratedFiles::new();

        // Generate Cargo.toml
        files.add_text_file("Cargo.toml", generate_state_machine_cargo_toml(ctx));

        // Generate main.rs
        files.add_text_file("src/main.rs", generate_state_machine_main(ctx));

        // Generate state machine implementation
        files.add_text_file("src/agent/mod.rs", generate_state_machine_mod());
        files.add_text_file("src/agent/state.rs", generate_state_definitions(ctx));
        files.add_text_file("src/agent/transitions.rs", generate_transitions());
        files.add_text_file("src/agent/invariants.rs", generate_state_invariants());

        // Generate tests
        files.add_text_file("tests/state_transitions.rs", generate_state_tests());
        files.add_text_file("tests/invariants.rs", generate_invariant_tests());

        Ok(files)
    }

    fn validate_context(&self, ctx: &AgentContext) -> Result<()> {
        if ctx.name.is_empty() {
            bail!("Agent name is required");
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn generate_state_machine_cargo_toml(ctx: &AgentContext) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
async-trait = "0.1"
tokio = {{ version = "1.40", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
anyhow = "1.0"
tracing = "0.1"

[dev-dependencies]
proptest = "1.5"
tokio-test = "0.4"
"#,
        ctx.name
    )
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn generate_state_machine_main(ctx: &AgentContext) -> String {
    format!(
        r#"//! {} - State Machine Agent

use anyhow::Result;

mod agent;

#[tokio::main]
async fn main() -> Result<()> {{
    println!("Starting {} state machine");
    // Implementation here
    Ok(())
}}
"#,
        ctx.name, ctx.name
    )
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn generate_state_machine_mod() -> String {
    r"//! State machine implementation.

pub mod state;
pub mod transitions;
pub mod invariants;
"
    .to_string()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn generate_state_definitions(ctx: &AgentContext) -> String {
    format!(
        r"//! State definitions for {}.

use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Refactoring state machine lifecycle state.
pub enum State {{
    Initial,
    Processing,
    Complete,
    Error(String),
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Event.
pub enum Event {{
    Start,
    Process,
    Finish,
    Fail(String),
}}
",
        ctx.name
    )
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn generate_transitions() -> String {
    r"//! State transition logic.

use anyhow::Result;
use super::state::{State, Event};

/// Apply a transition.
pub fn transition(state: &State, event: &Event) -> Result<State> {
    match (state, event) {
        (State::Initial, Event::Start) => Ok(State::Processing),
        (State::Processing, Event::Finish) => Ok(State::Complete),
        (State::Processing, Event::Fail(msg)) => Ok(State::Error(msg.clone())),
        _ => Ok(state.clone()),
    }
}
"
    .to_string()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn generate_state_invariants() -> String {
    r#"//! State invariants.

use anyhow::Result;
use super::state::State;

/// Check state invariants.
pub fn check_invariants(state: &State) -> Result<()> {
    match state {
        State::Error(msg) if msg.is_empty() => {
            anyhow::bail!("Error state must have a message");
        }
        _ => Ok(()),
    }
}
"#
    .to_string()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn generate_state_tests() -> String {
    r"//! State transition tests.

#[test]
fn test_valid_transitions() {
    // Test valid state transitions
}

#[test]
fn test_invalid_transitions() {
    // Test invalid state transitions
}
"
    .to_string()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn generate_invariant_tests() -> String {
    r"//! Invariant tests.

#[test]
fn test_state_invariants() {
    // Test state invariants
}
"
    .to_string()
}
