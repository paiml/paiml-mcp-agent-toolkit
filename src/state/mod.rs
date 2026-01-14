// Hybrid event sourcing with snapshots for state management
pub mod event_store;
// TODO: Fix async_raft v0.6 API compatibility
// pub mod raft_consensus;
pub mod recovery;
pub mod snapshot_store;

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

pub type EventId = u64;
pub type SnapshotId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEvent {
    pub id: EventId,
    pub timestamp: SystemTime,
    pub partition_key: String,
    pub event_type: String,
    pub data: serde_json::Value,
}

impl StateEvent {
    pub fn new(partition_key: String, event_type: String, data: serde_json::Value) -> Self {
        Self {
            id: 0, // Will be assigned by event store
            timestamp: SystemTime::now(),
            partition_key,
            event_type,
            data,
        }
    }

    pub fn partition_key(&self) -> String {
        self.partition_key.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: SnapshotId,
    pub timestamp: SystemTime,
    pub event_id: EventId,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoredState<S> {
    pub state: S,
    pub snapshot_id: SnapshotId,
    pub events_to_replay: usize,
}

pub trait AgentState: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync {
    fn apply_event(&mut self, event: &StateEvent);
    fn last_event_id(&self) -> EventId;
    fn events_since_snapshot(&self) -> usize;
    fn time_since_snapshot(&self) -> std::time::Duration;
    fn merge_partition(&mut self, partition: Self);
}

// Example implementation of AgentState
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleState {
    pub data: std::collections::HashMap<String, serde_json::Value>,
    pub last_event_id: EventId,
    pub event_count: usize,
    pub last_snapshot_time: SystemTime,
}

impl Default for ExampleState {
    fn default() -> Self {
        Self {
            data: std::collections::HashMap::new(),
            last_event_id: 0,
            event_count: 0,
            last_snapshot_time: SystemTime::now(),
        }
    }
}

impl AgentState for ExampleState {
    fn apply_event(&mut self, event: &StateEvent) {
        self.data
            .insert(event.partition_key.clone(), event.data.clone());
        self.last_event_id = event.id;
        self.event_count += 1;
    }

    fn last_event_id(&self) -> EventId {
        self.last_event_id
    }

    fn events_since_snapshot(&self) -> usize {
        self.event_count
    }

    fn time_since_snapshot(&self) -> std::time::Duration {
        SystemTime::now()
            .duration_since(self.last_snapshot_time)
            .unwrap_or_default()
    }

    fn merge_partition(&mut self, partition: Self) {
        for (key, value) in partition.data {
            self.data.insert(key, value);
        }
        self.last_event_id = self.last_event_id.max(partition.last_event_id);
        self.event_count += partition.event_count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_event_new() {
        let event = StateEvent::new(
            "partition1".to_string(),
            "test_type".to_string(),
            serde_json::json!({"key": "value"}),
        );

        assert_eq!(event.id, 0); // Assigned by store
        assert_eq!(event.partition_key, "partition1");
        assert_eq!(event.event_type, "test_type");
    }

    #[test]
    fn test_state_event_partition_key() {
        let event = StateEvent::new(
            "my_partition".to_string(),
            "event".to_string(),
            serde_json::json!({}),
        );

        assert_eq!(event.partition_key(), "my_partition");
    }

    #[test]
    fn test_state_event_clone() {
        let event = StateEvent::new(
            "partition".to_string(),
            "type".to_string(),
            serde_json::json!({"data": 123}),
        );

        let cloned = event.clone();
        assert_eq!(cloned.partition_key, event.partition_key);
        assert_eq!(cloned.event_type, event.event_type);
    }

    #[test]
    fn test_snapshot_creation() {
        let snapshot = Snapshot {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            event_id: 42,
            checksum: "abc123".to_string(),
        };

        assert_eq!(snapshot.event_id, 42);
        assert_eq!(snapshot.checksum, "abc123");
    }

    #[test]
    fn test_snapshot_clone() {
        let snapshot = Snapshot {
            id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            event_id: 100,
            checksum: "checksum".to_string(),
        };

        let cloned = snapshot.clone();
        assert_eq!(cloned.id, snapshot.id);
        assert_eq!(cloned.event_id, snapshot.event_id);
    }

    #[test]
    fn test_restored_state_creation() {
        let state = ExampleState::default();
        let restored: RestoredState<ExampleState> = RestoredState {
            state,
            snapshot_id: Uuid::new_v4(),
            events_to_replay: 10,
        };

        assert_eq!(restored.events_to_replay, 10);
    }

    #[test]
    fn test_example_state_default() {
        let state = ExampleState::default();

        assert!(state.data.is_empty());
        assert_eq!(state.last_event_id, 0);
        assert_eq!(state.event_count, 0);
    }

    #[test]
    fn test_example_state_apply_event() {
        let mut state = ExampleState::default();
        let mut event = StateEvent::new(
            "key1".to_string(),
            "update".to_string(),
            serde_json::json!({"value": 42}),
        );
        event.id = 5;

        state.apply_event(&event);

        assert_eq!(state.last_event_id, 5);
        assert_eq!(state.event_count, 1);
        assert!(state.data.contains_key("key1"));
    }

    #[test]
    fn test_example_state_last_event_id() {
        let mut state = ExampleState::default();
        let mut event = StateEvent::new(
            "p".to_string(),
            "t".to_string(),
            serde_json::json!({}),
        );
        event.id = 123;

        state.apply_event(&event);
        assert_eq!(state.last_event_id(), 123);
    }

    #[test]
    fn test_example_state_events_since_snapshot() {
        let mut state = ExampleState::default();

        for i in 0..5 {
            let mut event = StateEvent::new(
                format!("p{}", i),
                "t".to_string(),
                serde_json::json!({}),
            );
            event.id = i;
            state.apply_event(&event);
        }

        assert_eq!(state.events_since_snapshot(), 5);
    }

    #[test]
    fn test_example_state_time_since_snapshot() {
        let state = ExampleState::default();
        let duration = state.time_since_snapshot();

        // Should be very small, just created
        assert!(duration.as_secs() < 1);
    }

    #[test]
    fn test_example_state_merge_partition() {
        let mut state1 = ExampleState::default();
        state1.data.insert("key1".to_string(), serde_json::json!(1));
        state1.last_event_id = 10;
        state1.event_count = 5;

        let mut state2 = ExampleState::default();
        state2.data.insert("key2".to_string(), serde_json::json!(2));
        state2.last_event_id = 20;
        state2.event_count = 3;

        state1.merge_partition(state2);

        assert_eq!(state1.data.len(), 2);
        assert_eq!(state1.last_event_id, 20);
        assert_eq!(state1.event_count, 8);
    }

    #[test]
    fn test_example_state_clone() {
        let mut state = ExampleState::default();
        state.data.insert("k".to_string(), serde_json::json!({"x": 1}));
        state.last_event_id = 50;

        let cloned = state.clone();
        assert_eq!(cloned.last_event_id, 50);
        assert!(cloned.data.contains_key("k"));
    }

    #[test]
    fn test_state_event_serialization() {
        let event = StateEvent::new(
            "p".to_string(),
            "t".to_string(),
            serde_json::json!({"value": 42}),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: StateEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.partition_key, "p");
        assert_eq!(deserialized.event_type, "t");
    }

    #[test]
    fn test_example_state_serialization() {
        let mut state = ExampleState::default();
        state.data.insert("key".to_string(), serde_json::json!(123));
        state.last_event_id = 99;

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: ExampleState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.last_event_id, 99);
    }
}
