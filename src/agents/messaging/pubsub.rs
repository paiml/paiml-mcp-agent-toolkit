#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;
use dashmap::DashMap;
use parking_lot::RwLock;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Topic(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub topic: String,
    pub data: serde_json::Value,
    pub timestamp: u64,
}

pub struct PubSubBroker {
    topics: Arc<DashMap<Topic, Vec<Uuid>>>,
    subscribers: Arc<DashMap<Uuid, Recipient<AgentMessage>>>,
}

impl Default for PubSubBroker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct TopicStats {
    pub topic_name: String,
    pub subscriber_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum PubSubError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error("No subscribers for topic")]
    NoSubscribers,
}

// Wildcard subscription support
pub struct WildcardMatcher {
    patterns: Vec<(String, Uuid)>,
}

impl Default for WildcardMatcher {
    fn default() -> Self {
        Self::new()
    }
}

// Event sourcing support
pub struct EventStore {
    events: Arc<RwLock<Vec<StoredEvent>>>,
    max_events: usize,
}

#[derive(Debug, Clone)]
struct StoredEvent {
    event: Event,
    topic: Topic,
    timestamp: u64,
}

// --- Implementation methods split into include files ---
include!("pubsub_broker.rs");

// --- Tests ---
include!("pubsub_tests.rs");
