/// Public async function to analyze a C++ file and return FileContext
#[cfg(feature = "cpp-ast")]
#[allow(clippy::cast_possible_truncation)]
pub async fn analyze_cpp_file(
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
    let visitor = CppAstVisitor::new(path);
    let items = visitor
        .analyze_cpp_source(&content)
        .map_err(TemplateError::InvalidUtf8)?;

    // Analyze complexity
    let mut analyzer = CppComplexityAnalyzer::new();
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
        language: "cpp".to_string(),
        items,
        complexity_metrics,
    })
}
