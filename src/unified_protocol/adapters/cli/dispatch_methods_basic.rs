impl CliAdapter {
    /// Toyota Way Extract Method: Basic analysis commands dispatch
    /// Handles core metrics: churn, complexity, dead code, SATD, TDG, lint hotspots
    fn dispatch_basic_analysis(
        analyze_cmd: &AnalyzeCommands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        debug_assert!(true, "contract: dispatch_basic_analysis");
        match analyze_cmd {
            AnalyzeCommands::Churn {
                path,
                project_path,
                days,
                format,
                output,
                top_files,
                include: _,
                exclude: _,
            } => {
                let resolved = project_path.as_deref().unwrap_or(path);
                Self::decode_analyze_churn(resolved, *days, format, output, *top_files)
            }
            AnalyzeCommands::Complexity {
                path,
                project_path,
                file,
                files,
                toolchain,
                format,
                output,
                max_cyclomatic,
                max_cognitive,
                include,
                watch,
                top_files,
                fail_on_violation: _,
                timeout: _,
                ml: _, // GH-97: ML flag
            } => Self::decode_analyze_complexity_with_migration(
                path,
                project_path,
                file,
                files,
                toolchain,
                format,
                output,
                max_cyclomatic,
                max_cognitive,
                include,
                *watch,
                *top_files,
            ),
            AnalyzeCommands::DeadCode {
                path,
                format,
                top_files,
                include_unreachable,
                min_dead_lines,
                include_tests,
                output,
                fail_on_violation: _,
                max_percentage: _,
                timeout: _,
                include: _,
                exclude: _,
                max_depth: _,
            } => Self::decode_analyze_dead_code(
                path,
                format,
                top_files,
                *include_unreachable,
                *min_dead_lines,
                *include_tests,
                output,
            ),
            AnalyzeCommands::Satd {
                path,
                format,
                severity,
                critical_only,
                include_tests,
                strict,
                evolution,
                days,
                metrics,
                output,
                top_files,
                fail_on_violation: _,
                timeout: _,
                include: _,
                exclude: _,
                extended: _,
            } => Self::decode_analyze_satd(
                path,
                format,
                severity,
                *critical_only,
                *include_tests,
                *strict,
                *evolution,
                *days,
                *metrics,
                output,
                *top_files,
            ),
            AnalyzeCommands::Tdg {
                path,
                threshold,
                top_files,
                format,
                include_components,
                output,
                critical_only,
                verbose,
                ml: _, // GH-97: ML flag
            } => Self::decode_analyze_tdg(
                path,
                output,
                format,
                *threshold,
                *critical_only,
                *top_files,
                *include_components,
                *verbose,
            ),
            AnalyzeCommands::LintHotspot {
                path,
                project_path,
                file,
                format,
                max_density,
                min_confidence,
                enforce,
                dry_run,
                enforcement_metadata,
                output,
                perf,
                clippy_flags,
                top_files,
                include: _,
                exclude: _,
            } => {
                let resolved = project_path.as_ref().unwrap_or(path);
                Self::decode_analyze_lint_hotspot(
                    resolved,
                    file,
                    format,
                    max_density,
                    min_confidence,
                    enforce,
                    dry_run,
                    enforcement_metadata,
                    output,
                    perf,
                    clippy_flags,
                    top_files,
                )
            }
            _ => Err(ProtocolError::UnsupportedProtocol(
                "Command not supported in basic analysis dispatch".to_string(),
            )),
        }
    }

    /// Toyota Way Extract Method: Advanced analysis commands dispatch
    /// Handles comprehensive analysis: deep context, comprehensive, defect prediction, duplicates, `BigO`
    fn dispatch_advanced_analysis(
        analyze_cmd: &AnalyzeCommands,
    ) -> Result<(Method, String, Value, Option<OutputFormat>), ProtocolError> {
        debug_assert!(true, "contract: dispatch_advanced_analysis");
        match analyze_cmd {
            AnalyzeCommands::DeepContext {
                path,
                project_path,
                output,
                format,
                full,
                include,
                exclude,
                period_days,
                dag_type,
                max_depth,
                include_patterns,
                exclude_patterns,
                cache_strategy,
                parallel,
                verbose,
                top_files: _,
            } => {
                let resolved = project_path.as_deref().unwrap_or(path);
                Self::decode_analyze_deep_context(
                resolved,
                output,
                format,
                *full,
                include,
                exclude,
                *period_days,
                dag_type,
                max_depth,
                include_patterns,
                exclude_patterns,
                cache_strategy,
                parallel,
                *verbose,
            )
            }
            AnalyzeCommands::Comprehensive {
                path,
                project_path,
                file,
                files,
                format,
                include_duplicates,
                include_dead_code,
                include_defects,
                include_complexity,
                include_tdg,
                confidence_threshold,
                min_lines,
                include,
                exclude,
                output,
                perf,
                executive_summary,
                top_files,
            } => {
                let resolved = project_path.as_ref().unwrap_or(path);
                Self::decode_analyze_comprehensive(
                resolved,
                file,
                files,
                format,
                include_duplicates,
                include_dead_code,
                include_defects,
                include_complexity,
                include_tdg,
                confidence_threshold,
                min_lines,
                include,
                exclude,
                output,
                perf,
                executive_summary,
                top_files,
            )
            }
            AnalyzeCommands::DefectPrediction {
                path,
                project_path,
                confidence_threshold,
                min_lines,
                include_low_confidence,
                format,
                high_risk_only,
                include_recommendations,
                include,
                exclude,
                output,
                perf,
                top_files,
            } => {
                let resolved = project_path.as_ref().unwrap_or(path);
                Self::decode_analyze_defect_prediction(
                resolved,
                confidence_threshold,
                min_lines,
                include_low_confidence,
                format,
                high_risk_only,
                include_recommendations,
                include,
                exclude,
                output,
                perf,
                top_files,
            )
            }
            AnalyzeCommands::Duplicates {
                path,
                project_path,
                detection_type,
                threshold,
                min_lines,
                max_tokens,
                format,
                perf,
                include,
                exclude,
                output,
                top_files,
            } => {
                let resolved = project_path.as_ref().unwrap_or(path);
                Self::decode_analyze_duplicates(
                    resolved,
                    detection_type,
                    threshold,
                    min_lines,
                    max_tokens,
                    format,
                    perf,
                    include,
                    exclude,
                    output,
                    top_files,
                )
            }
            AnalyzeCommands::BigO {
                path,
                project_path,
                format,
                confidence_threshold,
                analyze_space,
                include,
                exclude,
                output,
                perf,
                high_complexity_only,
                top_files,
            } => {
                let resolved = project_path.as_deref().unwrap_or(path);
                Self::decode_analyze_big_o(
                    resolved,
                    format,
                    confidence_threshold,
                    analyze_space,
                    include,
                    exclude,
                    output,
                    perf,
                    high_complexity_only,
                    top_files,
                )
            }
            _ => Err(ProtocolError::UnsupportedProtocol(
                "Command not supported in advanced analysis dispatch".to_string(),
            )),
        }
    }
}
