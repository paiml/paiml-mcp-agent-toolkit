use crate::models::refactor::RefactorStateMachine;

// Note: This module provides Cap'n Proto conversion functions
// The actual capnp generated code would be included when capnp is available

pub fn serialize_state_to_capnp(state: &RefactorStateMachine) -> Result<Vec<u8>, String> {
    // For now, use JSON serialization as it's the most reliable
    // Cap'n Proto implementation can be added when the capnp compiler is available
    // This provides a clean interface for future Cap'n Proto integration
    serde_json::to_vec(state).map_err(|e| format!("Serialization error: {}", e))
}

pub fn deserialize_state_from_capnp(data: &[u8]) -> Result<RefactorStateMachine, String> {
    // For now, use JSON deserialization as it's the most reliable
    // Cap'n Proto implementation can be added when the capnp compiler is available
    serde_json::from_slice(data).map_err(|e| format!("Deserialization error: {}", e))
}

// Helper functions for testing and development
pub fn is_capnp_available() -> bool {
    // For now, Cap'n Proto is not available, using JSON fallback
    false
}

pub fn get_serialization_format() -> &'static str {
    if is_capnp_available() {
        "Cap'n Proto"
    } else {
        "JSON"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::refactor::RefactorConfig;
    use std::path::PathBuf;

    #[test]
    fn test_json_fallback_serialization() {
        let state =
            RefactorStateMachine::new(vec![PathBuf::from("test.rs")], RefactorConfig::default());

        let serialized = serialize_state_to_capnp(&state).unwrap();
        let deserialized = deserialize_state_from_capnp(&serialized).unwrap();

        assert_eq!(state.targets.len(), deserialized.targets.len());
        assert_eq!(
            state.config.target_complexity,
            deserialized.config.target_complexity
        );
    }

    #[test]
    fn test_serialization_format_detection() {
        let format = get_serialization_format();
        assert!(format == "Cap'n Proto" || format == "JSON");
    }
}
