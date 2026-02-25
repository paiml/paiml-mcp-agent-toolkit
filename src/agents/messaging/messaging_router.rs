// MessageRouter with priority queue and RouterError

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
