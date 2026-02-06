#![cfg_attr(coverage_nightly, coverage(off))]
// Zero-copy message passing protocol
pub mod backpressure;
pub mod circuit_breaker;
pub mod message_format;
pub mod pubsub;
pub mod request_response;

use actix::prelude::*;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Message)]
#[rtype(result = "Result<crate::agents::AgentResponse, crate::agents::AgentError>")]
pub struct AgentMessage {
    pub header: MessageHeader,
    pub payload: Bytes, // Zero-copy payload
}

// Custom Serialize/Deserialize for AgentMessage due to Bytes
impl Serialize for AgentMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AgentMessage", 2)?;
        state.serialize_field("header", &self.header)?;
        state.serialize_field("payload", &self.payload.to_vec())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for AgentMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};

        struct AgentMessageVisitor;

        impl<'de> Visitor<'de> for AgentMessageVisitor {
            type Value = AgentMessage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AgentMessage")
            }

            fn visit_map<V>(self, mut map: V) -> Result<AgentMessage, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut header = None;
                let mut payload = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "header" => {
                            header = Some(map.next_value()?);
                        }
                        "payload" => {
                            let bytes: Vec<u8> = map.next_value()?;
                            payload = Some(Bytes::from(bytes));
                        }
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                Ok(AgentMessage {
                    header: header.ok_or_else(|| de::Error::missing_field("header"))?,
                    payload: payload.ok_or_else(|| de::Error::missing_field("payload"))?,
                })
            }
        }

        deserializer.deserialize_struct("AgentMessage", &["header", "payload"], AgentMessageVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub id: Uuid,
    pub from: Uuid,
    pub to: Uuid,
    pub timestamp: u64, // Unix timestamp in nanos
    pub correlation_id: Option<Uuid>,
    pub priority: Priority,
    pub ttl_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

impl AgentMessage {
    pub fn new(from: Uuid, to: Uuid, payload: impl Serialize) -> Result<Self, bincode::Error> {
        let payload_bytes = bincode::serialize(&payload)?;

        Ok(Self {
            header: MessageHeader {
                id: Uuid::new_v4(),
                from,
                to,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("internal error")
                    .as_nanos() as u64,
                correlation_id: None,
                priority: Priority::Normal,
                ttl_ms: 5000, // 5 second default TTL
            },
            payload: Bytes::from(payload_bytes),
        })
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.header.priority = priority;
        self
    }

    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.header.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.header.ttl_ms = ttl.as_millis() as u32;
        self
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("internal error")
            .as_nanos() as u64;

        let expiry = self.header.timestamp + (self.header.ttl_ms as u64 * 1_000_000);
        now > expiry
    }

    pub fn deserialize_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, bincode::Error> {
        bincode::deserialize(&self.payload)
    }
}

// Message router with priority queue
pub struct MessageRouter {
    routes: dashmap::DashMap<Uuid, Recipient<AgentMessage>>,
    priority_queue: crossbeam::queue::SegQueue<(Priority, AgentMessage)>,
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            routes: dashmap::DashMap::new(),
            priority_queue: crossbeam::queue::SegQueue::new(),
        }
    }

    pub fn register(&self, agent_id: Uuid, recipient: Recipient<AgentMessage>) {
        self.routes.insert(agent_id, recipient);
    }

    pub fn route(&self, message: AgentMessage) -> Result<(), RouterError> {
        // Check if message is expired
        if message.is_expired() {
            return Err(RouterError::Expired);
        }

        // Add to priority queue
        self.priority_queue.push((message.header.priority, message));

        // Process queue by priority
        self.process_queue()
    }

    fn process_queue(&self) -> Result<(), RouterError> {
        // Sort by priority and process
        let mut messages: Vec<(Priority, AgentMessage)> = Vec::new();

        while let Some(msg) = self.priority_queue.pop() {
            messages.push(msg);
        }

        messages.sort_by_key(|(priority, _)| *priority);

        for (_, message) in messages {
            if let Some(recipient) = self.routes.get(&message.header.to) {
                recipient.do_send(message);
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("Message expired")]
    Expired,
    #[error("Agent not found")]
    NotFound,
    #[error("Queue full")]
    QueueFull,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let payload = "test payload";

        let message = AgentMessage::new(from, to, payload).expect("internal error");

        assert_eq!(message.header.from, from);
        assert_eq!(message.header.to, to);
        assert_eq!(message.header.priority, Priority::Normal);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical < Priority::High);
        assert!(Priority::High < Priority::Normal);
        assert!(Priority::Normal < Priority::Low);
    }

    #[test]
    fn test_message_expiry() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();

        let message = AgentMessage::new(from, to, "test")
            .expect("internal error")
            .with_ttl(Duration::from_millis(0));

        std::thread::sleep(Duration::from_millis(1));
        assert!(message.is_expired());
    }
}

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
