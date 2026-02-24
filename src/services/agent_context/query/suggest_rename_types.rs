// ── Types ──────────────────────────────────────────────────────────────────

/// Signal type used to determine the suggested name
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum RenameSignal {
    /// Single dominant struct/enum/trait → snake_case of that name
    DominantType,
    /// Existing suffix expansion (_attn → attention, _ops → operations)
    ExistingSuffix,
    /// Original filename base (before _part_ splitting) is meaningful
    OriginalBase,
    /// >70% of functions share a keyword theme (forward, serialize, test, etc.)
    FunctionTheme,
    /// Longest common prefix across all function names (min 4 chars)
    CommonPrefix,
    /// Dominant keyword extracted from doc comments
    DocCommentConsensus,
    /// Multiple weak signals combined
    Mixed,
    /// No meaningful signal found
    NoSignal,
}

/// A rename suggestion for a single `_part_` file
#[derive(Debug, Clone, Serialize)]
pub struct RenameSuggestion {
    /// Current file path (relative to project root)
    pub current_path: String,
    /// Suggested new filename (just the stem, no directory)
    pub suggested_name: String,
    /// Full suggested path
    pub suggested_path: String,
    /// Confidence score 0.0-1.0
    pub confidence: f32,
    /// Human-readable reasoning
    pub reasoning: String,
    /// Signal type that produced this suggestion
    pub signal: RenameSignal,
    /// Parent file that include!()s or #[path=] this file (if detected)
    pub parent_file: Option<String>,
    /// Inclusion pattern (include! or #[path])
    pub inclusion_pattern: Option<String>,
    /// Number of definitions in the file
    pub definition_count: usize,
}

// ── Suffix expansion table ─────────────────────────────────────────────────

const SUFFIX_EXPANSIONS: &[(&str, &str)] = &[
    ("_attn", "attention"),
    ("_ops", "operations"),
    ("_impl", "implementation"),
    ("_util", "utilities"),
    ("_cfg", "config"),
    ("_fmt", "formatting"),
    ("_conv", "conversion"),
    ("_init", "initialization"),
    ("_exec", "execution"),
    ("_proc", "processing"),
    ("_gen", "generation"),
    ("_val", "validation"),
    ("_ser", "serialization"),
    ("_deser", "deserialization"),
    ("_alloc", "allocation"),
    ("_disp", "dispatch"),
    ("_fwd", "forward"),
    ("_bwd", "backward"),
    ("_norm", "normalization"),
    ("_trans", "transform"),
];

// ── Theme keywords ─────────────────────────────────────────────────────────

const THEME_KEYWORDS: &[(&str, &str)] = &[
    ("forward", "forward"),
    ("backward", "backward"),
    ("serialize", "serialization"),
    ("deserialize", "deserialization"),
    ("parse", "parsing"),
    ("format", "formatting"),
    ("render", "rendering"),
    ("validate", "validation"),
    ("encode", "encoding"),
    ("decode", "decoding"),
    ("build", "builder"),
    ("create", "construction"),
    ("init", "initialization"),
    ("load", "loading"),
    ("save", "persistence"),
    ("read", "reading"),
    ("write", "writing"),
    ("convert", "conversion"),
    ("transform", "transform"),
    ("display", "display"),
    ("test", "tests"),
    ("bench", "benchmarks"),
    ("config", "config"),
    ("error", "errors"),
    ("handle", "handler"),
    ("dispatch", "dispatch"),
    ("compute", "computation"),
    ("calculate", "calculation"),
    ("process", "processing"),
    ("analyze", "analysis"),
    ("cache", "cache"),
    ("index", "indexing"),
    ("search", "search"),
    ("query", "query"),
    ("emit", "emission"),
    ("collect", "collection"),
    ("merge", "merging"),
    ("sort", "sorting"),
    ("filter", "filtering"),
    ("map", "mapping"),
    ("reduce", "reduction"),
];
