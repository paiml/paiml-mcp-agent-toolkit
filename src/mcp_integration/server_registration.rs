impl McpServer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn register_defaults(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Register default tools
        self.register_agent_tools().await?;

        // Register hallucination detection tools (Sprint 40a)
        self.register_hallucination_detection_tools().await?;

        // Register TDG analysis tools (Sprint 40c)
        self.register_tdg_tools().await?;

        // Register JVM language tools (Sprint 51)
        self.register_jvm_tools().await?;

        // Register cross-language analysis tools (Sprint 52)
        self.register_polyglot_tools().await?;

        // Register semantic search tools (PMAT-SEARCH-012)
        // Uses local embeddings - no API keys required
        self.register_semantic_tools().await?;

        // Register default resources
        self.register_agent_resources().await?;

        // Register default prompts
        self.register_agent_prompts().await?;

        Ok(())
    }

    async fn register_agent_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::tools::*;

        let mut tools = self.context.tools.write();

        // Register analyze tool
        tools.register(Arc::new(AnalyzeTool::new(
            self.context.agent_registry.clone(),
        )));

        // Register transform tool
        tools.register(Arc::new(TransformTool::new(
            self.context.agent_registry.clone(),
        )));

        // Register validate tool
        tools.register(Arc::new(ValidateTool::new(
            self.context.agent_registry.clone(),
        )));

        // Register orchestrate tool
        tools.register(Arc::new(OrchestrateTool::new(
            self.context.agent_registry.clone(),
        )));

        Ok(())
    }

    /// Register hallucination detection tools (Sprint 40a)
    ///
    /// Exposes Sprint 37's hallucination detection system via MCP to enable
    /// AI agents to validate documentation claims against the actual codebase.
    ///
    /// Based on peer-reviewed research:
    /// - Semantic Entropy (Farquhar et al., Nature 2024)
    /// - MIND framework (IJCAI 2025)
    /// - Unified Detection Framework (Complex & Intelligent Systems 2025)
    async fn register_hallucination_detection_tools(
        &self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::hallucination_detection_tools::*;

        let mut tools = self.context.tools.write();

        // Register validate_documentation tool
        tools.register(Arc::new(ValidateDocumentationTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered validate_documentation tool");

        // Register check_claim tool
        tools.register(Arc::new(CheckClaimTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered check_claim tool");

        tracing::info!("✅ Hallucination detection tools registered successfully (2 tools)");

        Ok(())
    }

    /// Register TDG (Technical Debt Gradient) analysis tools (Sprint 40c)
    ///
    /// Exposes the TDG quality analysis system via MCP to enable AI agents
    /// to assess code quality and receive actionable improvement recommendations.
    ///
    /// Tools:
    /// - `analyze_technical_debt`: Analyze quality scores for files/projects
    /// - `get_quality_recommendations`: Get actionable refactoring suggestions
    async fn register_tdg_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::tdg_tools::*;

        let mut tools = self.context.tools.write();

        // Register analyze_technical_debt tool
        tools.register(Arc::new(AnalyzeTechnicalDebtTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered analyze_technical_debt tool");

        // Register get_quality_recommendations tool
        tools.register(Arc::new(GetQualityRecommendationsTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered get_quality_recommendations tool");

        tracing::info!("✅ TDG analysis tools registered successfully (2 tools)");

        Ok(())
    }

    /// Register JVM language tools (Sprint 51)
    ///
    /// Sprint 51 added comprehensive JVM language support with Java and Scala
    /// analysis. These tools provide:
    ///
    /// - Java analysis: Class, interface, and method detection with complexity metrics
    /// - Scala analysis: Classes, traits, objects, and case classes with functional metrics
    /// - Mutation testing for Java and Scala code
    ///
    /// Both tools are feature-gated behind the "java-ast" and "scala-ast" features.
    async fn register_jvm_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut tools = self.context.tools.write();
        let mut registered_tools = 0;

        // Register Java analysis tools if feature is enabled
        #[cfg(feature = "java-ast")]
        {
            use crate::mcp_integration::java_tools::*;

            // Register Java analysis tool
            tools.register(Arc::new(JavaAnalysisTool::new(
                self.context.agent_registry.clone(),
            )));
            tracing::info!("✓ Registered analyze_java tool");
            registered_tools += 1;

            // Register Java mutation testing tool
            tools.register(Arc::new(JavaMutationTool::new(
                self.context.agent_registry.clone(),
            )));
            tracing::info!("✓ Registered mutation_test_java tool");
            registered_tools += 1;
        }

        // Register Scala analysis tools if feature is enabled
        #[cfg(feature = "scala-ast")]
        {
            use crate::mcp_integration::scala_tools::*;

            // Register Scala analysis tool
            tools.register(Arc::new(ScalaAnalysisTool::new(
                self.context.agent_registry.clone(),
            )));
            tracing::info!("✓ Registered analyze_scala tool");
            registered_tools += 1;

            // Register Scala mutation testing tool
            tools.register(Arc::new(ScalaMutationTool::new(
                self.context.agent_registry.clone(),
            )));
            tracing::info!("✓ Registered mutation_test_scala tool");
            registered_tools += 1;
        }

        if registered_tools > 0 {
            tracing::info!(
                "✅ JVM language tools registered successfully ({} tools)",
                registered_tools
            );
        } else {
            tracing::info!("ℹ️ No JVM language tools registered (features not enabled)");
        }

        Ok(())
    }

    /// Register cross-language analysis tools (Sprint 52)
    ///
    /// Sprint 52 adds cross-language analysis capabilities to detect relationships
    /// between different programming languages in a project. These tools build upon
    /// the JVM language support from Sprint 51 and extend it to create a unified
    /// polyglot analysis framework.
    ///
    /// Tools:
    /// - analyze_polyglot: Analyzes cross-language relationships in a project
    /// - detect_language_boundaries: Detects language boundaries and interop points
    async fn register_polyglot_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::polyglot_tools::*;

        let mut tools = self.context.tools.write();

        // Register polyglot analysis tool
        tools.register(Arc::new(PolyglotAnalysisTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered analyze_polyglot tool");

        // Register language boundary tool
        tools.register(Arc::new(LanguageBoundaryTool::new(
            self.context.agent_registry.clone(),
        )));
        tracing::info!("✓ Registered detect_language_boundaries tool");

        tracing::info!("✅ Cross-language analysis tools registered successfully (2 tools)");

        Ok(())
    }

    /// Register semantic search tools (PMAT-SEARCH-012)
    ///
    /// GREEN Phase: Partial implementation (needs adapter layer)
    ///
    /// The semantic search tools are fully implemented in `src/mcp/tools/semantic_search_tools.rs`
    /// but they implement a simpler `crate::mcp::McpTool` trait.
    ///
    /// The mcp_integration framework expects tools to implement `mcp_integration::McpTool` trait
    /// which has a different interface (metadata() vs name()/schema(), and different error types).
    ///
    /// **Architecture Note**: There are two MCP tool systems in the codebase:
    /// 1. **Simple MCP** (`src/mcp/`) - Used for semantic search tools
    ///    - Trait: `crate::mcp::McpTool`
    ///    - Methods: name(), schema(), execute() -> Result<Value, String>
    ///
    /// 2. **MCP Integration** (`src/mcp_integration/`) - Used for agent-based tools
    ///    - Trait: `mcp_integration::McpTool`
    ///    - Methods: metadata(), execute() -> Result<Value, McpError>
    ///
    /// **TODO**: Create adapter layer to bridge the two systems. Options:
    /// 1. Create wrapper structs that implement mcp_integration::McpTool and delegate to semantic tools
    /// 2. Unify the two MCP tool systems into a single interface
    /// 3. Keep them separate and register semantic tools via a different mechanism
    ///
    /// **Current Status**:
    /// - ✅ Configuration system complete (ServerConfig has semantic fields)
    /// - ✅ HybridSearchEngine initialization works
    /// - ✅ Semantic tools fully implemented and tested (149 tests passing)
    /// - ✅ CLI integration complete and working
    /// - 🔧 MCP integration requires adapter layer (estimated: 2-3 hours)
    ///
    /// Configuration is loaded from environment variables:
    /// - PMAT_SEMANTIC_ENABLED: Enable semantic search (default: false)
    /// - PMAT_VECTOR_DB_PATH: Path to vector database (default: ~/.pmat/embeddings.db)
    /// - PMAT_WORKSPACE: Workspace path for code indexing (default: current directory)
    ///
    /// NOTE: No API keys required - uses local embeddings via aprender/trueno-rag
    async fn register_semantic_tools(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if semantic search is enabled
        if !self.config.semantic_enabled {
            tracing::info!(
                "Semantic search is disabled (set PMAT_SEMANTIC_ENABLED=true to enable)"
            );
            return Ok(());
        }

        // Sprint 33 (PMAT-SEARCH-012 GREEN): Adapter layer implementation complete
        use crate::mcp_integration::tools::*;
        use crate::services::semantic::HybridSearchEngine;

        let db_path = self.config.semantic_db_path.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| {
                    h.join(".pmat")
                        .join("embeddings.db")
                        .to_string_lossy()
                        .to_string()
                })
                .unwrap_or_else(|| "embeddings.db".to_string())
        });

        let workspace = self
            .config
            .semantic_workspace
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Initialize hybrid search engine (no API key required - uses local embeddings)
        tracing::info!(
            "Initializing semantic search engine: db={}, workspace={}",
            db_path,
            workspace.display()
        );

        let engine = match HybridSearchEngine::new(&db_path, &workspace).await {
            Ok(engine) => Arc::new(engine),
            Err(e) => {
                tracing::warn!("Failed to initialize semantic search engine: {}", e);
                tracing::info!("Semantic search tools will not be available");
                return Ok(());
            }
        };

        // Register semantic search tools via adapter layer
        let mut tools = self.context.tools.write();

        tools.register(Arc::new(SemanticSearchToolAdapter::new(engine.clone())));
        tracing::info!("✓ Registered semantic_search tool");

        tools.register(Arc::new(FindSimilarCodeToolAdapter::new(engine.clone())));
        tracing::info!("✓ Registered find_similar_code tool");

        tools.register(Arc::new(ClusterCodeToolAdapter::new(engine.clone())));
        tracing::info!("✓ Registered cluster_code tool");

        tools.register(Arc::new(AnalyzeTopicsToolAdapter::new(engine)));
        tracing::info!("✓ Registered analyze_topics tool");

        tracing::info!("✅ Semantic search tools registered successfully (4 tools)");

        Ok(())
    }

    async fn register_agent_resources(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::resources::*;

        let mut resources = self.context.resources.write();

        // Register agent state resource
        resources.register(Arc::new(AgentStateResource::new(
            self.context.agent_registry.clone(),
        )));

        // Register metrics resource
        resources.register(Arc::new(MetricsResource::new(
            self.context.agent_registry.clone(),
        )));

        // Register quality report resource
        resources.register(Arc::new(QualityReportResource::new()));

        Ok(())
    }

    async fn register_agent_prompts(&self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::mcp_integration::prompts::*;

        let mut prompts = self.context.prompts.write();

        // Register code analysis prompt
        prompts.register(Arc::new(CodeAnalysisPrompt::new()));

        // Register refactoring prompt
        prompts.register(Arc::new(RefactoringPrompt::new()));

        // Register quality assessment prompt
        prompts.register(Arc::new(QualityAssessmentPrompt::new()));

        // Register repository health scoring prompt
        prompts.register(Arc::new(RepoScorePrompt::new()));

        Ok(())
    }
}
