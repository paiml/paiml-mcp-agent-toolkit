use super::*;
use crc32fast::Hasher;
use parking_lot::RwLock;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

// ============================================================================
// Persistence Trait (Batuta Pattern: Trait Abstraction for Testability)
// ============================================================================

/// Trait for event persistence backends.
/// Enables testing with InMemoryPersistence and production with JsonFilePersistence.
#[async_trait::async_trait]
pub trait EventPersistence: Send + Sync {
    /// Append a single event to the persistence layer
    async fn append_event(&self, event: &StateEvent) -> Result<(), EventStoreError>;

    /// Append multiple events in a batch
    async fn append_batch(&self, events: &[StateEvent]) -> Result<(), EventStoreError>;

    /// Load all events from the persistence layer
    async fn load_all(&self) -> Result<Vec<StateEvent>, EventStoreError>;

    /// Compact the event log (rewrite with only current events)
    async fn compact(&self, events: &BTreeMap<EventId, StateEvent>) -> Result<(), EventStoreError>;
}

// ============================================================================
// In-Memory Persistence (For Testing - No I/O)
// ============================================================================

/// In-memory persistence backend for testing.
/// Stores events in a Vec, no file I/O required.
#[derive(Default)]
pub struct InMemoryPersistence {
    events: RwLock<Vec<StateEvent>>,
}

impl InMemoryPersistence {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of persisted events (for testing)
    pub fn len(&self) -> usize {
        self.events.read().len()
    }

    /// Check if empty (for testing)
    pub fn is_empty(&self) -> bool {
        self.events.read().is_empty()
    }

    /// Clear all persisted events (for testing)
    pub fn clear(&self) {
        self.events.write().clear();
    }
}

#[async_trait::async_trait]
impl EventPersistence for InMemoryPersistence {
    async fn append_event(&self, event: &StateEvent) -> Result<(), EventStoreError> {
        self.events.write().push(event.clone());
        Ok(())
    }

    async fn append_batch(&self, events: &[StateEvent]) -> Result<(), EventStoreError> {
        self.events.write().extend(events.iter().cloned());
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<StateEvent>, EventStoreError> {
        Ok(self.events.read().clone())
    }

    async fn compact(&self, events: &BTreeMap<EventId, StateEvent>) -> Result<(), EventStoreError> {
        let mut persisted = self.events.write();
        persisted.clear();
        persisted.extend(events.values().cloned());
        Ok(())
    }
}

// ============================================================================
// JSON File Persistence (Production - Uses serde_json instead of bincode)
// ============================================================================

/// JSON-based file persistence backend.
/// Uses newline-delimited JSON (NDJSON) format with CRC32 checksums.
/// Solves the bincode serialization issue with serde_json::Value.
pub struct JsonFilePersistence {
    log_file: Arc<tokio::sync::RwLock<File>>,
    file_path: String,
}

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

// ============================================================================
// JsonFilePersistence Implementation (Uses serde_json - no bincode limitation!)
// ============================================================================

impl JsonFilePersistence {
    pub async fn new(file_path: &str) -> Result<Self, EventStoreError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(file_path)
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        Ok(Self {
            log_file: Arc::new(tokio::sync::RwLock::new(file)),
            file_path: file_path.to_string(),
        })
    }

    /// Serialize an event to JSON with CRC32 checksum.
    /// Format: JSON\tCHECKSUM\n (tab-separated, newline-terminated)
    fn serialize_event(event: &StateEvent) -> Result<String, EventStoreError> {
        let json = serde_json::to_string(event)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        let mut hasher = Hasher::new();
        hasher.update(json.as_bytes());
        let checksum = hasher.finalize();

        Ok(format!("{}\t{}\n", json, checksum))
    }

    /// Deserialize an event from a line (JSON\tCHECKSUM format).
    fn deserialize_line(line: &str) -> Result<StateEvent, EventStoreError> {
        let parts: Vec<&str> = line.rsplitn(2, '\t').collect();
        if parts.len() != 2 {
            return Err(EventStoreError::CorruptedData(
                "Invalid line format: missing checksum".to_string(),
            ));
        }

        let checksum_str = parts[0].trim();
        let json = parts[1];

        // Verify checksum
        let expected_checksum: u32 = checksum_str
            .parse()
            .map_err(|_| EventStoreError::CorruptedData("Invalid checksum format".to_string()))?;

        let mut hasher = Hasher::new();
        hasher.update(json.as_bytes());
        let actual_checksum = hasher.finalize();

        if expected_checksum != actual_checksum {
            return Err(EventStoreError::CorruptedData(format!(
                "Checksum mismatch: expected {}, got {}",
                expected_checksum, actual_checksum
            )));
        }

        // Deserialize JSON (works with serde_json::Value!)
        serde_json::from_str(json).map_err(|e| EventStoreError::SerializationError(e.to_string()))
    }
}

#[async_trait::async_trait]
impl EventPersistence for JsonFilePersistence {
    async fn append_event(&self, event: &StateEvent) -> Result<(), EventStoreError> {
        let line = Self::serialize_event(event)?;

        let mut file = self.log_file.write().await;
        file.write_all(line.as_bytes())
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;
        file.flush()
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    async fn append_batch(&self, events: &[StateEvent]) -> Result<(), EventStoreError> {
        let mut buffer = String::new();

        for event in events {
            buffer.push_str(&Self::serialize_event(event)?);
        }

        let mut file = self.log_file.write().await;
        file.write_all(buffer.as_bytes())
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;
        file.flush()
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<StateEvent>, EventStoreError> {
        use tokio::io::{AsyncBufReadExt, BufReader};

        // Open a fresh file handle for reading (more reliable than seeking)
        let file = tokio::fs::File::open(&self.file_path)
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut events = Vec::new();

        while let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| EventStoreError::PersistenceError(e.to_string()))?
        {
            if line.trim().is_empty() {
                continue;
            }
            events.push(Self::deserialize_line(&line)?);
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
            let line = Self::serialize_event(event)?;
            temp_file
                .write_all(line.as_bytes())
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_store_config_default() {
        let config = EventStoreConfig::default();
        assert_eq!(config.max_events_in_memory, 100_000);
        assert_eq!(config.compaction_threshold, 10_000);
        assert!(config.persistence_enabled);
        assert!(!config.sync_writes);
        assert_eq!(config.batch_size, 1000);
    }

    #[test]
    fn test_event_store_config_clone() {
        let config = EventStoreConfig {
            max_events_in_memory: 5000,
            compaction_threshold: 1000,
            persistence_enabled: false,
            sync_writes: true,
            batch_size: 100,
        };
        let cloned = config.clone();
        assert_eq!(cloned.max_events_in_memory, 5000);
        assert_eq!(cloned.compaction_threshold, 1000);
        assert!(!cloned.persistence_enabled);
        assert!(cloned.sync_writes);
    }

    #[test]
    fn test_compaction_result_default() {
        let result = CompactionResult::default();
        assert_eq!(result.events_before, 0);
        assert_eq!(result.events_after, 0);
        assert_eq!(result.bytes_saved, 0);
        assert_eq!(result.duration, std::time::Duration::ZERO);
    }

    #[test]
    fn test_event_store_stats_clone() {
        let stats = EventStoreStats {
            total_events: 100,
            total_partitions: 5,
            next_event_id: 101,
            memory_usage_bytes: 50000,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.total_events, 100);
        assert_eq!(cloned.total_partitions, 5);
        assert_eq!(cloned.next_event_id, 101);
        assert_eq!(cloned.memory_usage_bytes, 50000);
    }

    #[test]
    fn test_event_store_error_display() {
        let persistence_err = EventStoreError::PersistenceError("disk full".to_string());
        assert!(persistence_err.to_string().contains("disk full"));

        let serialization_err = EventStoreError::SerializationError("invalid format".to_string());
        assert!(serialization_err.to_string().contains("invalid format"));

        let corrupted_err = EventStoreError::CorruptedData("checksum mismatch".to_string());
        assert!(corrupted_err.to_string().contains("checksum mismatch"));

        let not_found_err = EventStoreError::EventNotFound(42);
        assert!(not_found_err.to_string().contains("42"));
    }

    #[test]
    fn test_estimate_memory_usage() {
        let events: BTreeMap<EventId, StateEvent> = BTreeMap::new();
        let usage = estimate_memory_usage(&events);
        assert_eq!(usage, 0);

        let mut events_map: BTreeMap<EventId, StateEvent> = BTreeMap::new();
        events_map.insert(
            1,
            StateEvent::new("p".to_string(), "e".to_string(), serde_json::json!({})),
        );
        let usage2 = estimate_memory_usage(&events_map);
        assert!(usage2 > 0);
    }

    #[actix_rt::test]
    async fn test_event_append_and_retrieve() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.expect("internal error");

        let event = StateEvent::new(
            "partition1".to_string(),
            "test_event".to_string(),
            serde_json::json!({"data": "test"}),
        );

        let id = store.append(event.clone()).await.expect("internal error");
        assert_eq!(id, 1);

        let retrieved = store.get_event(id).expect("internal error");
        assert_eq!(retrieved.partition_key, "partition1");
        assert_eq!(retrieved.event_type, "test_event");
    }

    #[actix_rt::test]
    async fn test_batch_append() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.expect("internal error");

        let events = vec![
            StateEvent::new("p1".to_string(), "e1".to_string(), serde_json::json!({})),
            StateEvent::new("p1".to_string(), "e2".to_string(), serde_json::json!({})),
            StateEvent::new("p2".to_string(), "e3".to_string(), serde_json::json!({})),
        ];

        let ids = store.append_batch(events).await.expect("internal error");
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
        let store = EventStore::new(config).await.expect("internal error");

        for i in 0..10 {
            let event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({"index": i}),
            );
            store.append(event).await.expect("internal error");
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
        let store = EventStore::new(config).await.expect("internal error");

        for i in 0..10 {
            let event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({"index": i}),
            );
            store.append(event).await.expect("internal error");
        }

        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 5); // Only 5 events in memory
        assert_eq!(stats.next_event_id, 11); // But ID counter continues
    }

    #[actix_rt::test]
    async fn test_get_latest_event_id() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.expect("internal error");

        // Initially should be 0 (1 - 1)
        let initial_id = store.get_latest_event_id();
        assert_eq!(initial_id, 0);

        // Add some events
        for i in 0..5 {
            let event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({}),
            );
            store.append(event).await.expect("internal error");
        }

        let latest_id = store.get_latest_event_id();
        assert_eq!(latest_id, 5);
    }

    #[actix_rt::test]
    async fn test_get_partition_events_with_since() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.expect("internal error");

        for i in 0..5 {
            let event = StateEvent::new(
                "my_partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({"index": i}),
            );
            store.append(event).await.expect("internal error");
        }

        // Get events since ID 2
        let events = store.get_partition_events("my_partition", Some(2));
        assert_eq!(events.len(), 3); // IDs 3, 4, 5
    }

    #[actix_rt::test]
    async fn test_get_partition_events_nonexistent_partition() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.expect("internal error");

        let events = store.get_partition_events("nonexistent", None);
        assert!(events.is_empty());
    }

    #[actix_rt::test]
    async fn test_get_event_nonexistent() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.expect("internal error");

        let event = store.get_event(999);
        assert!(event.is_none());
    }

    #[actix_rt::test]
    async fn test_get_events_since_no_limit() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.expect("internal error");

        for i in 0..5 {
            let event = StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({}));
            store.append(event).await.expect("internal error");
        }

        // Get all events since ID 2 with no limit
        let events = store.get_events_since(2, None);
        assert_eq!(events.len(), 3); // IDs 3, 4, 5
    }

    #[actix_rt::test]
    async fn test_compact_no_persistence() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.expect("internal error");

        let result = store.compact().await.expect("internal error");
        assert_eq!(result.events_before, 0);
        assert_eq!(result.events_after, 0);
    }

    #[actix_rt::test]
    async fn test_batch_append_memory_limit() {
        let config = EventStoreConfig {
            max_events_in_memory: 3,
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.expect("internal error");

        let events: Vec<StateEvent> = (0..10)
            .map(|i| StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({})))
            .collect();

        let ids = store.append_batch(events).await.expect("internal error");
        assert_eq!(ids.len(), 10);

        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 3); // Only 3 events retained
    }

    #[actix_rt::test]
    async fn test_get_statistics() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store = EventStore::new(config).await.expect("internal error");

        // Add events to different partitions
        store
            .append(StateEvent::new(
                "p1".to_string(),
                "e1".to_string(),
                serde_json::json!({}),
            ))
            .await
            .unwrap();
        store
            .append(StateEvent::new(
                "p1".to_string(),
                "e2".to_string(),
                serde_json::json!({}),
            ))
            .await
            .unwrap();
        store
            .append(StateEvent::new(
                "p2".to_string(),
                "e3".to_string(),
                serde_json::json!({}),
            ))
            .await
            .unwrap();

        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.total_partitions, 2);
        assert_eq!(stats.next_event_id, 4);
        assert!(stats.memory_usage_bytes > 0);
    }

    // ===== InMemoryPersistence Tests =====

    #[test]
    fn test_in_memory_persistence_new() {
        let persistence = InMemoryPersistence::new();
        assert!(persistence.is_empty());
        assert_eq!(persistence.len(), 0);
    }

    #[actix_rt::test]
    async fn test_in_memory_persistence_append_event() {
        let persistence = InMemoryPersistence::new();

        let event = StateEvent::new(
            "test_partition".to_string(),
            "test_type".to_string(),
            serde_json::json!({"key": "value"}),
        );

        let result = persistence.append_event(&event).await;
        assert!(result.is_ok());
        assert_eq!(persistence.len(), 1);
    }

    #[actix_rt::test]
    async fn test_in_memory_persistence_append_batch() {
        let persistence = InMemoryPersistence::new();

        let events: Vec<StateEvent> = (0..5)
            .map(|i| {
                StateEvent::new(
                    format!("partition_{}", i),
                    format!("type_{}", i),
                    serde_json::json!({"index": i}),
                )
            })
            .collect();

        let result = persistence.append_batch(&events).await;
        assert!(result.is_ok());
        assert_eq!(persistence.len(), 5);
    }

    #[actix_rt::test]
    async fn test_in_memory_persistence_load_all() {
        let persistence = InMemoryPersistence::new();

        // Append multiple events
        for i in 0..3 {
            let event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({"data": i}),
            );
            persistence.append_event(&event).await.unwrap();
        }

        // Load all events
        let loaded = persistence.load_all().await.unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].event_type, "event_0");
        assert_eq!(loaded[1].event_type, "event_1");
        assert_eq!(loaded[2].event_type, "event_2");
    }

    #[actix_rt::test]
    async fn test_in_memory_persistence_compact() {
        let persistence = InMemoryPersistence::new();

        // Add some events
        let mut events_map = BTreeMap::new();
        for i in 1..=5 {
            let mut event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({}),
            );
            event.id = i;
            events_map.insert(i, event.clone());
            persistence.append_event(&event).await.unwrap();
        }

        // Compact (should replace all events with provided map)
        let result = persistence.compact(&events_map).await;
        assert!(result.is_ok());

        // Load and verify
        let loaded = persistence.load_all().await.unwrap();
        assert_eq!(loaded.len(), 5);
    }

    #[actix_rt::test]
    async fn test_in_memory_persistence_clear() {
        let persistence = InMemoryPersistence::new();

        for i in 0..3 {
            let event = StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({}));
            persistence.append_event(&event).await.unwrap();
        }

        assert_eq!(persistence.len(), 3);
        persistence.clear();
        assert!(persistence.is_empty());
    }

    // ===== JsonFilePersistence Tests (Now work with serde_json!) =====

    #[actix_rt::test]
    async fn test_json_file_persistence_new() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_events.log");
        let persistence = JsonFilePersistence::new(file_path.to_str().unwrap()).await;
        assert!(persistence.is_ok());
    }

    #[actix_rt::test]
    async fn test_json_file_persistence_append_event() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_events.log");
        let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
            .await
            .unwrap();

        let event = StateEvent::new(
            "test_partition".to_string(),
            "test_type".to_string(),
            serde_json::json!({"key": "value"}),
        );

        let result = persistence.append_event(&event).await;
        assert!(result.is_ok());

        // Verify file was written
        let metadata = std::fs::metadata(&file_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[actix_rt::test]
    async fn test_json_file_persistence_append_batch() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_batch.log");
        let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
            .await
            .unwrap();

        let events: Vec<StateEvent> = (0..5)
            .map(|i| {
                StateEvent::new(
                    format!("partition_{}", i),
                    format!("type_{}", i),
                    serde_json::json!({"index": i}),
                )
            })
            .collect();

        let result = persistence.append_batch(&events).await;
        assert!(result.is_ok());

        // Load and verify
        let loaded = persistence.load_all().await.unwrap();
        assert_eq!(loaded.len(), 5);
    }

    #[actix_rt::test]
    async fn test_json_file_persistence_load_all() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_load.log");
        let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
            .await
            .unwrap();

        // Append multiple events
        for i in 0..3 {
            let event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({"data": i}),
            );
            persistence.append_event(&event).await.unwrap();
        }

        // Load all events
        let loaded = persistence.load_all().await.unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].event_type, "event_0");
        assert_eq!(loaded[1].event_type, "event_1");
        assert_eq!(loaded[2].event_type, "event_2");
    }

    #[actix_rt::test]
    async fn test_json_file_persistence_load_empty_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.log");
        let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
            .await
            .unwrap();

        let loaded = persistence.load_all().await.unwrap();
        assert!(loaded.is_empty());
    }

    #[actix_rt::test]
    async fn test_json_file_persistence_compact() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("compact.log");
        let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
            .await
            .unwrap();

        // Add some events
        let mut events_map = BTreeMap::new();
        for i in 1..=5 {
            let mut event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({}),
            );
            event.id = i;
            events_map.insert(i, event.clone());
            persistence.append_event(&event).await.unwrap();
        }

        // Compact
        let result = persistence.compact(&events_map).await;
        assert!(result.is_ok());

        // Load and verify
        let loaded = persistence.load_all().await.unwrap();
        assert_eq!(loaded.len(), 5);
    }

    #[actix_rt::test]
    async fn test_json_file_serialize_deserialize_roundtrip() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("roundtrip.log");
        let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
            .await
            .unwrap();

        // Create event with complex JSON data (this failed with bincode!)
        let event = StateEvent::new(
            "test_partition".to_string(),
            "complex_event".to_string(),
            serde_json::json!({
                "nested": {
                    "array": [1, 2, 3],
                    "object": {"key": "value"},
                    "number": 42,
                    "boolean": true,
                    "null": null
                }
            }),
        );

        persistence.append_event(&event).await.unwrap();
        let loaded = persistence.load_all().await.unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].event_type, "complex_event");
        assert_eq!(loaded[0].data["nested"]["array"][0], 1);
        assert_eq!(loaded[0].data["nested"]["object"]["key"], "value");
    }

    // ===== EventStore with InMemoryPersistence Tests =====

    #[actix_rt::test]
    async fn test_event_store_with_in_memory_persistence() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let config = EventStoreConfig {
            persistence_enabled: true,
            ..Default::default()
        };
        let store = EventStore::new_with_persistence(config, Some(persistence.clone()));

        // Add events
        for i in 0..3 {
            let event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({"index": i}),
            );
            store.append(event).await.unwrap();
        }

        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 3);
        assert_eq!(persistence.len(), 3);
    }

    #[actix_rt::test]
    async fn test_event_store_batch_with_in_memory_persistence() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let config = EventStoreConfig {
            persistence_enabled: true,
            ..Default::default()
        };
        let store = EventStore::new_with_persistence(config, Some(persistence.clone()));

        let events: Vec<StateEvent> = (0..5)
            .map(|i| StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({})))
            .collect();

        let ids = store.append_batch(events).await.unwrap();
        assert_eq!(ids.len(), 5);
        assert_eq!(persistence.len(), 5);
    }

    #[actix_rt::test]
    async fn test_event_store_compact_with_in_memory_persistence() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let config = EventStoreConfig {
            persistence_enabled: true,
            ..Default::default()
        };
        let store = EventStore::new_with_persistence(config, Some(persistence.clone()));

        // Add events
        for i in 0..10 {
            let event = StateEvent::new(
                "partition".to_string(),
                format!("event_{}", i),
                serde_json::json!({}),
            );
            store.append(event).await.unwrap();
        }

        // Compact
        let result = store.compact().await.unwrap();
        assert_eq!(result.events_before, 10);
        assert_eq!(result.events_after, 10);
    }

    #[actix_rt::test]
    async fn test_event_store_recovery_with_in_memory_persistence() {
        let persistence = Arc::new(InMemoryPersistence::new());

        // First store - create events
        {
            let config = EventStoreConfig {
                persistence_enabled: true,
                ..Default::default()
            };
            let store = EventStore::new_with_persistence(config, Some(persistence.clone()));

            for i in 0..5 {
                let event = StateEvent::new(
                    "partition".to_string(),
                    format!("event_{}", i),
                    serde_json::json!({"index": i}),
                );
                store.append(event).await.unwrap();
            }
        }

        // Second store - recover events (same persistence instance)
        {
            let config = EventStoreConfig {
                persistence_enabled: true,
                ..Default::default()
            };
            let mut store = EventStore::new_with_persistence(config, Some(persistence.clone()));
            store.recover().await.unwrap();

            let stats = store.get_statistics();
            assert_eq!(stats.total_events, 5);
            assert_eq!(stats.next_event_id, 6); // Should continue from 6

            // Verify events can be retrieved
            let event = store.get_event(1);
            assert!(event.is_some());
            assert_eq!(event.unwrap().event_type, "event_0");
        }
    }

    #[test]
    fn test_event_store_error_debug() {
        let err = EventStoreError::PersistenceError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("PersistenceError"));
    }

    #[test]
    fn test_compaction_result_debug() {
        let result = CompactionResult {
            events_before: 100,
            events_after: 50,
            bytes_saved: 1024,
            duration: std::time::Duration::from_secs(1),
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("events_before"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_event_store_stats_debug() {
        let stats = EventStoreStats {
            total_events: 100,
            total_partitions: 5,
            next_event_id: 101,
            memory_usage_bytes: 50000,
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("total_events"));
        assert!(debug_str.contains("100"));
    }

    // ===== Additional Comprehensive Tests =====

    #[actix_rt::test]
    async fn test_event_store_no_persistence_backend() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store: EventStore<InMemoryPersistence> = EventStore::new_with_persistence(config, None);

        let event = StateEvent::new(
            "partition".to_string(),
            "event_type".to_string(),
            serde_json::json!({}),
        );

        let id = store.append(event).await.unwrap();
        assert_eq!(id, 1);

        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 1);
    }

    #[actix_rt::test]
    async fn test_event_store_get_events_since() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let config = EventStoreConfig::default();
        let store = EventStore::new_with_persistence(config, Some(persistence));

        // Add 5 events
        for i in 0..5 {
            let event = StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({}));
            store.append(event).await.unwrap();
        }

        // Get events since ID 2 (should get events 3, 4, 5)
        let events = store.get_events_since(2, None);
        assert_eq!(events.len(), 3);

        // Get events since ID 2 with limit
        let events_limited = store.get_events_since(2, Some(2));
        assert_eq!(events_limited.len(), 2);
    }

    #[actix_rt::test]
    async fn test_event_store_get_partition_events() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let config = EventStoreConfig::default();
        let store = EventStore::new_with_persistence(config, Some(persistence));

        // Add events to different partitions
        for i in 0..5 {
            let event = StateEvent::new(
                format!("partition_{}", i % 2),
                format!("e{}", i),
                serde_json::json!({}),
            );
            store.append(event).await.unwrap();
        }

        // Get events for partition_0
        let events_p0 = store.get_partition_events("partition_0", None);
        assert_eq!(events_p0.len(), 3); // indices 0, 2, 4

        // Get events for partition_1
        let events_p1 = store.get_partition_events("partition_1", None);
        assert_eq!(events_p1.len(), 2); // indices 1, 3

        // Get events for non-existent partition
        let events_none = store.get_partition_events("nonexistent", None);
        assert!(events_none.is_empty());

        // Get events with since filter
        let events_since = store.get_partition_events("partition_0", Some(2));
        assert_eq!(events_since.len(), 2); // only events with id > 2
    }

    #[actix_rt::test]
    async fn test_event_store_get_event() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let config = EventStoreConfig::default();
        let store = EventStore::new_with_persistence(config, Some(persistence));

        let event = StateEvent::new(
            "p".to_string(),
            "test_event".to_string(),
            serde_json::json!({"data": 42}),
        );
        let id = store.append(event).await.unwrap();

        // Get existing event
        let retrieved = store.get_event(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().event_type, "test_event");

        // Get non-existent event
        let missing = store.get_event(999);
        assert!(missing.is_none());
    }

    #[actix_rt::test]
    async fn test_event_store_get_latest_event_id() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let config = EventStoreConfig::default();
        let store = EventStore::new_with_persistence(config, Some(persistence));

        // Initially 0 (1 - 1)
        assert_eq!(store.get_latest_event_id(), 0);

        // After adding events
        for _ in 0..5 {
            let event = StateEvent::new("p".to_string(), "e".to_string(), serde_json::json!({}));
            store.append(event).await.unwrap();
        }

        assert_eq!(store.get_latest_event_id(), 5);
    }

    #[actix_rt::test]
    async fn test_event_store_memory_limit_enforcement() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let config = EventStoreConfig {
            max_events_in_memory: 5,
            ..Default::default()
        };
        let store = EventStore::new_with_persistence(config, Some(persistence));

        // Add more events than the limit
        for i in 0..10 {
            let event = StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({}));
            store.append(event).await.unwrap();
        }

        // Stats should show only max_events_in_memory events
        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 5);

        // Oldest events should be removed (events 1-5 removed, 6-10 remain)
        assert!(store.get_event(1).is_none());
        assert!(store.get_event(6).is_some());
    }

    #[actix_rt::test]
    async fn test_event_store_batch_memory_limit_enforcement() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let config = EventStoreConfig {
            max_events_in_memory: 3,
            ..Default::default()
        };
        let store = EventStore::new_with_persistence(config, Some(persistence));

        // Add batch that exceeds limit
        let events: Vec<StateEvent> = (0..5)
            .map(|i| StateEvent::new("p".to_string(), format!("e{}", i), serde_json::json!({})))
            .collect();

        store.append_batch(events).await.unwrap();

        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 3);
    }

    #[actix_rt::test]
    async fn test_event_store_compact_without_persistence() {
        let config = EventStoreConfig {
            persistence_enabled: false,
            ..Default::default()
        };
        let store: EventStore<InMemoryPersistence> = EventStore::new_with_persistence(config, None);

        // Compact should return default result
        let result = store.compact().await.unwrap();
        assert_eq!(result.events_before, 0);
        assert_eq!(result.events_after, 0);
    }

    #[test]
    fn test_compaction_result_defaults() {
        let result = CompactionResult::default();
        assert_eq!(result.events_before, 0);
        assert_eq!(result.events_after, 0);
        assert_eq!(result.bytes_saved, 0);
    }

    #[test]
    fn test_event_store_error_all_variants() {
        let ser_err = EventStoreError::SerializationError("invalid json".to_string());
        let persist_err = EventStoreError::PersistenceError("connection lost".to_string());
        let corrupt_err = EventStoreError::CorruptedData("bad format".to_string());
        let not_found_err = EventStoreError::EventNotFound(42);

        assert!(format!("{:?}", ser_err).contains("SerializationError"));
        assert!(format!("{:?}", persist_err).contains("PersistenceError"));
        assert!(format!("{:?}", corrupt_err).contains("CorruptedData"));
        assert!(format!("{:?}", not_found_err).contains("EventNotFound"));
    }

    #[actix_rt::test]
    async fn test_json_file_persistence_serialize_event() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("serialize.log");
        let _persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
            .await
            .unwrap();

        let event = StateEvent::new(
            "test".to_string(),
            "serialize_test".to_string(),
            serde_json::json!({"nested": {"key": "value"}}),
        );

        // Test serialization
        let serialized = JsonFilePersistence::serialize_event(&event).unwrap();
        assert!(serialized.contains("serialize_test"));
        assert!(serialized.contains('\t')); // Tab separator
        assert!(serialized.ends_with('\n')); // Newline at end
    }

    #[actix_rt::test]
    async fn test_event_store_statistics_multiple_partitions() {
        let persistence = Arc::new(InMemoryPersistence::new());
        let config = EventStoreConfig::default();
        let store = EventStore::new_with_persistence(config, Some(persistence));

        // Add events to multiple partitions
        for i in 0..10 {
            let event = StateEvent::new(
                format!("partition_{}", i % 3),
                format!("e{}", i),
                serde_json::json!({}),
            );
            store.append(event).await.unwrap();
        }

        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 10);
        assert_eq!(stats.total_partitions, 3);
        assert_eq!(stats.next_event_id, 11);
        assert!(stats.memory_usage_bytes > 0);
    }

    #[actix_rt::test]
    async fn test_json_file_persistence_handles_corrupt_line() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("corrupt.log");

        // Write a corrupt line directly to the file
        std::fs::write(&file_path, "corrupt line without tab\n").unwrap();

        let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
            .await
            .unwrap();
        let result = persistence.load_all().await;

        // Should return an error for corrupt lines
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(format!("{:?}", e).contains("CorruptedData"));
        }
    }

    #[actix_rt::test]
    async fn test_json_file_persistence_handles_checksum_mismatch() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bad_checksum.log");

        // Write a line with invalid checksum
        std::fs::write(&file_path, "{\"event_type\":\"test\"}\t999999999\n").unwrap();

        let persistence = JsonFilePersistence::new(file_path.to_str().unwrap())
            .await
            .unwrap();
        let result = persistence.load_all().await;

        // Should return an error for checksum mismatch
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(format!("{:?}", e).contains("CorruptedData"));
        }
    }
}
