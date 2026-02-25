// Tests for OrchestratorActor: state management, trait impls,
// handler routing, and actor runtime behavior.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // OrchestratorState Tests
    // ========================================

    #[test]
    fn test_orchestrator_state_default() {
        let state = OrchestratorState::default();
        assert_eq!(state.last_event_id, 0);
        assert_eq!(state.events_since_snapshot, 0);
        assert_eq!(state.time_since_snapshot, Duration::ZERO);
    }

    #[test]
    fn test_orchestrator_state_custom_values() {
        let state = OrchestratorState {
            last_event_id: 42,
            events_since_snapshot: 10,
            time_since_snapshot: Duration::from_secs(60),
        };
        assert_eq!(state.last_event_id, 42);
        assert_eq!(state.events_since_snapshot, 10);
        assert_eq!(state.time_since_snapshot, Duration::from_secs(60));
    }

    #[test]
    fn test_orchestrator_state_clone() {
        let state = OrchestratorState {
            last_event_id: 100,
            events_since_snapshot: 5,
            time_since_snapshot: Duration::from_millis(500),
        };
        let cloned = state.clone();
        assert_eq!(state.last_event_id, cloned.last_event_id);
        assert_eq!(state.events_since_snapshot, cloned.events_since_snapshot);
        assert_eq!(state.time_since_snapshot, cloned.time_since_snapshot);
    }

    #[test]
    fn test_orchestrator_state_serialization() {
        let state = OrchestratorState {
            last_event_id: 99,
            events_since_snapshot: 3,
            time_since_snapshot: Duration::from_secs(30),
        };
        let serialized = serde_json::to_string(&state).expect("serialization should succeed");
        let deserialized: OrchestratorState =
            serde_json::from_str(&serialized).expect("deserialization should succeed");
        assert_eq!(state.last_event_id, deserialized.last_event_id);
        assert_eq!(
            state.events_since_snapshot,
            deserialized.events_since_snapshot
        );
    }

    #[test]
    fn test_orchestrator_state_deserialization_from_json() {
        let json = r#"{"last_event_id":55,"events_since_snapshot":7,"time_since_snapshot":{"secs":45,"nanos":0}}"#;
        let state: OrchestratorState =
            serde_json::from_str(json).expect("deserialization should succeed");
        assert_eq!(state.last_event_id, 55);
        assert_eq!(state.events_since_snapshot, 7);
        assert_eq!(state.time_since_snapshot, Duration::from_secs(45));
    }

    #[test]
    fn test_orchestrator_state_zero_duration() {
        let state = OrchestratorState {
            last_event_id: 1,
            events_since_snapshot: 1,
            time_since_snapshot: Duration::ZERO,
        };
        assert_eq!(state.time_since_snapshot.as_secs(), 0);
        assert_eq!(state.time_since_snapshot.as_nanos(), 0);
    }

    #[test]
    fn test_orchestrator_state_nanos_precision() {
        let state = OrchestratorState {
            last_event_id: 0,
            events_since_snapshot: 0,
            time_since_snapshot: Duration::from_nanos(123456789),
        };
        assert_eq!(state.time_since_snapshot.as_nanos(), 123456789);
    }

    // ========================================
    // AgentState Trait Implementation Tests
    // ========================================

    #[test]
    fn test_agent_state_last_event_id() {
        let state = OrchestratorState {
            last_event_id: 123,
            events_since_snapshot: 0,
            time_since_snapshot: Duration::ZERO,
        };
        assert_eq!(state.last_event_id(), 123);
    }

    #[test]
    fn test_agent_state_events_since_snapshot() {
        let state = OrchestratorState {
            last_event_id: 0,
            events_since_snapshot: 42,
            time_since_snapshot: Duration::ZERO,
        };
        assert_eq!(state.events_since_snapshot(), 42);
    }

    #[test]
    fn test_agent_state_time_since_snapshot() {
        let state = OrchestratorState {
            last_event_id: 0,
            events_since_snapshot: 0,
            time_since_snapshot: Duration::from_secs(120),
        };
        assert_eq!(state.time_since_snapshot(), Duration::from_secs(120));
    }

    #[test]
    fn test_agent_state_trait_all_methods() {
        let state = OrchestratorState {
            last_event_id: 999,
            events_since_snapshot: 50,
            time_since_snapshot: Duration::from_millis(5000),
        };

        // Test that trait methods work correctly
        assert_eq!(state.last_event_id(), 999);
        assert_eq!(state.events_since_snapshot(), 50);
        assert_eq!(state.time_since_snapshot(), Duration::from_millis(5000));
    }

    // ========================================
    // OrchestratorActor Tests
    // ========================================

    #[test]
    fn test_orchestrator_actor_new() {
        let actor = OrchestratorActor::new();
        // Verify actor was created (internal state is private, so we just verify construction)
        let _ = actor;
    }

    #[test]
    fn test_orchestrator_actor_default() {
        let actor = OrchestratorActor::default();
        // Verify default implementation delegates to new()
        let _ = actor;
    }

    #[test]
    fn test_orchestrator_actor_default_equals_new() {
        // Both should create equivalent actors
        let _actor1 = OrchestratorActor::new();
        let _actor2 = OrchestratorActor::default();
        // Both should be valid actors - construction doesn't panic
    }

    // ========================================
    // Edge Case Tests
    // ========================================

    #[test]
    fn test_orchestrator_state_max_values() {
        let state = OrchestratorState {
            last_event_id: u64::MAX,
            events_since_snapshot: usize::MAX,
            time_since_snapshot: Duration::MAX,
        };
        assert_eq!(state.last_event_id(), u64::MAX);
        assert_eq!(state.events_since_snapshot(), usize::MAX);
        assert_eq!(state.time_since_snapshot(), Duration::MAX);
    }

    #[test]
    fn test_orchestrator_state_min_values() {
        let state = OrchestratorState {
            last_event_id: 0,
            events_since_snapshot: 0,
            time_since_snapshot: Duration::ZERO,
        };
        assert_eq!(state.last_event_id(), 0);
        assert_eq!(state.events_since_snapshot(), 0);
        assert_eq!(state.time_since_snapshot(), Duration::ZERO);
    }

    #[test]
    fn test_orchestrator_state_clone_independence() {
        let original = OrchestratorState {
            last_event_id: 100,
            events_since_snapshot: 5,
            time_since_snapshot: Duration::from_secs(10),
        };
        let cloned = original.clone();

        // Verify cloned values match original
        assert_eq!(cloned.last_event_id, original.last_event_id);
        assert_eq!(cloned.events_since_snapshot, original.events_since_snapshot);
        assert_eq!(cloned.time_since_snapshot, original.time_since_snapshot);
    }

    #[test]
    fn test_orchestrator_state_serialization_roundtrip() {
        let original = OrchestratorState {
            last_event_id: 12345,
            events_since_snapshot: 678,
            time_since_snapshot: Duration::from_millis(9999),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&original).expect("serialization should succeed");

        // Deserialize back
        let restored: OrchestratorState =
            serde_json::from_str(&json).expect("deserialization should succeed");

        // Verify all fields match
        assert_eq!(original.last_event_id, restored.last_event_id);
        assert_eq!(
            original.events_since_snapshot,
            restored.events_since_snapshot
        );
        assert_eq!(original.time_since_snapshot, restored.time_since_snapshot);
    }

    #[test]
    fn test_orchestrator_state_serialization_json_format() {
        let state = OrchestratorState {
            last_event_id: 42,
            events_since_snapshot: 3,
            time_since_snapshot: Duration::from_secs(60),
        };

        let json = serde_json::to_string(&state).expect("serialization should succeed");

        // Verify JSON contains expected fields
        assert!(json.contains("last_event_id"));
        assert!(json.contains("events_since_snapshot"));
        assert!(json.contains("time_since_snapshot"));
        assert!(json.contains("42"));
    }

    // ========================================
    // Actor Runtime Tests (require actix runtime)
    // ========================================

    #[actix_rt::test]
    async fn test_orchestrator_actor_starts() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();
        // Actor should be running
        assert!(addr.connected());
    }

    #[actix_rt::test]
    async fn test_orchestrator_actor_default_starts() {
        let actor = OrchestratorActor::default();
        let addr = actor.start();
        assert!(addr.connected());
    }

    #[actix_rt::test]
    async fn test_handle_analyze_request_not_implemented() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        let request = AnalyzeRequest {
            code: "fn main() {}".to_string(),
        };

        let result = addr.send(request).await;
        assert!(result.is_ok());
        let inner_result = result.unwrap();
        assert!(inner_result.is_err());
        match inner_result {
            Err(AgentError::ProcessingFailed(msg)) => {
                assert_eq!(msg, "Not implemented");
            }
            _ => panic!("Expected ProcessingFailed error"),
        }
    }

    #[actix_rt::test]
    async fn test_handle_transform_request_not_implemented() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        let request = TransformRequest {
            code: "fn main() {}".to_string(),
            transform_type: "refactor".to_string(),
        };

        let result = addr.send(request).await;
        assert!(result.is_ok());
        let inner_result = result.unwrap();
        assert!(inner_result.is_err());
        match inner_result {
            Err(AgentError::ProcessingFailed(msg)) => {
                assert_eq!(msg, "Not implemented");
            }
            _ => panic!("Expected ProcessingFailed error"),
        }
    }

    #[actix_rt::test]
    async fn test_handle_validate_request_not_implemented() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        let request = ValidateRequest {
            code: "fn main() {}".to_string(),
            rules: vec!["rule1".to_string()],
        };

        let result = addr.send(request).await;
        assert!(result.is_ok());
        let inner_result = result.unwrap();
        assert!(inner_result.is_err());
        match inner_result {
            Err(AgentError::ProcessingFailed(msg)) => {
                assert_eq!(msg, "Not implemented");
            }
            _ => panic!("Expected ProcessingFailed error"),
        }
    }

    #[actix_rt::test]
    async fn test_handle_analyze_request_empty_code() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        let request = AnalyzeRequest {
            code: String::new(),
        };

        let result = addr.send(request).await;
        assert!(result.is_ok());
        // Even empty code returns not implemented
        assert!(result.unwrap().is_err());
    }

    #[actix_rt::test]
    async fn test_handle_transform_request_empty_transform_type() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        let request = TransformRequest {
            code: "fn test() {}".to_string(),
            transform_type: String::new(),
        };

        let result = addr.send(request).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    #[actix_rt::test]
    async fn test_handle_validate_request_empty_rules() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        let request = ValidateRequest {
            code: "fn main() {}".to_string(),
            rules: vec![],
        };

        let result = addr.send(request).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    #[actix_rt::test]
    async fn test_orchestrator_handles_sequential_requests() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        // Send requests sequentially
        let request1 = AnalyzeRequest {
            code: "fn a() {}".to_string(),
        };
        let r1 = addr.send(request1).await;
        assert!(r1.is_ok());
        assert!(r1.unwrap().is_err());

        let request2 = TransformRequest {
            code: "fn b() {}".to_string(),
            transform_type: "test".to_string(),
        };
        let r2 = addr.send(request2).await;
        assert!(r2.is_ok());
        assert!(r2.unwrap().is_err());

        let request3 = ValidateRequest {
            code: "fn c() {}".to_string(),
            rules: vec!["rule".to_string()],
        };
        let r3 = addr.send(request3).await;
        assert!(r3.is_ok());
        assert!(r3.unwrap().is_err());
    }

    #[actix_rt::test]
    async fn test_analyze_request_large_code() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        let large_code = format!("fn main() {{ {} }}", "x;".repeat(10000));
        let request = AnalyzeRequest { code: large_code };

        let result = addr.send(request).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    #[actix_rt::test]
    async fn test_validate_request_many_rules() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        let rules: Vec<String> = (0..100).map(|i| format!("rule_{}", i)).collect();
        let request = ValidateRequest {
            code: "fn test() {}".to_string(),
            rules,
        };

        let result = addr.send(request).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    #[actix_rt::test]
    async fn test_transform_request_special_chars() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        let request = TransformRequest {
            code: "fn test() { let s = \"hello\\nworld\"; }".to_string(),
            transform_type: "format".to_string(),
        };

        let result = addr.send(request).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }

    #[actix_rt::test]
    async fn test_actor_remains_connected_after_error() {
        let actor = OrchestratorActor::new();
        let addr = actor.start();

        // Send a request that returns an error
        let request = AnalyzeRequest {
            code: "test".to_string(),
        };
        let _ = addr.send(request).await;

        // Actor should still be connected
        assert!(addr.connected());

        // Should be able to send another request
        let request2 = AnalyzeRequest {
            code: "test2".to_string(),
        };
        let result = addr.send(request2).await;
        assert!(result.is_ok());
    }
}
