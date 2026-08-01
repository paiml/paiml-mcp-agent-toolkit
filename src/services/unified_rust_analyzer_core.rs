impl UnifiedRustAnalyzer {
    /// Create new analyzer for a file
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            #[cfg(test)]
            parse_count: AtomicUsize::new(0),
        }
    }

    /// Get the file path being analyzed
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Analyze file with single parse
    ///
    /// This is the core GREEN phase implementation: minimal but correct.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze(&self) -> Result<UnifiedAnalysis, AnalysisError> {
        // Track parse count for testing
        #[cfg(test)]
        {
            self.parse_count.fetch_add(1, Ordering::SeqCst);
        }

        // 1. Read file content (single I/O operation)
        let content = tokio::fs::read_to_string(&self.file_path)
            .await
            .map_err(AnalysisError::Io)?;

        // 2. Parse ONCE with syn
        let syntax_tree =
            syn::parse_file(&content).map_err(|e| AnalysisError::Parse(e.to_string()))?;

        // 3. Extract AST items using existing EnhancedAstVisitor
        let ast_items = self.extract_ast_items(&syntax_tree, &content);

        // 4. Extract complexity metrics (measured against the same source text)
        let file_metrics = self.extract_complexity_metrics(&syntax_tree, &content);

        Ok(UnifiedAnalysis {
            ast_items,
            file_metrics,
            parsed_at: std::time::Instant::now(),
        })
    }

    /// Get parse count (test-only, for verifying single parse)
    #[cfg(test)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn parse_count(&self) -> usize {
        self.parse_count.load(Ordering::SeqCst)
    }

    /// Extract AST items from parsed syntax tree
    fn extract_ast_items(&self, syntax_tree: &syn::File, source: &str) -> Vec<AstItem> {
        // `source` is required: without it the visitor cannot resolve real line
        // numbers and previously fell back to an enumeration index.
        let visitor = EnhancedAstVisitor::new(&self.file_path, source);
        visitor.extract_items(syntax_tree)
    }
}
