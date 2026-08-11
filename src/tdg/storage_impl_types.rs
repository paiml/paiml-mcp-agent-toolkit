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

/// Serialise an unmeasured compression ratio as `null`, never as the sentinel.
///
/// The text renderers were taught to say "not measured" for a `0.0` ratio, but
/// the derived `Serialize` kept emitting the sentinel verbatim — so `tdg
/// storage stats` printed "Compression ratio: not measured (nothing stored)"
/// while `tdg diagnostics --format json` reported `"compression_ratio": 0.0`
/// for the very same store. 0.0 is not a ratio any store can have (it would
/// mean zero compressed bytes), so the sentinel must not survive the
/// serializer: delegating that to each renderer is what let the two surfaces
/// disagree.
fn serialize_measured_ratio<S>(ratio: &f32, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if *ratio > 0.0 {
        serializer.serialize_some(ratio)
    } else {
        serializer.serialize_none()
    }
}

/// Read back a ratio written by `serialize_measured_ratio`, mapping `null` to
/// the unmeasured sentinel so a serialize/deserialize round trip is lossless.
fn deserialize_measured_ratio<'de, D>(deserializer: D) -> std::result::Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(<Option<f32> as serde::Deserialize>::deserialize(deserializer)?.unwrap_or(0.0))
}

/// Storage performance and usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatistics {
    pub hot_entries: usize,
    pub warm_entries: usize,
    pub cold_entries: usize,
    pub total_entries: usize,
    pub hot_memory_kb: usize,
    /// Measured compressed:uncompressed ratio of the warm tier.
    ///
    /// `0.0` means **not measured** — nothing is stored, or the tier is too
    /// large to read back cheaply. It was previously a hardcoded 0.33 that
    /// rendered as "Compression ratio: 33.0%" over empty stores; renderers must
    /// say "not measured" for 0.0 rather than print "0.0%" as a measurement,
    /// and it serialises as `null` — see `serialize_measured_ratio`.
    #[serde(
        serialize_with = "serialize_measured_ratio",
        deserialize_with = "deserialize_measured_ratio"
    )]
    pub compression_ratio: f32,
    pub warm_backend: String,
    pub cold_backend: String,
    pub backend_stats: HashMap<String, HashMap<String, String>>,
}

impl StorageStatistics {
    /// The compression ratio as text, or why there is no ratio to show.
    ///
    /// A ratio cannot be derived from zero stored entries, yet this line read
    /// "Compression ratio: 33.0%" for every store pmat has ever printed
    /// diagnostics for — see the `compression_ratio` field.
    #[must_use]
    pub fn compression_ratio_display(&self) -> String {
        if self.compression_ratio > 0.0 {
            format!("{:.1}%", self.compression_ratio * 100.0)
        } else if self.total_entries == 0 {
            "not measured (nothing stored)".to_string()
        } else {
            "not measured".to_string()
        }
    }

    /// Names of the fields in this report that serialise as `null` because they
    /// could not be measured.
    ///
    /// A `null` on its own is ambiguous — absent, unsupported, or unmeasured —
    /// so the unmeasured fields travel with the payload by name, the convention
    /// `analyze tdg` already uses for `average_score`/`average_grade`.
    #[must_use]
    pub fn not_measured(&self) -> Vec<&'static str> {
        if self.compression_ratio > 0.0 {
            Vec::new()
        } else {
            vec!["compression_ratio"]
        }
    }

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
             - Compression ratio: {}",
            self.hot_entries,
            self.hot_memory_kb,
            self.warm_backend,
            self.warm_entries,
            self.cold_backend,
            self.cold_entries,
            self.total_entries,
            self.compression_ratio_display()
        )
    }
}
