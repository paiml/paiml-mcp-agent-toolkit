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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod storage_impl_tests {
    use super::*;
    use crate::tdg::Grade;
    use crate::tdg::language_simple::Language;

    // ============================================================================
    // FileIdentity Tests
    // ============================================================================

    fn create_test_file_identity() -> FileIdentity {
        FileIdentity {
            path: PathBuf::from("src/test.rs"),
            content_hash: blake3::hash(b"test content"),
            size_bytes: 1024,
            modified_time: SystemTime::now(),
        }
    }

    #[test]
    fn test_file_identity_creation() {
        let identity = create_test_file_identity();
        assert_eq!(identity.path, PathBuf::from("src/test.rs"));
        assert_eq!(identity.size_bytes, 1024);
    }

    #[test]
    fn test_file_identity_clone() {
        let identity = create_test_file_identity();
        let cloned = identity.clone();
        assert_eq!(identity.path, cloned.path);
        assert_eq!(identity.size_bytes, cloned.size_bytes);
        assert_eq!(identity.content_hash, cloned.content_hash);
    }

    #[test]
    fn test_file_identity_serialization() {
        let identity = create_test_file_identity();
        let json = serde_json::to_string(&identity).unwrap();
        let deserialized: FileIdentity = serde_json::from_str(&json).unwrap();
        assert_eq!(identity.path, deserialized.path);
        assert_eq!(identity.size_bytes, deserialized.size_bytes);
    }

    // ============================================================================
    // ComponentScores Tests
    // ============================================================================

    #[test]
    fn test_component_scores_default() {
        let scores = ComponentScores::default();
        assert!(scores.complexity_breakdown.is_empty());
        assert!(scores.duplication_sources.is_empty());
        assert!(scores.coupling_dependencies.is_empty());
        assert!(scores.doc_missing_items.is_empty());
        assert!(scores.consistency_violations.is_empty());
    }

    #[test]
    fn test_component_scores_with_data() {
        let mut complexity = HashMap::new();
        complexity.insert("function_a".to_string(), 15.0);
        complexity.insert("function_b".to_string(), 8.5);

        let scores = ComponentScores {
            complexity_breakdown: complexity,
            duplication_sources: vec!["file_a.rs".to_string()],
            coupling_dependencies: vec!["mod_x".to_string(), "mod_y".to_string()],
            doc_missing_items: vec!["function_c".to_string()],
            consistency_violations: vec![],
        };

        assert_eq!(scores.complexity_breakdown.len(), 2);
        assert_eq!(scores.duplication_sources.len(), 1);
        assert_eq!(scores.coupling_dependencies.len(), 2);
        assert_eq!(scores.doc_missing_items.len(), 1);
    }

    #[test]
    fn test_component_scores_serialization() {
        let scores = ComponentScores::default();
        let json = serde_json::to_string(&scores).unwrap();
        let deserialized: ComponentScores = serde_json::from_str(&json).unwrap();
        assert!(deserialized.complexity_breakdown.is_empty());
    }

    // ============================================================================
    // SemanticSignature Tests
    // ============================================================================

    fn create_test_semantic_signature() -> SemanticSignature {
        SemanticSignature {
            ast_structure_hash: 12345678,
            identifier_pattern: "snake_case".to_string(),
            control_flow_pattern: "linear".to_string(),
            import_dependencies: vec!["std".to_string(), "tokio".to_string()],
        }
    }

    #[test]
    fn test_semantic_signature_creation() {
        let sig = create_test_semantic_signature();
        assert_eq!(sig.ast_structure_hash, 12345678);
        assert_eq!(sig.identifier_pattern, "snake_case");
        assert_eq!(sig.control_flow_pattern, "linear");
        assert_eq!(sig.import_dependencies.len(), 2);
    }

    #[test]
    fn test_semantic_signature_clone() {
        let sig = create_test_semantic_signature();
        let cloned = sig.clone();
        assert_eq!(sig.ast_structure_hash, cloned.ast_structure_hash);
        assert_eq!(sig.identifier_pattern, cloned.identifier_pattern);
    }

    #[test]
    fn test_semantic_signature_serialization() {
        let sig = create_test_semantic_signature();
        let json = serde_json::to_string(&sig).unwrap();
        let deserialized: SemanticSignature = serde_json::from_str(&json).unwrap();
        assert_eq!(sig.ast_structure_hash, deserialized.ast_structure_hash);
    }

    // ============================================================================
    // AnalysisMetadata Tests
    // ============================================================================

    fn create_test_analysis_metadata() -> AnalysisMetadata {
        AnalysisMetadata {
            analyzer_version: "2.0.0".to_string(),
            analysis_duration_ms: 150,
            language_confidence: 0.95,
            analysis_timestamp: SystemTime::now(),
            cache_hit: false,
        }
    }

    #[test]
    fn test_analysis_metadata_creation() {
        let metadata = create_test_analysis_metadata();
        assert_eq!(metadata.analyzer_version, "2.0.0");
        assert_eq!(metadata.analysis_duration_ms, 150);
        assert!((metadata.language_confidence - 0.95).abs() < 0.001);
        assert!(!metadata.cache_hit);
    }

    #[test]
    fn test_analysis_metadata_with_cache_hit() {
        let metadata = AnalysisMetadata {
            analyzer_version: "2.0.0".to_string(),
            analysis_duration_ms: 5,
            language_confidence: 1.0,
            analysis_timestamp: SystemTime::now(),
            cache_hit: true,
        };
        assert!(metadata.cache_hit);
        assert_eq!(metadata.analysis_duration_ms, 5);
    }

    #[test]
    fn test_analysis_metadata_serialization() {
        let metadata = create_test_analysis_metadata();
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: AnalysisMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata.analyzer_version, deserialized.analyzer_version);
        assert_eq!(metadata.analysis_duration_ms, deserialized.analysis_duration_ms);
    }

    // ============================================================================
    // HotCacheEntry Tests
    // ============================================================================

    fn create_test_full_tdg_record() -> FullTdgRecord {
        FullTdgRecord {
            identity: create_test_file_identity(),
            score: TdgScore {
                structural_complexity: 5.0,
                semantic_complexity: 3.0,
                duplication_ratio: 0.1,
                coupling_score: 2.0,
                doc_coverage: 0.8,
                consistency_score: 0.9,
                entropy_score: 0.5,
                total: 82.5,
                grade: Grade::B,
                confidence: 0.95,
                language: Language::Rust,
                file_path: Some(PathBuf::from("src/test.rs")),
                penalties_applied: vec![],
                critical_defects_count: 0,
                has_critical_defects: false,
            },
            components: ComponentScores::default(),
            semantic_sig: create_test_semantic_signature(),
            metadata: create_test_analysis_metadata(),
            git_context: None,
        }
    }

    #[test]
    fn test_hot_cache_entry_from_record() {
        let record = create_test_full_tdg_record();
        let entry = HotCacheEntry::from_record(&record);

        assert_eq!(entry.grade, Grade::B as u8);
        assert!((entry.total_score - 82.5).abs() < 0.001);
        assert!(entry.timestamp > 0);
    }

    #[test]
    fn test_hot_cache_entry_hash_bytes() {
        let record = create_test_full_tdg_record();
        let entry = HotCacheEntry::from_record(&record);

        // Hash bytes should be 32 bytes (Blake3)
        assert_eq!(entry.content_hash.len(), 32);
    }

    #[test]
    fn test_hot_cache_entry_copy() {
        let record = create_test_full_tdg_record();
        let entry = HotCacheEntry::from_record(&record);
        let copied = entry; // Should work since HotCacheEntry is Copy

        assert_eq!(entry.grade, copied.grade);
        assert_eq!(entry.total_score, copied.total_score);
    }

    // ============================================================================
    // FullTdgRecord Tests
    // ============================================================================

    #[test]
    fn test_full_tdg_record_creation() {
        let record = create_test_full_tdg_record();
        assert_eq!(record.score.grade, Grade::B);
        assert!((record.score.total - 82.5).abs() < 0.001);
        assert!(record.git_context.is_none());
    }

    #[test]
    fn test_full_tdg_record_clone() {
        let record = create_test_full_tdg_record();
        let cloned = record.clone();
        assert_eq!(record.score.grade, cloned.score.grade);
        assert_eq!(record.identity.path, cloned.identity.path);
    }

    #[test]
    fn test_full_tdg_record_serialization() {
        let record = create_test_full_tdg_record();
        let json = serde_json::to_string(&record).unwrap();
        let deserialized: FullTdgRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record.score.grade, deserialized.score.grade);
    }

    // ============================================================================
    // TieredStore Tests
    // ============================================================================

    #[test]
    fn test_tiered_store_in_memory() {
        let store = TieredStore::in_memory();
        let stats = store.get_statistics();
        assert_eq!(stats.hot_entries, 0);
        assert_eq!(stats.total_entries, 0);
    }

    #[tokio::test]
    async fn test_tiered_store_store_and_get_hot() {
        let store = TieredStore::in_memory();
        let record = create_test_full_tdg_record();
        let hash = record.identity.content_hash;

        store.store(record).await.unwrap();

        let hot_entry = store.get_hot(&hash);
        assert!(hot_entry.is_some());

        let entry = hot_entry.unwrap();
        assert_eq!(entry.grade, Grade::B as u8);
    }

    #[tokio::test]
    async fn test_tiered_store_retrieve_full() {
        let store = TieredStore::in_memory();
        let record = create_test_full_tdg_record();
        let hash = record.identity.content_hash;

        store.store(record.clone()).await.unwrap();

        let retrieved = store.retrieve_full(&hash).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_record = retrieved.unwrap();
        assert_eq!(retrieved_record.score.grade, record.score.grade);
    }

    #[tokio::test]
    async fn test_tiered_store_retrieve_nonexistent() {
        let store = TieredStore::in_memory();
        let hash = blake3::hash(b"nonexistent");

        let retrieved = store.retrieve_full(&hash).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_tiered_store_cleanup_hot_cache() {
        let store = TieredStore::in_memory();

        // Add some entries directly to hot cache
        let hash1 = blake3::hash(b"test1");
        let hash2 = blake3::hash(b"test2");

        // Entry with old timestamp
        let old_entry = HotCacheEntry {
            content_hash: [0u8; 32],
            grade: 0,
            total_score: 50.0,
            timestamp: 0, // Very old
        };

        // Entry with current timestamp
        let new_entry = HotCacheEntry {
            content_hash: [0u8; 32],
            grade: 0,
            total_score: 50.0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        store.hot.insert(hash1, old_entry);
        store.hot.insert(hash2, new_entry);

        // Cleanup entries older than 10 seconds
        let removed = store.cleanup_hot_cache(10);

        // Old entry should be removed
        assert_eq!(removed, 1);
        assert!(store.hot.get(&hash1).is_none());
        assert!(store.hot.get(&hash2).is_some());
    }

    #[test]
    fn test_tiered_store_flush() {
        let store = TieredStore::in_memory();
        let result = store.flush();
        assert!(result.is_ok());
    }

    // ============================================================================
    // StorageStatistics Tests
    // ============================================================================

    #[test]
    fn test_storage_statistics_creation() {
        let stats = StorageStatistics {
            hot_entries: 100,
            warm_entries: 500,
            cold_entries: 2000,
            total_entries: 2600,
            hot_memory_kb: 50,
            compression_ratio: 0.33,
            warm_backend: "sled".to_string(),
            cold_backend: "sled".to_string(),
            backend_stats: HashMap::new(),
        };

        assert_eq!(stats.hot_entries, 100);
        assert_eq!(stats.warm_entries, 500);
        assert_eq!(stats.cold_entries, 2000);
        assert_eq!(stats.total_entries, 2600);
    }

    #[test]
    fn test_storage_statistics_format_diagnostic() {
        let stats = StorageStatistics {
            hot_entries: 100,
            warm_entries: 500,
            cold_entries: 2000,
            total_entries: 2600,
            hot_memory_kb: 50,
            compression_ratio: 0.33,
            warm_backend: "sled".to_string(),
            cold_backend: "sled".to_string(),
            backend_stats: HashMap::new(),
        };

        let diagnostic = stats.format_diagnostic();
        assert!(diagnostic.contains("Hot (memory): 100 entries"));
        assert!(diagnostic.contains("Warm (sled backend): 500 entries"));
        assert!(diagnostic.contains("Cold (sled backend): 2000 entries"));
        assert!(diagnostic.contains("Total: 2600 entries"));
        assert!(diagnostic.contains("Compression ratio: 33.0%"));
    }

    #[test]
    fn test_storage_statistics_serialization() {
        let stats = StorageStatistics {
            hot_entries: 100,
            warm_entries: 500,
            cold_entries: 2000,
            total_entries: 2600,
            hot_memory_kb: 50,
            compression_ratio: 0.33,
            warm_backend: "sled".to_string(),
            cold_backend: "sled".to_string(),
            backend_stats: HashMap::new(),
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: StorageStatistics = serde_json::from_str(&json).unwrap();
        assert_eq!(stats.hot_entries, deserialized.hot_entries);
        assert_eq!(stats.total_entries, deserialized.total_entries);
    }

    // ============================================================================
    // TieredStorageFactory Tests
    // ============================================================================

    #[test]
    fn test_tiered_storage_factory_in_memory() {
        let store = TieredStorageFactory::create_in_memory();
        let stats = store.get_statistics();
        assert_eq!(stats.hot_entries, 0);
    }

    // ============================================================================
    // Debug Trait Tests
    // ============================================================================

    #[test]
    fn test_file_identity_debug() {
        let identity = create_test_file_identity();
        let debug = format!("{:?}", identity);
        assert!(debug.contains("FileIdentity"));
        assert!(debug.contains("src/test.rs"));
    }

    #[test]
    fn test_component_scores_debug() {
        let scores = ComponentScores::default();
        let debug = format!("{:?}", scores);
        assert!(debug.contains("ComponentScores"));
    }

    #[test]
    fn test_semantic_signature_debug() {
        let sig = create_test_semantic_signature();
        let debug = format!("{:?}", sig);
        assert!(debug.contains("SemanticSignature"));
        assert!(debug.contains("snake_case"));
    }

    #[test]
    fn test_analysis_metadata_debug() {
        let metadata = create_test_analysis_metadata();
        let debug = format!("{:?}", metadata);
        assert!(debug.contains("AnalysisMetadata"));
        assert!(debug.contains("2.0.0"));
    }

    #[test]
    fn test_hot_cache_entry_debug() {
        let record = create_test_full_tdg_record();
        let entry = HotCacheEntry::from_record(&record);
        let debug = format!("{:?}", entry);
        assert!(debug.contains("HotCacheEntry"));
    }
}

