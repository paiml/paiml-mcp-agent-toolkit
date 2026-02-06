#![cfg_attr(coverage_nightly, coverage(off))]
//! EventStore main implementation
//!
//! Append-only event log with strong ordering guarantees.

use super::*;
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

// ============================================================================
// EventStore (Main Implementation)
// ============================================================================

/// Append-only event log with strong ordering guarantees.
/// Uses trait-based persistence for testability (Batuta pattern).
pub struct EventStore<P: EventPersistence = JsonFilePersistence> {
    events: Arc<RwLock<BTreeMap<EventId, StateEvent>>>,
    partitions: Arc<RwLock<HashMap<String, Vec<EventId>>>>,
    next_event_id: Arc<RwLock<EventId>>,
    persistence: Option<Arc<P>>,
    config: EventStoreConfig,
}

#[derive(Clone)]
pub struct EventStoreConfig {
    pub max_events_in_memory: usize,
    pub compaction_threshold: usize,
    pub persistence_enabled: bool,
    pub sync_writes: bool,
    pub batch_size: usize,
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            max_events_in_memory: 100_000,
            compaction_threshold: 10_000,
            persistence_enabled: true,
            sync_writes: false,
            batch_size: 1000,
        }
    }
}

impl<P: EventPersistence> EventStore<P> {
    /// Create a new EventStore with a custom persistence backend.
    /// Use this for testing with InMemoryPersistence.
    pub fn new_with_persistence(config: EventStoreConfig, persistence: Option<Arc<P>>) -> Self {
        Self {
            events: Arc::new(RwLock::new(BTreeMap::new())),
            partitions: Arc::new(RwLock::new(HashMap::new())),
            next_event_id: Arc::new(RwLock::new(1)),
            persistence,
            config,
        }
    }

    /// Recover events from the persistence layer.
    /// Call this after creating the store if persistence is enabled.
    pub async fn recover(&mut self) -> Result<(), EventStoreError> {
        if let Some(persistence) = &self.persistence {
            let recovered = persistence.load_all().await?;

            let mut events = self.events.write();
            let mut partitions = self.partitions.write();
            let mut max_id = 0;

            for event in recovered {
                max_id = max_id.max(event.id);

                partitions
                    .entry(event.partition_key.clone())
                    .or_default()
                    .push(event.id);

                events.insert(event.id, event);
            }

            *self.next_event_id.write() = max_id + 1;
        }

        Ok(())
    }

    pub async fn append(&self, mut event: StateEvent) -> Result<EventId, EventStoreError> {
        // Assign event ID
        let event_id = {
            let mut next_id = self.next_event_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };
        event.id = event_id;
        event.timestamp = SystemTime::now();

        // Store in memory
        {
            let mut events = self.events.write();
            events.insert(event_id, event.clone());

            // Enforce memory limit
            if events.len() > self.config.max_events_in_memory {
                let to_remove = events.len() - self.config.max_events_in_memory;
                let keys_to_remove: Vec<_> = events.keys().take(to_remove).cloned().collect();
                for key in keys_to_remove {
                    events.remove(&key);
                }
            }
        }

        // Update partition index
        {
            let mut partitions = self.partitions.write();
            partitions
                .entry(event.partition_key.clone())
                .or_default()
                .push(event_id);
        }

        // Persist to storage
        if let Some(persistence) = &self.persistence {
            persistence.append_event(&event).await?;
        }

        Ok(event_id)
    }

    pub async fn append_batch(
        &self,
        events: Vec<StateEvent>,
    ) -> Result<Vec<EventId>, EventStoreError> {
        let mut ids = Vec::with_capacity(events.len());
        let mut persisted_events = Vec::with_capacity(events.len());

        // Assign IDs and store in memory
        {
            let mut events_map = self.events.write();
            let mut next_id = self.next_event_id.write();
            let mut partitions = self.partitions.write();

            for mut event in events {
                let event_id = *next_id;
                *next_id += 1;
                event.id = event_id;
                event.timestamp = SystemTime::now();

                events_map.insert(event_id, event.clone());
                partitions
                    .entry(event.partition_key.clone())
                    .or_default()
                    .push(event_id);

                ids.push(event_id);
                persisted_events.push(event);
            }

            // Enforce memory limit
            if events_map.len() > self.config.max_events_in_memory {
                let to_remove = events_map.len() - self.config.max_events_in_memory;
                let keys_to_remove: Vec<_> = events_map.keys().take(to_remove).cloned().collect();
                for key in keys_to_remove {
                    events_map.remove(&key);
                }
            }
        }

        // Persist batch to storage
        if let Some(persistence) = &self.persistence {
            persistence.append_batch(&persisted_events).await?;
        }

        Ok(ids)
    }

    pub fn get_events_since(&self, event_id: EventId, limit: Option<usize>) -> Vec<StateEvent> {
        let events = self.events.read();
        let iter = events.range((event_id + 1)..);

        if let Some(limit) = limit {
            iter.take(limit).map(|(_, e)| e.clone()).collect()
        } else {
            iter.map(|(_, e)| e.clone()).collect()
        }
    }

    pub fn get_partition_events(
        &self,
        partition_key: &str,
        since: Option<EventId>,
    ) -> Vec<StateEvent> {
        let partitions = self.partitions.read();
        let events = self.events.read();

        if let Some(event_ids) = partitions.get(partition_key) {
            event_ids
                .iter()
                .filter(|&&id| since.map_or(true, |s| id > s))
                .filter_map(|id| events.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_event(&self, event_id: EventId) -> Option<StateEvent> {
        self.events.read().get(&event_id).cloned()
    }

    pub fn get_latest_event_id(&self) -> EventId {
        *self.next_event_id.read() - 1
    }

    pub async fn compact(&self) -> Result<CompactionResult, EventStoreError> {
        if self.persistence.is_none() {
            return Ok(CompactionResult::default());
        }

        let events_before = self.events.read().len();
        let start_time = std::time::Instant::now();

        // Create new compacted file
        let persistence = self.persistence.as_ref().expect("internal error");
        let events = { self.events.read().clone() };
        persistence.compact(&events).await?;

        let events_after = self.events.read().len();
        let duration = start_time.elapsed();

        Ok(CompactionResult {
            events_before,
            events_after,
            bytes_saved: 0, // Would calculate from file sizes
            duration,
        })
    }

    pub fn get_statistics(&self) -> EventStoreStats {
        let events = self.events.read();
        let partitions = self.partitions.read();

        EventStoreStats {
            total_events: events.len(),
            total_partitions: partitions.len(),
            next_event_id: *self.next_event_id.read(),
            memory_usage_bytes: estimate_memory_usage(&events),
        }
    }
}

// Backward-compatible constructor for JsonFilePersistence
impl EventStore<JsonFilePersistence> {
    /// Create a new EventStore with file-based JSON persistence.
    /// This is the default production constructor.
    pub async fn new(config: EventStoreConfig) -> Result<Self, EventStoreError> {
        let persistence = if config.persistence_enabled {
            Some(Arc::new(JsonFilePersistence::new("events.log").await?))
        } else {
            None
        };

        let mut store = Self {
            events: Arc::new(RwLock::new(BTreeMap::new())),
            partitions: Arc::new(RwLock::new(HashMap::new())),
            next_event_id: Arc::new(RwLock::new(1)),
            persistence,
            config,
        };

        // Recover from persistent storage
        if store.config.persistence_enabled {
            store.recover().await?;
        }

        Ok(store)
    }
}

#[derive(Debug, Default)]
pub struct CompactionResult {
    pub events_before: usize,
    pub events_after: usize,
    pub bytes_saved: usize,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct EventStoreStats {
    pub total_events: usize,
    pub total_partitions: usize,
    pub next_event_id: EventId,
    pub memory_usage_bytes: usize,
}

pub fn estimate_memory_usage(events: &BTreeMap<EventId, StateEvent>) -> usize {
    events.len() * std::mem::size_of::<(EventId, StateEvent)>()
}
