/// Main code intelligence interface
pub struct CodeIntelligence {
    dag: Arc<RwLock<AstDag>>,
    deadcode: Arc<RwLock<DeadCodeAnalyzer>>,
    cache: Arc<UnifiedCache>,
}

impl Default for CodeIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeIntelligence {
    /// Creates a new code intelligence instance with default configuration.
    ///
    /// Initializes an empty AST DAG, dead code analyzer with 10,000 initial capacity,
    /// and unified cache with 100 report capacity.
    ///
    /// # Performance Characteristics
    ///
    /// - Memory: ~100MB initial allocation for internal structures
    /// - Cache: 100 analysis reports (configurable via `UnifiedCache::new`)
    /// - Dead code analyzer: 10,000 node capacity (grows dynamically)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::code_intelligence::CodeIntelligence;
    ///
    /// let intelligence = CodeIntelligence::new();
    ///
    /// // Verify initial state
    /// # tokio_test::block_on(async {
    /// let (nodes, generation) = intelligence.get_dag_stats().await;
    /// assert_eq!(nodes, 0); // No nodes initially
    /// assert_eq!(generation, 0); // No analysis runs yet
    /// # });
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        let dag = Arc::new(RwLock::new(AstDag::new()));

        Self {
            dag,
            deadcode: Arc::new(RwLock::new(DeadCodeAnalyzer::new(10000))), // Initial capacity
            cache: Arc::new(UnifiedCache::new(100)),
        }
    }

    /// Retrieves current AST DAG statistics for monitoring and debugging.
    ///
    /// Returns the current state of the internal DAG structure, useful for
    /// performance monitoring and understanding analysis scope.
    ///
    /// # Returns
    ///
    /// * `(node_count, generation)` - Tuple containing:
    ///   - `node_count`: Total number of AST nodes currently in the DAG
    ///   - `generation`: Number of analysis runs performed (increments with each analysis)
    ///
    /// # Performance
    ///
    /// - Time: O(1) - direct field access
    /// - Space: O(1) - no allocation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::services::code_intelligence::CodeIntelligence;
    ///
    /// # tokio_test::block_on(async {
    /// let intelligence = CodeIntelligence::new();
    ///
    /// // Initially empty
    /// let (nodes, gen) = intelligence.get_dag_stats().await;
    /// assert_eq!(nodes, 0);
    /// assert_eq!(gen, 0);
    ///
    /// // After analysis, stats should reflect changes
    /// // (This example assumes a successful analysis has been run)
    /// # });
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn get_dag_stats(&self) -> (usize, u32) {
        let dag = self.dag.read().await;
        (dag.nodes.len(), dag.generation())
    }
}

include!("code_intelligence_engine_analysis.rs");
include!("code_intelligence_engine_dag.rs");
