// State serialization for the refactor state machine.
// Included from handlers.rs — shares parent module scope.

fn serialize_state(
    state_machine: &crate::models::refactor::RefactorStateMachine,
) -> Result<Value, Box<dyn std::error::Error>> {
    let state_json = match &state_machine.current {
        crate::models::refactor::State::Scan { targets } => {
            json!({
                "current": "Scan",
                "targets": targets,
                "current_target_index": state_machine.current_target_index,
                "config": state_machine.config
            })
        }
        crate::models::refactor::State::Analyze { current } => {
            json!({
                "current": "Analyze",
                "current_file": current,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Plan { violations } => {
            json!({
                "current": "Plan",
                "violations": violations,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Refactor { operation } => {
            json!({
                "current": "Refactor",
                "operation": operation,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Test { command } => {
            json!({
                "current": "Test",
                "command": command,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Lint { strict } => {
            json!({
                "current": "Lint",
                "strict": strict,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Emit { payload } => {
            json!({
                "current": "Emit",
                "payload": payload,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Checkpoint { reason } => {
            json!({
                "current": "Checkpoint",
                "reason": reason,
                "targets": state_machine.targets,
                "current_target_index": state_machine.current_target_index
            })
        }
        crate::models::refactor::State::Complete { summary } => {
            json!({
                "current": "Complete",
                "summary": summary
            })
        }
    };

    Ok(state_json)
}
