// Included from accurate_complexity_analyzer.rs — NO use imports or #! attributes

impl AccurateComplexityAnalyzer {
    /// Analyze a single Rust file
    pub async fn analyze_file(&self, path: &Path) -> Result<FileComplexityResult> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = tokio::fs::read_to_string(path).await?;
        let ast = syn::parse_file(&content)?;

        // Build a lookup of function name -> line number from source text
        let line_map = build_function_line_map(&content);

        let mut functions = Vec::new();

        for item in ast.items {
            if let Item::Fn(func) = item {
                let name = func.sig.ident.to_string();
                let line_start = line_map.get(&name).copied().unwrap_or(0);
                let metrics = self.analyze_function(&func, line_start);
                functions.push(metrics);
            }
        }

        Ok(FileComplexityResult {
            functions,
            file_path: path.display().to_string(),
        })
    }

    /// Analyze an entire project
    pub async fn analyze_project(&self, path: &Path) -> Result<ProjectComplexityResult> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let mut file_metrics = Vec::new();
        let mut files_analyzed = 0;

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let file_path = entry.path();

            // Skip test files if requested
            if self.exclude_tests && self.is_test_file(file_path) {
                continue;
            }

            if let Ok(result) = self.analyze_file(file_path).await {
                files_analyzed += 1;
                file_metrics.push(result);
            }
        }

        Ok(ProjectComplexityResult {
            files_analyzed,
            file_metrics,
        })
    }

    /// Analyze a single function
    fn analyze_function(&self, func: &ItemFn, line_start: u32) -> FunctionMetrics {
        let name = func.sig.ident.to_string();
        let suppressed = self.respect_annotations && self.has_suppress_annotation(&func.attrs);

        let mut visitor = ComplexityVisitor::new().with_function_name(name.clone());
        visitor.visit_item_fn(func);

        FunctionMetrics {
            name,
            cyclomatic_complexity: visitor.cyclomatic,
            cognitive_complexity: visitor.cognitive,
            suppressed,
            line_start,
        }
    }

    /// Check if file is a test file
    fn is_test_file(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let path_str = path.to_string_lossy();
        path_str.contains("/tests/")
            || path_str.contains("/test/")
            || path_str.ends_with("_test.rs")
            || path_str.ends_with("_tests.rs")
            || path_str.contains("test_")
            || path_str.contains("tests.rs")
    }

    /// Check if function has suppression annotation
    fn has_suppress_annotation(&self, attrs: &[Attribute]) -> bool {
        debug_assert!(!attrs.is_empty(), "attrs must not be empty");
        attrs.iter().any(|attr| {
            // Check if it's an allow attribute
            if attr.path().is_ident("allow") {
                // Check if it contains complex_function
                // In syn 2.0, we need to parse the token stream differently
                let tokens_str = attr
                    .meta
                    .require_list()
                    .map(|list| list.tokens.to_string())
                    .unwrap_or_default();
                tokens_str.contains("complex_function")
            } else {
                false
            }
        })
    }
}
