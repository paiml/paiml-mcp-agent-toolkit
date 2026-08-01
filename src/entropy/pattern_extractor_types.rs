/// Types of patterns we detect
///
/// `PartialOrd`/`Ord` are derived so that every collection keyed by `PatternType`
/// can be a `BTreeMap` and therefore serialize in a fixed order (see
/// `EntropyMetrics::patterns_by_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PatternType {
    ErrorHandling,      // try/catch, Result handling patterns
    DataValidation,     // Input validation patterns
    ResourceManagement, // open/close, lock/unlock patterns
    ControlFlow,        // if/else chains, match statements
    DataTransformation, // map/filter/reduce patterns
    ApiCall,            // HTTP/RPC call patterns
}

/// Location of a pattern in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

/// Represents an AST pattern found in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstPattern {
    pub pattern_type: PatternType,
    pub pattern_hash: String,
    /// Number of instances of this pattern that were counted.
    pub frequency: usize,
    pub locations: Vec<Location>,
    pub variation_score: f64, // How much patterns vary (0=identical, 1=very different)
    pub example_code: String,
    /// Estimated lines of code covered by *all* `frequency` instances together.
    pub estimated_loc: usize,
}

/// Collection of patterns found in project
///
/// `BTreeMap` (not `HashMap`) is deliberate: every downstream number — metrics,
/// violations, JSON — is produced by iterating these maps, and `HashMap`
/// iteration order changes between processes.
#[derive(Debug, Clone)]
pub struct PatternCollection {
    pub patterns: BTreeMap<String, AstPattern>,
    pub file_patterns: BTreeMap<PathBuf, Vec<String>>,
    pub total_files: usize,
    /// Measured non-blank source lines across every file actually analyzed.
    /// This is a measurement of the input, not an estimate derived from patterns.
    pub total_loc: usize,
}

impl Default for PatternCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternCollection {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            patterns: BTreeMap::new(),
            file_patterns: BTreeMap::new(),
            total_files: 0,
            total_loc: 0,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// File count.
    pub fn file_count(&self) -> usize {
        self.total_files
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Summary.
    pub fn summary(&self) -> super::violation_detector::PatternSummary {
        // Most common pattern, with the hash as an explicit tie-break so two runs
        // over the same input cannot pick different "most common" patterns.
        let most_common = self
            .patterns
            .values()
            .max_by(|a, b| {
                a.frequency
                    .cmp(&b.frequency)
                    .then_with(|| b.pattern_hash.cmp(&a.pattern_hash))
            })
            .cloned()
            .unwrap_or_else(|| AstPattern {
                pattern_type: PatternType::ControlFlow,
                pattern_hash: String::new(),
                frequency: 0,
                locations: vec![],
                variation_score: 0.0,
                example_code: String::new(),
                estimated_loc: 0,
            });

        super::violation_detector::PatternSummary {
            pattern_type: most_common.pattern_type,
            repetitions: most_common.frequency,
            variation_score: most_common.variation_score,
            example_code: most_common.example_code,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Add a pattern, merging it into any pattern that already has the same
    /// structural hash.
    ///
    /// NONDETERMINISM FIX: this used to be `self.patterns.insert(hash, pattern)`.
    /// A structural hash recurs in many files, so the file processed last
    /// silently discarded every earlier file's occurrences — and file order came
    /// from a `HashMap` walk. Two consecutive runs over the same 938-file tree
    /// therefore reported total_instances 173 then 187, total_loc 2627 then 3451,
    /// ControlFlow 35 then 43, while total_patterns stayed at 43 (the number of
    /// distinct hashes never changed — only which file's copy survived).
    ///
    /// Merging is order-independent by construction (add, max, min, sort), so the
    /// result does not depend on the order files are handed to us.
    pub fn add_pattern(&mut self, pattern: AstPattern) {
        self.record_file_edges(&pattern);

        let hash = pattern.pattern_hash.clone();
        match self.patterns.get_mut(&hash) {
            Some(existing) => Self::merge_pattern(existing, pattern),
            None => {
                self.patterns.insert(hash, pattern);
            }
        }
    }

    /// Record which files a pattern was seen in, so file-level entropy has data.
    ///
    /// `file_patterns` was previously never written to, which made
    /// `file_level_entropy` a hard 0.0 for every project — a constant rendered as
    /// a measurement.
    fn record_file_edges(&mut self, pattern: &AstPattern) {
        let mut files: Vec<&PathBuf> = pattern.locations.iter().map(|l| &l.file).collect();
        files.sort();
        files.dedup();
        for file in files {
            let entry = self.file_patterns.entry(file.clone()).or_default();
            if !entry.contains(&pattern.pattern_hash) {
                entry.push(pattern.pattern_hash.clone());
                entry.sort();
            }
        }
    }

    /// Merge `incoming` into `existing`. Every operation is commutative and
    /// associative so the merged result is independent of file processing order.
    fn merge_pattern(existing: &mut AstPattern, incoming: AstPattern) {
        existing.frequency = existing.frequency.saturating_add(incoming.frequency);
        existing.estimated_loc = existing.estimated_loc.saturating_add(incoming.estimated_loc);
        existing.variation_score = existing.variation_score.max(incoming.variation_score);
        existing.pattern_type = existing.pattern_type.min(incoming.pattern_type);
        if incoming.example_code < existing.example_code {
            existing.example_code = incoming.example_code;
        }
        existing.locations.extend(incoming.locations);
        existing
            .locations
            .sort_by(|a, b| (&a.file, a.line, a.column).cmp(&(&b.file, b.line, b.column)));
        existing
            .locations
            .dedup_by(|a, b| a.file == b.file && a.line == b.line && a.column == b.column);
    }

    /// Number of distinct files this pattern was observed in.
    #[must_use]
    pub fn distinct_files(pattern: &AstPattern) -> usize {
        let mut files: Vec<&PathBuf> = pattern.locations.iter().map(|l| &l.file).collect();
        files.sort();
        files.dedup();
        files.len()
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Get patterns for file.
    pub fn get_patterns_for_file(&self, file: &Path) -> Vec<&AstPattern> {
        self.file_patterns
            .get(file)
            .map(|hashes| hashes.iter().filter_map(|h| self.patterns.get(h)).collect())
            .unwrap_or_default()
    }
}

/// The source files that will be analyzed, keyed by path.
///
/// `BTreeMap` so the walk order is the path order — a `HashMap` here meant the
/// files were handed to the extractor in a different order on every run.
#[derive(Debug)]
struct ProjectContext {
    files: BTreeMap<PathBuf, String>,
}
