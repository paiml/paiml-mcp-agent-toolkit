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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// From record.
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
