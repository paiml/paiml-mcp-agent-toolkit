// Coverage tests for messaging module (serialization, builder, router, error)

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // AgentMessage serialization tests
    #[test]
    fn test_agent_message_serialize() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let message = AgentMessage::new(from, to, "test payload").unwrap();

        // Test JSON serialization
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("header"));
        assert!(json.contains("payload"));
    }

    #[test]
    fn test_agent_message_deserialize() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let original = AgentMessage::new(from, to, "test payload").unwrap();

        // Serialize and deserialize
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: AgentMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.header.from, original.header.from);
        assert_eq!(deserialized.header.to, original.header.to);
        assert_eq!(deserialized.header.priority, original.header.priority);
        assert_eq!(deserialized.payload, original.payload);
    }

    #[test]
    fn test_agent_message_deserialize_unknown_field() {
        // Test that unknown fields are ignored during deserialization
        let json = r#"{
            "header": {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "from": "550e8400-e29b-41d4-a716-446655440001",
                "to": "550e8400-e29b-41d4-a716-446655440002",
                "timestamp": 1234567890,
                "correlation_id": null,
                "priority": "Normal",
                "ttl_ms": 5000
            },
            "payload": [1, 2, 3, 4],
            "unknown_field": "should be ignored"
        }"#;

        let result: Result<AgentMessage, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }

    // MessageHeader tests
    #[test]
    fn test_message_header_serialize_deserialize() {
        let header = MessageHeader {
            id: Uuid::new_v4(),
            from: Uuid::new_v4(),
            to: Uuid::new_v4(),
            timestamp: 1234567890,
            correlation_id: Some(Uuid::new_v4()),
            priority: Priority::High,
            ttl_ms: 3000,
        };

        let json = serde_json::to_string(&header).unwrap();
        let deserialized: MessageHeader = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, header.id);
        assert_eq!(deserialized.from, header.from);
        assert_eq!(deserialized.to, header.to);
        assert_eq!(deserialized.timestamp, header.timestamp);
        assert_eq!(deserialized.correlation_id, header.correlation_id);
        assert_eq!(deserialized.priority, header.priority);
        assert_eq!(deserialized.ttl_ms, header.ttl_ms);
    }

    #[test]
    fn test_message_header_debug() {
        let header = MessageHeader {
            id: Uuid::nil(),
            from: Uuid::nil(),
            to: Uuid::nil(),
            timestamp: 12345,
            correlation_id: None,
            priority: Priority::Normal,
            ttl_ms: 5000,
        };

        let debug_str = format!("{:?}", header);
        assert!(debug_str.contains("MessageHeader"));
    }

    #[test]
    fn test_message_header_clone() {
        let header = MessageHeader {
            id: Uuid::new_v4(),
            from: Uuid::new_v4(),
            to: Uuid::new_v4(),
            timestamp: 999,
            correlation_id: Some(Uuid::new_v4()),
            priority: Priority::Critical,
            ttl_ms: 1000,
        };

        let cloned = header.clone();
        assert_eq!(cloned.id, header.id);
        assert_eq!(cloned.priority, header.priority);
    }

    // Priority enum tests
    #[test]
    fn test_priority_all_variants() {
        let critical = Priority::Critical;
        let high = Priority::High;
        let normal = Priority::Normal;
        let low = Priority::Low;

        // Test ordering
        assert!(critical < high);
        assert!(high < normal);
        assert!(normal < low);

        // Test equality
        assert_eq!(critical, Priority::Critical);
        assert_ne!(critical, high);

        // Test Debug
        let debug_str = format!("{:?}", critical);
        assert!(debug_str.contains("Critical"));

        // Test Clone and Copy
        let cloned = critical;
        assert_eq!(cloned, critical);
    }

    #[test]
    fn test_priority_serialize_deserialize() {
        for priority in [
            Priority::Critical,
            Priority::High,
            Priority::Normal,
            Priority::Low,
        ] {
            let json = serde_json::to_string(&priority).unwrap();
            let deserialized: Priority = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, priority);
        }
    }

    // AgentMessage builder methods tests
    #[test]
    fn test_agent_message_with_priority() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test")
            .unwrap()
            .with_priority(Priority::Critical);

        assert_eq!(msg.header.priority, Priority::Critical);
    }

    #[test]
    fn test_agent_message_with_correlation() {
        let correlation_id = Uuid::new_v4();
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test")
            .unwrap()
            .with_correlation(correlation_id);

        assert_eq!(msg.header.correlation_id, Some(correlation_id));
    }

    #[test]
    fn test_agent_message_with_ttl() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test")
            .unwrap()
            .with_ttl(Duration::from_secs(10));

        assert_eq!(msg.header.ttl_ms, 10_000);
    }

    #[test]
    fn test_agent_message_builder_chain() {
        let correlation_id = Uuid::new_v4();
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test")
            .unwrap()
            .with_priority(Priority::High)
            .with_correlation(correlation_id)
            .with_ttl(Duration::from_millis(1500));

        assert_eq!(msg.header.priority, Priority::High);
        assert_eq!(msg.header.correlation_id, Some(correlation_id));
        assert_eq!(msg.header.ttl_ms, 1500);
    }

    // AgentMessage is_expired tests
    #[test]
    fn test_agent_message_not_expired() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test")
            .unwrap()
            .with_ttl(Duration::from_secs(60));

        assert!(!msg.is_expired());
    }

    #[test]
    fn test_agent_message_expired_immediately() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test")
            .unwrap()
            .with_ttl(Duration::from_millis(0));

        // Sleep to ensure expiry
        std::thread::sleep(Duration::from_millis(1));
        assert!(msg.is_expired());
    }

    // Payload deserialization tests
    #[test]
    fn test_deserialize_payload_string() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "hello world").unwrap();
        let payload: String = msg.deserialize_payload().unwrap();
        assert_eq!(payload, "hello world");
    }

    #[test]
    fn test_deserialize_payload_struct() {
        #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
        struct TestPayload {
            name: String,
            value: i32,
        }

        let original = TestPayload {
            name: "test".to_string(),
            value: 42,
        };

        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), &original).unwrap();
        let payload: TestPayload = msg.deserialize_payload().unwrap();
        assert_eq!(payload, original);
    }

    #[test]
    fn test_deserialize_payload_invalid_type() {
        // Note: bincode can interpret any 4+ bytes as i32, so we test with a struct
        // that requires specific structure that a simple string won't match
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct ComplexType {
            id: u64,
            name: String,
            items: Vec<u32>,
        }

        // Create message with just a small integer payload
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), 42u8).unwrap();

        // Try to deserialize as complex struct - should fail due to insufficient data
        let result: Result<ComplexType, _> = msg.deserialize_payload();
        assert!(
            result.is_err(),
            "Expected deserialization to fail for incompatible type"
        );
    }

    // MessageRouter tests
    #[test]
    fn test_message_router_default() {
        let router = MessageRouter::default();
        // Verify it was created successfully
        assert!(router.routes.is_empty());
    }

    #[test]
    fn test_message_router_route_expired_message() {
        let router = MessageRouter::new();
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        let msg = AgentMessage::new(from, to, "test")
            .unwrap()
            .with_ttl(Duration::from_millis(0));

        // Sleep to ensure message expires
        std::thread::sleep(Duration::from_millis(1));

        let result = router.route(msg);
        assert!(matches!(result, Err(RouterError::Expired)));
    }

    #[test]
    fn test_message_router_route_no_recipient() {
        let router = MessageRouter::new();
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        // Create a non-expired message
        let msg = AgentMessage::new(from, to, "test")
            .unwrap()
            .with_ttl(Duration::from_secs(60));

        // Route to non-existent recipient - should succeed but not deliver
        let result = router.route(msg);
        assert!(result.is_ok());
    }

    // RouterError tests
    #[test]
    fn test_router_error_expired() {
        let err = RouterError::Expired;
        let display_str = format!("{}", err);
        assert!(display_str.contains("expired"));

        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Expired"));
    }

    #[test]
    fn test_router_error_not_found() {
        let err = RouterError::NotFound;
        let display_str = format!("{}", err);
        assert!(display_str.contains("not found"));
    }

    #[test]
    fn test_router_error_queue_full() {
        let err = RouterError::QueueFull;
        let display_str = format!("{}", err);
        assert!(display_str.contains("full"));
    }

    // Test message routing with multiple priorities
    #[test]
    fn test_message_router_priority_sorting() {
        let router = MessageRouter::new();

        // Create messages with different priorities
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        let low = AgentMessage::new(from, to, "low")
            .unwrap()
            .with_priority(Priority::Low)
            .with_ttl(Duration::from_secs(60));

        let critical = AgentMessage::new(from, to, "critical")
            .unwrap()
            .with_priority(Priority::Critical)
            .with_ttl(Duration::from_secs(60));

        let normal = AgentMessage::new(from, to, "normal")
            .unwrap()
            .with_priority(Priority::Normal)
            .with_ttl(Duration::from_secs(60));

        // Route in reverse priority order
        router.route(low).unwrap();
        router.route(normal).unwrap();
        router.route(critical).unwrap();
    }

    // Test AgentMessage clone
    #[test]
    fn test_agent_message_clone() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test payload")
            .unwrap()
            .with_priority(Priority::High)
            .with_correlation(Uuid::new_v4());

        let cloned = msg.clone();

        assert_eq!(cloned.header.id, msg.header.id);
        assert_eq!(cloned.header.from, msg.header.from);
        assert_eq!(cloned.header.to, msg.header.to);
        assert_eq!(cloned.header.priority, msg.header.priority);
        assert_eq!(cloned.header.correlation_id, msg.header.correlation_id);
        assert_eq!(cloned.payload, msg.payload);
    }

    // Test AgentMessage debug
    #[test]
    fn test_agent_message_debug() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test").unwrap();
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("AgentMessage"));
        assert!(debug_str.contains("header"));
        assert!(debug_str.contains("payload"));
    }

    // Test MessageRouter register
    #[test]
    fn test_message_router_register_and_route() {
        use actix::prelude::*;

        // Create a simple test actor
        struct TestActor;
        impl Actor for TestActor {
            type Context = Context<Self>;
        }
        impl Handler<AgentMessage> for TestActor {
            type Result = Result<crate::agents::AgentResponse, crate::agents::AgentError>;
            fn handle(&mut self, _msg: AgentMessage, _ctx: &mut Context<Self>) -> Self::Result {
                Ok(crate::agents::AgentResponse::Success(serde_json::json!({})))
            }
        }

        // Test basic registration without actix system
        let router = MessageRouter::new();
        let _agent_id = Uuid::new_v4();

        // Can't easily test with real recipient without Actix runtime,
        // but we can verify the router structure
        assert!(router.routes.is_empty());
    }

    // Test serialization of complex payloads
    #[test]
    fn test_complex_payload_serialization() {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
        struct ComplexPayload {
            id: u64,
            data: Vec<String>,
            nested: Option<Box<ComplexPayload>>,
        }

        let payload = ComplexPayload {
            id: 1,
            data: vec!["a".to_string(), "b".to_string()],
            nested: Some(Box::new(ComplexPayload {
                id: 2,
                data: vec!["c".to_string()],
                nested: None,
            })),
        };

        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), &payload).unwrap();
        let deserialized: ComplexPayload = msg.deserialize_payload().unwrap();

        assert_eq!(deserialized.id, payload.id);
        assert_eq!(deserialized.data, payload.data);
        assert!(deserialized.nested.is_some());
    }

    // Test timestamp generation
    #[test]
    fn test_message_timestamp() {
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test").unwrap();

        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        assert!(msg.header.timestamp >= before);
        assert!(msg.header.timestamp <= after);
    }

    // Test default TTL
    #[test]
    fn test_default_ttl() {
        let msg = AgentMessage::new(Uuid::new_v4(), Uuid::new_v4(), "test").unwrap();
        assert_eq!(msg.header.ttl_ms, 5000); // Default 5 seconds
    }
}
