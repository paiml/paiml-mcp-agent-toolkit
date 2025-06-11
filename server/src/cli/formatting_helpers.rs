use crate::models::project_meta::{BuildInfo, ProjectOverview};
use crate::services::deep_context::DeepContext;
use std::fmt::Write;

/// Format executive summary section
pub fn format_executive_summary(context: &DeepContext) -> String {
    let mut output = String::new();

    output.push_str("## Executive Summary\n\n");
    let _ = writeln!(&mut output, "Generated: {}", context.metadata.generated_at);
    let _ = writeln!(&mut output, "Version: {}", context.metadata.tool_version);
    let _ = writeln!(
        &mut output,
        "Analysis Time: {:.2}s",
        context.metadata.analysis_duration.as_secs_f64()
    );
    let _ = writeln!(
        &mut output,
        "Cache Hit Rate: {:.1}%",
        context.metadata.cache_stats.hit_rate * 100.0
    );

    output
}

/// Format quality scorecard section
pub fn format_quality_scorecard(context: &DeepContext) -> String {
    let mut output = String::new();

    output.push_str("\n## Quality Scorecard\n\n");

    let health_emoji = match context.quality_scorecard.overall_health {
        h if h >= 80.0 => "✅",
        h if h >= 60.0 => "⚠️",
        _ => "❌",
    };

    let _ = writeln!(
        &mut output,
        "- **Overall Health**: {} ({:.1}/100)",
        health_emoji, context.quality_scorecard.overall_health
    );
    let _ = writeln!(
        &mut output,
        "- **Maintainability Index**: {:.1}",
        context.quality_scorecard.maintainability_index
    );
    let _ = writeln!(
        &mut output,
        "- **Technical Debt**: {:.1} hours estimated",
        context.quality_scorecard.technical_debt_hours
    );

    output
}

/// Format project overview from README
pub fn format_project_overview(overview: &ProjectOverview) -> String {
    let mut output = String::new();

    output.push_str("### README.md (Documentation)\n\n");

    if !overview.compressed_description.is_empty() {
        output.push_str(&overview.compressed_description);
        output.push_str("\n\n");
    }

    if !overview.key_features.is_empty() {
        output.push_str("**Key Features:**\n");
        for feature in &overview.key_features {
            let _ = writeln!(&mut output, "- {feature}");
        }
        output.push('\n');
    }

    if let Some(arch) = &overview.architecture_summary {
        output.push_str("**Architecture:**\n");
        output.push_str(arch);
        output.push_str("\n\n");
    }

    output
}

/// Format build info from Makefile
pub fn format_build_info(build_info: &BuildInfo) -> String {
    let mut output = String::new();

    output.push_str("### Makefile (Build Configuration)\n\n");
    let _ = writeln!(&mut output, "**Toolchain:** {}", build_info.toolchain);

    if !build_info.targets.is_empty() {
        output.push_str("**Key Targets:**\n");
        for target in &build_info.targets {
            let _ = writeln!(&mut output, "- {target}");
        }
        output.push('\n');
    }

    if !build_info.dependencies.is_empty() {
        output.push_str("**Dependencies:**\n");
        for dep in &build_info.dependencies {
            let _ = writeln!(&mut output, "- {dep}");
        }
        output.push('\n');
    }

    output
}

/// Format defect summary section
pub fn format_defect_summary(context: &DeepContext) -> String {
    let mut output = String::new();

    if context.defect_summary.total_defects > 0 {
        output.push_str("\n## Defect Summary\n\n");
        let _ = writeln!(
            &mut output,
            "- **Total Defects Found**: {}",
            context.defect_summary.total_defects
        );
        let _ = writeln!(
            &mut output,
            "- **Defect Density**: {:.2}",
            context.defect_summary.defect_density
        );

        // Show severity breakdown if available
        if !context.defect_summary.by_severity.is_empty() {
            output.push_str("\n**By Severity:**\n");
            for (severity, count) in &context.defect_summary.by_severity {
                let _ = writeln!(&mut output, "- {severity}: {count}");
            }
        }

        // Show type breakdown if available
        if !context.defect_summary.by_type.is_empty() {
            output.push_str("\n**By Type:**\n");
            for (defect_type, count) in &context.defect_summary.by_type {
                let _ = writeln!(&mut output, "- {defect_type}: {count}");
            }
        }
    }

    output
}

/// Format recommendations section
pub fn format_recommendations(context: &DeepContext) -> String {
    let mut output = String::new();

    if !context.recommendations.is_empty() {
        output.push_str("\n## Recommendations\n\n");

        for (i, rec) in context.recommendations.iter().enumerate() {
            let _ = writeln!(
                &mut output,
                "{}. **{}** (Priority: {:?})",
                i + 1,
                rec.title,
                rec.priority
            );
            let _ = writeln!(&mut output, "   - Impact: {:?}", rec.impact);
            let _ = writeln!(
                &mut output,
                "   - Effort: {:.1} hours",
                rec.estimated_effort.as_secs_f64() / 3600.0
            );
            let _ = writeln!(&mut output, "   - {}", rec.description);
            output.push('\n');
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project_meta::{BuildInfo, ProjectOverview};
    use crate::services::deep_context::{
        AnalysisResults, AnnotatedFileTree, AnnotatedNode, CacheStats, ContextMetadata,
        DeepContext, DefectSummary, Impact, PrioritizedRecommendation, Priority, QualityScorecard,
    };
    use chrono::Utc;
    use rustc_hash::FxHashMap;
    use std::time::Duration;

    fn create_test_deep_context() -> DeepContext {
        DeepContext {
            metadata: ContextMetadata {
                generated_at: Utc::now(),
                tool_version: "1.0.0".to_string(),
                project_root: std::path::PathBuf::from("/test"),
                analysis_duration: Duration::from_secs(5),
                cache_stats: CacheStats {
                    hit_rate: 0.85,
                    memory_efficiency: 0.9,
                    time_saved_ms: 150,
                },
            },
            file_tree: AnnotatedFileTree {
                root: AnnotatedNode {
                    name: "root".to_string(),
                    path: std::path::PathBuf::from("/"),
                    node_type: crate::services::deep_context::NodeType::Directory,
                    children: vec![],
                    annotations: crate::services::deep_context::NodeAnnotations {
                        defect_score: Some(0.1),
                        complexity_score: Some(10.0),
                        cognitive_complexity: Some(8),
                        churn_score: Some(0.2),
                        dead_code_items: 0,
                        satd_items: 0,
                        centrality: Some(0.5),
                        test_coverage: Some(0.8),
                        big_o_complexity: Some("O(n)".to_string()),
                        memory_complexity: Some("O(1)".to_string()),
                        duplication_score: Some(0.1),
                    },
                },
                total_files: 5,
                total_size_bytes: 1000,
            },
            analyses: AnalysisResults {
                ast_contexts: vec![],
                complexity_report: None,
                churn_analysis: None,
                dependency_graph: None,
                dead_code_results: None,
                duplicate_code_results: None,
                satd_results: None,
                provability_results: None,
                cross_language_refs: vec![],
                big_o_analysis: None,
            },
            quality_scorecard: QualityScorecard {
                overall_health: 75.5,
                complexity_score: 8.2,
                maintainability_index: 68.2,
                modularity_score: 7.5,
                test_coverage: Some(65.0),
                technical_debt_hours: 24.5,
            },
            template_provenance: None,
            defect_summary: DefectSummary {
                total_defects: 5,
                defect_density: 1.2,
                by_severity: FxHashMap::from_iter([
                    ("High".to_string(), 1),
                    ("Medium".to_string(), 2),
                    ("Low".to_string(), 2),
                ]),
                by_type: FxHashMap::from_iter([
                    ("Logic Error".to_string(), 2),
                    ("Resource Leak".to_string(), 1),
                    ("Type Safety".to_string(), 2),
                ]),
            },
            hotspots: vec![],
            recommendations: vec![
                PrioritizedRecommendation {
                    title: "Reduce complexity in main module".to_string(),
                    description: "Break down large functions into smaller ones".to_string(),
                    priority: Priority::High,
                    impact: Impact::High,
                    estimated_effort: Duration::from_secs(3600 * 4), // 4 hours
                    prerequisites: vec![],
                },
                PrioritizedRecommendation {
                    title: "Add unit tests".to_string(),
                    description: "Increase test coverage for core modules".to_string(),
                    priority: Priority::Medium,
                    impact: Impact::Medium,
                    estimated_effort: Duration::from_secs(3600 * 8), // 8 hours
                    prerequisites: vec![],
                },
            ],
            qa_verification: None,
            build_info: None,
            project_overview: None,
        }
    }

    #[test]
    fn test_format_executive_summary() {
        let context = create_test_deep_context();
        let result = format_executive_summary(&context);

        assert!(result.contains("## Executive Summary"));
        assert!(result.contains("Generated:"));
        assert!(result.contains("Version: 1.0.0"));
        assert!(result.contains("Analysis Time: 5.00s"));
        assert!(result.contains("Cache Hit Rate: 85.0%"));
    }

    #[test]
    fn test_format_quality_scorecard_high_health() {
        let mut context = create_test_deep_context();
        context.quality_scorecard.overall_health = 85.0;

        let result = format_quality_scorecard(&context);

        assert!(result.contains("## Quality Scorecard"));
        assert!(result.contains("✅"));
        assert!(result.contains("85.0/100"));
        assert!(result.contains("Maintainability Index"));
        assert!(result.contains("Technical Debt"));
    }

    #[test]
    fn test_format_quality_scorecard_medium_health() {
        let mut context = create_test_deep_context();
        context.quality_scorecard.overall_health = 65.0;

        let result = format_quality_scorecard(&context);

        assert!(result.contains("⚠️"));
        assert!(result.contains("65.0/100"));
    }

    #[test]
    fn test_format_quality_scorecard_low_health() {
        let mut context = create_test_deep_context();
        context.quality_scorecard.overall_health = 45.0;

        let result = format_quality_scorecard(&context);

        assert!(result.contains("❌"));
        assert!(result.contains("45.0/100"));
    }

    #[test]
    fn test_format_project_overview_complete() {
        let overview = ProjectOverview {
            compressed_description: "A test project for demonstration".to_string(),
            key_features: vec![
                "Feature 1".to_string(),
                "Feature 2".to_string(),
                "Feature 3".to_string(),
            ],
            architecture_summary: Some("Layered architecture with clean separation".to_string()),
            api_summary: Some("REST API with JSON responses".to_string()),
        };

        let result = format_project_overview(&overview);

        assert!(result.contains("### README.md"));
        assert!(result.contains("A test project for demonstration"));
        assert!(result.contains("**Key Features:**"));
        assert!(result.contains("- Feature 1"));
        assert!(result.contains("- Feature 2"));
        assert!(result.contains("- Feature 3"));
        assert!(result.contains("**Architecture:**"));
        assert!(result.contains("Layered architecture"));
    }

    #[test]
    fn test_format_project_overview_minimal() {
        let overview = ProjectOverview {
            compressed_description: "".to_string(),
            key_features: vec![],
            architecture_summary: None,
            api_summary: None,
        };

        let result = format_project_overview(&overview);

        assert!(result.contains("### README.md"));
        assert!(!result.contains("**Key Features:**"));
        assert!(!result.contains("**Architecture:**"));
    }

    #[test]
    fn test_format_build_info_complete() {
        let build_info = BuildInfo {
            toolchain: "Rust 1.70".to_string(),
            targets: vec!["build".to_string(), "test".to_string(), "clean".to_string()],
            dependencies: vec![
                "serde".to_string(),
                "tokio".to_string(),
                "anyhow".to_string(),
            ],
            primary_command: Some("cargo build".to_string()),
        };

        let result = format_build_info(&build_info);

        assert!(result.contains("### Makefile"));
        assert!(result.contains("**Toolchain:** Rust 1.70"));
        assert!(result.contains("**Key Targets:**"));
        assert!(result.contains("- build"));
        assert!(result.contains("- test"));
        assert!(result.contains("- clean"));
        assert!(result.contains("**Dependencies:**"));
        assert!(result.contains("- serde"));
        assert!(result.contains("- tokio"));
        assert!(result.contains("- anyhow"));
    }

    #[test]
    fn test_format_build_info_minimal() {
        let build_info = BuildInfo {
            toolchain: "Unknown".to_string(),
            targets: vec![],
            dependencies: vec![],
            primary_command: None,
        };

        let result = format_build_info(&build_info);

        assert!(result.contains("### Makefile"));
        assert!(result.contains("**Toolchain:** Unknown"));
        assert!(!result.contains("**Key Targets:**"));
        assert!(!result.contains("**Dependencies:**"));
    }

    #[test]
    fn test_format_defect_summary_with_defects() {
        let context = create_test_deep_context();
        let result = format_defect_summary(&context);

        assert!(result.contains("## Defect Summary"));
        assert!(result.contains("**Total Defects Found**: 5"));
        assert!(result.contains("**Defect Density**: 1.20"));
        assert!(result.contains("**By Severity:**"));
        assert!(result.contains("High: 1"));
        assert!(result.contains("Medium: 2"));
        assert!(result.contains("Low: 2"));
        assert!(result.contains("**By Type:**"));
        assert!(result.contains("Logic Error: 2"));
        assert!(result.contains("Resource Leak: 1"));
        assert!(result.contains("Type Safety: 2"));
    }

    #[test]
    fn test_format_defect_summary_no_defects() {
        let mut context = create_test_deep_context();
        context.defect_summary.total_defects = 0;

        let result = format_defect_summary(&context);

        assert!(result.is_empty());
    }

    #[test]
    fn test_format_defect_summary_minimal() {
        let mut context = create_test_deep_context();
        context.defect_summary.by_severity.clear();
        context.defect_summary.by_type.clear();

        let result = format_defect_summary(&context);

        assert!(result.contains("## Defect Summary"));
        assert!(result.contains("**Total Defects Found**: 5"));
        assert!(!result.contains("**By Severity:**"));
        assert!(!result.contains("**By Type:**"));
    }

    #[test]
    fn test_format_recommendations_with_items() {
        let context = create_test_deep_context();
        let result = format_recommendations(&context);

        assert!(result.contains("## Recommendations"));
        assert!(result.contains("1. **Reduce complexity in main module** (Priority: High)"));
        assert!(result.contains("Impact: High"));
        assert!(result.contains("Effort: 4.0 hours"));
        assert!(result.contains("Break down large functions"));
        assert!(result.contains("2. **Add unit tests** (Priority: Medium)"));
        assert!(result.contains("Impact: Medium"));
        assert!(result.contains("Effort: 8.0 hours"));
        assert!(result.contains("Increase test coverage"));
    }

    #[test]
    fn test_format_recommendations_empty() {
        let mut context = create_test_deep_context();
        context.recommendations.clear();

        let result = format_recommendations(&context);

        assert!(result.is_empty());
    }

    #[test]
    fn test_format_recommendations_single_item() {
        let mut context = create_test_deep_context();
        context.recommendations.truncate(1);

        let result = format_recommendations(&context);

        assert!(result.contains("## Recommendations"));
        assert!(result.contains("1. **Reduce complexity"));
        assert!(!result.contains("2. **Add unit tests"));
    }
}
