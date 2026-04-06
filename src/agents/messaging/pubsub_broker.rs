// PubSubBroker implementation methods

impl PubSubBroker {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            topics: Arc::new(DashMap::new()),
            subscribers: Arc::new(DashMap::new()),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn subscribe(&self, agent_id: Uuid, topic: Topic, recipient: Recipient<AgentMessage>) {
        // Register subscriber
        self.subscribers.insert(agent_id, recipient);

        // Add to topic
        self.topics.entry(topic).or_default().push(agent_id);
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn unsubscribe(&self, agent_id: Uuid, topic: &Topic) {
        if let Some(mut subscribers) = self.topics.get_mut(topic) {
            subscribers.retain(|id| *id != agent_id);
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_pattern(&mut self, pattern: String, agent_id: Uuid) {
        self.patterns.push((pattern, agent_id));
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            max_events,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    pub fn replay(&self, topic: &Topic, since: u64) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.topic == *topic && e.timestamp >= since)
            .map(|e| e.event.clone())
            .collect()
    }
}
