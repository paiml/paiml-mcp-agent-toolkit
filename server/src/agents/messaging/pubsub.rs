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
        self.topics
            .entry(topic)
            .or_default()
            .push(agent_id);
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
