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
