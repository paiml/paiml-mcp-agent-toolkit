/// Storage backend abstraction for flexible persistence options
///
/// Supports multiple backend implementations:
/// - Sled: Embedded database with good performance (default)
/// - RocksDB: Facebook's embedded database with excellent performance
/// - InMemory: Fast testing and development backend
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Type alias for key-value pair
pub type KeyValuePair = (Vec<u8>, Vec<u8>);

/// Type alias for storage iterator
pub type StorageIterator<'a> = Box<dyn Iterator<Item = Result<KeyValuePair>> + 'a>;

/// Trait for storage backend implementations
pub trait StorageBackend: Send + Sync {
    /// Store a key-value pair
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;

    /// Retrieve a value by key
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Delete a key-value pair
    fn delete(&self, key: &[u8]) -> Result<()>;

    /// Check if a key exists
    fn contains(&self, key: &[u8]) -> Result<bool>;

    /// Iterate over all key-value pairs
    fn iter(&self) -> Result<StorageIterator<'_>>;

    /// Get approximate size in bytes
    fn size_on_disk(&self) -> Result<u64>;

    /// Flush any pending writes
    fn flush(&self) -> Result<()>;

    /// Clear all data
    fn clear(&self) -> Result<()>;

    /// Get backend name for diagnostics
    fn backend_name(&self) -> &'static str;

    /// Get backend-specific statistics
    fn get_stats(&self) -> HashMap<String, String>;
}

/// Sled database backend
pub struct SledBackend {
    db: sled::Db,
    tree: sled::Tree,
}

impl SledBackend {
    pub fn new(path: &Path) -> Result<Self> {
        let db = sled::open(path)?;
        let tree = db.open_tree("tdg_storage")?;
        Ok(Self { db, tree })
    }

    pub fn new_temporary() -> Result<Self> {
        let db = sled::Config::new().temporary(true).open()?;
        let tree = db.open_tree("tdg_storage")?;
        Ok(Self { db, tree })
    }
}

impl StorageBackend for SledBackend {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.tree.insert(key, value)?;
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.tree.get(key)?.map(|v| v.to_vec()))
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        self.tree.remove(key)?;
        Ok(())
    }

    fn contains(&self, key: &[u8]) -> Result<bool> {
        Ok(self.tree.contains_key(key)?)
    }

    fn iter(&self) -> Result<StorageIterator<'_>> {
        Ok(Box::new(self.tree.iter().map(|res| {
            res.map(|(k, v)| (k.to_vec(), v.to_vec()))
                .map_err(|e| anyhow::anyhow!("Iteration error: {}", e))
        })))
    }

    fn size_on_disk(&self) -> Result<u64> {
        Ok(self.db.size_on_disk()?)
    }

    fn flush(&self) -> Result<()> {
        self.tree.flush()?;
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        self.tree.clear()?;
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "sled"
    }

    fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("entries".to_string(), self.tree.len().to_string());
        if let Ok(size) = self.db.size_on_disk() {
            stats.insert("size_bytes".to_string(), size.to_string());
        }
        stats.insert("tree_name".to_string(), "tdg_storage".to_string());
        stats
    }
}

/// In-memory backend for testing and development
pub struct InMemoryBackend {
    data: Arc<DashMap<Vec<u8>, Vec<u8>>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for InMemoryBackend {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.data.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(key).map(|v| v.clone()))
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        self.data.remove(key);
        Ok(())
    }

    fn contains(&self, key: &[u8]) -> Result<bool> {
        Ok(self.data.contains_key(key))
    }

    fn iter(&self) -> Result<StorageIterator<'_>> {
        let entries: Vec<_> = self
            .data
            .iter()
            .map(|entry| Ok((entry.key().clone(), entry.value().clone())))
            .collect();
        Ok(Box::new(entries.into_iter()))
    }

    fn size_on_disk(&self) -> Result<u64> {
        let size: usize = self
            .data
            .iter()
            .map(|entry| entry.key().len() + entry.value().len())
            .sum();
        Ok(size as u64)
    }

    fn flush(&self) -> Result<()> {
        // No-op for in-memory backend
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        self.data.clear();
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "in-memory"
    }

    fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("entries".to_string(), self.data.len().to_string());
        let size: usize = self
            .data
            .iter()
            .map(|entry| entry.key().len() + entry.value().len())
            .sum();
        stats.insert("memory_bytes".to_string(), size.to_string());
        stats
    }
}

/// RocksDB backend for high-performance production use
#[cfg(feature = "rocksdb-backend")]
pub struct RocksDbBackend {
    db: rocksdb::DB,
}

#[cfg(feature = "rocksdb-backend")]
impl RocksDbBackend {
    pub fn new(path: &Path) -> Result<Self> {
        use rocksdb::{Options, DB};

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB
        opts.set_max_write_buffer_number(3);
        opts.set_target_file_size_base(64 * 1024 * 1024); // 64MB
        opts.set_level_compaction_dynamic_level_bytes(true);

        let db = DB::open(&opts, path)?;
        Ok(Self { db })
    }
}

#[cfg(feature = "rocksdb-backend")]
impl StorageBackend for RocksDbBackend {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.put(key, value)?;
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key)?)
    }

    fn delete(&self, key: &[u8]) -> Result<()> {
        self.db.delete(key)?;
        Ok(())
    }

    fn contains(&self, key: &[u8]) -> Result<bool> {
        Ok(self.db.get(key)?.is_some())
    }

    fn iter(&self) -> Result<StorageIterator<'_>> {
        use rocksdb::IteratorMode;

        let iter = self.db.iterator(IteratorMode::Start);
        Ok(Box::new(iter.map(|res| {
            res.map(|(k, v)| (k.to_vec(), v.to_vec()))
                .map_err(|e| anyhow::anyhow!("RocksDB iteration error: {}", e))
        })))
    }

    fn size_on_disk(&self) -> Result<u64> {
        // Get live files size
        let size = self
            .db
            .property_int_value("rocksdb.live-sst-files-size")?
            .unwrap_or(0) as u64;
        Ok(size)
    }

    fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        // Delete all keys (expensive operation)
        let keys: Vec<Vec<u8>> = self
            .iter()?
            .filter_map(|res| res.ok())
            .map(|(k, _)| k)
            .collect();

        for key in keys {
            self.delete(&key)?;
        }
        Ok(())
    }

    fn backend_name(&self) -> &'static str {
        "rocksdb"
    }

    fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();

        // Add various RocksDB statistics
        if let Ok(Some(v)) = self.db.property_int_value("rocksdb.estimate-num-keys") {
            stats.insert("estimated_keys".to_string(), v.to_string());
        }
        if let Ok(Some(v)) = self.db.property_int_value("rocksdb.live-sst-files-size") {
            stats.insert("sst_files_size".to_string(), v.to_string());
        }
        if let Ok(Some(v)) = self.db.property_int_value("rocksdb.total-sst-files-size") {
            stats.insert("total_sst_size".to_string(), v.to_string());
        }
        if let Ok(Some(v)) = self.db.property_int_value("rocksdb.num-live-versions") {
            stats.insert("live_versions".to_string(), v.to_string());
        }

        stats
    }
}

/// Storage backend factory for creating appropriate backends
pub struct StorageBackendFactory;

impl StorageBackendFactory {
    /// Create default backend (sled)
    pub fn create_default(path: &Path) -> Result<Box<dyn StorageBackend>> {
        Ok(Box::new(SledBackend::new(path)?))
    }

    /// Create in-memory backend for testing
    pub fn create_in_memory() -> Box<dyn StorageBackend> {
        Box::new(InMemoryBackend::new())
    }

    /// Create sled backend
    pub fn create_sled(path: &Path) -> Result<Box<dyn StorageBackend>> {
        Ok(Box::new(SledBackend::new(path)?))
    }

    /// Create temporary sled backend
    pub fn create_sled_temporary() -> Result<Box<dyn StorageBackend>> {
        Ok(Box::new(SledBackend::new_temporary()?))
    }

    /// Create RocksDB backend (if feature enabled)
    #[cfg(feature = "rocksdb-backend")]
    pub fn create_rocksdb(path: &Path) -> Result<Box<dyn StorageBackend>> {
        Ok(Box::new(RocksDbBackend::new(path)?))
    }

    /// Create backend from configuration
    pub fn create_from_config(config: &StorageConfig) -> Result<Box<dyn StorageBackend>> {
        match config.backend_type {
            StorageBackendType::Sled => {
                if let Some(path) = &config.path {
                    Self::create_sled(path)
                } else {
                    Self::create_sled_temporary()
                }
            }
            StorageBackendType::InMemory => Ok(Self::create_in_memory()),
            #[cfg(feature = "rocksdb-backend")]
            StorageBackendType::RocksDb => {
                if let Some(path) = &config.path {
                    Self::create_rocksdb(path)
                } else {
                    Err(anyhow::anyhow!("RocksDB requires a path"))
                }
            }
            #[cfg(not(feature = "rocksdb-backend"))]
            StorageBackendType::RocksDb => Err(anyhow::anyhow!(
                "RocksDB support not compiled in. Enable the 'rocksdb-backend' feature."
            )),
        }
    }
}

/// Storage backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend_type: StorageBackendType,
    pub path: Option<std::path::PathBuf>,
    pub cache_size_mb: Option<u32>,
    pub compression: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend_type: StorageBackendType::Sled,
            path: None,
            cache_size_mb: Some(128),
            compression: true,
        }
    }
}

/// Available storage backend types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StorageBackendType {
    Sled,
    InMemory,
    RocksDb,
}

impl std::fmt::Display for StorageBackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageBackendType::Sled => write!(f, "sled"),
            StorageBackendType::InMemory => write!(f, "in-memory"),
            StorageBackendType::RocksDb => write!(f, "rocksdb"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_in_memory_backend() {
        let backend = InMemoryBackend::new();

        // Test basic operations
        let key = b"test_key";
        let value = b"test_value";

        backend.put(key, value).unwrap();
        assert!(backend.contains(key).unwrap());

        let retrieved = backend.get(key).unwrap().unwrap();
        assert_eq!(retrieved, value);

        backend.delete(key).unwrap();
        assert!(!backend.contains(key).unwrap());
    }

    #[test]
    fn test_sled_backend() {
        let temp_dir = TempDir::new().unwrap();
        let backend = SledBackend::new(temp_dir.path()).unwrap();

        // Test basic operations
        let key = b"sled_key";
        let value = b"sled_value";

        backend.put(key, value).unwrap();
        backend.flush().unwrap();

        let retrieved = backend.get(key).unwrap().unwrap();
        assert_eq!(retrieved, value);

        // Test iteration
        let mut count = 0;
        for result in backend.iter().unwrap() {
            let (k, v) = result.unwrap();
            if k == key.to_vec() {
                assert_eq!(v, value);
                count += 1;
            }
        }
        assert_eq!(count, 1);

        // Test stats
        let stats = backend.get_stats();
        assert!(stats.contains_key("entries"));
        assert_eq!(stats.get("entries").unwrap(), "1");
    }

    #[test]
    fn test_backend_factory() {
        // Test in-memory creation
        let backend = StorageBackendFactory::create_in_memory();
        assert_eq!(backend.backend_name(), "in-memory");

        // Test temporary sled creation
        let backend = StorageBackendFactory::create_sled_temporary().unwrap();
        assert_eq!(backend.backend_name(), "sled");

        // Test config-based creation
        let config = StorageConfig {
            backend_type: StorageBackendType::InMemory,
            path: None,
            cache_size_mb: None,
            compression: false,
        };
        let backend = StorageBackendFactory::create_from_config(&config).unwrap();
        assert_eq!(backend.backend_name(), "in-memory");
    }

    #[test]
    fn test_backend_clear() {
        let backend = InMemoryBackend::new();

        // Add multiple entries
        for i in 0..10 {
            let key = format!("key_{}", i);
            let value = format!("value_{}", i);
            backend.put(key.as_bytes(), value.as_bytes()).unwrap();
        }

        // Verify entries exist
        assert!(backend.contains(b"key_5").unwrap());

        // Clear all data
        backend.clear().unwrap();

        // Verify all entries are gone
        assert!(!backend.contains(b"key_5").unwrap());

        let stats = backend.get_stats();
        assert_eq!(stats.get("entries").unwrap(), "0");
    }
}
