// Unit tests for messaging module (basic tests)

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
