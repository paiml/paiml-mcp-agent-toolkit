impl TdgAnalyzerAst {
    pub fn new() -> Result<Self> {
        Ok(Self {
            config: TdgConfig::default(),
            storage: None,
            scheduler: None,
            adaptive_manager: None,
            resource_controller: None,
            git_context: None,
        })
    }

    pub fn with_config(config: TdgConfig) -> Result<Self> {
        Ok(Self {
            config,
            storage: None,
            scheduler: None,
            adaptive_manager: None,
            resource_controller: None,
            git_context: None,
        })
    }

    pub fn with_storage(config: TdgConfig) -> Result<Self> {
        let storage = TieredStorageFactory::create_default()?;
        let scheduler = SchedulerFactory::create_balanced();
        let adaptive_manager = AdaptiveThresholdFactory::create_default();
        let resource_controller = ResourceControllerFactory::create_default();
        Ok(Self {
            config,
            storage: Some(storage),
            scheduler: Some(scheduler),
            adaptive_manager: Some(adaptive_manager),
            resource_controller: Some(resource_controller),
            git_context: None,
        })
    }

    /// Create analyzer with in-memory storage for testing (no file I/O conflicts)
    #[cfg(test)]
    pub fn with_in_memory_storage(config: TdgConfig) -> Self {
        let storage = TieredStorageFactory::create_in_memory();
        let scheduler = SchedulerFactory::create_balanced();
        let adaptive_manager = AdaptiveThresholdFactory::create_default();
        let resource_controller = ResourceControllerFactory::create_default();
        Self {
            config,
            storage: Some(storage),
            scheduler: Some(scheduler),
            adaptive_manager: Some(adaptive_manager),
            resource_controller: Some(resource_controller),
            git_context: None,
        }
    }

    /// Create analyzer with full resource management for production
    pub async fn with_full_resource_management(config: TdgConfig) -> Result<Self> {
        let storage = TieredStorageFactory::create_default()?;
        let scheduler = SchedulerFactory::create_background_optimized();
        let adaptive_manager = AdaptiveThresholdFactory::create_prod_optimized();
        let resource_controller = ResourceControllerFactory::create_prod_optimized();

        // Start resource monitoring
        resource_controller.start_monitoring().await?;

        Ok(Self {
            config,
            storage: Some(storage),
            scheduler: Some(scheduler),
            adaptive_manager: Some(adaptive_manager),
            resource_controller: Some(resource_controller),
            git_context: None,
        })
    }

    /// Sprint 65: Set git context for commit correlation
    /// This should be called before analyze_file() when --with-git-context flag is enabled
    pub fn set_git_context(&mut self, git_context: Option<crate::models::git_context::GitContext>) {
        self.git_context = git_context;
    }

    /// Sprint 65: Get git context for output formatting
    pub fn get_git_context(&self) -> Option<&crate::models::git_context::GitContext> {
        self.git_context.as_ref()
    }

    /// Sprint 65 Phase 3: Get storage reference for history queries
    pub fn storage(&self) -> Option<&TieredStore> {
        self.storage.as_ref()
    }

    pub async fn analyze_file(&self, path: &Path) -> Result<TdgScore> {
        self.analyze_file_with_priority(path, OperationPriority::Medium)
            .await
    }

    pub async fn analyze_file_with_priority(
        &self,
        path: &Path,
        priority: OperationPriority,
    ) -> Result<TdgScore> {
        let start_time = SystemTime::now();
        let language = Language::from_extension(path);

        // Toyota Way Extract Method: Resource allocation
        let _resource_allocation = self.request_analysis_resources(path, priority).await?;

        let source = fs::read_to_string(path)?;
        let content_hash = blake3::hash(source.as_bytes());

        // Toyota Way Extract Method: Cache check and return if hit
        if let Some(cached_score) = self
            .check_cache_and_return(&content_hash, language, path, start_time)
            .await?
        {
            return Ok(cached_score);
        }

        // Toyota Way Extract Method: Fresh analysis and storage
        let score = self
            .perform_analysis_and_store(path, &source, language, content_hash, start_time)
            .await?;

        Ok(score)
    }

    /// Toyota Way Extract Method: Request analysis resources if controller available
    async fn request_analysis_resources(
        &self,
        path: &Path,
        priority: OperationPriority,
    ) -> Result<Option<crate::tdg::resource_control::ResourceAllocation>> {
        if let Some(controller) = &self.resource_controller {
            let estimated_memory = self.estimate_analysis_memory(path)?;
            Ok(Some(
                controller
                    .request_resources(
                        format!("analyze_{}", path.display()),
                        crate::tdg::resource_control::OperationType::Analysis,
                        priority,
                        estimated_memory,
                    )
                    .await?,
            ))
        } else {
            Ok(None)
        }
    }

    /// Toyota Way Extract Method: Check cache and return score if hit
    async fn check_cache_and_return(
        &self,
        content_hash: &blake3::Hash,
        language: Language,
        path: &Path,
        start_time: SystemTime,
    ) -> Result<Option<TdgScore>> {
        if let Some(storage) = &self.storage {
            if let Some(hot_entry) = storage.get_hot(content_hash) {
                // Record performance sample for cache hit
                if let Some(adaptive) = &self.adaptive_manager {
                    let duration = start_time.elapsed().unwrap_or_default();
                    let sample = adaptive.create_sample(duration, true, 0).await;
                    adaptive.record_sample(sample).await?;
                }

                // Return cached score with updated timestamp
                let mut cached_score = TdgScore {
                    total: hot_entry.total_score,
                    grade: Grade::from_score(hot_entry.total_score),
                    language,
                    confidence: language.confidence(),
                    file_path: Some(path.to_path_buf()),
                    ..Default::default()
                };
                cached_score.calculate_total();
                return Ok(Some(cached_score));
            }
        }
        Ok(None)
    }

    /// Toyota Way Extract Method: Perform fresh analysis and store results
    async fn perform_analysis_and_store(
        &self,
        path: &Path,
        source: &str,
        language: Language,
        content_hash: blake3::Hash,
        start_time: SystemTime,
    ) -> Result<TdgScore> {
        // Perform fresh analysis
        let analysis_start = SystemTime::now();
        let score = self.analyze_source(source, language, Some(path.to_path_buf()))?;
        let analysis_duration = analysis_start.elapsed().unwrap_or_default();

        // Store in tiered storage if enabled
        self.store_analysis_record(path, &score, content_hash, analysis_duration, language)
            .await?;

        // Record performance sample for fresh analysis
        if let Some(adaptive) = &self.adaptive_manager {
            let total_duration = start_time.elapsed().unwrap_or_default();
            let sample = adaptive.create_sample(total_duration, false, 0).await;
            adaptive.record_sample(sample).await?;
        }

        Ok(score)
    }

    /// Toyota Way Extract Method: Store analysis record in tiered storage
    async fn store_analysis_record(
        &self,
        path: &Path,
        score: &TdgScore,
        content_hash: blake3::Hash,
        analysis_duration: Duration,
        language: Language,
    ) -> Result<()> {
        if let Some(storage) = &self.storage {
            let file_metadata = fs::metadata(path)?;
            let record = FullTdgRecord {
                identity: FileIdentity {
                    path: path.to_path_buf(),
                    content_hash,
                    size_bytes: file_metadata.len(),
                    modified_time: file_metadata.modified().unwrap_or(SystemTime::now()),
                },
                score: score.clone(),
                components: ComponentScores {
                    complexity_breakdown: std::collections::HashMap::new(),
                    duplication_sources: Vec::new(),
                    coupling_dependencies: Vec::new(),
                    doc_missing_items: Vec::new(),
                    consistency_violations: Vec::new(),
                },
                semantic_sig: SemanticSignature {
                    ast_structure_hash: u64::from_le_bytes(
                        content_hash.as_bytes()[0..8]
                            .try_into()
                            .expect("slice with incorrect length"),
                    ),
                    identifier_pattern: String::new(),
                    control_flow_pattern: String::new(),
                    import_dependencies: Vec::new(),
                },
                metadata: AnalysisMetadata {
                    analyzer_version: env!("CARGO_PKG_VERSION").to_string(),
                    analysis_duration_ms: analysis_duration.as_millis() as u64,
                    language_confidence: language.confidence(),
                    analysis_timestamp: SystemTime::now(),
                    cache_hit: false,
                },
                git_context: self.git_context.clone(), // Sprint 65: Git-commit correlation
            };

            storage.store(record).await?;
        }
        Ok(())
    }

    /// Analyze file with commit priority (for git hooks, CI/CD)
    pub async fn analyze_file_commit(&self, path: &Path) -> Result<TdgScore> {
        let _guard = if let Some(scheduler) = &self.scheduler {
            Some(
                scheduler
                    .schedule_commit(path.to_path_buf())
                    .await
                    .map_err(|e| anyhow::anyhow!("Scheduling failed: {e}"))?,
            )
        } else {
            None
        };

        self.analyze_file_with_priority(path, OperationPriority::Critical)
            .await
    }

    /// Analyze file with background priority (for daemon, IDE plugins)
    pub async fn analyze_file_background(&self, path: &Path) -> Result<TdgScore> {
        let _guard = if let Some(scheduler) = &self.scheduler {
            Some(
                scheduler
                    .schedule_background(path.to_path_buf())
                    .await
                    .map_err(|e| anyhow::anyhow!("Scheduling failed: {e}"))?,
            )
        } else {
            None
        };

        self.analyze_file_with_priority(path, OperationPriority::Low)
            .await
    }

    /// Get scheduler statistics for diagnostics
    pub async fn get_scheduler_stats(&self) -> Option<crate::tdg::SchedulingStatistics> {
        if let Some(scheduler) = &self.scheduler {
            Some(scheduler.get_statistics().await)
        } else {
            None
        }
    }

    // GREEN Phase: Public methods for TDG dogfooding - accessing stored scores

    /// Get a reference to the storage system for querying stored scores
    #[must_use]
    pub fn get_storage(&self) -> Option<&TieredStore> {
        self.storage.as_ref()
    }

    /// Get stored score for a specific file
    pub async fn get_stored_score(&self, path: &Path) -> Result<Option<TdgScore>> {
        if let Some(storage) = &self.storage {
            // Calculate content hash for the file
            let source = fs::read_to_string(path)?;
            let content_hash = blake3::hash(source.as_bytes());

            // Check hot cache first
            if let Some(hot_entry) = storage.get_hot(&content_hash) {
                let language = Language::from_extension(path);
                let score = TdgScore {
                    total: hot_entry.total_score,
                    grade: Grade::from_score(hot_entry.total_score),
                    language,
                    confidence: language.confidence(),
                    file_path: Some(path.to_path_buf()),
                    ..Default::default()
                };
                return Ok(Some(score));
            }

            // Check warm/cold storage
            if let Some(record) = storage.retrieve_full(&content_hash).await? {
                return Ok(Some(record.score));
            }
        }
        Ok(None)
    }

    /// Get storage statistics for monitoring
    #[must_use]
    pub fn get_storage_stats(&self) -> Option<crate::tdg::StorageStatistics> {
        self.storage
            .as_ref()
            .map(super::storage::TieredStore::get_statistics)
    }

    /// Get adaptive threshold statistics for diagnostics
    pub async fn get_adaptive_stats(&self) -> Option<crate::tdg::PerformanceStatistics> {
        if let Some(adaptive) = &self.adaptive_manager {
            Some(adaptive.get_performance_stats().await)
        } else {
            None
        }
    }

    /// Get current adaptive thresholds
    pub async fn get_current_thresholds(&self) -> Option<crate::tdg::CurrentThresholds> {
        if let Some(adaptive) = &self.adaptive_manager {
            Some(adaptive.get_current_thresholds().await)
        } else {
            None
        }
    }

    /// Reset adaptive thresholds to defaults
    pub async fn reset_adaptive_thresholds(&self) -> Result<()> {
        if let Some(adaptive) = &self.adaptive_manager {
            adaptive.reset_to_defaults().await?;
        }
        Ok(())
    }

    /// Get resource controller statistics for diagnostics
    pub async fn get_resource_stats(&self) -> Option<crate::tdg::ResourceEnforcementStats> {
        if let Some(controller) = &self.resource_controller {
            Some(controller.get_enforcement_stats().await)
        } else {
            None
        }
    }

    /// Get current resource usage
    pub async fn get_resource_usage(&self) -> Option<crate::tdg::ResourceUsage> {
        if let Some(controller) = &self.resource_controller {
            Some(controller.get_current_usage().await)
        } else {
            None
        }
    }

    /// Estimate memory required for file analysis
    fn estimate_analysis_memory(&self, path: &Path) -> Result<f64> {
        let metadata = fs::metadata(path)?;
        let file_size_mb = metadata.len() as f64 / (1024.0 * 1024.0);

        // Estimate memory as 3-5x file size (for AST parsing, analysis structures)
        let base_memory = file_size_mb * 4.0;

        // Add language-specific memory overhead
        let language = Language::from_extension(path);
        let language_overhead = match language {
            Language::Rust => 20.0,              // Rust AST is memory intensive
            Language::Cpp | Language::C => 15.0, // C++ templates can be large
            Language::Java => 12.0,              // Java reflection analysis
            Language::TypeScript => 10.0,        // TS type checking
            Language::Python => 8.0,             // Python AST is relatively compact
            Language::JavaScript => 6.0,         // JS AST is simple
            _ => 5.0,                            // Default overhead for other languages
        };

        Ok((base_memory + language_overhead).max(5.0)) // Minimum 5MB estimate
    }

    pub fn analyze_source(
        &self,
        source: &str,
        language: Language,
        file_path: Option<PathBuf>,
    ) -> Result<TdgScore> {
        let mut tracker = PenaltyTracker::new();
        let mut score = TdgScore {
            language,
            confidence: language.confidence(),
            file_path,
            ..Default::default()
        };

        // Use proper AST-based analysis
        match language {
            Language::Rust => self.analyze_rust_ast(source, &mut score, &mut tracker)?,
            Language::Python => self.analyze_python_ast(source, &mut score, &mut tracker)?,
            Language::JavaScript | Language::TypeScript => {
                self.analyze_javascript_ast(source, &mut score, &mut tracker)?;
            }
            Language::Go => self.analyze_go_ast(source, &mut score, &mut tracker)?,
            Language::Java => self.analyze_java_ast(source, &mut score, &mut tracker)?,
            Language::C | Language::Cpp => self.analyze_c_ast(source, &mut score, &mut tracker)?,
            Language::Ruchy => self.analyze_ruchy_ast(source, &mut score, &mut tracker)?,
            Language::Lua => self.analyze_lua_ast(source, &mut score, &mut tracker)?,
            _ => {
                // Fallback to heuristics for unsupported languages
                // but with reduced confidence
                score.confidence *= 0.5;
                self.analyze_heuristic(source, &mut score, &mut tracker)?;
            }
        }

        score.penalties_applied = tracker.get_attributions();

        // Known Defects v2.1: Detect critical defects for auto-fail
        if language == Language::Rust {
            if let Some(ref path) = score.file_path {
                let detector = RustDefectDetector::new();
                let defects = detector.detect(source, path);

                // Count critical defects
                let critical_count: usize = defects
                    .iter()
                    .filter(|d| d.severity == DefectSeverity::Critical)
                    .map(|d| d.instances.len())
                    .sum();

                score.critical_defects_count = critical_count;
                score.has_critical_defects = critical_count > 0;
            }
        }

        score.calculate_total();

        Ok(score)
    }

    fn analyze_rust_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "rust-ast")]
        {
            use syn::{visit::Visit, File};

            let ast = syn::parse_str::<File>(source)?;

            let mut visitor = RustComplexityVisitor::new();
            visitor.visit_file(&ast);

            // Calculate structural complexity based on AST
            let cyclomatic = visitor.cyclomatic_complexity;
            let cognitive = visitor.cognitive_complexity;

            score.structural_complexity = self.score_structural_complexity(
                cyclomatic,
                cognitive,
                visitor.max_nesting_depth,
                visitor.max_method_length,
                tracker,
            );

            // Calculate semantic complexity
            score.semantic_complexity = self.score_semantic_complexity(
                visitor.max_params,
                visitor.generic_count,
                visitor.abstraction_levels,
                tracker,
            );

            // Calculate duplication (requires AST comparison)
            score.duplication_ratio = self.analyze_duplication_ast(source, Language::Rust, tracker);

            // Calculate coupling
            score.coupling_score = self.score_coupling(
                visitor.import_count,
                visitor.external_calls,
                visitor.interface_implementations,
                tracker,
            );

            // Calculate documentation coverage
            score.doc_coverage = self.score_documentation(
                visitor.documented_items,
                visitor.total_public_items,
                visitor.comment_lines,
                visitor.total_lines,
                tracker,
            );

            // Calculate consistency
            score.consistency_score = self.score_consistency_rust(&ast, tracker);

            // Calculate entropy - pattern analysis for code quality
            score.entropy_score = self.score_entropy_analysis(source, Language::Rust, tracker);
        }
        #[cfg(not(feature = "rust-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_python_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "python-ast")]
        {
            // Modern tree-sitter-python parsing (replaces rustpython-parser)
            use tree_sitter::Parser as TsParser;

            let mut parser = TsParser::new();
            parser
                .set_language(&tree_sitter_python::LANGUAGE.into())
                .map_err(|e| anyhow::anyhow!("Failed to set Python language: {e}"))?;

            let tree = parser
                .parse(source, None)
                .ok_or_else(|| anyhow::anyhow!("Failed to parse Python code"))?;

            let mut visitor = PythonComplexityVisitor::new(source);
            visitor.analyze_tree(&tree);

            score.structural_complexity = self.score_structural_complexity(
                visitor.cyclomatic_complexity,
                visitor.cognitive_complexity,
                visitor.max_nesting_depth,
                visitor.max_method_length,
                tracker,
            );

            score.semantic_complexity = self.score_semantic_complexity(
                visitor.max_params,
                visitor.decorator_count,
                visitor.metaclass_count,
                tracker,
            );

            score.duplication_ratio =
                self.analyze_duplication_ast(source, Language::Python, tracker);

            score.coupling_score = self.score_coupling(
                visitor.import_count,
                visitor.external_calls,
                0, // Python doesn't have explicit interfaces
                tracker,
            );

            score.doc_coverage = self.score_documentation(
                visitor.documented_functions,
                visitor.total_functions,
                visitor.docstring_lines,
                visitor.total_lines,
                tracker,
            );

            score.consistency_score = self.score_consistency_python(source, tracker);

            score.entropy_score = self.score_entropy_analysis(source, Language::Python, tracker);
        }
        #[cfg(not(feature = "python-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_javascript_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "typescript-ast")]
        {
            use swc_common::{sync::Lrc, FileName, SourceMap};
            use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
            use swc_ecma_visit::VisitWith;

            let cm: Lrc<SourceMap> = Default::default();
            let fm = cm.new_source_file(
                FileName::Custom("test.js".into()).into(),
                source.to_string(),
            );

            let lexer = Lexer::new(
                Syntax::Typescript(TsSyntax {
                    tsx: true,
                    decorators: true,
                    ..Default::default()
                }),
                Default::default(),
                StringInput::from(&*fm),
                None,
            );

            let mut parser = Parser::new_from(lexer);

            match parser.parse_module() {
                Ok(module) => {
                    let mut visitor = JavaScriptComplexityVisitor::new();
                    module.visit_with(&mut visitor);

                    score.structural_complexity = self.score_structural_complexity(
                        visitor.cyclomatic_complexity,
                        visitor.cognitive_complexity,
                        visitor.max_nesting_depth,
                        visitor.max_function_length,
                        tracker,
                    );

                    score.semantic_complexity = self.score_semantic_complexity(
                        visitor.max_params,
                        visitor.async_count,
                        visitor.callback_depth,
                        tracker,
                    );

                    score.duplication_ratio =
                        self.analyze_duplication_ast(source, Language::JavaScript, tracker);

                    score.coupling_score = self.score_coupling(
                        visitor.import_count,
                        visitor.external_calls,
                        visitor.class_count,
                        tracker,
                    );

                    score.doc_coverage = self.score_documentation(
                        visitor.jsdoc_count,
                        visitor.function_count,
                        visitor.comment_lines,
                        visitor.total_lines,
                        tracker,
                    );

                    score.consistency_score = self.score_consistency_javascript(source, tracker);

                    score.entropy_score =
                        self.score_entropy_analysis(source, Language::JavaScript, tracker);
                }
                Err(_) => {
                    self.analyze_heuristic(source, score, tracker)?;
                }
            }
        }
        #[cfg(not(feature = "typescript-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_go_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "go-ast")]
        {
            use tree_sitter::Parser as TsParser;

            let mut parser = TsParser::new();
            parser
                .set_language(&tree_sitter_go::LANGUAGE.into())
                .map_err(|e| anyhow::anyhow!("Failed to set Go language: {e}"))?;

            if let Some(tree) = parser.parse(source, None) {
                let mut visitor = GoComplexityVisitor::new(source);
                visitor.analyze_tree(&tree);

                score.structural_complexity = self.score_structural_complexity(
                    visitor.cyclomatic_complexity,
                    visitor.cognitive_complexity,
                    visitor.max_nesting_depth,
                    visitor.max_method_length,
                    tracker,
                );

                score.semantic_complexity = self.score_semantic_complexity(
                    visitor.max_params,
                    visitor.interface_count,
                    0,
                    tracker,
                );

                score.duplication_ratio =
                    self.analyze_duplication_ast(source, Language::Go, tracker);

                score.coupling_score = self.score_coupling(
                    visitor.import_count,
                    visitor.external_calls,
                    visitor.interface_count,
                    tracker,
                );

                score.doc_coverage = self.score_documentation(
                    visitor.documented_functions,
                    visitor.total_functions,
                    visitor.comment_lines,
                    visitor.total_lines,
                    tracker,
                );

                score.consistency_score = self.config.weights.consistency;

                score.entropy_score =
                    self.score_entropy_analysis(source, Language::Go, tracker);
            } else {
                self.analyze_heuristic(source, score, tracker)?;
            }
        }
        #[cfg(not(feature = "go-ast"))]
        {
            self.analyze_tree_sitter_generic(source, Language::Go, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_java_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        // Java requires tree-sitter-java which we don't have yet
        // Use tree-sitter generic parser as fallback
        self.analyze_tree_sitter_generic(source, Language::Java, score, tracker)
    }

    fn analyze_lua_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "lua-ast")]
        {
            use tree_sitter::Parser as TsParser;

            let mut parser = TsParser::new();
            parser
                .set_language(&tree_sitter_lua::LANGUAGE.into())
                .map_err(|e| anyhow::anyhow!("Failed to set Lua language: {e}"))?;

            if let Some(tree) = parser.parse(source, None) {
                let mut visitor = LuaComplexityVisitor::new(source);
                visitor.analyze_tree(&tree);

                score.structural_complexity = self.score_structural_complexity(
                    visitor.cyclomatic_complexity,
                    visitor.cognitive_complexity,
                    visitor.max_nesting_depth,
                    visitor.max_method_length,
                    tracker,
                );

                score.semantic_complexity = self.score_semantic_complexity(
                    visitor.max_params,
                    visitor.metatable_count,
                    0, // Lua has no abstraction levels like generics
                    tracker,
                );

                score.duplication_ratio =
                    self.analyze_duplication_ast(source, Language::Lua, tracker);

                score.coupling_score = self.score_coupling(
                    visitor.import_count,
                    visitor.external_calls,
                    0, // Lua has no explicit interfaces
                    tracker,
                );

                score.doc_coverage = self.score_documentation(
                    visitor.documented_functions,
                    visitor.total_functions,
                    visitor.comment_lines,
                    visitor.total_lines,
                    tracker,
                );

                score.consistency_score = self.score_consistency_lua(source, tracker);

                score.entropy_score = self.score_entropy_analysis(source, Language::Lua, tracker);
            } else {
                self.analyze_heuristic(source, score, tracker)?;
            }
        }
        #[cfg(not(feature = "lua-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }

    fn analyze_c_ast(
        &self,
        source: &str,
        score: &mut TdgScore,
        tracker: &mut PenaltyTracker,
    ) -> Result<()> {
        #[cfg(feature = "c-ast")]
        {
            use tree_sitter::{Parser, Query, QueryCursor};

            let mut parser = Parser::new();
            let language = if score.language == Language::Cpp {
                tree_sitter_cpp::LANGUAGE.into()
            } else {
                tree_sitter_c::LANGUAGE.into()
            };

            parser
                .set_language(&language)
                .map_err(|e| anyhow::anyhow!("Failed to set language: {e:?}"))?;

            if let Some(tree) = parser.parse(source, None) {
                let root_node = tree.root_node();

                // Count control flow statements for cyclomatic complexity
                let query_str = r"
                    (if_statement) @if
                    (while_statement) @while
                    (for_statement) @for
                    (do_statement) @do
                    (switch_statement) @switch
                    (case_statement) @case
                    (conditional_expression) @ternary
                ";

                let query = Query::new(&language, query_str)
                    .map_err(|e| anyhow::anyhow!("Query error: {e:?}"))?;

                let mut cursor = QueryCursor::new();
                let matches = cursor.matches(&query, root_node, source.as_bytes());
                let cyclomatic = 1 + matches.count() as u32;

                score.structural_complexity = self.score_structural_complexity(
                    cyclomatic,
                    self.calculate_cognitive_complexity(&root_node),
                    self.calculate_max_nesting(&root_node),
                    self.calculate_max_function_length(&root_node, source),
                    tracker,
                );

                // Count includes for coupling
                let include_query = Query::new(&language, "(preproc_include) @include")
                    .map_err(|e| anyhow::anyhow!("Query error: {e:?}"))?;

                let mut cursor = QueryCursor::new();
                let include_matches = cursor.matches(&include_query, root_node, source.as_bytes());
                let import_count = include_matches.count() as u32;

                score.coupling_score = self.score_coupling(import_count, 0, 0, tracker);

                // Basic documentation coverage
                let comment_query = Query::new(&language, "(comment) @comment")
                    .map_err(|e| anyhow::anyhow!("Query error: {e:?}"))?;

                let mut cursor = QueryCursor::new();
                let comment_matches = cursor.matches(&comment_query, root_node, source.as_bytes());
                let comment_count = comment_matches.count() as u32;

                score.doc_coverage = (comment_count as f32 / source.lines().count() as f32 * 10.0)
                    .min(self.config.weights.documentation);

                score.semantic_complexity = self.config.weights.semantic_complexity;
                score.duplication_ratio =
                    self.analyze_duplication_ast(source, score.language, tracker);
                score.consistency_score = self.config.weights.consistency;

                score.entropy_score =
                    self.score_entropy_analysis(source, score.language, tracker);
            } else {
                self.analyze_heuristic(source, score, tracker)?;
            }
        }
        #[cfg(not(feature = "c-ast"))]
        {
            self.analyze_heuristic(source, score, tracker)?;
        }

        Ok(())
    }
}
