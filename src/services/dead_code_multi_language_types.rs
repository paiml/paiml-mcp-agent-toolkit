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

/// Analyze dead code using appropriate strategy for the project language
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn analyze_dead_code_multi_language(path: &Path) -> Result<DeadCodeResult> {
    info!("Starting multi-language dead code analysis at: {:?}", path);

    // Step 1: Detect language using enhanced detection from BUG-011
    let detection =
        crate::services::enhanced_language_detection::detect_project_language_enhanced(path);

    debug!(
        "Detected language: {} (confidence: {:.1}%)",
        detection.language, detection.confidence
    );

    // Step 2: Select appropriate strategy
    let strategy: Box<dyn DeadCodeStrategy> = match detection.language.as_str() {
        "rust" => Box::new(RustDeadCodeStrategy),
        "c" => Box::new(CDeadCodeStrategy),
        "cpp" => Box::new(CppDeadCodeStrategy),
        "python" => Box::new(PythonDeadCodeStrategy),
        "lua" => Box::new(LuaDeadCodeStrategy),
        _ => {
            return Err(anyhow::anyhow!(
                "Dead code analysis not supported for language: {}. Supported: rust, c, cpp, python, lua",
                detection.language
            ));
        }
    };

    // Step 3: Run analysis
    strategy.analyze(path)
}
