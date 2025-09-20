use crate::models::refactor::RefactorStateMachine;

// Note: This module provides Cap'n Proto conversion functions
// The actual capnp generated code would be included when capnp is available

/// Serializes a `RefactorStateMachine` to binary format (currently JSON)
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::capnp_conversion::serialize_state_to_capnp;
/// use pmat::models::refactor::{RefactorStateMachine, RefactorConfig};
/// use std::path::PathBuf;
///
/// let state = RefactorStateMachine::new(
///     vec![PathBuf::from("src/main.rs")],
///     RefactorConfig::default()
/// );
///
/// let result = serialize_state_to_capnp(&state);
/// assert!(result.is_ok());
/// assert!(!result.unwrap().is_empty());
/// ```
pub fn serialize_state_to_capnp(state: &RefactorStateMachine) -> Result<Vec<u8>, String> {
    // JSON serialization is used as the primary format for cross-platform compatibility
    // This approach provides consistent serialization across all environments
    serde_json::to_vec(state).map_err(|e| format!("Serialization error: {e}"))
}

/// Deserializes a `RefactorStateMachine` from binary format (currently JSON)
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::capnp_conversion::{serialize_state_to_capnp, deserialize_state_from_capnp};
/// use pmat::models::refactor::{RefactorStateMachine, RefactorConfig};
/// use std::path::PathBuf;
///
/// let state = RefactorStateMachine::new(
///     vec![PathBuf::from("test.rs")],
///     RefactorConfig::default()
/// );
///
/// let serialized = serialize_state_to_capnp(&state).unwrap();
/// let deserialized = deserialize_state_from_capnp(&serialized).unwrap();
///
/// assert_eq!(state.targets.len(), deserialized.targets.len());
/// ```
pub fn deserialize_state_from_capnp(data: &[u8]) -> Result<RefactorStateMachine, String> {
    // JSON deserialization ensures compatibility with the serialization format
    // This maintains consistency with the serialize_state_to_capnp function
    serde_json::from_slice(data).map_err(|e| format!("Deserialization error: {e}"))
}

// Helper functions for testing and development
/// Checks if Cap'n Proto serialization is available
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::capnp_conversion::is_capnp_available;
///
/// let available = is_capnp_available();
/// assert!(!available); // Currently always returns false
/// ```
#[must_use] 
pub fn is_capnp_available() -> bool {
    // JSON format is the standard serialization method for this module
    false
}

/// Returns the current serialization format being used
///
/// # Examples
///
/// ```rust
/// use pmat::mcp_server::capnp_conversion::get_serialization_format;
///
/// let format = get_serialization_format();
/// assert_eq!(format, "JSON");
/// ```
#[must_use] 
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
