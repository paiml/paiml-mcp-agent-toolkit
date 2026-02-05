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

impl PubSubBroker {
    pub fn new() -> Self {
        Self {
            topics: Arc::new(DashMap::new()),
            subscribers: Arc::new(DashMap::new()),
        }
    }

    pub fn subscribe(&self, agent_id: Uuid, topic: Topic, recipient: Recipient<AgentMessage>) {
        // Register subscriber
        self.subscribers.insert(agent_id, recipient);

        // Add to topic
        self.topics.entry(topic).or_default().push(agent_id);
    }

    pub fn unsubscribe(&self, agent_id: Uuid, topic: &Topic) {
        if let Some(mut subscribers) = self.topics.get_mut(topic) {
            subscribers.retain(|id| *id != agent_id);
        }
    }

    pub async fn publish(&self, topic: Topic, event: Event) -> Result<usize, PubSubError> {
        let publisher_id = Uuid::new_v4();
        let mut sent_count = 0;

        if let Some(subscribers) = self.topics.get(&topic) {
            // Create message once
            let message = AgentMessage::new(publisher_id, Uuid::nil(), event)?;

            // Parallel broadcast using rayon
            subscribers.par_iter().for_each(|agent_id| {
                if let Some(recipient) = self.subscribers.get(agent_id) {
                    let mut msg = message.clone();
                    msg.header.to = *agent_id;
                    recipient.do_send(msg);
                }
            });

            sent_count = subscribers.len();
        }

        Ok(sent_count)
    }

    pub fn get_topic_stats(&self) -> HashMap<String, TopicStats> {
        let mut stats = HashMap::new();

        for entry in self.topics.iter() {
            let topic = entry.key();
            let subscribers = entry.value();

            stats.insert(
                topic.0.clone(),
                TopicStats {
                    subscriber_count: subscribers.len(),
                    topic_name: topic.0.clone(),
                },
            );
        }

        stats
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

impl WildcardMatcher {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    pub fn add_pattern(&mut self, pattern: String, agent_id: Uuid) {
        self.patterns.push((pattern, agent_id));
    }

    pub fn matches(&self, topic: &str) -> Vec<Uuid> {
        self.patterns
            .iter()
            .filter(|(pattern, _)| self.pattern_matches(pattern, topic))
            .map(|(_, id)| *id)
            .collect()
    }

    fn pattern_matches(&self, pattern: &str, topic: &str) -> bool {
        // Simple wildcard matching (* and ?)
        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let topic_parts: Vec<&str> = topic.split('.').collect();

        if pattern_parts.len() != topic_parts.len() {
            return false;
        }

        pattern_parts
            .iter()
            .zip(topic_parts.iter())
            .all(|(p, t)| p == &"*" || p == t)
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

impl EventStore {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
        }
    }

    pub fn store(&self, topic: Topic, event: Event) {
        let mut events = self.events.write();

        events.push(StoredEvent {
            event: event.clone(),
            topic,
            timestamp: event.timestamp, // Use the event's own timestamp
        });

        // Trim if exceeds max
        let event_count = events.len();
        if event_count > self.max_events {
            events.drain(0..event_count - self.max_events);
        }
    }

    pub fn replay(&self, topic: &Topic, since: u64) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.topic == *topic && e.timestamp >= since)
            .map(|e| e.event.clone())
            .collect()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_matching() {
        let matcher = WildcardMatcher::new();

        assert!(matcher.pattern_matches("logs.*", "logs.error"));
        assert!(matcher.pattern_matches("logs.*", "logs.info"));
        assert!(!matcher.pattern_matches("logs.*", "metrics.cpu"));
        assert!(matcher.pattern_matches("*.*", "logs.error"));
    }

    // TODO: Fix test - recipient() is an Actix method not available on tokio::sync::mpsc::Sender
    // #[actix_rt::test]
    // async fn test_pubsub() {
    //     let broker = PubSubBroker::new();
    //     let topic = Topic("test.topic".to_string());

    //     // Create mock recipient
    //     let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    //     let recipient = tx.clone().recipient();

    //     let agent_id = Uuid::new_v4();
    //     broker.subscribe(agent_id, topic.clone(), recipient);

    //     let event = Event {
    //         topic: "test.topic".to_string(),
    //         data: serde_json::json!({"test": "data"}),
    //         timestamp: 0,
    //     };

    //     let sent = broker.publish(topic, event).await.unwrap();
    //     assert_eq!(sent, 1);
    // }

    #[test]
    fn test_event_store() {
        let store = EventStore::new(100);
        let topic = Topic("test".to_string());

        for i in 0..10 {
            let event = Event {
                topic: "test".to_string(),
                data: serde_json::json!({"index": i}),
                timestamp: i,
            };
            store.store(topic.clone(), event);
        }

        let replayed = store.replay(&topic, 5);
        assert_eq!(replayed.len(), 5);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;

    // Topic struct tests
    #[test]
    fn test_topic_creation_and_traits() {
        let topic1 = Topic("test.topic".to_string());
        let topic2 = Topic("test.topic".to_string());
        let topic3 = Topic("other.topic".to_string());

        // Test Clone
        let topic1_clone = topic1.clone();
        assert_eq!(topic1_clone.0, topic1.0);

        // Test Eq and PartialEq
        assert_eq!(topic1, topic2);
        assert_ne!(topic1, topic3);

        // Test Hash (indirectly via HashMap)
        let mut map = HashMap::new();
        map.insert(topic1.clone(), 1);
        assert_eq!(map.get(&topic2), Some(&1));
        assert_eq!(map.get(&topic3), None);

        // Test Debug
        let debug_str = format!("{:?}", topic1);
        assert!(debug_str.contains("test.topic"));
    }

    // Event struct tests
    #[test]
    fn test_event_creation_and_traits() {
        let event = Event {
            topic: "test.topic".to_string(),
            data: serde_json::json!({"key": "value", "number": 42}),
            timestamp: 1234567890,
        };

        // Test Clone
        let event_clone = event.clone();
        assert_eq!(event_clone.topic, event.topic);
        assert_eq!(event_clone.data, event.data);
        assert_eq!(event_clone.timestamp, event.timestamp);

        // Test Debug
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("test.topic"));

        // Test Serialize/Deserialize
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.topic, event.topic);
        assert_eq!(deserialized.timestamp, event.timestamp);
    }

    // PubSubBroker tests
    #[test]
    fn test_pubsub_broker_default() {
        let broker = PubSubBroker::default();
        let stats = broker.get_topic_stats();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_pubsub_broker_unsubscribe_from_nonexistent_topic() {
        let broker = PubSubBroker::new();
        let agent_id = Uuid::new_v4();
        let topic = Topic("nonexistent".to_string());

        // Should not panic when unsubscribing from non-existent topic
        broker.unsubscribe(agent_id, &topic);
        let stats = broker.get_topic_stats();
        assert!(stats.is_empty());
    }

    #[test]
    fn test_pubsub_broker_get_topic_stats_with_subscribers() {
        let broker = PubSubBroker::new();
        let topic1 = Topic("topic.one".to_string());
        let topic2 = Topic("topic.two".to_string());

        // We can't easily add subscribers without Actix recipients,
        // but we can test the empty stats path
        let stats = broker.get_topic_stats();
        assert!(stats.is_empty());

        // Manually add to topics map for stats testing
        broker.topics.entry(topic1.clone()).or_default();
        broker.topics.entry(topic2.clone()).or_default();

        let stats = broker.get_topic_stats();
        assert_eq!(stats.len(), 2);
        assert!(stats.contains_key("topic.one"));
        assert!(stats.contains_key("topic.two"));
    }

    // TopicStats tests
    #[test]
    fn test_topic_stats_traits() {
        let stats = TopicStats {
            topic_name: "test.topic".to_string(),
            subscriber_count: 5,
        };

        // Test Clone
        let stats_clone = stats.clone();
        assert_eq!(stats_clone.topic_name, stats.topic_name);
        assert_eq!(stats_clone.subscriber_count, stats.subscriber_count);

        // Test Debug
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("test.topic"));
        assert!(debug_str.contains("5"));
    }

    // PubSubError tests
    #[test]
    fn test_pubsub_error_display() {
        let err = PubSubError::NoSubscribers;
        let display_str = format!("{}", err);
        assert!(display_str.contains("No subscribers"));

        // Test Debug
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NoSubscribers"));
    }

    #[test]
    fn test_pubsub_error_serialization_variant() {
        // Create a serialization error by serializing a known-bad value
        // Use a size hint that will trigger a bincode error during deserialization
        let bad_data = vec![0xFFu8; 16]; // Invalid serialized data
        let result: Result<String, _> = bincode::deserialize(&bad_data);
        if let Err(e) = result {
            let pubsub_err = PubSubError::Serialization(e);
            let display_str = format!("{}", pubsub_err);
            assert!(display_str.contains("Serialization error"));
        }
    }

    // WildcardMatcher tests
    #[test]
    fn test_wildcard_matcher_default() {
        let matcher = WildcardMatcher::default();
        assert!(matcher.matches("any.topic").is_empty());
    }

    #[test]
    fn test_wildcard_matcher_add_pattern() {
        let mut matcher = WildcardMatcher::new();
        let agent1 = Uuid::new_v4();
        let agent2 = Uuid::new_v4();

        matcher.add_pattern("logs.*".to_string(), agent1);
        matcher.add_pattern("metrics.*".to_string(), agent2);
        matcher.add_pattern("logs.*".to_string(), agent2); // Duplicate pattern

        // Test matches returns correct agents
        let logs_agents = matcher.matches("logs.error");
        assert!(logs_agents.contains(&agent1));
        assert!(logs_agents.contains(&agent2));
        assert_eq!(logs_agents.len(), 2);

        let metrics_agents = matcher.matches("metrics.cpu");
        assert!(metrics_agents.contains(&agent2));
        assert_eq!(metrics_agents.len(), 1);

        // No match
        let no_match = matcher.matches("other.topic");
        assert!(no_match.is_empty());
    }

    #[test]
    fn test_wildcard_pattern_matching_edge_cases() {
        let matcher = WildcardMatcher::new();

        // Test length mismatch
        assert!(!matcher.pattern_matches("a.b.c", "a.b"));
        assert!(!matcher.pattern_matches("a.b", "a.b.c"));

        // Test exact match
        assert!(matcher.pattern_matches("exact.match", "exact.match"));

        // Test wildcard in different positions
        assert!(matcher.pattern_matches("*.second", "first.second"));
        assert!(matcher.pattern_matches("first.*", "first.anything"));

        // Test multiple wildcards
        assert!(matcher.pattern_matches("*.*", "any.thing"));
        assert!(matcher.pattern_matches("*.*.*", "a.b.c"));

        // Test no wildcards - exact match required
        assert!(!matcher.pattern_matches("exact.match", "exact.other"));
    }

    // EventStore tests
    #[test]
    fn test_event_store_trim_on_max_exceeded() {
        let store = EventStore::new(5);
        let topic = Topic("test".to_string());

        // Add more events than max
        for i in 0..10 {
            let event = Event {
                topic: "test".to_string(),
                data: serde_json::json!({"index": i}),
                timestamp: i as u64,
            };
            store.store(topic.clone(), event);
        }

        // Should only have 5 most recent events (timestamps 5-9)
        let all_events = store.replay(&topic, 0);
        assert_eq!(all_events.len(), 5);

        // Verify we have the most recent events
        let timestamps: Vec<u64> = all_events.iter().map(|e| e.timestamp).collect();
        assert!(timestamps.iter().all(|&t| t >= 5));
    }

    #[test]
    fn test_event_store_replay_with_different_topics() {
        let store = EventStore::new(100);
        let topic1 = Topic("topic1".to_string());
        let topic2 = Topic("topic2".to_string());

        // Store events for different topics
        for i in 0..5 {
            let event1 = Event {
                topic: "topic1".to_string(),
                data: serde_json::json!({"topic": 1, "index": i}),
                timestamp: i as u64,
            };
            store.store(topic1.clone(), event1);

            let event2 = Event {
                topic: "topic2".to_string(),
                data: serde_json::json!({"topic": 2, "index": i}),
                timestamp: i as u64,
            };
            store.store(topic2.clone(), event2);
        }

        // Replay should filter by topic
        let topic1_events = store.replay(&topic1, 0);
        assert_eq!(topic1_events.len(), 5);
        assert!(topic1_events.iter().all(|e| e.topic == "topic1"));

        let topic2_events = store.replay(&topic2, 0);
        assert_eq!(topic2_events.len(), 5);
        assert!(topic2_events.iter().all(|e| e.topic == "topic2"));
    }

    #[test]
    fn test_event_store_replay_with_since_filter() {
        let store = EventStore::new(100);
        let topic = Topic("test".to_string());

        for i in 0..10 {
            let event = Event {
                topic: "test".to_string(),
                data: serde_json::json!({"index": i}),
                timestamp: i as u64 * 100, // 0, 100, 200, ..., 900
            };
            store.store(topic.clone(), event);
        }

        // Replay since timestamp 500 should return events with timestamps >= 500
        let recent_events = store.replay(&topic, 500);
        assert_eq!(recent_events.len(), 5); // timestamps 500, 600, 700, 800, 900

        // Replay since 0 should return all
        let all_events = store.replay(&topic, 0);
        assert_eq!(all_events.len(), 10);

        // Replay since future should return none
        let future_events = store.replay(&topic, 1000);
        assert_eq!(future_events.len(), 0);
    }

    #[test]
    fn test_event_store_empty_replay() {
        let store = EventStore::new(100);
        let topic = Topic("empty".to_string());

        let events = store.replay(&topic, 0);
        assert!(events.is_empty());
    }

    #[test]
    fn test_stored_event_clone() {
        let event = Event {
            topic: "test".to_string(),
            data: serde_json::json!({"key": "value"}),
            timestamp: 12345,
        };

        let stored = StoredEvent {
            event: event.clone(),
            topic: Topic("test".to_string()),
            timestamp: 12345,
        };

        // Test Clone
        let stored_clone = stored.clone();
        assert_eq!(stored_clone.timestamp, stored.timestamp);
        assert_eq!(stored_clone.topic, stored.topic);
    }
}
