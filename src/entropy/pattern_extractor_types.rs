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

/// Detection thresholds for one Rust construct.
///
/// `min_matches_in_file` is how many raw regex matches a file must contain before
/// the construct is considered at all; `min_identical` is how many *structurally
/// identical* occurrences must then land in one group for a pattern to be
/// emitted. A user therefore needs `effective_minimum()` identical copies in one
/// file before anything is reported.
///
/// This table exists so that `EntropyAnalyzer::measurement_note` quotes the real
/// numbers. Before it existed the note said "needs at least 3 structurally
/// identical occurrences" for every construct, which is false for four of the six:
/// a fixture with 5 identical validation lines measured nothing while the note
/// promised 3 would be enough.
#[derive(Debug, Clone, Copy)]
pub struct RustPatternThreshold {
    /// Human-readable construct name used in the measurement note.
    pub name: &'static str,
    /// Minimum raw matches of the construct's regex within one file.
    pub min_matches_in_file: usize,
    /// Minimum structurally identical occurrences within one group.
    pub min_identical: usize,
}

impl RustPatternThreshold {
    /// Identical copies a single file must contain before this construct is reported.
    #[must_use]
    pub const fn effective_minimum(&self) -> usize {
        if self.min_matches_in_file > self.min_identical {
            self.min_matches_in_file
        } else {
            self.min_identical
        }
    }
}

/// Thresholds actually applied by the Rust extractors, in `PatternType` order.
///
/// Both the extractors and the measurement note read this table, so the
/// documented rule and the enforced rule cannot diverge.
pub const RUST_PATTERN_THRESHOLDS: [RustPatternThreshold; 6] = [
    RustPatternThreshold {
        name: "Result handling",
        min_matches_in_file: 2,
        min_identical: 3,
    },
    RustPatternThreshold {
        name: "input validation",
        min_matches_in_file: 6,
        min_identical: 3,
    },
    RustPatternThreshold {
        name: "resource management",
        min_matches_in_file: 6,
        min_identical: 3,
    },
    RustPatternThreshold {
        name: "if/else chains",
        min_matches_in_file: 3,
        min_identical: 3,
    },
    RustPatternThreshold {
        name: "iterator chains",
        min_matches_in_file: 9,
        min_identical: 3,
    },
    RustPatternThreshold {
        name: "API calls",
        min_matches_in_file: 4,
        min_identical: 3,
    },
];

/// Threshold for one `PatternType`, by its position in the enum.
#[must_use]
pub const fn rust_threshold(pattern_type: PatternType) -> RustPatternThreshold {
    RUST_PATTERN_THRESHOLDS[pattern_type as usize]
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
    /// The most-repeated pattern found, or `None` when no pattern was found.
    ///
    /// A project with no patterns used to get a synthesised summary —
    /// `{"pattern_type": "ControlFlow", "repetitions": 0, "variation_score": 0.0,
    /// "example_code": ""}` — which reads as "the dominant construct here is
    /// control flow, seen zero times". Nothing was measured, so nothing is
    /// reported.
    pub fn summary(&self) -> Option<super::violation_detector::PatternSummary> {
        // Most common pattern, with the hash as an explicit tie-break so two runs
        // over the same input cannot pick different "most common" patterns.
        let most_common = self.patterns.values().max_by(|a, b| {
            a.frequency
                .cmp(&b.frequency)
                .then_with(|| b.pattern_hash.cmp(&a.pattern_hash))
        })?;

        Some(super::violation_detector::PatternSummary {
            pattern_type: most_common.pattern_type,
            repetitions: most_common.frequency,
            variation_score: most_common.variation_score,
            example_code: most_common.example_code.clone(),
        })
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
