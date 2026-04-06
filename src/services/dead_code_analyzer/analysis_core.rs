impl DeadCodeAnalyzer {
    /// Default capacity for small to medium projects
    pub const DEFAULT_CAPACITY: usize = 100_000;

    /// Large capacity for enterprise projects
    pub const LARGE_CAPACITY: usize = 1_000_000;

    /// Create a new dead code analyzer
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::dead_code_analyzer::DeadCodeAnalyzer;
    ///
    /// let analyzer = DeadCodeAnalyzer::new(1000);
    /// // Analyzer is ready to analyze up to 1000 nodes
    /// ```
    #[must_use]
    pub fn new(total_nodes: usize) -> Self {
        Self {
            reachability: Arc::new(RwLock::new(HierarchicalBitSet::new(total_nodes))),
            references: Arc::new(RwLock::new(CrossLangReferenceGraph::new())),
            vtable_analysis: Arc::new(RwLock::new(VTableResolver::new())),
            coverage_map: None,
            entry_points: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Add coverage data to improve dead code detection accuracy
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::dead_code_analyzer::{DeadCodeAnalyzer, CoverageData};
    /// use std::collections::{HashMap, HashSet};
    ///
    /// let mut covered_lines = HashMap::new();
    /// covered_lines.insert("main.rs".to_string(), HashSet::new());
    ///
    /// let coverage = CoverageData {
    ///     covered_lines,
    ///     execution_counts: HashMap::new(),
    /// };
    ///
    /// let analyzer = DeadCodeAnalyzer::new(100).with_coverage(coverage);
    /// ```
    #[must_use]
    pub fn with_coverage(mut self, coverage: CoverageData) -> Self {
        self.coverage_map = Some(Arc::new(coverage));
        self
    }

    /// Perform complete dead code analysis
    #[inline]
    pub fn analyze(&mut self, dag: &AstDag) -> DeadCodeReport {
        debug_assert!(!dag.nodes.is_empty(), "AST DAG must have at least one node");
        // Phase 1: Build reference graph from AST
        self.build_reference_graph(dag);

        // Phase 2: Resolve dynamic dispatch
        self.resolve_dynamic_calls();

        // Phase 3: Mark reachable from entry points
        self.mark_reachable_vectorized();

        // Phase 4: Classify dead code by type
        self.classify_dead_code(dag)
    }

    #[inline]
    /// Perform dead code analysis on a dependency graph
    pub fn analyze_dependency_graph(&mut self, dag: &DependencyGraph) -> DeadCodeReport {
        // Phase 1: Build reference graph from dependency graph
        self.build_reference_graph_from_dep_graph(dag);

        // Phase 2: Resolve dynamic dispatch
        self.resolve_dynamic_calls();

        // Phase 3: Mark reachable from entry points
        self.mark_reachable_vectorized();

        // Phase 4: Classify dead code by type for dependency graph
        self.classify_dead_code_from_dep_graph(dag)
    }
}
