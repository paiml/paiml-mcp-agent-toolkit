use super::storage_backend::{StorageBackend, StorageBackendFactory, StorageConfig};
use super::TdgScore;
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

    /// Git context (Sprint 65 - Git-Commit Correlation)
    /// None if not in a git repository or --no-git-context flag used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_context: Option<crate::models::git_context::GitContext>,
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
    #[must_use]
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
    /// Create new tiered storage instance with default Libsql backend
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let warm_config = StorageConfig {
            backend_type: crate::tdg::storage_backend::StorageBackendType::Libsql,
            path: Some(db_path.as_ref().join(".pmat/tdg-warm.db")),
            cache_size_mb: Some(128),
            compression: true,
        };

        let cold_config = StorageConfig {
            backend_type: crate::tdg::storage_backend::StorageBackendType::Libsql,
            path: Some(db_path.as_ref().join(".pmat/tdg-cold.db")),
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
    #[must_use]
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
        // NOTE: Using serde_json instead of bincode due to incompatibility with FullTdgRecord
        // (see git commit 46968e5f - bincode causes "unexpected end of file" errors)
        let serialized = serde_json::to_vec(&record)?;
        let compressed = compress_prepend_size(&serialized);
        self.warm_backend.put(hash.as_bytes(), &compressed)?;

        // Schedule cold archival if record is old enough
        if self.should_archive(&record) {
            self.archive_to_cold(record).await?;
        }

        Ok(())
    }

    /// Retrieve hot cache entry (fastest access)
    #[must_use]
    pub fn get_hot(&self, hash: &Blake3Hash) -> Option<HotCacheEntry> {
        self.hot.get(hash).map(|entry| *entry.value())
    }

    /// Retrieve full record from any tier
    pub async fn retrieve_full(&self, hash: &Blake3Hash) -> Result<Option<FullTdgRecord>> {
        // Check warm storage first (compressed but fast)
        if let Some(compressed) = self.warm_backend.get(hash.as_bytes())? {
            let decompressed = decompress_size_prepended(&compressed)?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            return Ok(Some(serde_json::from_slice(&decompressed)?));
        }

        // Check cold storage (full historical records)
        if let Some(archived) = self.cold_backend.get(hash.as_bytes())? {
            // NOTE: Using serde_json instead of bincode (see store() method)
            return Ok(Some(serde_json::from_slice(&archived)?));
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

        age_days > u64::from(self.archive_after_days)
    }

    /// Archive record to cold storage and remove from warm
    async fn archive_to_cold(&self, record: FullTdgRecord) -> Result<()> {
        let hash = record.identity.content_hash;

        // Store in cold storage (uncompressed for long-term access)
        // NOTE: Using serde_json instead of bincode (see store() method)
        let serialized = serde_json::to_vec(&record)?;
        self.cold_backend.put(hash.as_bytes(), &serialized)?;

        // OLAP-Compatible Pattern (Issue #79, P0-4):
        // Remove from warm storage to save space (data lifecycle management)
        // This is NOT an OLTP update - we're moving data between storage tiers
        // The record remains immutable; we're just changing its storage location
        self.warm_backend.delete(hash.as_bytes())?;

        Ok(())
    }

    /// Clean up expired hot cache entries
    #[must_use]
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

    /// Get storage statistics for monitoring and dogfooding
    #[must_use]
    pub fn get_statistics(&self) -> StorageStatistics {
        let hot_entries = self.hot.len();
        let hot_memory_kb = (hot_entries * std::mem::size_of::<HotCacheEntry>()) / 1024;

        // Get backend statistics (if available)
        let warm_stats = self.warm_backend.get_stats();
        let cold_stats = self.cold_backend.get_stats();

        let warm_entries = warm_stats
            .get("entry_count")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);
        let cold_entries = cold_stats
            .get("entry_count")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let total_entries = hot_entries + warm_entries + cold_entries;

        let mut backend_stats = HashMap::new();
        backend_stats.insert("warm".to_string(), warm_stats);
        backend_stats.insert("cold".to_string(), cold_stats);

        StorageStatistics {
            hot_entries,
            warm_entries,
            cold_entries,
            total_entries,
            hot_memory_kb,
            compression_ratio: 0.33,          // Default compression ratio
            warm_backend: "sled".to_string(), // Default backend type
            cold_backend: "sled".to_string(), // Default backend type
            backend_stats,
        }
    }

    /// Query TDG records by git commit SHA (Sprint 65 Phase 3)
    ///
    /// Searches both warm and cold storage for records matching the specified commit.
    /// Supports full SHA, short SHA (7 chars), or git tags.
    pub async fn get_by_commit(&self, commit_ref: &str) -> Result<Vec<FullTdgRecord>> {
        let mut results = Vec::new();

        // Search warm storage
        for item in self.warm_backend.iter()? {
            let (_key, value) = item?;
            let decompressed = decompress_size_prepended(&value)?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&decompressed) {
                if let Some(git_ctx) = &record.git_context {
                    if git_ctx.commit_sha == commit_ref
                        || git_ctx.commit_sha_short == commit_ref
                        || git_ctx.tags.contains(&commit_ref.to_string())
                    {
                        results.push(record);
                    }
                }
            }
        }

        // Search cold storage
        for item in self.cold_backend.iter()? {
            let (_key, value) = item?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&value) {
                if let Some(git_ctx) = &record.git_context {
                    if git_ctx.commit_sha == commit_ref
                        || git_ctx.commit_sha_short == commit_ref
                        || git_ctx.tags.contains(&commit_ref.to_string())
                    {
                        results.push(record);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Query TDG records in a git commit range (Sprint 65 Phase 3)
    ///
    /// Returns all records with git context, allowing the caller to filter by commit range.
    /// The actual git range resolution happens in the handler (using git2).
    pub async fn get_all_with_git_context(&self) -> Result<Vec<FullTdgRecord>> {
        let mut results = Vec::new();

        // Search warm storage
        for item in self.warm_backend.iter()? {
            let (_key, value) = item?;
            let decompressed = decompress_size_prepended(&value)?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&decompressed) {
                if record.git_context.is_some() {
                    results.push(record);
                }
            }
        }

        // Search cold storage
        for item in self.cold_backend.iter()? {
            let (_key, value) = item?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&value) {
                if record.git_context.is_some() {
                    results.push(record);
                }
            }
        }

        // Sort by commit timestamp (newest first)
        results.sort_by(|a, b| {
            let a_time = a
                .git_context
                .as_ref()
                .map(|g| g.commit_timestamp.timestamp())
                .unwrap_or(0);
            let b_time = b
                .git_context
                .as_ref()
                .map(|g| g.commit_timestamp.timestamp())
                .unwrap_or(0);
            b_time.cmp(&a_time) // Reverse order (newest first)
        });

        Ok(results)
    }

    /// Query TDG records by file path (Sprint 65 Phase 3)
    ///
    /// Filters records to only those matching the specified file path.
    pub async fn get_by_path(&self, target_path: &Path) -> Result<Vec<FullTdgRecord>> {
        let mut results = Vec::new();

        // Search warm storage
        for item in self.warm_backend.iter()? {
            let (_key, value) = item?;
            let decompressed = decompress_size_prepended(&value)?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&decompressed) {
                if record.identity.path == target_path {
                    results.push(record);
                }
            }
        }

        // Search cold storage
        for item in self.cold_backend.iter()? {
            let (_key, value) = item?;
            // NOTE: Using serde_json instead of bincode (see store() method)
            if let Ok(record) = serde_json::from_slice::<FullTdgRecord>(&value) {
                if record.identity.path == target_path {
                    results.push(record);
                }
            }
        }

        // Sort by commit timestamp (newest first)
        results.sort_by(|a, b| {
            let a_time = a
                .git_context
                .as_ref()
                .map(|g| g.commit_timestamp.timestamp())
                .unwrap_or(0);
            let b_time = b
                .git_context
                .as_ref()
                .map(|g| g.commit_timestamp.timestamp())
                .unwrap_or(0);
            b_time.cmp(&a_time) // Reverse order (newest first)
        });

        Ok(results)
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
    #[must_use]
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
    #[must_use]
    pub fn create_in_memory() -> TieredStore {
        TieredStore::in_memory()
    }
}

