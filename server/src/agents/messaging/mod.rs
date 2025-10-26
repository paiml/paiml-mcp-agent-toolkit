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
                    .unwrap()
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
            .unwrap()
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
        self.priority_queue
            .push((message.header.priority, message));

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let payload = "test payload";

        let message = AgentMessage::new(from, to, payload).unwrap();

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
            .unwrap()
            .with_ttl(Duration::from_millis(0));

        std::thread::sleep(Duration::from_millis(1));
        assert!(message.is_expired());
    }
}
