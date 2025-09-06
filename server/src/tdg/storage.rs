use crate::tdg::storage_backend::{StorageBackend, StorageBackendFactory, StorageConfig};
use crate::tdg::TdgScore;
use anyhow::{anyhow, Result};
use blake3::Hash as Blake3Hash;
use dashmap::DashMap;
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Complete file identity for transactional tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIdentity {
    pub path: PathBuf,
    pub content_hash: Blake3Hash,
    pub size_bytes: u64,
    pub modified_time: SystemTime,
}

/// Component-level score breakdown for detailed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentScores {
    pub complexity_breakdown: HashMap<String, f32>,
    pub duplication_sources: Vec<String>,
    pub coupling_dependencies: Vec<String>,
    pub doc_missing_items: Vec<String>,
    pub consistency_violations: Vec<String>,
}

/// Semantic signature for efficient similarity detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSignature {
    pub ast_structure_hash: u64,
    pub identifier_pattern: String,
    pub control_flow_pattern: String,
    pub import_dependencies: Vec<String>,
}

/// Analysis metadata for quality tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub analyzer_version: String,
    pub analysis_duration_ms: u64,
    pub language_confidence: f32,
    pub analysis_timestamp: SystemTime,
    pub cache_hit: bool,
}

/// Full TDG record for transactional storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTdgRecord {
    pub identity: FileIdentity,
    pub score: TdgScore,
    pub components: ComponentScores,
    pub semantic_sig: SemanticSignature,
    pub metadata: AnalysisMetadata,
}

/// Hot cache entry for high-speed access (in-memory)
#[derive(Debug, Clone, Copy)]
pub struct HotCacheEntry {
    pub content_hash: [u8; 32],
    pub grade: u8,
    pub total_score: f32,
    pub timestamp: i64,
}

impl HotCacheEntry {
    pub fn from_record(record: &FullTdgRecord) -> Self {
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(record.identity.content_hash.as_bytes());

        Self {
            content_hash: hash_bytes,
            grade: record.score.grade as u8,
            total_score: record.score.total,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        }
    }
}

/// Tiered storage system with Hot/Warm/Cold tiers using flexible backends
pub struct TieredStore {
    /// Hot cache - recent files (in-memory)
    hot: Arc<DashMap<Blake3Hash, HotCacheEntry>>,
    /// Warm storage - compressed recent records (backend-agnostic)
    warm_backend: Box<dyn StorageBackend>,
    /// Cold storage - full historical records (backend-agnostic)
    cold_backend: Box<dyn StorageBackend>,
    /// Archival configuration
    archive_after_days: u32,
}

impl TieredStore {
    /// Create new tiered storage instance with default Sled backend
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let warm_config = StorageConfig {
            backend_type: crate::tdg::storage_backend::StorageBackendType::Sled,
            path: Some(db_path.as_ref().join(".pmat/tdg-warm")),
            cache_size_mb: Some(128),
            compression: true,
        };

        let cold_config = StorageConfig {
            backend_type: crate::tdg::storage_backend::StorageBackendType::Sled,
            path: Some(db_path.as_ref().join(".pmat/tdg-cold")),
            cache_size_mb: Some(64),
            compression: false, // Cold storage doesn't need additional compression
        };

        Self::with_config(warm_config, cold_config)
    }

    /// Create tiered storage with specific backend configurations
    pub fn with_config(warm_config: StorageConfig, cold_config: StorageConfig) -> Result<Self> {
        let warm_backend = StorageBackendFactory::create_from_config(&warm_config)?;
        let cold_backend = StorageBackendFactory::create_from_config(&cold_config)?;

        Ok(Self {
            hot: Arc::new(DashMap::new()),
            warm_backend,
            cold_backend,
            archive_after_days: 30,
        })
    }

    /// Create in-memory tiered storage for testing
    pub fn in_memory() -> Self {
        Self {
            hot: Arc::new(DashMap::new()),
            warm_backend: StorageBackendFactory::create_in_memory(),
            cold_backend: StorageBackendFactory::create_in_memory(),
            archive_after_days: 30,
        }
    }

    /// Store a complete TDG record in all tiers
    pub async fn store(&self, record: FullTdgRecord) -> Result<()> {
        let hash = record.identity.content_hash;

        // Hot cache entry (immediate access)
        let hot_entry = HotCacheEntry::from_record(&record);
        self.hot.insert(hash, hot_entry);

        // Warm storage - compress with LZ4 for space efficiency
        let serialized = bincode::serialize(&record)?;
        let compressed = compress_prepend_size(&serialized);
        self.warm_backend.put(hash.as_bytes(), &compressed)?;

        // Schedule cold archival if record is old enough
        if self.should_archive(&record) {
            self.archive_to_cold(record).await?;
        }

        Ok(())
    }

    /// Retrieve hot cache entry (fastest access)
    pub fn get_hot(&self, hash: &Blake3Hash) -> Option<HotCacheEntry> {
        self.hot.get(hash).map(|entry| *entry.value())
    }

    /// Retrieve full record from any tier
    pub async fn retrieve_full(&self, hash: &Blake3Hash) -> Result<Option<FullTdgRecord>> {
        // Check warm storage first (compressed but fast)
        if let Some(compressed) = self.warm_backend.get(hash.as_bytes())? {
            let decompressed = decompress_size_prepended(&compressed)?;
            return Ok(Some(bincode::deserialize(&decompressed)?));
        }

        // Check cold storage (full historical records)
        if let Some(archived) = self.cold_backend.get(hash.as_bytes())? {
            return Ok(Some(bincode::deserialize(&archived)?));
        }

        Ok(None)
    }

    /// Check if record should be archived to cold storage
    fn should_archive(&self, record: &FullTdgRecord) -> bool {
        let age_days = record
            .metadata
            .analysis_timestamp
            .elapsed()
            .unwrap_or_default()
            .as_secs()
            / (24 * 60 * 60);

        age_days > self.archive_after_days as u64
    }

    /// Archive record to cold storage and remove from warm
    async fn archive_to_cold(&self, record: FullTdgRecord) -> Result<()> {
        let hash = record.identity.content_hash;

        // Store in cold storage (uncompressed for long-term access)
        let serialized = bincode::serialize(&record)?;
        self.cold_backend.put(hash.as_bytes(), &serialized)?;

        // Remove from warm storage to save space
        self.warm_backend.delete(hash.as_bytes())?;

        Ok(())
    }

    /// Get storage statistics for diagnostics
    pub fn get_statistics(&self) -> StorageStatistics {
        let hot_count = self.hot.len();

        // Count warm entries
        let warm_count = self
            .warm_backend
            .iter()
            .map(|iter| iter.count())
            .unwrap_or(0);

        // Count cold entries
        let cold_count = self
            .cold_backend
            .iter()
            .map(|iter| iter.count())
            .unwrap_or(0);

        // Get backend-specific stats
        let warm_stats = self.warm_backend.get_stats();
        let cold_stats = self.cold_backend.get_stats();

        StorageStatistics {
            hot_entries: hot_count,
            warm_entries: warm_count,
            cold_entries: cold_count,
            total_entries: hot_count + warm_count + cold_count,
            hot_memory_kb: (hot_count * std::mem::size_of::<HotCacheEntry>()) / 1024,
            compression_ratio: self.estimate_compression_ratio(),
            warm_backend: self.warm_backend.backend_name().to_string(),
            cold_backend: self.cold_backend.backend_name().to_string(),
            backend_stats: HashMap::from([
                ("warm".to_string(), warm_stats),
                ("cold".to_string(), cold_stats),
            ]),
        }
    }

    /// Estimate compression ratio for warm storage
    fn estimate_compression_ratio(&self) -> f32 {
        // Sample a few entries to estimate compression
        let mut total_original = 0usize;
        let mut total_compressed = 0usize;
        let mut samples = 0;

        if let Ok(iter) = self.warm_backend.iter() {
            for (_, compressed) in iter.take(10).flatten() {
                total_compressed += compressed.len();
                // Estimate original size (this is approximate)
                total_original += compressed.len() * 3; // Typical compression is ~3:1
                samples += 1;
            }
        }

        if samples > 0 && total_original > 0 {
            total_compressed as f32 / total_original as f32
        } else {
            0.33 // Default estimate for LZ4 compression
        }
    }

    /// Clean up expired hot cache entries
    pub fn cleanup_hot_cache(&self, max_age_seconds: u64) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut removed = 0;
        self.hot.retain(|_, entry| {
            let age = now - entry.timestamp;
            if age > max_age_seconds as i64 {
                removed += 1;
                false
            } else {
                true
            }
        });

        removed
    }

    /// Migrate between storage backends
    pub async fn migrate_backend(
        &mut self,
        new_warm_config: StorageConfig,
        new_cold_config: StorageConfig,
    ) -> Result<()> {
        // Create new backends
        let new_warm = StorageBackendFactory::create_from_config(&new_warm_config)?;
        let new_cold = StorageBackendFactory::create_from_config(&new_cold_config)?;

        // Migrate warm storage
        if let Ok(iter) = self.warm_backend.iter() {
            for result in iter {
                let (key, value) = result?;
                new_warm.put(&key, &value)?;
            }
        }

        // Migrate cold storage
        if let Ok(iter) = self.cold_backend.iter() {
            for result in iter {
                let (key, value) = result?;
                new_cold.put(&key, &value)?;
            }
        }

        // Swap backends
        self.warm_backend = new_warm;
        self.cold_backend = new_cold;

        Ok(())
    }

    /// Flush all pending writes
    pub fn flush(&self) -> Result<()> {
        self.warm_backend.flush()?;
        self.cold_backend.flush()?;
        Ok(())
    }
}

/// Storage performance and usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    pub hot_entries: usize,
    pub warm_entries: usize,
    pub cold_entries: usize,
    pub total_entries: usize,
    pub hot_memory_kb: usize,
    pub compression_ratio: f32,
    pub warm_backend: String,
    pub cold_backend: String,
    pub backend_stats: HashMap<String, HashMap<String, String>>,
}

impl StorageStatistics {
    /// Format statistics for diagnostic display
    pub fn format_diagnostic(&self) -> String {
        format!(
            "Storage Tiers:\n\
             - Hot (memory): {} entries, {} KB\n\
             - Warm ({} backend): {} entries\n\
             - Cold ({} backend): {} entries\n\
             - Total: {} entries\n\
             - Compression ratio: {:.1}%",
            self.hot_entries,
            self.hot_memory_kb,
            self.warm_backend,
            self.warm_entries,
            self.cold_backend,
            self.cold_entries,
            self.total_entries,
            self.compression_ratio * 100.0
        )
    }
}

/// Factory for creating tiered storage instances
pub struct TieredStorageFactory;

impl TieredStorageFactory {
    /// Create storage instance with default configuration
    pub fn create_default() -> Result<TieredStore> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        TieredStore::new(home_dir)
    }

    /// Create storage instance at specific path
    pub fn create_at_path(path: impl AsRef<Path>) -> Result<TieredStore> {
        TieredStore::new(path)
    }

    /// Create in-memory storage for testing
    pub fn create_in_memory() -> TieredStore {
        TieredStore::in_memory()
    }

    /// Create with RocksDB backend (if feature enabled)
    #[cfg(feature = "rocksdb-backend")]
    pub fn create_with_rocksdb(path: impl AsRef<Path>) -> Result<TieredStore> {
        use crate::tdg::storage_backend::StorageBackendType;

        let warm_config = StorageConfig {
            backend_type: StorageBackendType::RocksDb,
            path: Some(path.as_ref().join(".pmat/tdg-warm-rocks")),
            cache_size_mb: Some(256),
            compression: true,
        };

        let cold_config = StorageConfig {
            backend_type: StorageBackendType::RocksDb,
            path: Some(path.as_ref().join(".pmat/tdg-cold-rocks")),
            cache_size_mb: Some(128),
            compression: false,
        };

        TieredStore::with_config(warm_config, cold_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::language_simple::Language;
    use crate::tdg::Grade;
    use tempfile::TempDir;

    fn create_test_record() -> FullTdgRecord {
        let content = b"fn test() { println!(\"hello\"); }";
        let hash = blake3::hash(content);

        FullTdgRecord {
            identity: FileIdentity {
                path: PathBuf::from("test.rs"),
                content_hash: hash,
                size_bytes: content.len() as u64,
                modified_time: SystemTime::now(),
            },
            score: TdgScore {
                structural_complexity: 20.0,
                semantic_complexity: 18.0,
                duplication_ratio: 19.0,
                coupling_score: 14.0,
                doc_coverage: 9.0,
                consistency_score: 8.0,
                total: 88.0,
                grade: Grade::AMinus,
                confidence: 0.95,
                language: Language::Rust,
                file_path: Some(PathBuf::from("test.rs")),
                penalties_applied: Vec::new(),
            },
            components: ComponentScores {
                complexity_breakdown: HashMap::new(),
                duplication_sources: Vec::new(),
                coupling_dependencies: Vec::new(),
                doc_missing_items: Vec::new(),
                consistency_violations: Vec::new(),
            },
            semantic_sig: SemanticSignature {
                ast_structure_hash: 123456789,
                identifier_pattern: "test,println".to_string(),
                control_flow_pattern: "function_call".to_string(),
                import_dependencies: Vec::new(),
            },
            metadata: AnalysisMetadata {
                analyzer_version: "2.38.0".to_string(),
                analysis_duration_ms: 5,
                language_confidence: 1.0,
                analysis_timestamp: SystemTime::now(),
                cache_hit: false,
            },
        }
    }

    #[tokio::test]
    async fn test_tiered_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = TieredStore::new(temp_dir.path()).unwrap();

        let stats = storage.get_statistics();
        assert_eq!(stats.hot_entries, 0);
        assert_eq!(stats.warm_entries, 0);
        assert_eq!(stats.cold_entries, 0);
    }

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = TieredStore::in_memory();
        let record = create_test_record();
        let hash = record.identity.content_hash;

        // Store record
        storage.store(record.clone()).await.unwrap();

        // Check hot cache
        let hot_entry = storage.get_hot(&hash).unwrap();
        assert_eq!(hot_entry.total_score, 88.0);
        assert_eq!(hot_entry.grade, Grade::AMinus as u8);

        // Retrieve full record
        let retrieved = storage.retrieve_full(&hash).await.unwrap().unwrap();
        assert_eq!(retrieved.score.total, record.score.total);
        assert_eq!(retrieved.identity.path, record.identity.path);
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let storage = TieredStore::new(temp_dir.path()).unwrap();
        let record = create_test_record();
        let hash = record.identity.content_hash;

        // Store record
        storage.store(record.clone()).await.unwrap();

        // Check hot cache
        let hot_entry = storage.get_hot(&hash).unwrap();
        assert_eq!(hot_entry.total_score, 88.0);
        assert_eq!(hot_entry.grade, Grade::AMinus as u8);

        // Retrieve full record
        let retrieved = storage.retrieve_full(&hash).await.unwrap().unwrap();
        assert_eq!(retrieved.score.total, record.score.total);
        assert_eq!(retrieved.identity.path, record.identity.path);
    }

    #[tokio::test]
    async fn test_compression() {
        let temp_dir = TempDir::new().unwrap();
        let storage = TieredStore::new(temp_dir.path()).unwrap();
        let record = create_test_record();

        // Store and verify compression
        storage.store(record.clone()).await.unwrap();
        storage.flush().unwrap();

        let stats = storage.get_statistics();
        assert!(stats.compression_ratio > 0.0);
        assert!(stats.compression_ratio < 1.0); // Should be compressed
    }

    #[test]
    fn test_hot_cache_cleanup() {
        let storage = TieredStore::in_memory();

        // Add some entries with old timestamps
        let old_hash = blake3::hash(b"old content");
        let old_entry = HotCacheEntry {
            content_hash: *old_hash.as_bytes(),
            grade: Grade::B as u8,
            total_score: 75.0,
            timestamp: (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64)
                - 3600, // 1 hour ago
        };
        storage.hot.insert(old_hash, old_entry);

        // Cleanup entries older than 30 minutes
        let removed = storage.cleanup_hot_cache(1800);
        assert_eq!(removed, 1);
        assert!(storage.hot.is_empty());
    }

    #[tokio::test]
    async fn test_backend_migration() {
        use crate::tdg::storage_backend::StorageBackendType;

        let temp_dir = TempDir::new().unwrap();
        let mut storage = TieredStore::new(temp_dir.path()).unwrap();

        // Store some records
        let record1 = create_test_record();
        let record2 = create_test_record();
        storage.store(record1.clone()).await.unwrap();
        storage.store(record2.clone()).await.unwrap();

        // Migrate to in-memory backend
        let new_warm = StorageConfig {
            backend_type: StorageBackendType::InMemory,
            path: None,
            cache_size_mb: None,
            compression: true,
        };

        let new_cold = StorageConfig {
            backend_type: StorageBackendType::InMemory,
            path: None,
            cache_size_mb: None,
            compression: false,
        };

        storage.migrate_backend(new_warm, new_cold).await.unwrap();

        // Verify data still accessible
        let retrieved = storage
            .retrieve_full(&record1.identity.content_hash)
            .await
            .unwrap();
        assert!(retrieved.is_some());
    }
}
