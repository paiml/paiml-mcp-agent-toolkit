/// Dead code analysis result
#[derive(Debug, Clone, PartialEq)]
pub struct DeadCodeResult {
    pub language: String,
    pub dead_functions: Vec<DeadFunction>,
    pub total_functions: usize,
    /// Source files the strategy actually walked (#720).
    ///
    /// The caller used to report `total_functions.max(1)` as its file count, so a
    /// 2-file Python fixture with 4 functions printed "Files Analyzed | 4" while
    /// the summary beside it correctly said 2. Every strategy already has the
    /// file list in hand, so this is the count of that list -- never a function
    /// count, and never `.max(1)` inventing a file for an empty project.
    pub total_files: usize,
    pub dead_code_percentage: f64,
    /// Whether the analysed target is a LIBRARY, and hence whether its exported
    /// items were treated as reachability roots rather than as dead code.
    ///
    /// A library's public API is un-called by construction — its callers are
    /// outside the tree — so an engine that has only "nothing calls it" to go on
    /// reports the whole API as dead. This records which way the question was
    /// answered, INCLUDING when it could not be answered, because the reader has
    /// to know whether an un-called export was kept or listed. See
    /// [`LibraryTarget`].
    pub library_target: LibraryTarget,
    /// How many exported definitions `library_target` turned into reachability
    /// roots. Always `0` unless the verdict is
    /// [`LibraryTarget::Library`] — an undetermined verdict seeds nothing, which
    /// is the fact the disclosure exists to convey.
    pub exported_roots: usize,
}

#[derive(Debug, Clone, PartialEq)]
/// Dead function.
pub struct DeadFunction {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub reason: String,
}

/// Strategy trait for language-specific dead code detection
pub trait DeadCodeStrategy {
    /// Analyze dead code in the given project
    fn analyze(&self, path: &Path) -> Result<DeadCodeResult>;

    /// Get the language this strategy handles
    fn language(&self) -> &str;
}

/// Languages with a dead-code strategy, and the extensions that identify them.
///
/// Used to answer "is there anything here I *can* analyse?" when the project's
/// dominant language has no strategy.
const DEAD_CODE_SUPPORTED_LANGUAGES: &[(&str, &[&str])] = &[
    ("rust", &["rs"]),
    ("c", &["c", "h"]),
    ("cpp", &["cpp", "cc", "cxx", "hpp", "hxx"]),
    ("python", &["py"]),
    ("lua", &["lua"]),
];

fn dead_code_strategy_for(language: &str) -> Option<Box<dyn DeadCodeStrategy>> {
    match language {
        "rust" => Some(Box::new(RustDeadCodeStrategy)),
        "c" => Some(Box::new(CDeadCodeStrategy)),
        "cpp" => Some(Box::new(CppDeadCodeStrategy)),
        "python" => Some(Box::new(PythonDeadCodeStrategy)),
        "lua" => Some(Box::new(LuaDeadCodeStrategy)),
        _ => None,
    }
}

/// Analyze dead code using appropriate strategy for the project language
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn analyze_dead_code_multi_language(path: &Path) -> Result<DeadCodeResult> {
    info!("Starting multi-language dead code analysis at: {:?}", path);

    // A missing path used to fall through to language detection, which returned
    // "unknown", so the user was told "not supported for language: unknown"
    // instead of that the path does not exist.
    if !path.exists() {
        return Err(anyhow::anyhow!("Path not found: {}", path.display()));
    }

    // Step 1: Detect language using enhanced detection from BUG-011
    let detection =
        crate::services::enhanced_language_detection::detect_project_language_enhanced(path);

    debug!(
        "Detected language: {} (confidence: {:.1}%)",
        detection.language, detection.confidence
    );

    // Step 2: Select a strategy. Detection reports ONE dominant language, so a
    // tree whose plurality language has no strategy (a TypeScript app with a
    // handful of Python scripts) used to abort the whole run even though every
    // supported file in it was analysable. Fall back to a language that is
    // actually present rather than refusing outright.
    let strategy: Box<dyn DeadCodeStrategy> = match dead_code_strategy_for(&detection.language) {
        Some(s) => s,
        None => {
            let fallback = DEAD_CODE_SUPPORTED_LANGUAGES
                .iter()
                .find(|(_, exts)| !find_files_by_extension(path, exts).is_empty())
                .and_then(|(lang, _)| {
                    debug!(
                        "No strategy for detected language {}; falling back to {lang}",
                        detection.language
                    );
                    dead_code_strategy_for(lang)
                });
            match fallback {
                Some(s) => s,
                None => {
                    return Err(anyhow::anyhow!(
                        "Dead code analysis not supported for language: {}, and no rust, c, cpp, python or lua source files were found under {}",
                        detection.language,
                        path.display()
                    ));
                }
            }
        }
    };

    // Step 3: Run analysis
    strategy.analyze(path)
}
