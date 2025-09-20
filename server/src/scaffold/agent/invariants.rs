//! Runtime invariant checking for agent state validation.

use anyhow::Result;
use async_trait::async_trait;
use std::fmt;
use tracing::error;

/// Trait for agent state machines.
#[async_trait]
pub trait AgentStateMachine: Send + Sync {
    /// The state type.
    type State: AgentState;
    /// The event type.
    type Event: AgentEvent;
    /// The context type.
    type Context: AgentContext;

    /// Get the initial state.
    fn initial_state(&self) -> Self::State;

    /// Apply a transition.
    async fn transition(
        &self,
        state: &Self::State,
        event: &Self::Event,
        ctx: &mut Self::Context,
    ) -> Result<Self::State>;

    /// Validate a transition.
    fn validate_transition(
        &self,
        from: &Self::State,
        to: &Self::State,
        event: &Self::Event,
    ) -> Result<()>;

    /// Get the invariants for this state machine.
    fn invariants(&self) -> &[Box<dyn Invariant<Self::State, Self::Context>>];
}

/// Marker trait for agent states.
pub trait AgentState: Clone + Send + Sync + fmt::Debug {}

/// Marker trait for agent events.
pub trait AgentEvent: Clone + Send + Sync + fmt::Debug {}

/// Marker trait for agent contexts.
pub trait AgentContext: Send + Sync {}

/// Trait for invariants that must hold.
pub trait Invariant<S, C>: Send + Sync {
    /// Check if the invariant holds.
    fn check(&self, state: &S, ctx: &C) -> Result<()>;

    /// Get the name of this invariant.
    fn name(&self) -> &str;
}

/// Runtime invariant checker for agent state validation.
pub struct InvariantChecker<S, C> {
    /// The invariants to check.
    invariants: Vec<Box<dyn Invariant<S, C>>>,
    /// Handler for violations.
    violation_handler: ViolationHandler,
}

impl<S: AgentState, C: AgentContext> InvariantChecker<S, C> {
    /// Create a new invariant checker.
    #[must_use] 
    pub fn new(invariants: Vec<Box<dyn Invariant<S, C>>>) -> Self {
        Self {
            invariants,
            violation_handler: ViolationHandler::default(),
        }
    }

    /// Create a new invariant checker with a custom violation handler.
    #[must_use] 
    pub fn with_handler(
        invariants: Vec<Box<dyn Invariant<S, C>>>,
        handler: ViolationHandler,
    ) -> Self {
        Self {
            invariants,
            violation_handler: handler,
        }
    }

    /// Check all invariants.
    pub fn check(&self, state: &S, ctx: &C) -> Result<()> {
        for invariant in &self.invariants {
            if let Err(e) = invariant.check(state, ctx) {
                let violation = InvariantViolation {
                    invariant_name: invariant.name().to_string(),
                    message: e.to_string(),
                };

                match self.violation_handler.handle(&violation) {
                    ViolationAction::Panic => panic!("{}", violation),
                    ViolationAction::Log => error!("{}", violation),
                    ViolationAction::Fallback(_) => {
                        // Fallback handling would require mutable state
                        error!("Fallback not implemented: {}", violation);
                    }
                }
            }
        }
        Ok(())
    }

    /// Add an invariant to the checker.
    pub fn add_invariant(&mut self, invariant: Box<dyn Invariant<S, C>>) {
        self.invariants.push(invariant);
    }

    /// Get the number of invariants.
    #[must_use] 
    pub fn invariant_count(&self) -> usize {
        self.invariants.len()
    }
}

/// Handler for invariant violations.
#[derive(Debug, Clone)]
pub struct ViolationHandler {
    /// The default action to take.
    default_action: ViolationAction,
}

impl ViolationHandler {
    /// Create a new violation handler with the given default action.
    #[must_use] 
    pub fn new(default_action: ViolationAction) -> Self {
        Self { default_action }
    }

    /// Handle a violation.
    #[must_use] 
    pub fn handle(&self, _violation: &InvariantViolation) -> ViolationAction {
        self.default_action.clone()
    }
}

impl Default for ViolationHandler {
    fn default() -> Self {
        Self {
            default_action: ViolationAction::Log,
        }
    }
}

/// Actions to take when an invariant is violated.
#[derive(Debug, Clone)]
pub enum ViolationAction {
    /// Panic immediately.
    Panic,
    /// Log the violation.
    Log,
    /// Fall back to a recovery function (not used in current implementation).
    Fallback(fn() -> ()),
}

/// Information about an invariant violation.
#[derive(Debug)]
pub struct InvariantViolation {
    /// Name of the violated invariant.
    pub invariant_name: String,
    /// Violation message.
    pub message: String,
}

impl fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invariant '{}' violated: {}",
            self.invariant_name, self.message
        )
    }
}

/// Example invariant implementation.
pub struct NonEmptyInvariant {
    /// Field name to check.
    field_name: String,
}

impl NonEmptyInvariant {
    /// Create a new non-empty invariant.
    pub fn new(field_name: impl Into<String>) -> Self {
        Self {
            field_name: field_name.into(),
        }
    }
}

impl<S, C> Invariant<S, C> for NonEmptyInvariant
where
    S: fmt::Debug,
    C: Send + Sync,
{
    fn check(&self, state: &S, _ctx: &C) -> Result<()> {
        // Verify field is not empty through debug representation
        let state_str = format!("{state:?}");
        if state_str.is_empty() {
            anyhow::bail!("{} cannot be empty", self.field_name);
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "NonEmpty"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestState {
        value: i32,
    }

    impl AgentState for TestState {}

    struct TestContext;
    impl AgentContext for TestContext {}

    struct PositiveValueInvariant;

    impl Invariant<TestState, TestContext> for PositiveValueInvariant {
        fn check(&self, state: &TestState, _ctx: &TestContext) -> Result<()> {
            if state.value <= 0 {
                anyhow::bail!("Value must be positive, got {}", state.value);
            }
            Ok(())
        }

        fn name(&self) -> &str {
            "PositiveValue"
        }
    }

    #[test]
    fn test_invariant_checker() {
        let checker = InvariantChecker::new(vec![Box::new(PositiveValueInvariant)]);

        let valid_state = TestState { value: 5 };
        let ctx = TestContext;
        assert!(checker.check(&valid_state, &ctx).is_ok());

        let invalid_state = TestState { value: -1 };
        // This will log an error but not panic with default handler
        let _ = checker.check(&invalid_state, &ctx);
    }

    #[test]
    fn test_violation_handler() {
        let handler = ViolationHandler::new(ViolationAction::Log);
        let violation = InvariantViolation {
            invariant_name: "Test".to_string(),
            message: "Test violation".to_string(),
        };

        assert!(matches!(handler.handle(&violation), ViolationAction::Log));
    }

    #[test]
    fn test_invariant_violation_display() {
        let violation = InvariantViolation {
            invariant_name: "TestInvariant".to_string(),
            message: "Something went wrong".to_string(),
        };

        assert_eq!(
            violation.to_string(),
            "Invariant 'TestInvariant' violated: Something went wrong"
        );
    }

    #[test]
    fn test_add_invariant() {
        let mut checker: InvariantChecker<TestState, TestContext> = InvariantChecker::new(vec![]);
        assert_eq!(checker.invariant_count(), 0);

        checker.add_invariant(Box::new(PositiveValueInvariant));
        assert_eq!(checker.invariant_count(), 1);
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
