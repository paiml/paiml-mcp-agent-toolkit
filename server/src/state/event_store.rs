use super::*;
use bincode::{deserialize, serialize};
use crc32fast::Hasher;
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

// Append-only event log with strong ordering guarantees
pub struct EventStore {
    events: Arc<RwLock<BTreeMap<EventId, StateEvent>>>,
    partitions: Arc<RwLock<HashMap<String, Vec<EventId>>>>,
    next_event_id: Arc<RwLock<EventId>>,
    persistence: Option<Arc<PersistenceLayer>>,
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

impl EventStore {
    pub async fn new(config: EventStoreConfig) -> Result<Self, EventStoreError> {
        let persistence = if config.persistence_enabled {
            Some(Arc::new(PersistenceLayer::new("events.log").await?))
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
            store.recover_from_disk().await?;
        }

        Ok(store)
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

        // Persist to disk
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

        // Persist batch to disk
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
        let persistence = self.persistence.as_ref().unwrap();
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

    async fn recover_from_disk(&mut self) -> Result<(), EventStoreError> {
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

// Persistence layer for durability
struct PersistenceLayer {
    log_file: Arc<RwLock<File>>,
    file_path: String,
}

impl PersistenceLayer {
    async fn new(file_path: &str) -> Result<Self, EventStoreError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(file_path)
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        Ok(Self {
            log_file: Arc::new(RwLock::new(file)),
            file_path: file_path.to_string(),
        })
    }

    #[allow(clippy::await_holding_lock)]
    async fn append_event(&self, event: &StateEvent) -> Result<(), EventStoreError> {
        let serialized =
            serialize(event).map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        let mut hasher = Hasher::new();
        hasher.update(&serialized);
        let checksum = hasher.finalize();

        let mut file = self.log_file.write();

        // Write length, checksum, and data
        file.write_all(&(serialized.len() as u32).to_le_bytes())
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;
        file.write_all(&checksum.to_le_bytes())
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;
        file.write_all(&serialized)
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        file.flush()
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    #[allow(clippy::await_holding_lock)]
    async fn append_batch(&self, events: &[StateEvent]) -> Result<(), EventStoreError> {
        let mut buffer = Vec::new();

        for event in events {
            let serialized =
                serialize(event).map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

            let mut hasher = Hasher::new();
            hasher.update(&serialized);
            let checksum = hasher.finalize();

            buffer.extend_from_slice(&(serialized.len() as u32).to_le_bytes());
            buffer.extend_from_slice(&checksum.to_le_bytes());
            buffer.extend_from_slice(&serialized);
        }

        let mut file = self.log_file.write();
        file.write_all(&buffer)
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;
        file.flush()
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    #[allow(clippy::await_holding_lock)]
    async fn load_all(&self) -> Result<Vec<StateEvent>, EventStoreError> {
        let mut file = self.log_file.write();
        file.seek(std::io::SeekFrom::Start(0))
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        let mut events = Vec::new();
        let mut length_buf = [0u8; 4];
        let mut checksum_buf = [0u8; 4];

        loop {
            // Read length
            match file.read_exact(&mut length_buf).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(EventStoreError::PersistenceError(e.to_string())),
            }
            let length = u32::from_le_bytes(length_buf) as usize;

            // Read checksum
            file.read_exact(&mut checksum_buf)
                .await
                .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;
            let expected_checksum = u32::from_le_bytes(checksum_buf);

            // Read data
            let mut data = vec![0u8; length];
            file.read_exact(&mut data)
                .await
                .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

            // Verify checksum
            let mut hasher = Hasher::new();
            hasher.update(&data);
            let actual_checksum = hasher.finalize();

            if expected_checksum != actual_checksum {
                return Err(EventStoreError::CorruptedData(format!(
                    "Checksum mismatch: expected {}, got {}",
                    expected_checksum, actual_checksum
                )));
            }

            // Deserialize event
            let event: StateEvent = deserialize(&data)
                .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;
            events.push(event);
        }

        Ok(events)
    }

    async fn compact(&self, events: &BTreeMap<EventId, StateEvent>) -> Result<(), EventStoreError> {
        let temp_path = format!("{}.compact", self.file_path);

        let mut temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        // Write all events to new file
        for event in events.values() {
            let serialized =
                serialize(event).map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

            let mut hasher = Hasher::new();
            hasher.update(&serialized);
            let checksum = hasher.finalize();

            temp_file
                .write_all(&(serialized.len() as u32).to_le_bytes())
                .await
                .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;
            temp_file
                .write_all(&checksum.to_le_bytes())
                .await
                .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;
            temp_file
                .write_all(&serialized)
                .await
                .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;
        }

        temp_file
            .flush()
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;
        drop(temp_file);

        // Atomic rename
        tokio::fs::rename(&temp_path, &self.file_path)
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        Ok(())
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

fn estimate_memory_usage(events: &BTreeMap<EventId, StateEvent>) -> usize {
    events.len() * std::mem::size_of::<(EventId, StateEvent)>()
}

#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Persistence error: {0}")]
    PersistenceError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Corrupted data: {0}")]
    CorruptedData(String),
    #[error("Event not found: {0}")]
    EventNotFound(EventId),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn test_event_append_and_retrieve() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.unwrap();

        let event = StateEvent::new(
            "partition1".to_string(),
            "test_event".to_string(),
            serde_json::json!({"data": "test"}),
        );

        let id = store.append(event.clone()).await.unwrap();
        assert_eq!(id, 1);

        let retrieved = store.get_event(id).unwrap();
        assert_eq!(retrieved.partition_key, "partition1");
        assert_eq!(retrieved.event_type, "test_event");
    }

    #[actix_rt::test]
    async fn test_batch_append() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.unwrap();

        let events = vec![
            StateEvent::new("p1".to_string(), "e1".to_string(), serde_json::json!({})),
            StateEvent::new("p1".to_string(), "e2".to_string(), serde_json::json!({})),
            StateEvent::new("p2".to_string(), "e3".to_string(), serde_json::json!({})),
        ];

        let ids = store.append_batch(events).await.unwrap();
        assert_eq!(ids, vec![1, 2, 3]);

        let p1_events = store.get_partition_events("p1", None);
        assert_eq!(p1_events.len(), 2);

        let p2_events = store.get_partition_events("p2", None);
        assert_eq!(p2_events.len(), 1);
    }

    #[actix_rt::test]
    async fn test_get_events_since() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.unwrap();

        for i in 0..10 {
            let event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({"index": i}),
            );
            store.append(event).await.unwrap();
        }

        let events = store.get_events_since(5, Some(3));
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].event_type, "event_5");
        assert_eq!(events[1].event_type, "event_6");
        assert_eq!(events[2].event_type, "event_7");
    }

    #[actix_rt::test]
    async fn test_memory_limit_enforcement() {
        let config = EventStoreConfig {
            max_events_in_memory: 5,
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.unwrap();

        for i in 0..10 {
            let event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({"index": i}),
            );
            store.append(event).await.unwrap();
        }

        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 5); // Only 5 events in memory
        assert_eq!(stats.next_event_id, 11); // But ID counter continues
    }
}
