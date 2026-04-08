/// Types of patterns we detect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub frequency: usize,
    pub locations: Vec<Location>,
    pub variation_score: f64, // How much patterns vary (0=identical, 1=very different)
    pub example_code: String,
    pub estimated_loc: usize,
}

/// Collection of patterns found in project
#[derive(Debug, Clone)]
pub struct PatternCollection {
    pub patterns: HashMap<String, AstPattern>,
    pub file_patterns: HashMap<PathBuf, Vec<String>>,
    pub total_files: usize,
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
            patterns: HashMap::new(),
            file_patterns: HashMap::new(),
            total_files: 0,
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
        // For now, return a summary based on the most common pattern
        let most_common = self
            .patterns
            .values()
            .max_by_key(|p| p.frequency)
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
    /// Add pattern.
    pub fn add_pattern(&mut self, pattern: AstPattern) {
        let hash = pattern.pattern_hash.clone();
        self.patterns.insert(hash, pattern);
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

/// Temporary struct - will be replaced with actual context from pmat
#[derive(Debug)]
struct ProjectContext {
    files: HashMap<PathBuf, String>,
}
