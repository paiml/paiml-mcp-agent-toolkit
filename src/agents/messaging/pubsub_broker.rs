// PubSubBroker implementation methods

use std::sync::atomic::{AtomicUsize, Ordering};

/// Hand `msg` to `recipient`, reporting whether it was actually queued.
///
/// `Recipient::do_send` swallows its `SendError`, so it cannot tell a delivery
/// from a message dropped on the floor. `try_send` surfaces the distinction:
/// `Full` still gets queued via `do_send` (unbounded fallback, the historical
/// behaviour), while `Closed` means the subscriber's actor is gone and nothing
/// was delivered.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn deliver(recipient: &Recipient<AgentMessage>, msg: AgentMessage) -> bool {
    match recipient.try_send(msg) {
        Ok(()) => true,
        Err(SendError::Full(msg)) => {
            recipient.do_send(msg);
            true
        }
        Err(SendError::Closed(_)) => false,
    }
}

impl PubSubBroker {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            topics: Arc::new(DashMap::new()),
            subscribers: Arc::new(DashMap::new()),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Subscribe to updates.
    pub fn subscribe(&self, agent_id: Uuid, topic: Topic, recipient: Recipient<AgentMessage>) {
        // Register subscriber (latest recipient wins if the agent restarted)
        self.subscribers.insert(agent_id, recipient);

        // Add to topic. Subscribing is idempotent: a repeated subscribe must not
        // push the id twice, which would deliver the event twice and inflate
        // both `subscriber_count` and the count returned by `publish`.
        let mut ids = self.topics.entry(topic).or_default();
        if !ids.contains(&agent_id) {
            ids.push(agent_id);
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Unsubscribe.
    pub fn unsubscribe(&self, agent_id: Uuid, topic: &Topic) {
        {
            // Scoped so the per-entry lock is released before scanning `topics`
            // below — DashMap is not re-entrant.
            if let Some(mut subscribers) = self.topics.get_mut(topic) {
                subscribers.retain(|id| *id != agent_id);
            }
        }

        // Release the recipient once the agent holds no subscription at all.
        // Without this, `subscribers` grows without bound and keeps every
        // unsubscribed actor's `Addr` alive for the lifetime of the broker.
        let still_subscribed = self
            .topics
            .iter()
            .any(|entry| entry.value().contains(&agent_id));
        if !still_subscribed {
            self.subscribers.remove(&agent_id);
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn publish(&self, topic: Topic, event: Event) -> Result<usize, PubSubError> {
        let publisher_id = Uuid::new_v4();

        let Some(subscribers) = self.topics.get(&topic) else {
            return Ok(0);
        };

        // Create message once
        let message = AgentMessage::new(publisher_id, Uuid::nil(), event)?;

        // The returned count is the number of messages that were actually
        // handed to a live mailbox — NOT `subscribers.len()`. A subscriber id
        // with no registered recipient, or one whose actor has stopped, is an
        // absence; counting it would report a delivery that never happened.
        let delivered = AtomicUsize::new(0);

        // Parallel broadcast using rayon
        subscribers.par_iter().for_each(|agent_id| {
            if let Some(recipient) = self.subscribers.get(agent_id) {
                let mut msg = message.clone();
                msg.header.to = *agent_id;
                if deliver(recipient.value(), msg) {
                    delivered.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        Ok(delivered.into_inner())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Get topic stats.
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

// WildcardMatcher implementation methods

impl WildcardMatcher {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Add pattern.
    pub fn add_pattern(&mut self, pattern: String, agent_id: Uuid) {
        self.patterns.push((pattern, agent_id));
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Matches.
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

// EventStore implementation methods

impl EventStore {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Store.
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Replay.
    pub fn replay(&self, topic: &Topic, since: u64) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.topic == *topic && e.timestamp >= since)
            .map(|e| e.event.clone())
            .collect()
    }
}
