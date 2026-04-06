#![cfg_attr(coverage_nightly, coverage(off))]
use crate::state::*;
use crc32fast::Hasher;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

use super::persistence::EventPersistence;
use super::store::EventStoreError;

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
// JsonFilePersistence Implementation (Uses serde_json - no bincode limitation!)
// ============================================================================

impl JsonFilePersistence {
    pub async fn new(file_path: &str) -> Result<Self, EventStoreError> {
        debug_assert!(!file_path.is_empty(), "file_path must not be empty");
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
    pub(super) fn serialize_event(event: &StateEvent) -> Result<String, EventStoreError> {
        let json = serde_json::to_string(event)
            .map_err(|e| EventStoreError::SerializationError(e.to_string()))?;

        let mut hasher = Hasher::new();
        hasher.update(json.as_bytes());
        let checksum = hasher.finalize();

        Ok(format!("{}\t{}\n", json, checksum))
    }

    /// Deserialize an event from a line (JSON\tCHECKSUM format).
    fn deserialize_line(line: &str) -> Result<StateEvent, EventStoreError> {
        debug_assert!(!line.is_empty(), "line must not be empty");
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
