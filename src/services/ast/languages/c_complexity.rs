// CComplexityAnalyzer implementation and analyze_c_file async function.
// This file is included into c.rs and shares its module scope (no `use` imports here).

impl CComplexityAnalyzer {
    /// Creates a new C complexity analyzer
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
        }
    }

    /// Analyzes complexity of C source code (complexity ≤10)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_complexity(&mut self, source: &str) -> Result<(u32, u32), String> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        self.cyclomatic_complexity = 1;
        self.cognitive_complexity = 0;

        let mut nesting_depth = 0;

        for line in source.lines() {
            let trimmed = line.trim();

            // Skip comments and preprocessor directives
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#") {
                continue;
            }

            // Check for control flow statements
            let is_if_stmt = trimmed.starts_with("if ") || trimmed.starts_with("if(");
            let is_else_if = trimmed.starts_with("else if") || trimmed.contains("} else if");
            let _is_else = trimmed.starts_with("else") && !is_else_if;
            let is_switch = trimmed.starts_with("switch ");
            let is_case = trimmed.starts_with("case ") || trimmed.starts_with("default:");
            let is_loop = trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("do ");
            let is_goto = trimmed.starts_with("goto ");

            // Increment cyclomatic complexity for decision points
            if is_if_stmt || is_else_if || is_switch || is_case || is_loop || is_goto {
                self.cyclomatic_complexity += 1;
            }

            // Cognitive complexity considers nesting
            if is_if_stmt || is_else_if || is_switch || is_loop {
                self.cognitive_complexity += 1 + nesting_depth;
                nesting_depth += 1;
            } else if trimmed.contains("{") && !trimmed.contains("}") {
                // Opening a block (not a one-line block)
                nesting_depth += 1;
            }

            // Track closing braces
            if trimmed.contains("}") {
                nesting_depth = nesting_depth.saturating_sub(1);
            }
        }

        Ok((self.cyclomatic_complexity, self.cognitive_complexity))
    }
}

/// Public async function to analyze a C file and return FileContext
#[cfg(feature = "c-ast")]
#[allow(clippy::cast_possible_truncation)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn analyze_c_file(
    path: &Path,
) -> Result<crate::services::context::FileContext, crate::models::error::TemplateError> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    use crate::models::error::TemplateError;
    use crate::services::complexity::ComplexityMetrics;
    use crate::services::context::FileContext;

    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(TemplateError::Io)?;

    // Create visitor and analyze
    let visitor = CAstVisitor::new(path);
    let items = visitor
        .analyze_c_source(&content)
        .map_err(TemplateError::InvalidUtf8)?;

    // Analyze complexity
    let mut analyzer = CComplexityAnalyzer::new();
    let (cyclomatic, cognitive) = analyzer
        .analyze_complexity(&content)
        .map_err(TemplateError::InvalidUtf8)?;

    // Convert to correct types for ComplexityMetrics::new
    // Create function complexity metrics
    let func_metrics = ComplexityMetrics::new(
        (cyclomatic & 0xFFFF) as u16,             // Convert to u16 with clamping
        (cognitive & 0xFFFF) as u16,              // Convert to u16 with clamping
        0,                                        // nesting_max (not calculated)
        std::cmp::min(items.len(), 65535) as u16, // lines (clamped to u16 max)
    );

    // Create a file complexity metrics object
    let file_complexity = crate::services::complexity::FileComplexityMetrics {
        path: path.display().to_string(),
        total_complexity: func_metrics,
        functions: vec![crate::services::complexity::FunctionComplexity {
            name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            line_start: 1,
            line_end: std::cmp::min(items.len(), 65535) as u32,
            metrics: func_metrics,
        }],
        classes: vec![],
    };

    let complexity_metrics = Some(file_complexity);

    // Return FileContext
    Ok(FileContext {
        path: path.display().to_string(),
        language: "c".to_string(),
        items,
        complexity_metrics,
    })
}
