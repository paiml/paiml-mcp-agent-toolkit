// DeepContextAnalyzer core analysis methods - extracted for file health (CB-040)
impl DeepContextAnalyzer {
    pub async fn analyze_project(&self, project_path: &PathBuf) -> anyhow::Result<DeepContext> {
        let start_time = std::time::Instant::now();
        info!(
            "Starting deep context analysis for project: {:?}",
            project_path
        );

        // Create progress tracker
        let progress = crate::services::progress::ProgressTracker::new(true);
        let main_progress = progress.create_spinner("Analyzing project...");

        // Execute all analysis phases using extracted methods
        let mut file_tree = self
            .execute_discovery_phase(project_path, &main_progress)
            .await?;
        let analyses = self
            .execute_analysis_phase(project_path, &progress, &main_progress)
            .await?;
        self.enrich_file_tree_if_dag_present(&mut file_tree, &analyses, &main_progress)?;
        let cross_refs = self
            .execute_cross_reference_phase(&analyses, &main_progress)
            .await?;
        let (defect_summary, hotspots) = self
            .execute_defect_correlation_phase(&analyses, &main_progress)
            .await?;
        let quality_scorecard = self
            .execute_quality_scoring_phase(&analyses, &defect_summary, &main_progress)
            .await?;
        let recommendations = self
            .execute_recommendations_phase(&analyses, &defect_summary, &main_progress)
            .await?;
        let template_provenance = self
            .execute_template_provenance_phase(&analyses, &main_progress)
            .await?;
        let (build_info, project_overview) = self
            .execute_metadata_analysis_phase(project_path, &main_progress)
            .await?;

        // Build the deep context from all phases
        let analysis_duration = start_time.elapsed();
        let build_params = DeepContextBuildParams {
            project_path,
            file_tree,
            analyses,
            cross_refs,
            quality_scorecard,
            template_provenance,
            defect_summary,
            hotspots,
            recommendations,
            build_info,
            project_overview,
            analysis_duration,
        };
        let mut deep_context = self.build_deep_context(build_params);

        // Execute final QA verification phase
        deep_context.qa_verification = Some(
            self.execute_qa_verification_phase(&deep_context, &main_progress)
                .await?,
        );

        // Complete progress tracking
        main_progress.finish_with_message("Analysis complete!");
        progress.clear();

        info!("Deep context analysis completed in {:?}", analysis_duration);
        Ok(deep_context)
    }

    // EXTRACTED METHODS - Toyota Way Extract Method Pattern
    // Each method has single responsibility and <10 complexity

    async fn execute_discovery_phase(
        &self,
        project_path: &PathBuf,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<AnnotatedFileTree> {
        progress.set_message("Discovering project structure...");
        let file_tree = self.discover_project_structure(project_path).await?;
        debug!("Discovery phase completed");
        Ok(file_tree)
    }

    async fn execute_analysis_phase(
        &self,
        project_path: &Path,
        tracker: &crate::services::progress::ProgressTracker,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        progress.set_message("Running parallel analyses...");
        let analysis_start = std::time::Instant::now();
        let analyses = self
            .execute_parallel_analyses_with_progress(project_path, tracker)
            .await?;
        info!("Analysis phase completed in {:?}", analysis_start.elapsed());
        debug!("Analysis phase completed");
        Ok(analyses)
    }

    fn enrich_file_tree_if_dag_present(
        &self,
        file_tree: &mut AnnotatedFileTree,
        analyses: &ParallelAnalysisResults,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<()> {
        if let Some(ref dag) = analyses.dependency_graph {
            progress.set_message("Enriching file tree with centrality scores...");
            self.enrich_file_tree_with_centrality(file_tree, dag)?;
            debug!("File tree enriched with centrality scores");
        }
        Ok(())
    }

    async fn execute_cross_reference_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<FxHashMap<String, Vec<CrossLangReference>>> {
        progress.set_message("Resolving cross-language references...");
        let cross_refs = self.build_cross_language_references(analyses).await?;
        debug!("Cross-reference resolution completed");
        Ok(cross_refs)
    }

    async fn execute_defect_correlation_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<(DefectSummary, Vec<DefectHotspot>)> {
        progress.set_message("Correlating defects...");
        let (defect_summary, hotspots) = self.correlate_defects(analyses).await?;
        debug!("Defect correlation completed");
        Ok((defect_summary, hotspots))
    }

    async fn execute_quality_scoring_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        defect_summary: &DefectSummary,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<QualityScorecard> {
        progress.set_message("Calculating quality scores...");
        let quality_scorecard = self
            .calculate_quality_scorecard(analyses, defect_summary)
            .await?;
        debug!("Quality scoring completed");
        Ok(quality_scorecard)
    }

    async fn execute_recommendations_phase(
        &self,
        analyses: &ParallelAnalysisResults,
        defect_summary: &DefectSummary,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<Vec<PrioritizedRecommendation>> {
        progress.set_message("Generating recommendations...");
        let recommendations = self
            .generate_recommendations(analyses, defect_summary)
            .await?;
        debug!("Recommendations generated");
        Ok(recommendations)
    }

    async fn execute_template_provenance_phase(
        &self,
        _analyses: &ParallelAnalysisResults,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<Option<TemplateProvenance>> {
        progress.set_message("Analyzing template provenance...");
        // Legacy function - returns None for now
        Ok(None)
    }

    async fn execute_metadata_analysis_phase(
        &self,
        project_path: &Path,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<(Option<BuildInfo>, Option<ProjectOverview>)> {
        progress.set_message("Analyzing project metadata...");
        let (build_info, project_overview) = self.analyze_project_metadata(project_path).await?;
        debug!("Project metadata analysis completed");
        Ok((build_info, project_overview))
    }

    fn build_deep_context(&self, params: DeepContextBuildParams) -> DeepContext {
        DeepContext {
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: env!("CARGO_PKG_VERSION").to_string(),
                project_root: params.project_path.to_path_buf(),
                cache_stats: CacheStats {
                    hit_rate: 0.0,
                    memory_efficiency: 0.0,
                    time_saved_ms: 0,
                },
                analysis_duration: params.analysis_duration,
            },
            file_tree: params.file_tree,
            analyses: AnalysisResults {
                ast_contexts: params.analyses.ast_contexts.unwrap_or_default(),
                complexity_report: params.analyses.complexity_report,
                churn_analysis: params.analyses.churn_analysis,
                dependency_graph: params.analyses.dependency_graph,
                dead_code_results: params.analyses.dead_code_results,
                duplicate_code_results: params.analyses.duplicate_code_results,
                satd_results: params.analyses.satd_results,
                provability_results: params.analyses.provability_results,
                cross_language_refs: params
                    .cross_refs
                    .into_iter()
                    .flat_map(|(_, refs)| refs)
                    .collect(),
                big_o_analysis: params.analyses.big_o_analysis,
            },
            quality_scorecard: params.quality_scorecard,
            template_provenance: params.template_provenance,
            defect_summary: params.defect_summary,
            hotspots: params.hotspots,
            recommendations: params.recommendations,
            qa_verification: None,
            build_info: params.build_info,
            project_overview: params.project_overview,
        }
    }

    async fn execute_qa_verification_phase(
        &self,
        deep_context: &DeepContext,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<QAVerificationResult> {
        progress.set_message("Running QA verification...");
        let qa_result = self.run_qa_verification(deep_context).await?;
        info!("QA verification completed");
        Ok(qa_result)
    }

    // New helper methods for phases that didn't exist before
    async fn calculate_quality_scorecard(
        &self,
        analyses: &ParallelAnalysisResults,
        _defect_summary: &DefectSummary,
    ) -> anyhow::Result<QualityScorecard> {
        // Calculate quality scores based on analyses
        let complexity_score = if let Some(ref report) = analyses.complexity_report {
            // Calculate based on the number of violations
            let violation_penalty = (report.violations.len() as f64 * 5.0).min(50.0);
            100.0 - violation_penalty
        } else {
            75.0
        };

        let maintainability_index = 70.0; // Placeholder for now
        let modularity_score = 85.0; // Placeholder for now
        let test_coverage = Some(65.0); // Placeholder for now
        let technical_debt_hours = 40.0; // Placeholder for now

        Ok(QualityScorecard {
            overall_health: (complexity_score + maintainability_index + modularity_score) / 3.0,
            complexity_score,
            maintainability_index,
            modularity_score,
            test_coverage,
            technical_debt_hours,
        })
    }

    async fn generate_recommendations(
        &self,
        analyses: &ParallelAnalysisResults,
        defect_summary: &DefectSummary,
    ) -> anyhow::Result<Vec<PrioritizedRecommendation>> {
        let mut recommendations = Vec::new();

        // Extract Method: Each recommendation type is handled by a focused method
        self.add_complexity_recommendations(&mut recommendations, analyses);
        self.add_defect_recommendations(&mut recommendations, defect_summary);
        self.add_satd_recommendations(&mut recommendations, analyses);

        Ok(recommendations)
    }

    fn add_complexity_recommendations(
        &self,
        recommendations: &mut Vec<PrioritizedRecommendation>,
        analyses: &ParallelAnalysisResults,
    ) {
        if let Some(complexity) = &analyses.complexity_report {
            for violation in &complexity.violations {
                if let Some(recommendation) = self.create_complexity_recommendation(violation) {
                    recommendations.push(recommendation);
                }
            }
        }
    }

    fn create_complexity_recommendation(
        &self,
        violation: &crate::services::complexity::Violation,
    ) -> Option<PrioritizedRecommendation> {
        match violation {
            crate::services::complexity::Violation::Error {
                function,
                value,
                threshold,
                message,
                ..
            }
            | crate::services::complexity::Violation::Warning {
                function,
                value,
                threshold,
                message,
                ..
            } => {
                function
                    .as_ref()
                    .map(|func_name| PrioritizedRecommendation {
                        title: format!("Refactor high-complexity function: {func_name}"),
                        description: format!(
                            "{message} (complexity: {value}, threshold: {threshold})"
                        ),
                        priority: self.determine_complexity_priority(*value),
                        estimated_effort: Duration::from_secs(3600), // 1 hour estimate
                        impact: Impact::High,
                        prerequisites: vec![],
                    })
            }
        }
    }

    fn determine_complexity_priority(&self, value: u16) -> Priority {
        if value > 25 {
            Priority::Critical
        } else if value > 20 {
            Priority::High
        } else {
            Priority::Medium
        }
    }

    fn add_defect_recommendations(
        &self,
        recommendations: &mut Vec<PrioritizedRecommendation>,
        defect_summary: &DefectSummary,
    ) {
        if defect_summary.total_defects > 50 {
            recommendations.push(PrioritizedRecommendation {
                title: "High defect count detected".to_string(),
                description: format!(
                    "Project has {} total defects. Consider a focused quality improvement sprint.",
                    defect_summary.total_defects
                ),
                priority: Priority::High,
                estimated_effort: Duration::from_secs(7200), // 2 hours
                impact: Impact::High,
                prerequisites: vec![],
            });
        }
    }

    fn add_satd_recommendations(
        &self,
        recommendations: &mut Vec<PrioritizedRecommendation>,
        analyses: &ParallelAnalysisResults,
    ) {
        if let Some(satd) = &analyses.satd_results {
            if satd.summary.total_items > 0 {
                recommendations.push(PrioritizedRecommendation {
                    title: "Technical debt detected".to_string(),
                    description: format!(
                        "Found {} SATD comments. Zero-tolerance policy requires immediate remediation.",
                        satd.summary.total_items
                    ),
                    priority: Priority::Critical,
                    estimated_effort: Duration::from_secs(satd.summary.total_items as u64 * 1800), // 30 min per SATD
                    impact: Impact::High,
                    prerequisites: vec![],
                });
            }
        }
    }

    async fn discover_project_structure(
        &self,
        project_path: &PathBuf,
    ) -> anyhow::Result<AnnotatedFileTree> {
        let mut total_files = 0;
        let mut total_size_bytes = 0;

        let root =
            self.build_file_tree_recursive(project_path, &mut total_files, &mut total_size_bytes)?;

        Ok(AnnotatedFileTree {
            root,
            total_files,
            total_size_bytes,
        })
    }

    fn build_file_tree_recursive(
        &self,
        path: &PathBuf,
        total_files: &mut usize,
        total_size: &mut u64,
    ) -> anyhow::Result<AnnotatedNode> {
        let metadata = std::fs::metadata(path)?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        if metadata.is_dir() {
            let mut children = Vec::new();

            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let child_path = entry.path();

                    // Apply exclude patterns
                    if self.should_exclude_path(&child_path) {
                        continue;
                    }

                    if let Ok(child_node) =
                        self.build_file_tree_recursive(&child_path, total_files, total_size)
                    {
                        children.push(child_node);
                    }
                }
            }

            Ok(AnnotatedNode {
                name,
                path: path.clone(),
                node_type: NodeType::Directory,
                children,
                annotations: NodeAnnotations {
                    defect_score: None,
                    complexity_score: None,
                    cognitive_complexity: None,
                    churn_score: None,
                    dead_code_items: 0,
                    satd_items: 0,
                    centrality: None,
                    test_coverage: None,
                    big_o_complexity: None,
                    memory_complexity: None,
                    duplication_score: None,
                },
            })
        } else {
            *total_files += 1;
            *total_size += metadata.len();

            Ok(AnnotatedNode {
                name,
                path: path.clone(),
                node_type: NodeType::File,
                children: Vec::new(),
                annotations: NodeAnnotations {
                    defect_score: None,
                    complexity_score: None,
                    cognitive_complexity: None,
                    churn_score: None,
                    dead_code_items: 0,
                    satd_items: 0,
                    centrality: None,
                    test_coverage: None,
                    big_o_complexity: None,
                    memory_complexity: None,
                    duplication_score: None,
                },
            })
        }
    }

    fn should_exclude_path(&self, path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.config.exclude_patterns {
            if path_str.contains(pattern.trim_matches('*')) {
                return true;
            }
        }

        false
    }

    /// Enrich the file tree with centrality scores from the dependency graph
    fn enrich_file_tree_with_centrality(
        &self,
        file_tree: &mut AnnotatedFileTree,
        dag: &DependencyGraph,
    ) -> anyhow::Result<()> {
        // Create a map of file paths to centrality scores
        let mut centrality_map: FxHashMap<PathBuf, f32> = FxHashMap::default();

        for node in dag.nodes.values() {
            if let Some(centrality_str) = node.metadata.get("centrality") {
                if let Ok(centrality) = centrality_str.parse::<f32>() {
                    let file_path = PathBuf::from(&node.file_path);
                    centrality_map.insert(file_path, centrality);
                }
            }
        }

        // Recursively update the file tree with centrality scores
        Self::update_node_centrality(&mut file_tree.root, &centrality_map);

        Ok(())
    }

    /// Recursively update node centrality scores
    fn update_node_centrality(node: &mut AnnotatedNode, centrality_map: &FxHashMap<PathBuf, f32>) {
        // Update this node's centrality if it's a file
        if node.node_type == NodeType::File {
            if let Some(&centrality) = centrality_map.get(&node.path) {
                node.annotations.centrality = Some(centrality);
            }
        }

        // Recursively update children
        for child in &mut node.children {
            Self::update_node_centrality(child, centrality_map);
        }
    }

    async fn execute_parallel_analyses_with_progress(
        &self,
        project_path: &std::path::Path,
        progress: &crate::services::progress::ProgressTracker,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        // Step 1: Spawn all analysis tasks with progress tracking
        let mut join_set = self.spawn_analysis_tasks(project_path)?;

        // Create sub-progress bars for different analyses
        let analysis_count = self.config.include_analyses.len() as u64;
        let analysis_progress = progress.create_sub_progress("Running analyses", analysis_count);

        // Step 2: Collect and process results - NO TIMEOUT!
        let results = self
            .collect_analysis_results_with_progress(&mut join_set, &analysis_progress)
            .await?;

        analysis_progress.finish_with_message("Analyses complete");
        Ok(results)
    }

    /// Spawn all configured analysis tasks
    fn spawn_analysis_tasks(
        &self,
        project_path: &std::path::Path,
    ) -> anyhow::Result<tokio::task::JoinSet<AnalysisResult>> {
        let mut join_set = tokio::task::JoinSet::new();

        for analysis_type in &self.config.include_analyses {
            self.spawn_analysis_task(&mut join_set, project_path, analysis_type)?;
        }

        Ok(join_set)
    }

    /// Spawn a single analysis task based on type
    fn spawn_analysis_task(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        project_path: &std::path::Path,
        analysis_type: &AnalysisType,
    ) -> anyhow::Result<()> {
        let path = project_path.to_path_buf();

        match analysis_type {
            AnalysisType::Ast => self.spawn_ast_analysis(join_set, path),
            AnalysisType::Complexity => self.spawn_complexity_analysis(join_set, path),
            AnalysisType::Churn => self.spawn_churn_analysis(join_set, path),
            AnalysisType::DeadCode => self.spawn_dead_code_analysis(join_set, path),
            AnalysisType::DuplicateCode => self.spawn_duplicate_analysis(join_set, path),
            AnalysisType::Satd => self.spawn_satd_analysis(join_set, path),
            AnalysisType::Provability => self.spawn_provability_analysis(join_set, path),
            AnalysisType::Dag => self.spawn_dag_analysis(join_set, path),
            AnalysisType::TechnicalDebtGradient => Ok(()), // Computed in correlate_defects
            AnalysisType::BigO => self.spawn_big_o_analysis(join_set, path),
        }
    }

    fn spawn_ast_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let file_classifier_config = self.config.file_classifier_config.clone();
        join_set.spawn(async move {
            AnalysisResult::Ast(analyze_ast_contexts(&path, file_classifier_config).await)
        });
        Ok(())
    }

    fn spawn_complexity_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move { AnalysisResult::Complexity(analyze_complexity(&path).await) });
        Ok(())
    }

    fn spawn_churn_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let days = self.config.period_days;
        join_set.spawn(async move { AnalysisResult::Churn(analyze_churn(&path, days).await) });
        Ok(())
    }

    fn spawn_dead_code_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move { AnalysisResult::DeadCode(analyze_dead_code(&path).await) });
        Ok(())
    }

    fn spawn_duplicate_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move {
            AnalysisResult::DuplicateCode(analyze_duplicate_code(&path).await)
        });
        Ok(())
    }

    fn spawn_satd_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                tokio::runtime::Handle::current().block_on(async { analyze_satd(&path).await })
            })
            .await
            .unwrap_or_else(|_| Err(anyhow::anyhow!("SATD analysis failed")));
            AnalysisResult::Satd(result)
        });
        Ok(())
    }

    fn spawn_provability_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set
            .spawn(async move { AnalysisResult::Provability(analyze_provability(&path).await) });
        Ok(())
    }

    fn spawn_dag_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        let dag_type = self.config.dag_type.clone();
        join_set.spawn(async move { AnalysisResult::Dag(analyze_dag(&path, dag_type).await) });
        Ok(())
    }

    fn spawn_big_o_analysis(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        path: PathBuf,
    ) -> anyhow::Result<()> {
        join_set.spawn(async move { AnalysisResult::BigO(analyze_big_o(&path).await) });
        Ok(())
    }

    async fn collect_analysis_results_with_progress(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        // Direct collection without timeout - let it complete naturally
        let results = self
            .process_analysis_results_with_progress(join_set, progress)
            .await?;
        debug!("Parallel analysis collection completed successfully");
        Ok(results)
    }

    /// Process all analysis results concurrently with progress
    async fn process_analysis_results_with_progress(
        &self,
        join_set: &mut tokio::task::JoinSet<AnalysisResult>,
        progress: &crate::services::progress::ProgressBar,
    ) -> anyhow::Result<ParallelAnalysisResults> {
        // Collect all results first
        let mut pending_results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            pending_results.push(result?);
            progress.inc(1);
        }

        // Process results concurrently
        let result_processors: Vec<_> = pending_results
            .into_iter()
            .map(|result| tokio::spawn(async move { result }))
            .collect();

        // Aggregate processed results
        let mut results = ParallelAnalysisResults::default();
        for processor in result_processors {
            if let Ok(processed) = processor.await {
                self.integrate_analysis_result(&mut results, processed);
            }
        }

        Ok(results)
    }

    /// Integrate a single analysis result into the final results
    fn integrate_analysis_result(
        &self,
        results: &mut ParallelAnalysisResults,
        result: AnalysisResult,
    ) {
        match &result {
            AnalysisResult::Ast(Ok(data)) => {
                results.ast_contexts = Some(data.clone());
            }
            AnalysisResult::Complexity(Ok(data)) => {
                results.complexity_report = Some(data.clone());
            }
            AnalysisResult::Churn(Ok(data)) => {
                results.churn_analysis = Some(data.clone());
            }
            AnalysisResult::DeadCode(Ok(data)) => {
                results.dead_code_results = Some(data.clone());
            }
            AnalysisResult::DuplicateCode(Ok(data)) => {
                results.duplicate_code_results = Some(data.clone());
            }
            AnalysisResult::Satd(Ok(data)) => {
                results.satd_results = Some(data.clone());
            }
            AnalysisResult::Provability(Ok(data)) => {
                results.provability_results = Some(data.clone());
            }
            AnalysisResult::Dag(Ok(data)) => {
                results.dependency_graph = Some(data.clone());
            }
            AnalysisResult::BigO(Ok(data)) => {
                results.big_o_analysis = Some(data.clone());
            }
            // Handle errors with helper
            _ => self.log_integration_error(&result),
        }
    }

    /// Log errors from analysis integration
    fn log_integration_error(&self, result: &AnalysisResult) {
        match result {
            AnalysisResult::Ast(Err(e))
            | AnalysisResult::Complexity(Err(e))
            | AnalysisResult::Churn(Err(e))
            | AnalysisResult::DeadCode(Err(e))
            | AnalysisResult::DuplicateCode(Err(e))
            | AnalysisResult::Satd(Err(e))
            | AnalysisResult::Provability(Err(e))
            | AnalysisResult::Dag(Err(e))
            | AnalysisResult::BigO(Err(e)) => {
                debug!("{} analysis failed: {}", self.get_analysis_name(result), e);
            }
            _ => {}
        }
    }

    /// Get analysis name for logging
    fn get_analysis_name(&self, result: &AnalysisResult) -> &'static str {
        match result {
            AnalysisResult::Ast(_) => "AST",
            AnalysisResult::Complexity(_) => "Complexity",
            AnalysisResult::Churn(_) => "Churn",
            AnalysisResult::DeadCode(_) => "Dead code",
            AnalysisResult::DuplicateCode(_) => "Duplicate code",
            AnalysisResult::Satd(_) => "SATD",
            AnalysisResult::Provability(_) => "Provability",
            AnalysisResult::Dag(_) => "DAG",
            AnalysisResult::BigO(_) => "Big-O",
        }
    }

    async fn build_cross_language_references(
        &self,
        _analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<FxHashMap<String, Vec<CrossLangReference>>> {
        // TRACKED: Implement cross-language reference detection
        // This would analyze FFI bindings, WASM exports, Python bindings, etc.
        Ok(FxHashMap::default())
    }

    async fn correlate_defects(
        &self,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<(DefectSummary, Vec<DefectHotspot>)> {
        // Step 1: Collect file TDG scores from all analyses
        let file_tdg_scores = self.collect_file_tdg_scores(analyses)?;

        // Step 2: Calculate TDG summary for the project
        let _tdg_calculator = TDGCalculator::new();
        let tdg_summary = self.calculate_tdg_summary(&file_tdg_scores)?;

        // Step 3: Build defect summary (now based on TDG)
        let defect_summary = self.build_tdg_defect_summary(&tdg_summary, analyses)?;

        // Step 4: Generate hotspots
        let hotspots = self.generate_tdg_hotspots(&file_tdg_scores)?;

        Ok((defect_summary, hotspots))
    }

    /// Collect file TDG scores from all available analyses
    fn collect_file_tdg_scores(
        &self,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<FxHashMap<String, TDGScore>> {
        let mut file_tdg_scores = FxHashMap::default();

        if let Some(ref ast_contexts) = analyses.ast_contexts {
            for enhanced_context in ast_contexts {
                let file_path = enhanced_context.base.path.clone();

                // Extract actual churn score for this file
                let churn_score = if let Some(ref churn_analysis) = analyses.churn_analysis {
                    churn_analysis
                        .files
                        .iter()
                        .find(|f| {
                            f.path.to_string_lossy() == file_path
                                || f.relative_path == file_path
                                || file_path.ends_with(&f.relative_path)
                        })
                        .map_or(0.0, |f| f.churn_score)
                } else {
                    0.0
                };

                // Use TDG calculator to compute score for this file
                let tdg_score = TDGScore {
                    value: 1.5, // Default value - could be computed from components
                    components: crate::models::tdg::TDGComponents {
                        complexity: 1.0,
                        churn: f64::from(churn_score),
                        coupling: 0.5,
                        domain_risk: 0.5,
                        duplication: 0.5,
                    },
                    severity: TDGSeverity::Normal,
                    percentile: 50.0,
                    confidence: 0.8,
                };

                file_tdg_scores.insert(file_path, tdg_score);
            }
        }

        Ok(file_tdg_scores)
    }

    /// Calculate TDG summary from individual file scores
    fn calculate_tdg_summary(
        &self,
        file_scores: &FxHashMap<String, TDGScore>,
    ) -> anyhow::Result<TDGSummary> {
        let total_files = file_scores.len();
        // Use parallel processing for score analysis
        let (values, severities): (Vec<_>, Vec<_>) = file_scores
            .par_iter()
            .map(|(_, score)| (score.value, &score.severity))
            .unzip();

        let mut tdg_values = values;

        // Count severities in parallel
        let critical_files = severities
            .par_iter()
            .filter(|s| matches!(s, TDGSeverity::Critical))
            .count();
        let warning_files = severities
            .par_iter()
            .filter(|s| matches!(s, TDGSeverity::Warning))
            .count();

        tdg_values.sort_unstable_by(|a, b| a.partial_cmp(b).expect("internal error"));

        let average_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            tdg_values.iter().sum::<f64>() / tdg_values.len() as f64
        };

        let p95_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            let index = ((tdg_values.len() - 1) as f64 * 0.95) as usize;
            tdg_values[index.min(tdg_values.len() - 1)]
        };

        let p99_tdg = if tdg_values.is_empty() {
            0.0
        } else {
            let index = ((tdg_values.len() - 1) as f64 * 0.99) as usize;
            tdg_values[index.min(tdg_values.len() - 1)]
        };

        // Create hotspots from top TDG scores
        let mut hotspots: Vec<_> = file_scores
            .iter()
            .map(|(path, score)| crate::models::tdg::TDGHotspot {
                path: path.clone(),
                tdg_score: score.value,
                primary_factor: "complexity".to_string(), // Default factor
                estimated_hours: score.value * 2.0,       // Simple estimation
            })
            .collect();
        hotspots.sort_unstable_by(|a, b| {
            b.tdg_score
                .partial_cmp(&a.tdg_score)
                .expect("internal error")
        });
        hotspots.truncate(10);

        Ok(TDGSummary {
            total_files,
            critical_files,
            warning_files,
            average_tdg,
            p95_tdg,
            p99_tdg,
            estimated_debt_hours: average_tdg * total_files as f64 * 2.0,
            hotspots,
        })
    }

    /// Build defect summary based on actual defect enumeration
    fn build_tdg_defect_summary(
        &self,
        tdg_summary: &TDGSummary,
        analyses: &ParallelAnalysisResults,
    ) -> anyhow::Result<DefectSummary> {
        let mut total_defects = 0usize;
        let mut by_severity = FxHashMap::default();
        let mut by_type = FxHashMap::default();
        let mut total_loc = 0usize;

        // Process each analysis type
        self.process_complexity_violations(
            analyses,
            &mut total_defects,
            &mut by_severity,
            &mut by_type,
            &mut total_loc,
        );
        self.process_satd_violations(analyses, &mut total_defects, &mut by_severity, &mut by_type);
        self.process_dead_code_violations(
            analyses,
            &mut total_defects,
            &mut by_severity,
            &mut by_type,
        );
        self.process_tdg_violations(
            tdg_summary,
            &mut total_defects,
            &mut by_severity,
            &mut by_type,
        );

        let defect_density = self.calculate_defect_density(total_defects, total_loc);

        debug!(
            "Calculated defect summary: {} total defects, {} LOC, density = {:.2}",
            total_defects, total_loc, defect_density
        );

        Ok(DefectSummary {
            total_defects,
            by_severity,
            by_type,
            defect_density,
        })
    }

    fn process_complexity_violations(
        &self,
        analyses: &ParallelAnalysisResults,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
        total_loc: &mut usize,
    ) {
        if let Some(ref complexity_report) = analyses.complexity_report {
            let complexity_violations = complexity_report.violations.len();
            *total_defects += complexity_violations;
            by_type.insert("Complexity".to_string(), complexity_violations);

            for violation in &complexity_report.violations {
                let severity = match violation {
                    crate::services::complexity::Violation::Error { .. } => "Critical",
                    crate::services::complexity::Violation::Warning { .. } => "Warning",
                };
                *by_severity.entry(severity.to_string()).or_insert(0) += 1;
            }

            for file in &complexity_report.files {
                *total_loc += file.total_complexity.lines as usize;
            }
        }
    }

    fn process_satd_violations(
        &self,
        analyses: &ParallelAnalysisResults,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
    ) {
        if let Some(ref satd_results) = analyses.satd_results {
            let satd_count = satd_results.items.len();
            *total_defects += satd_count;
            by_type.insert("TechnicalDebt".to_string(), satd_count);

            for item in &satd_results.items {
                let severity = match item.severity {
                    crate::services::satd_detector::Severity::Critical => "Critical",
                    crate::services::satd_detector::Severity::High => "Critical",
                    crate::services::satd_detector::Severity::Medium => "Warning",
                    crate::services::satd_detector::Severity::Low => "Normal",
                };
                *by_severity.entry(severity.to_string()).or_insert(0) += 1;
            }
        }
    }

    fn process_dead_code_violations(
        &self,
        analyses: &ParallelAnalysisResults,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
    ) {
        if let Some(ref dead_code_results) = analyses.dead_code_results {
            let dead_code_count = dead_code_results.summary.dead_functions
                + dead_code_results.summary.dead_classes
                + dead_code_results.summary.dead_modules;
            *total_defects += dead_code_count;
            by_type.insert("DeadCode".to_string(), dead_code_count);
            *by_severity.entry("Warning".to_string()).or_insert(0) += dead_code_count;
        }
    }

    fn process_tdg_violations(
        &self,
        tdg_summary: &TDGSummary,
        total_defects: &mut usize,
        by_severity: &mut FxHashMap<String, usize>,
        by_type: &mut FxHashMap<String, usize>,
    ) {
        let high_tdg_count = tdg_summary.critical_files + tdg_summary.warning_files;
        *total_defects += high_tdg_count;
        by_type.insert("TDG".to_string(), high_tdg_count);
        *by_severity.entry("Critical".to_string()).or_insert(0) += tdg_summary.critical_files;
        *by_severity.entry("Warning".to_string()).or_insert(0) += tdg_summary.warning_files;
    }

    fn calculate_defect_density(&self, total_defects: usize, total_loc: usize) -> f64 {
        if total_loc > 0 {
            (total_defects as f64 * 1000.0) / total_loc as f64
        } else {
            0.0
        }
    }

    /// Generate hotspots from TDG scores
    fn generate_tdg_hotspots(
        &self,
        file_scores: &FxHashMap<String, TDGScore>,
    ) -> anyhow::Result<Vec<DefectHotspot>> {
        let mut hotspots: Vec<_> = file_scores
            .par_iter()
            .filter(|(_, score)| score.value > 1.5) // Filter above threshold
            .map(|(path, score)| DefectHotspot {
                location: FileLocation {
                    file: std::path::PathBuf::from(path),
                    line: 1,
                    column: 1,
                },
                composite_score: score.value as f32,
                contributing_factors: vec![DefectFactor::TechnicalDebt {
                    category: TechnicalDebtCategory::Implementation,
                    severity: TechnicalDebtSeverity::High,
                    age_days: 0,
                }],
                refactoring_effort: RefactoringEstimate {
                    estimated_hours: score.value as f32 * 2.0,
                    priority: Priority::High,
                    impact: Impact::Medium,
                    suggested_actions: vec!["Reduce TDG score".to_string()],
                },
            })
            .collect();

        hotspots.sort_unstable_by(|a, b| {
            b.composite_score
                .partial_cmp(&a.composite_score)
                .expect("internal error")
        });
        hotspots.truncate(20);

        Ok(hotspots)
    }

    /// Analyze project metadata (Makefile and README)
    async fn analyze_project_metadata(
        &self,
        project_path: &Path,
    ) -> anyhow::Result<(
        Option<crate::models::project_meta::BuildInfo>,
        Option<crate::models::project_meta::ProjectOverview>,
    )> {
        use crate::services::{
            makefile_compressor::MakefileCompressor, project_meta_detector::ProjectMetaDetector,
            readme_compressor::ReadmeCompressor,
        };

        let detector = ProjectMetaDetector::new();
        let meta_files = detector.detect(project_path).await;

        let mut build_info = None;
        let mut project_overview = None;

        for meta_file in meta_files {
            match meta_file.file_type {
                crate::models::project_meta::MetaFileType::Makefile => {
                    let compressor = MakefileCompressor::new();
                    let compressed = compressor.compress(&meta_file.content);
                    build_info = Some(crate::models::project_meta::BuildInfo::from_makefile(
                        compressed,
                    ));
                    debug!("Makefile compressed and analyzed");
                }
                crate::models::project_meta::MetaFileType::Readme => {
                    let compressor = ReadmeCompressor::new();
                    let compressed = compressor.compress(&meta_file.content);
                    project_overview = Some(compressed.to_summary());
                    debug!("README compressed and analyzed");
                }
            }
        }

        Ok((build_info, project_overview))
    }

    /// Run QA verification on the deep context analysis results
    async fn run_qa_verification(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<QAVerificationResult> {
        // Convert DeepContext to the format expected by quality_gates
        let result = self.create_qa_compatible_result(context)?;

        // Create QA verification instance and generate report
        let qa_verification = QAVerification::new();
        let verification_report = qa_verification.generate_verification_report(&result);

        debug!(
            "QA verification report generated: overall status = {:?}",
            verification_report.overall
        );

        Ok(verification_report)
    }

    /// Create a `DeepContextResult` that's compatible with `quality_gates` expectations
    /// Convert complexity report to QA format
    fn convert_complexity_report_to_qa(&self, report: &ComplexityReport) -> ComplexityMetricsForQA {
        ComplexityMetricsForQA {
            files: report
                .files
                .iter()
                .map(|f| FileComplexityMetricsForQA {
                    path: std::path::PathBuf::from(&f.path),
                    functions: f
                        .functions
                        .iter()
                        .map(|func| FunctionComplexityForQA {
                            name: func.name.clone(),
                            cyclomatic: u32::from(func.metrics.cyclomatic),
                            cognitive: u32::from(func.metrics.cognitive),
                            nesting_depth: u32::from(func.metrics.nesting_max),
                            start_line: func.line_start as usize,
                            end_line: func.line_end as usize,
                        })
                        .collect(),
                    total_cyclomatic: u32::from(f.total_complexity.cyclomatic),
                    total_cognitive: u32::from(f.total_complexity.cognitive),
                    total_lines: f.total_complexity.lines as usize,
                })
                .collect(),
            summary: ComplexitySummaryForQA {
                total_files: report.files.len(),
                total_functions: report.files.par_iter().map(|f| f.functions.len()).sum(),
            },
        }
    }

    /// Create fallback complexity metrics from file discovery
    fn create_fallback_complexity_metrics(
        &self,
        context: &DeepContext,
    ) -> Option<ComplexityMetricsForQA> {
        let file_paths = self.collect_file_paths(&context.file_tree.root);
        let mut files_with_lines = Vec::new();
        let project_root = &context.metadata.project_root;

        debug!(
            "QA Fallback: Counting lines from {} files in {:?}",
            file_paths.len(),
            project_root
        );

        for path_str in &file_paths {
            if let Some(file_metrics) = self.process_file_for_fallback(path_str, project_root) {
                files_with_lines.push(file_metrics);
            }
        }

        if files_with_lines.is_empty() {
            None
        } else {
            Some(ComplexityMetricsForQA {
                files: files_with_lines,
                summary: ComplexitySummaryForQA {
                    total_files: 0,
                    total_functions: 0,
                },
            })
        }
    }

    /// Process single file for fallback metrics
    fn process_file_for_fallback(
        &self,
        path_str: &str,
        project_root: &std::path::Path,
    ) -> Option<FileComplexityMetricsForQA> {
        let full_path = if std::path::Path::new(path_str).is_absolute() {
            std::path::PathBuf::from(path_str)
        } else {
            project_root.join(path_str)
        };

        if full_path.exists() && full_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                let line_count = content.lines().count();

                if line_count > 0 {
                    return Some(FileComplexityMetricsForQA {
                        path: full_path,
                        functions: Vec::new(),
                        total_cyclomatic: 0,
                        total_cognitive: 0,
                        total_lines: line_count,
                    });
                }
            }
        }

        None
    }

    fn create_qa_compatible_result(
        &self,
        context: &DeepContext,
    ) -> anyhow::Result<DeepContextResult> {
        // Create complexity metrics from analysis results or fallback
        let complexity_metrics = if let Some(report) = context.analyses.complexity_report.as_ref() {
            Some(self.convert_complexity_report_to_qa(report))
        } else {
            self.create_fallback_complexity_metrics(context)
        };

        // Create dead code analysis from the results
        let dead_code_analysis = if let Some(ref dead_code) = context.analyses.dead_code_results {
            // Calculate total functions from complexity report if available
            let total_functions = context
                .analyses
                .complexity_report
                .as_ref()
                .map_or(0, |report| {
                    report
                        .files
                        .iter()
                        .map(|f| f.functions.len())
                        .sum::<usize>()
                });

            Some(DeadCodeAnalysis {
                summary: DeadCodeSummary {
                    total_functions,
                    dead_functions: dead_code.summary.dead_functions,
                    total_lines: dead_code
                        .ranked_files
                        .par_iter()
                        .map(|f| f.total_lines)
                        .sum(),
                    total_dead_lines: dead_code.summary.total_dead_lines,
                    dead_percentage: f64::from(dead_code.summary.dead_percentage),
                },
                dead_functions: vec![], // Not needed for QA verification
                warnings: vec![],
            })
        } else {
            None
        };

        // Create file paths list
        let file_paths = self.collect_file_paths(&context.file_tree.root);

        // Create AST summaries
        let ast_summaries = if context.analyses.ast_contexts.is_empty() {
            None
        } else {
            Some(
                context
                    .analyses
                    .ast_contexts
                    .iter()
                    .map(|ctx| AstSummary {
                        path: ctx.base.path.clone(),
                        language: ctx.base.language.clone(),
                        total_items: ctx.base.items.len(),
                        functions: ctx
                            .base
                            .items
                            .iter()
                            .filter(|item| {
                                matches!(item, crate::services::context::AstItem::Function { .. })
                            })
                            .count(),
                        classes: ctx
                            .base
                            .items
                            .iter()
                            .filter(|item| {
                                matches!(item, crate::services::context::AstItem::Struct { .. })
                            })
                            .count(),
                        imports: ctx
                            .base
                            .items
                            .iter()
                            .filter(|item| {
                                matches!(
                                    item,
                                    crate::services::context::AstItem::Use { .. }
                                        | crate::services::context::AstItem::Import { .. }
                                )
                            })
                            .count(),
                    })
                    .collect(),
            )
        };

        // Create language statistics
        let mut language_stats = FxHashMap::default();
        for ctx in &context.analyses.ast_contexts {
            *language_stats.entry(ctx.base.language.clone()).or_insert(0) += 1;
        }

        // Build the QA-compatible result
        Ok(DeepContextResult {
            metadata: context.metadata.clone(),
            file_tree: file_paths, // Vec<String> for quality_gates
            analyses: context.analyses.clone(),
            quality_scorecard: context.quality_scorecard.clone(),
            template_provenance: context.template_provenance.clone(),
            defect_summary: context.defect_summary.clone(),
            hotspots: context.hotspots.clone(),
            recommendations: context.recommendations.clone(),
            qa_verification: context.qa_verification.clone(),

            // Additional fields expected by quality_gates
            complexity_metrics,
            dead_code_analysis,
            ast_summaries,
            churn_analysis: context.analyses.churn_analysis.clone(),
            language_stats: Some(language_stats),

            // Project metadata fields
            build_info: context.build_info.clone(),
            project_overview: context.project_overview.clone(),
        })
    }

    /// Collect all file paths from the annotated tree
    fn collect_file_paths(&self, node: &AnnotatedNode) -> Vec<String> {
        let mut paths = Vec::new();
        Self::collect_paths_recursive(node, &mut paths);
        paths
    }

    fn collect_paths_recursive(node: &AnnotatedNode, paths: &mut Vec<String>) {
        match node.node_type {
            NodeType::File => {
                paths.push(node.path.to_string_lossy().to_string());
            }
            NodeType::Directory => {
                for child in &node.children {
                    Self::collect_paths_recursive(child, paths);
                }
            }
        }
    }
}

/// Structure for collecting parallel analysis results
#[derive(Default)]
struct ParallelAnalysisResults {
    ast_contexts: Option<Vec<EnhancedFileContext>>,
    complexity_report: Option<ComplexityReport>,
    churn_analysis: Option<CodeChurnAnalysis>,
    dependency_graph: Option<DependencyGraph>,
    dead_code_results: Option<crate::models::dead_code::DeadCodeRankingResult>,
    duplicate_code_results: Option<crate::services::duplicate_detector::CloneReport>,
    satd_results: Option<SATDAnalysisResult>,
    provability_results:
        Option<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>>,
    big_o_analysis: Option<crate::services::big_o_analyzer::BigOAnalysisReport>,
}

enum AnalysisResult {
    Ast(anyhow::Result<Vec<EnhancedFileContext>>),
    Complexity(anyhow::Result<ComplexityReport>),
    Churn(anyhow::Result<CodeChurnAnalysis>),
    DeadCode(anyhow::Result<crate::models::dead_code::DeadCodeRankingResult>),
    DuplicateCode(anyhow::Result<crate::services::duplicate_detector::CloneReport>),
    Satd(anyhow::Result<SATDAnalysisResult>),
    Provability(
        anyhow::Result<Vec<crate::services::lightweight_provability_analyzer::ProofSummary>>,
    ),
    Dag(anyhow::Result<DependencyGraph>),
    BigO(anyhow::Result<crate::services::big_o_analyzer::BigOAnalysisReport>),
}

// Analysis functions (simplified implementations)
