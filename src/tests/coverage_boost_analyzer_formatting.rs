#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/deep_context/analyzer_formatting.rs
//!
//! Tests the pure formatting methods of `DeepContextAnalyzer` which produce
//! markdown, JSON, and SARIF output from `DeepContext` structures.
//! All methods under test are pure (no filesystem access).

use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
use crate::models::dead_code::{DeadCodeAnalysisConfig, DeadCodeRankingResult, DeadCodeSummary};
use crate::models::project_meta::{BuildInfo, ProjectOverview};
use crate::services::complexity::{
    ComplexityMetrics, ComplexityReport, ComplexitySummary, FileComplexityMetrics,
    FunctionComplexity,
};
use crate::services::context::{AstItem, FileContext};
use crate::services::deep_context::{
    AnnotatedFileTree, AnnotatedNode, CrossLangReference,
    CrossLangReferenceType, DeepContext, DeepContextAnalyzer, DeepContextConfig, DefectAnnotations,
    DefectHotspot, DefectSummary, EnhancedFileContext, FileLocation, Impact, NodeAnnotations,
    NodeType, Priority, PrioritizedRecommendation, QualityScorecard, RefactoringEstimate,
};
use crate::services::satd_detector::{DebtCategory, SATDAnalysisResult, SATDSummary, Severity};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helper constructors
// ---------------------------------------------------------------------------

fn make_analyzer() -> DeepContextAnalyzer {
    DeepContextAnalyzer::new(DeepContextConfig::default())
}

fn make_empty_context() -> DeepContext {
    DeepContext::default()
}

fn make_scorecard(health: f64, maintainability: f64, debt_hours: f64) -> QualityScorecard {
    QualityScorecard {
        overall_health: health,
        complexity_score: 75.0,
        maintainability_index: maintainability,
        modularity_score: 80.0,
        test_coverage: Some(90.0),
        technical_debt_hours: debt_hours,
    }
}

fn make_recommendation(
    title: &str,
    priority: Priority,
    impact: Impact,
    prereqs: Vec<&str>,
) -> PrioritizedRecommendation {
    PrioritizedRecommendation {
        title: title.to_string(),
        description: format!("Description for {title}"),
        priority,
        estimated_effort: Duration::from_secs(3600),
        impact,
        prerequisites: prereqs.into_iter().map(String::from).collect(),
    }
}

fn make_project_overview() -> ProjectOverview {
    ProjectOverview {
        compressed_description: "A test project for analysis.".to_string(),
        key_features: vec!["Feature A".to_string(), "Feature B".to_string()],
        architecture_summary: Some("Microservices architecture".to_string()),
        api_summary: None,
    }
}

fn make_build_info() -> BuildInfo {
    BuildInfo {
        toolchain: "Rust".to_string(),
        targets: vec!["pmat".to_string(), "pmat-cli".to_string()],
        dependencies: vec!["serde".to_string(), "tokio".to_string()],
        primary_command: Some("cargo build --release".to_string()),
    }
}

fn make_annotated_node(name: &str, node_type: NodeType) -> AnnotatedNode {
    AnnotatedNode {
        name: name.to_string(),
        path: PathBuf::from(name),
        node_type,
        children: Vec::new(),
        annotations: NodeAnnotations::default(),
    }
}

fn make_annotated_tree(total_files: usize, total_size: u64) -> AnnotatedFileTree {
    AnnotatedFileTree {
        root: make_annotated_node("root", NodeType::Directory),
        total_files,
        total_size_bytes: total_size,
    }
}

fn make_file_context(path: &str, language: &str, items: Vec<AstItem>) -> FileContext {
    FileContext {
        path: path.to_string(),
        language: language.to_string(),
        items,
        complexity_metrics: None,
    }
}

fn make_enhanced_file_context(path: &str, language: &str) -> EnhancedFileContext {
    EnhancedFileContext {
        base: make_file_context(path, language, Vec::new()),
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: format!("sym_{path}"),
    }
}

fn make_complexity_report() -> ComplexityReport {
    ComplexityReport {
        summary: ComplexitySummary {
            total_files: 5,
            total_functions: 20,
            median_cyclomatic: 4.5,
            median_cognitive: 3.0,
            max_cyclomatic: 25,
            max_cognitive: 18,
            p90_cyclomatic: 12,
            p90_cognitive: 10,
            technical_debt_hours: 8.5,
        },
        violations: Vec::new(),
        hotspots: Vec::new(),
        files: vec![FileComplexityMetrics {
            path: "src/main.rs".to_string(),
            total_complexity: ComplexityMetrics::new(15, 12, 3, 100),
            functions: vec![
                FunctionComplexity {
                    name: "complex_function".to_string(),
                    line_start: 10,
                    line_end: 50,
                    metrics: ComplexityMetrics::new(15, 20, 4, 40),
                },
                FunctionComplexity {
                    name: "simple_function".to_string(),
                    line_start: 55,
                    line_end: 60,
                    metrics: ComplexityMetrics::new(2, 1, 1, 5),
                },
            ],
            classes: Vec::new(),
        }],
    }
}

fn make_churn_analysis() -> CodeChurnAnalysis {
    CodeChurnAnalysis {
        generated_at: Utc::now(),
        period_days: 30,
        repository_root: PathBuf::from("/test/project"),
        files: vec![FileChurnMetrics {
            path: PathBuf::from("src/lib.rs"),
            relative_path: "src/lib.rs".to_string(),
            commit_count: 42,
            unique_authors: vec!["alice".to_string(), "bob".to_string()],
            additions: 500,
            deletions: 200,
            churn_score: 0.75,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        }],
        summary: ChurnSummary {
            total_commits: 100,
            total_files_changed: 15,
            hotspot_files: vec![PathBuf::from("src/lib.rs")],
            stable_files: vec![PathBuf::from("src/config.rs")],
            author_contributions: HashMap::new(),
            mean_churn_score: 0.45,
            variance_churn_score: 0.12,
            stddev_churn_score: 0.35,
        },
    }
}

fn make_satd_result() -> SATDAnalysisResult {
    SATDAnalysisResult {
        items: vec![
            crate::services::satd_detector::TechnicalDebt {
                category: DebtCategory::Defect,
                severity: Severity::Critical,
                text: "  FIXME: critical bug here  ".to_string(),
                file: PathBuf::from("src/main.rs"),
                line: 42,
                column: 5,
                context_hash: [0u8; 16],
            },
            crate::services::satd_detector::TechnicalDebt {
                category: DebtCategory::Requirement,
                severity: Severity::Low,
                text: "TODO: add logging".to_string(),
                file: PathBuf::from("src/util.rs"),
                line: 10,
                column: 1,
                context_hash: [1u8; 16],
            },
            crate::services::satd_detector::TechnicalDebt {
                category: DebtCategory::Design,
                severity: Severity::High,
                text: "HACK: workaround for upstream".to_string(),
                file: PathBuf::from("src/hack.rs"),
                line: 5,
                column: 1,
                context_hash: [2u8; 16],
            },
            crate::services::satd_detector::TechnicalDebt {
                category: DebtCategory::Performance,
                severity: Severity::Medium,
                text: "SLOW: O(n^2) scan".to_string(),
                file: PathBuf::from("src/scan.rs"),
                line: 100,
                column: 1,
                context_hash: [3u8; 16],
            },
        ],
        summary: SATDSummary {
            total_items: 4,
            by_severity: HashMap::new(),
            by_category: HashMap::new(),
            files_with_satd: 4,
            avg_age_days: 15.0,
        },
        total_files_analyzed: 10,
        files_with_debt: 4,
        analysis_timestamp: Utc::now(),
    }
}

fn make_dead_code_result() -> DeadCodeRankingResult {
    DeadCodeRankingResult {
        summary: DeadCodeSummary {
            total_files_analyzed: 10,
            files_with_dead_code: 3,
            total_dead_lines: 150,
            dead_percentage: 5.0,
            dead_functions: 8,
            dead_classes: 1,
            dead_modules: 0,
            unreachable_blocks: 2,
        },
        ranked_files: vec![
            crate::models::dead_code::FileDeadCodeMetrics {
                path: "src/old_module.rs".to_string(),
                dead_lines: 80,
                total_lines: 200,
                dead_percentage: 40.0,
                dead_functions: 5,
                dead_classes: 1,
                dead_modules: 0,
                unreachable_blocks: 1,
                dead_score: 0.85,
                confidence: crate::models::dead_code::ConfidenceLevel::High,
                items: Vec::new(),
            },
            crate::models::dead_code::FileDeadCodeMetrics {
                path: "src/legacy.rs".to_string(),
                dead_lines: 50,
                total_lines: 300,
                dead_percentage: 16.7,
                dead_functions: 3,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 1,
                dead_score: 0.55,
                confidence: crate::models::dead_code::ConfidenceLevel::Medium,
                items: Vec::new(),
            },
            // File with zero dead functions (should be filtered out of SARIF)
            crate::models::dead_code::FileDeadCodeMetrics {
                path: "src/clean.rs".to_string(),
                dead_lines: 0,
                total_lines: 100,
                dead_percentage: 0.0,
                dead_functions: 0,
                dead_classes: 0,
                dead_modules: 0,
                unreachable_blocks: 0,
                dead_score: 0.0,
                confidence: crate::models::dead_code::ConfidenceLevel::Low,
                items: Vec::new(),
            },
        ],
        analysis_timestamp: Utc::now(),
        config: DeadCodeAnalysisConfig {
            include_unreachable: true,
            include_tests: false,
            min_dead_lines: 1,
        },
    }
}

fn make_defect_hotspot(file: &str, line: u32, score: f32, hours: f32) -> DefectHotspot {
    DefectHotspot {
        location: FileLocation {
            file: PathBuf::from(file),
            line,
            column: 1,
        },
        composite_score: score,
        contributing_factors: Vec::new(),
        refactoring_effort: RefactoringEstimate {
            estimated_hours: hours,
            priority: Priority::High,
            impact: Impact::High,
            suggested_actions: vec!["Refactor".to_string()],
        },
    }
}

fn make_cross_lang_ref(src: &str, tgt: &str, confidence: f32) -> CrossLangReference {
    CrossLangReference {
        source_file: PathBuf::from(src),
        target_file: PathBuf::from(tgt),
        reference_type: CrossLangReferenceType::FfiCall,
        confidence,
    }
}

fn make_populated_context() -> DeepContext {
    let mut ctx = DeepContext::default();
    ctx.quality_scorecard = make_scorecard(85.0, 72.0, 12.5);
    ctx.project_overview = Some(make_project_overview());
    ctx.build_info = Some(make_build_info());
    ctx.recommendations = vec![
        make_recommendation("Reduce complexity", Priority::High, Impact::High, vec![]),
        make_recommendation("Add tests", Priority::Medium, Impact::Medium, vec!["CI setup"]),
    ];
    ctx.file_tree = make_annotated_tree(42, 512_000);
    ctx.analyses.complexity_report = Some(make_complexity_report());
    ctx.analyses.churn_analysis = Some(make_churn_analysis());
    ctx.analyses.ast_contexts = vec![make_enhanced_file_context("src/lib.rs", "Rust")];
    ctx.defect_summary = DefectSummary {
        total_defects: 5,
        defect_density: 2.5,
        ..Default::default()
    };
    ctx.hotspots = vec![make_defect_hotspot("src/lib.rs", 100, 0.9, 4.0)];
    ctx
}

// ===========================================================================
// Constructor tests
// ===========================================================================

#[test]
fn test_new_with_default_config() {
    // Verify creation does not panic and the analyzer is usable
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let result = analyzer.format_as_json(&ctx);
    assert!(result.is_ok());
}

#[test]
fn test_new_with_custom_config() {
    let config = DeepContextConfig {
        period_days: 90,
        parallel: 2,
        ..Default::default()
    };
    let analyzer = DeepContextAnalyzer::new(config);
    // Verify the custom-config analyzer can produce output (config is private)
    let ctx = make_empty_context();
    let json = analyzer.format_as_json(&ctx).unwrap();
    assert!(json.contains("metadata"));
}

// ===========================================================================
// format_as_comprehensive_markdown (async) -- empty context
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_empty_context() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let result = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(result.starts_with("# Deep Context Analysis Report"));
    assert!(result.contains("Quality Scorecard"));
}

#[tokio::test]
async fn test_comprehensive_markdown_contains_header() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("# Deep Context Analysis Report"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- project overview
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_with_project_overview() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.project_overview = Some(make_project_overview());
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("## Project Overview"));
    assert!(md.contains("A test project for analysis."));
    assert!(md.contains("Feature A"));
    assert!(md.contains("Feature B"));
    assert!(md.contains("Microservices architecture"));
}

#[tokio::test]
async fn test_comprehensive_markdown_overview_no_description() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.project_overview = Some(ProjectOverview {
        compressed_description: String::new(),
        key_features: vec!["Only feature".to_string()],
        architecture_summary: None,
        api_summary: None,
    });
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("## Project Overview"));
    assert!(md.contains("Only feature"));
    // Empty description should not produce an extra paragraph
    assert!(!md.contains("\n\n\n\n"));
}

#[tokio::test]
async fn test_comprehensive_markdown_overview_no_features() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.project_overview = Some(ProjectOverview {
        compressed_description: "Desc".to_string(),
        key_features: Vec::new(),
        architecture_summary: None,
        api_summary: None,
    });
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("Desc"));
    assert!(!md.contains("Key Features"));
}

#[tokio::test]
async fn test_comprehensive_markdown_overview_no_architecture() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.project_overview = Some(ProjectOverview {
        compressed_description: "Desc".to_string(),
        key_features: vec!["A".to_string()],
        architecture_summary: None,
        api_summary: None,
    });
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(!md.contains("**Architecture:**"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- build info
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_with_build_info() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.build_info = Some(make_build_info());
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("## Build System"));
    assert!(md.contains("**Detected Toolchain:** Rust"));
    assert!(md.contains("pmat, pmat-cli"));
    assert!(md.contains("serde, tokio"));
    assert!(md.contains("`cargo build --release`"));
}

#[tokio::test]
async fn test_comprehensive_markdown_build_info_no_targets() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.build_info = Some(BuildInfo {
        toolchain: "Python".to_string(),
        targets: Vec::new(),
        dependencies: Vec::new(),
        primary_command: None,
    });
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("**Detected Toolchain:** Python"));
    assert!(!md.contains("Primary Targets"));
    assert!(!md.contains("Key Dependencies"));
    assert!(!md.contains("Build Command"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- quality scorecard
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_quality_scorecard_default() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("Overall Health: 0.0%"));
    assert!(md.contains("Maintainability Index: 0.0%"));
    assert!(md.contains("Refactoring Time: 0.0 hours"));
    assert!(md.contains("Complexity Score: 0.0%"));
}

#[tokio::test]
async fn test_comprehensive_markdown_quality_scorecard_high_health() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.quality_scorecard = make_scorecard(95.0, 88.0, 2.0);
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("Overall Health: 95.0%"));
    assert!(md.contains("Maintainability Index: 88.0%"));
    assert!(md.contains("Refactoring Time: 2.0 hours"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- project structure
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_project_structure() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.file_tree = make_annotated_tree(100, 1_000_000);
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("## Project Structure"));
    assert!(md.contains("Total Files: 100"));
    assert!(md.contains("Total Size: 1000000 bytes"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- analysis results
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_analysis_results_empty() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("## Analysis Results"));
    // No AST, complexity, or churn sub-headings when empty
    assert!(!md.contains("### AST Analysis"));
    assert!(!md.contains("### Complexity Analysis"));
    assert!(!md.contains("### Code Churn"));
}

#[tokio::test]
async fn test_comprehensive_markdown_with_ast_contexts() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.ast_contexts = vec![
        make_enhanced_file_context("a.rs", "Rust"),
        make_enhanced_file_context("b.rs", "Rust"),
        make_enhanced_file_context("c.rs", "Rust"),
    ];
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("### AST Analysis"));
    assert!(md.contains("Files analyzed: 3"));
}

#[tokio::test]
async fn test_comprehensive_markdown_with_complexity_report() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.complexity_report = Some(make_complexity_report());
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("### Complexity Analysis"));
    assert!(md.contains("Total files: 5"));
    assert!(md.contains("Total functions: 20"));
    assert!(md.contains("Median cyclomatic complexity: 4.5"));
}

#[tokio::test]
async fn test_comprehensive_markdown_with_churn_analysis() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.churn_analysis = Some(make_churn_analysis());
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("### Code Churn"));
    assert!(md.contains("Files analyzed: 1"));
    assert!(md.contains("Total commits: 100"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- recommendations
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_no_recommendations() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    // When empty, the recommendations section should NOT be appended
    assert!(!md.contains("## Recommendations"));
}

#[tokio::test]
async fn test_comprehensive_markdown_with_recommendations() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.recommendations = vec![
        make_recommendation("Fix bug", Priority::Critical, Impact::High, vec!["Deploy"]),
        make_recommendation("Add docs", Priority::Low, Impact::Low, vec![]),
    ];
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    assert!(md.contains("## Recommendations"));
    assert!(md.contains("**Fix bug**"));
    assert!(md.contains("Priority: Critical"));
    assert!(md.contains("**Add docs**"));
    assert!(md.contains("Priority: Low"));
}

#[tokio::test]
async fn test_comprehensive_markdown_fully_populated() {
    let analyzer = make_analyzer();
    let ctx = make_populated_context();
    let md = analyzer.format_as_comprehensive_markdown(&ctx).await.unwrap();
    // All sections should appear
    assert!(md.contains("# Deep Context Analysis Report"));
    assert!(md.contains("## Project Overview"));
    assert!(md.contains("## Build System"));
    assert!(md.contains("## Quality Scorecard"));
    assert!(md.contains("## Project Structure"));
    assert!(md.contains("## Analysis Results"));
    assert!(md.contains("## Recommendations"));
}

// ===========================================================================
// format_as_comprehensive_markdown_legacy -- empty context
// ===========================================================================

#[test]
fn test_legacy_markdown_empty_context() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let result = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(result.contains("# Deep Context:"));
    assert!(result.contains("## Quality Scorecard"));
}

#[test]
fn test_legacy_markdown_header_contains_metadata() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.metadata.tool_version = "1.2.3".to_string();
    ctx.metadata.project_root = PathBuf::from("/test/my_project");
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("# Deep Context: my_project"));
    assert!(md.contains("Version: 1.2.3"));
}

#[test]
fn test_legacy_markdown_header_analysis_duration() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.metadata.analysis_duration = Duration::from_millis(1500);
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("Analysis Time: 1.50s"));
}

#[test]
fn test_legacy_markdown_header_cache_hit_rate() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.metadata.cache_stats.hit_rate = 0.85;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("Cache Hit Rate: 85.0%"));
}

// ===========================================================================
// format_as_comprehensive_markdown_legacy -- quality scorecard + health emoji
// ===========================================================================

#[test]
fn test_legacy_markdown_scorecard_high_health() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.quality_scorecard = make_scorecard(90.0, 80.0, 5.0);
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    // Health >= 80 should produce the check emoji
    assert!(md.contains("(90.0/100)"));
    assert!(md.contains("Maintainability Index"));
    assert!(md.contains("80.0"));
    assert!(md.contains("Refactoring Time"));
    assert!(md.contains("5.0 hours"));
}

#[test]
fn test_legacy_markdown_scorecard_medium_health() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.quality_scorecard = make_scorecard(65.0, 55.0, 20.0);
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("(65.0/100)"));
}

#[test]
fn test_legacy_markdown_scorecard_low_health() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.quality_scorecard = make_scorecard(40.0, 30.0, 50.0);
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("(40.0/100)"));
}

// ===========================================================================
// format_as_comprehensive_markdown_legacy -- project structure (annotated tree)
// ===========================================================================

#[test]
fn test_legacy_markdown_project_structure_with_tree() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut root = make_annotated_node("project", NodeType::Directory);
    root.children.push(make_annotated_node("src", NodeType::Directory));
    root.children.push(make_annotated_node("README.md", NodeType::File));
    ctx.file_tree = AnnotatedFileTree {
        root,
        total_files: 2,
        total_size_bytes: 4096,
    };
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("## Project Structure"));
    assert!(md.contains("project/"));
    assert!(md.contains("src/"));
    assert!(md.contains("README.md"));
    assert!(md.contains("Total Files: 2"));
    assert!(md.contains("Total Size: 4096 bytes"));
}

#[test]
fn test_legacy_markdown_tree_nested_nodes() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();

    let mut child_dir = make_annotated_node("src", NodeType::Directory);
    child_dir
        .children
        .push(make_annotated_node("main.rs", NodeType::File));
    child_dir
        .children
        .push(make_annotated_node("lib.rs", NodeType::File));

    let mut root = make_annotated_node("root", NodeType::Directory);
    root.children.push(child_dir);

    ctx.file_tree = AnnotatedFileTree {
        root,
        total_files: 2,
        total_size_bytes: 2048,
    };
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("main.rs"));
    assert!(md.contains("lib.rs"));
}

// ===========================================================================
// format_as_comprehensive_markdown_legacy -- node annotations (indirect tests)
// ===========================================================================

#[test]
fn test_legacy_markdown_annotations_defect_score_high() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("buggy.rs", NodeType::File);
    node.annotations.defect_score = Some(0.9);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    // High defect score (>0.7) should show the red indicator
    assert!(md.contains("0.9"));
}

#[test]
fn test_legacy_markdown_annotations_defect_score_medium() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("risky.rs", NodeType::File);
    node.annotations.defect_score = Some(0.5);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("0.5"));
}

#[test]
fn test_legacy_markdown_annotations_defect_score_low_not_shown() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("clean.rs", NodeType::File);
    node.annotations.defect_score = Some(0.2);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    // Defect score <= 0.4 should NOT produce an indicator
    assert!(!md.contains("0.2"));
}

#[test]
fn test_legacy_markdown_annotations_cognitive_complexity_high() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("complex.rs", NodeType::File);
    node.annotations.cognitive_complexity = Some(35);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("35"));
}

#[test]
fn test_legacy_markdown_annotations_cognitive_complexity_medium() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("moderate.rs", NodeType::File);
    node.annotations.cognitive_complexity = Some(20);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("20"));
}

#[test]
fn test_legacy_markdown_annotations_cognitive_complexity_low_not_shown() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("simple.rs", NodeType::File);
    node.annotations.cognitive_complexity = Some(10);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    // Cognitive <= 15 produces no indicator
    assert!(!md.contains("[") || !md.contains("10"));
}

#[test]
fn test_legacy_markdown_annotations_satd_items() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("todo.rs", NodeType::File);
    node.annotations.satd_items = 3;
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("3"));
}

#[test]
fn test_legacy_markdown_annotations_dead_code_items() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("dead.rs", NodeType::File);
    node.annotations.dead_code_items = 7;
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("7"));
}

#[test]
fn test_legacy_markdown_annotations_test_coverage_low() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("uncovered.rs", NodeType::File);
    node.annotations.test_coverage = Some(0.3);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    // Coverage < 0.5 should produce the alert indicator
    assert!(md.contains("30%"));
}

#[test]
fn test_legacy_markdown_annotations_test_coverage_medium() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("partial.rs", NodeType::File);
    node.annotations.test_coverage = Some(0.65);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("65%"));
}

#[test]
fn test_legacy_markdown_annotations_test_coverage_high() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("covered.rs", NodeType::File);
    node.annotations.test_coverage = Some(0.95);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("95%"));
}

#[test]
fn test_legacy_markdown_annotations_big_o_constant() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("fast.rs", NodeType::File);
    node.annotations.big_o_complexity = Some("O(1)".to_string());
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("O(1)"));
}

#[test]
fn test_legacy_markdown_annotations_big_o_quadratic() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("slow.rs", NodeType::File);
    node.annotations.big_o_complexity = Some("O(n\u{00b2})".to_string());
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("O(n\u{00b2})"));
}

#[test]
fn test_legacy_markdown_annotations_churn_high() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("hot.rs", NodeType::File);
    node.annotations.churn_score = Some(0.9);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("0.9"));
}

#[test]
fn test_legacy_markdown_annotations_churn_medium() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("warm.rs", NodeType::File);
    node.annotations.churn_score = Some(0.6);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("0.6"));
}

#[test]
fn test_legacy_markdown_annotations_churn_low() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("calm.rs", NodeType::File);
    node.annotations.churn_score = Some(0.3);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("0.3"));
}

#[test]
fn test_legacy_markdown_annotations_churn_very_low_not_shown() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("stable.rs", NodeType::File);
    node.annotations.churn_score = Some(0.1);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    // Churn <= 0.2 should not produce an annotation
    assert!(!md.contains("[") || !md.contains("0.1"));
}

#[test]
fn test_legacy_markdown_annotations_memory_complexity() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("mem.rs", NodeType::File);
    node.annotations.memory_complexity = Some("O(n)".to_string());
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("O(n)"));
}

#[test]
fn test_legacy_markdown_annotations_duplication_high() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("dup.rs", NodeType::File);
    node.annotations.duplication_score = Some(0.5);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("50%"));
}

#[test]
fn test_legacy_markdown_annotations_duplication_medium() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("dup2.rs", NodeType::File);
    node.annotations.duplication_score = Some(0.15);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("15%"));
}

#[test]
fn test_legacy_markdown_annotations_duplication_low_not_shown() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("clean.rs", NodeType::File);
    node.annotations.duplication_score = Some(0.05);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    // Duplication <= 0.1 should not produce an indicator
    assert!(!md.contains("5%") || !md.contains("["));
}

#[test]
fn test_legacy_markdown_annotations_all_combined() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut node = make_annotated_node("all.rs", NodeType::File);
    node.annotations.defect_score = Some(0.8);
    node.annotations.cognitive_complexity = Some(40);
    node.annotations.satd_items = 2;
    node.annotations.dead_code_items = 4;
    node.annotations.test_coverage = Some(0.4);
    node.annotations.big_o_complexity = Some("O(n)".to_string());
    node.annotations.churn_score = Some(0.9);
    node.annotations.memory_complexity = Some("O(1)".to_string());
    node.annotations.duplication_score = Some(0.35);
    ctx.file_tree.root = node;
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("0.8"));
    assert!(md.contains("40"));
    assert!(md.contains("2"));
    assert!(md.contains("4"));
    assert!(md.contains("40%"));
    assert!(md.contains("O(n)"));
    assert!(md.contains("0.9"));
    assert!(md.contains("O(1)"));
    assert!(md.contains("35%"));
}

// ===========================================================================
// format_as_comprehensive_markdown_legacy -- analysis sections
// ===========================================================================

#[test]
fn test_legacy_markdown_complexity_hotspots() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.complexity_report = Some(make_complexity_report());
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("## Complexity Hotspots"));
    assert!(md.contains("complex_function"));
    assert!(md.contains("src/main.rs"));
    assert!(md.contains("| Function | File | Cyclomatic | Cognitive |"));
}

#[test]
fn test_legacy_markdown_churn_analysis() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.churn_analysis = Some(make_churn_analysis());
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("## Code Churn Analysis"));
    assert!(md.contains("Total Commits: 100"));
    assert!(md.contains("Files Changed: 1"));
    assert!(md.contains("src/lib.rs"));
}

#[test]
fn test_legacy_markdown_technical_debt() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.satd_results = Some(make_satd_result());
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("## Code Quality Analysis"));
    assert!(md.contains("SATD Summary"));
    // Critical item should appear
    assert!(md.contains("FIXME: critical bug here"));
}

#[test]
fn test_legacy_markdown_dead_code_analysis() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.dead_code_results = Some(make_dead_code_result());
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("## Dead Code Analysis"));
    assert!(md.contains("Dead Functions: 8"));
    assert!(md.contains("Total Dead Lines: 150"));
    assert!(md.contains("src/old_module.rs"));
}

#[test]
fn test_legacy_markdown_cross_references() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.cross_language_refs =
        vec![make_cross_lang_ref("binding.rs", "lib.py", 0.85)];
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("## Cross-Language References"));
    assert!(md.contains("binding.rs"));
    assert!(md.contains("lib.py"));
    assert!(md.contains("85.0%"));
}

#[test]
fn test_legacy_markdown_no_cross_references_when_empty() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(!md.contains("## Cross-Language References"));
}

#[test]
fn test_legacy_markdown_defect_predictions() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.defect_summary = DefectSummary {
        total_defects: 12,
        defect_density: 3.5,
        ..Default::default()
    };
    ctx.hotspots = vec![make_defect_hotspot("src/risky.rs", 50, 0.85, 6.0)];
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("## Defect Probability Analysis"));
    assert!(md.contains("Total Defects Predicted: 12"));
    assert!(md.contains("3.50 defects per 1000 lines"));
    assert!(md.contains("src/risky.rs:50"));
    // {:.1} rounds 0.85 to "0.9"
    assert!(md.contains("0.9"));
}

#[test]
fn test_legacy_markdown_recommendations_section() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.recommendations = vec![
        make_recommendation("Fix critical bug", Priority::Critical, Impact::High, vec!["Hotfix"]),
        make_recommendation("Update deps", Priority::Low, Impact::Low, vec![]),
    ];
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("## Prioritized Recommendations"));
    assert!(md.contains("Fix critical bug"));
    assert!(md.contains("Update deps"));
    assert!(md.contains("Hotfix"));
}

#[test]
fn test_legacy_markdown_no_recommendations_when_empty() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(!md.contains("## Prioritized Recommendations"));
}

// ===========================================================================
// format_as_json
// ===========================================================================

#[test]
fn test_json_empty_context() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let json_str = analyzer.format_as_json(&ctx).unwrap();
    assert!(json_str.starts_with('{'));
    assert!(json_str.contains("\"metadata\""));
    assert!(json_str.contains("\"quality_scorecard\""));
}

#[test]
fn test_json_populated_context() {
    let analyzer = make_analyzer();
    let ctx = make_populated_context();
    let json_str = analyzer.format_as_json(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.get("metadata").is_some());
    assert!(parsed.get("quality_scorecard").is_some());
    assert!(parsed.get("recommendations").is_some());
    let recs = parsed["recommendations"].as_array().unwrap();
    assert_eq!(recs.len(), 2);
}

#[test]
fn test_json_round_trip_preserves_scorecard() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.quality_scorecard = make_scorecard(88.8, 77.7, 6.6);
    let json_str = analyzer.format_as_json(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let health = parsed["quality_scorecard"]["overall_health"]
        .as_f64()
        .unwrap();
    assert!((health - 88.8).abs() < 0.1);
}

#[test]
fn test_json_with_analyses() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.complexity_report = Some(make_complexity_report());
    ctx.analyses.ast_contexts = vec![make_enhanced_file_context("test.rs", "Rust")];
    let json_str = analyzer.format_as_json(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed["analyses"]["complexity_report"].is_object());
    assert!(parsed["analyses"]["ast_contexts"].is_array());
}

// ===========================================================================
// format_as_sarif
// ===========================================================================

#[test]
fn test_sarif_empty_context() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    assert_eq!(parsed["version"], "2.1.0");
    let runs = parsed["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1);
    let driver = &runs[0]["tool"]["driver"];
    assert_eq!(driver["name"], "pmat");
    // No results when context is empty
    let results = runs[0]["results"].as_array().unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_sarif_with_complexity_violations() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    // Create a function with cyclomatic > 10 (triggers complexity warning)
    let report = ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: Vec::new(),
        hotspots: Vec::new(),
        files: vec![FileComplexityMetrics {
            path: "src/main.rs".to_string(),
            total_complexity: ComplexityMetrics::default(),
            functions: vec![
                FunctionComplexity {
                    name: "high_cyclomatic".to_string(),
                    line_start: 1,
                    line_end: 50,
                    metrics: ComplexityMetrics::new(15, 8, 3, 50),
                },
                FunctionComplexity {
                    name: "very_high_cyclomatic".to_string(),
                    line_start: 55,
                    line_end: 150,
                    metrics: ComplexityMetrics::new(25, 30, 5, 100),
                },
                FunctionComplexity {
                    name: "normal".to_string(),
                    line_start: 155,
                    line_end: 170,
                    metrics: ComplexityMetrics::new(3, 2, 1, 15),
                },
            ],
            classes: Vec::new(),
        }],
    };
    ctx.analyses.complexity_report = Some(report);
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let runs = parsed["runs"].as_array().unwrap();
    let results = runs[0]["results"].as_array().unwrap();
    // high_cyclomatic (15 > 10) -> 1 cyclomatic result
    // very_high_cyclomatic (25 > 10 AND 30 > 15) -> 2 results (cyclomatic + cognitive)
    // normal (3 <= 10) -> 0 results
    assert_eq!(results.len(), 3);
    // Check rule IDs
    let rule_ids: Vec<&str> = results
        .iter()
        .map(|r| r["ruleId"].as_str().unwrap())
        .collect();
    assert!(rule_ids.contains(&"complexity/high-cyclomatic"));
    assert!(rule_ids.contains(&"complexity/high-cognitive"));
}

#[test]
fn test_sarif_complexity_level_warning_vs_error() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.complexity_report = Some(ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: Vec::new(),
        hotspots: Vec::new(),
        files: vec![FileComplexityMetrics {
            path: "test.rs".to_string(),
            total_complexity: ComplexityMetrics::default(),
            functions: vec![
                // Cyclomatic 15 -> "warning" (between 10 and 20)
                FunctionComplexity {
                    name: "warning_func".to_string(),
                    line_start: 1,
                    line_end: 20,
                    metrics: ComplexityMetrics::new(15, 5, 2, 20),
                },
                // Cyclomatic 25 -> "error" (above 20)
                FunctionComplexity {
                    name: "error_func".to_string(),
                    line_start: 25,
                    line_end: 100,
                    metrics: ComplexityMetrics::new(25, 5, 4, 80),
                },
            ],
            classes: Vec::new(),
        }],
    });
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let results = parsed["runs"][0]["results"].as_array().unwrap();
    let warning_result = results
        .iter()
        .find(|r| {
            r["message"]["text"]
                .as_str()
                .unwrap()
                .contains("warning_func")
        })
        .unwrap();
    assert_eq!(warning_result["level"], "warning");
    let error_result = results
        .iter()
        .find(|r| {
            r["message"]["text"]
                .as_str()
                .unwrap()
                .contains("error_func")
        })
        .unwrap();
    assert_eq!(error_result["level"], "error");
}

#[test]
fn test_sarif_with_satd_items() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.satd_results = Some(make_satd_result());
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let runs = parsed["runs"].as_array().unwrap();
    let results = runs[0]["results"].as_array().unwrap();
    // 4 SATD items
    assert_eq!(results.len(), 4);
    // Check that the debt rule was registered
    let rules = runs[0]["tool"]["driver"]["rules"].as_array().unwrap();
    let rule_ids: Vec<&str> = rules.iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert!(rule_ids.contains(&"debt/technical-debt"));
}

#[test]
fn test_sarif_satd_severity_levels() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.satd_results = Some(make_satd_result());
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let results = parsed["runs"][0]["results"].as_array().unwrap();
    let levels: Vec<&str> = results.iter().map(|r| r["level"].as_str().unwrap()).collect();
    // Critical -> "error", High -> "warning", Medium -> "note", Low -> "note"
    assert!(levels.contains(&"error"));
    assert!(levels.contains(&"warning"));
    assert!(levels.contains(&"note"));
}

#[test]
fn test_sarif_with_dead_code() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.dead_code_results = Some(make_dead_code_result());
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let results = parsed["runs"][0]["results"].as_array().unwrap();
    // Only files with dead_functions > 0 should appear: old_module.rs (5), legacy.rs (3)
    // clean.rs has 0 dead_functions, so it's filtered out
    let dead_code_results: Vec<_> = results
        .iter()
        .filter(|r| r["ruleId"] == "dead-code/unused-code")
        .collect();
    assert_eq!(dead_code_results.len(), 2);
}

#[test]
fn test_sarif_properties_section() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.metadata.analysis_duration = Duration::from_secs(5);
    ctx.metadata.cache_stats.hit_rate = 0.75;
    ctx.quality_scorecard.overall_health = 82.0;
    ctx.quality_scorecard.technical_debt_hours = 10.5;
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let props = &parsed["runs"][0]["properties"];
    assert!((props["analysis_duration_seconds"].as_f64().unwrap() - 5.0).abs() < 0.1);
    assert!((props["cache_hit_rate"].as_f64().unwrap() - 0.75).abs() < 0.01);
    assert!((props["overall_health_score"].as_f64().unwrap() - 82.0).abs() < 0.1);
    assert!((props["technical_debt_hours"].as_f64().unwrap() - 10.5).abs() < 0.1);
}

#[test]
fn test_sarif_combined_all_analyses() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.complexity_report = Some(make_complexity_report());
    ctx.analyses.satd_results = Some(make_satd_result());
    ctx.analyses.dead_code_results = Some(make_dead_code_result());
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let runs = parsed["runs"].as_array().unwrap();
    let rules = runs[0]["tool"]["driver"]["rules"].as_array().unwrap();
    let results = runs[0]["results"].as_array().unwrap();
    // Rules: 2 complexity + 1 SATD + 1 dead-code = 4
    assert_eq!(rules.len(), 4);
    // Results: 2 complexity + 4 SATD + 2 dead code = 8
    assert_eq!(results.len(), 8);
}

#[test]
fn test_sarif_tool_version_matches_metadata() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.metadata.tool_version = "2.0.0-test".to_string();
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let version = parsed["runs"][0]["tool"]["driver"]["version"]
        .as_str()
        .unwrap();
    assert_eq!(version, "2.0.0-test");
}

// ===========================================================================
// format_enhanced_ast_section (public helper)
// ===========================================================================

#[test]
fn test_enhanced_ast_section_empty() {
    let analyzer = make_analyzer();
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[])
        .unwrap();
    assert!(output.contains("## Enhanced AST Analysis"));
}

#[test]
fn test_enhanced_ast_section_with_functions() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/lib.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![
                AstItem::Function {
                    name: "my_func".to_string(),
                    visibility: "pub".to_string(),
                    is_async: true,
                    line: 10,
                },
                AstItem::Function {
                    name: "private_func".to_string(),
                    visibility: "priv".to_string(),
                    is_async: false,
                    line: 20,
                },
            ],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_lib".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("### src/lib.rs"));
    assert!(output.contains("**Language:** Rust"));
    assert!(output.contains("Total Symbols:** 2"));
    assert!(output.contains("`my_func (async)`"));
    assert!(output.contains("`private_func`"));
}

#[test]
fn test_enhanced_ast_section_with_structs_and_enums() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/types.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![
                AstItem::Struct {
                    name: "MyStruct".to_string(),
                    visibility: "pub".to_string(),
                    fields_count: 3,
                    derives: vec!["Debug".to_string(), "Clone".to_string()],
                    line: 5,
                },
                AstItem::Enum {
                    name: "MyEnum".to_string(),
                    visibility: "pub".to_string(),
                    variants_count: 4,
                    line: 15,
                },
            ],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_types".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("MyStruct"));
    assert!(output.contains("3 fields"));
    assert!(output.contains("Debug, Clone"));
    assert!(output.contains("MyEnum"));
    assert!(output.contains("4 variants"));
}

#[test]
fn test_enhanced_ast_section_with_traits_and_impls() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/trait.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![
                AstItem::Trait {
                    name: "Processor".to_string(),
                    visibility: "pub".to_string(),
                    line: 1,
                },
                AstItem::Impl {
                    type_name: "MyStruct".to_string(),
                    trait_name: Some("Processor".to_string()),
                    line: 10,
                },
                AstItem::Impl {
                    type_name: "MyStruct".to_string(),
                    trait_name: None,
                    line: 20,
                },
            ],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_trait".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("Processor"));
    assert!(output.contains("Processor for MyStruct"));
    assert!(output.contains("impl MyStruct"));
}

#[test]
fn test_enhanced_ast_section_with_modules_and_imports() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/mod.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![
                AstItem::Module {
                    name: "submod".to_string(),
                    visibility: "pub".to_string(),
                    line: 1,
                },
                AstItem::Use {
                    path: "std::collections::HashMap".to_string(),
                    line: 3,
                },
                AstItem::Import {
                    module: "os".to_string(),
                    items: vec!["path".to_string(), "env".to_string()],
                    alias: None,
                    line: 5,
                },
                AstItem::Import {
                    module: "numpy".to_string(),
                    items: Vec::new(),
                    alias: Some("np".to_string()),
                    line: 6,
                },
            ],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_mod".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("submod"));
    assert!(output.contains("std::collections::HashMap"));
    // Import with items: "os (path, env)"
    assert!(output.contains("os (path, env)"));
    // Import with alias: "numpy as np"
    assert!(output.contains("numpy as np"));
}

#[test]
fn test_enhanced_ast_section_single_field_struct() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/single.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![AstItem::Struct {
                name: "Wrapper".to_string(),
                visibility: "pub".to_string(),
                fields_count: 1,
                derives: Vec::new(),
                line: 1,
            }],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_single".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    // Single field should not have an 's' suffix
    assert!(output.contains("1 field "));
    assert!(!output.contains("1 fields"));
}

#[test]
fn test_enhanced_ast_section_single_variant_enum() {
    let analyzer = make_analyzer();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/single_enum.rs".to_string(),
            language: "Rust".to_string(),
            items: vec![AstItem::Enum {
                name: "Single".to_string(),
                visibility: "pub".to_string(),
                variants_count: 1,
                line: 1,
            }],
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_single_enum".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("1 variant "));
    assert!(!output.contains("1 variants"));
}

// ===========================================================================
// Edge case: more than 10 functions triggers truncation message
// ===========================================================================

#[test]
fn test_enhanced_ast_section_truncated_functions() {
    let analyzer = make_analyzer();
    let items: Vec<AstItem> = (0..15)
        .map(|i| AstItem::Function {
            name: format!("func_{i}"),
            visibility: "pub".to_string(),
            is_async: false,
            line: i * 10,
        })
        .collect();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/many.rs".to_string(),
            language: "Rust".to_string(),
            items,
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_many".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("... and 5 more functions"));
}

// ===========================================================================
// Edge case: more than 8 imports triggers compact mode
// ===========================================================================

#[test]
fn test_enhanced_ast_section_many_imports_compact() {
    let analyzer = make_analyzer();
    let items: Vec<AstItem> = (0..12)
        .map(|i| AstItem::Use {
            path: format!("crate::module_{i}"),
            line: i,
        })
        .collect();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/imports.rs".to_string(),
            language: "Rust".to_string(),
            items,
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_imports".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    // With > 8 imports, should show compact form
    assert!(output.contains("**Imports:** 12 import statements"));
}

// ===========================================================================
// Edge case: more than 5 structs triggers truncation
// ===========================================================================

#[test]
fn test_enhanced_ast_section_truncated_structs() {
    let analyzer = make_analyzer();
    let items: Vec<AstItem> = (0..8)
        .map(|i| AstItem::Struct {
            name: format!("Struct_{i}"),
            visibility: "pub".to_string(),
            fields_count: i + 1,
            derives: Vec::new(),
            line: i * 5,
        })
        .collect();
    let efc = EnhancedFileContext {
        base: FileContext {
            path: "src/structs.rs".to_string(),
            language: "Rust".to_string(),
            items,
            complexity_metrics: None,
        },
        complexity_metrics: None,
        churn_metrics: None,
        defects: DefectAnnotations {
            dead_code: None,
            technical_debt: Vec::new(),
            complexity_violations: Vec::new(),
            tdg_score: None,
        },
        symbol_id: "sym_structs".to_string(),
    };
    let mut output = String::new();
    analyzer
        .format_enhanced_ast_section(&mut output, &[efc])
        .unwrap();
    assert!(output.contains("... and 3 more structs"));
}

// ===========================================================================
// Legacy markdown: file metrics (complexity, churn, TDG)
// ===========================================================================

#[test]
fn test_legacy_markdown_file_complexity_metrics() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut efc = make_enhanced_file_context("src/complex.rs", "Rust");
    efc.complexity_metrics = Some(FileComplexityMetrics {
        path: "src/complex.rs".to_string(),
        total_complexity: ComplexityMetrics::new(12, 15, 3, 80),
        functions: Vec::new(),
        classes: Vec::new(),
    });
    ctx.analyses.ast_contexts = vec![efc];
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("Complexity Metrics"));
    // u16 with {:.1} format renders without decimal point
    assert!(md.contains("Cyclomatic: 12"));
    assert!(md.contains("Cognitive: 15"));
    assert!(md.contains("Lines: 80"));
}

#[test]
fn test_legacy_markdown_file_churn_metrics() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    let mut efc = make_enhanced_file_context("src/churny.rs", "Rust");
    efc.churn_metrics = Some(crate::services::deep_context::FileChurnMetrics {
        commits: 55,
        authors: 3,
        lines_added: 200,
        lines_deleted: 80,
        last_modified: Utc::now(),
    });
    ctx.analyses.ast_contexts = vec![efc];
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("Code Churn"));
    assert!(md.contains("55 commits by 3 authors"));
}

// ===========================================================================
// Memory complexity indicator variations
// ===========================================================================

#[test]
fn test_memory_complexity_all_variants() {
    let analyzer = make_analyzer();
    let variants = vec![
        ("O(1)", true),
        ("O(log n)", true),
        ("O(n)", true),
        ("O(n log n)", true),
        ("O(n\u{00b2})", true),
        ("O(2\u{207f})", true), // unknown variant
    ];
    for (mem, _) in variants {
        let mut ctx = make_empty_context();
        let mut node = make_annotated_node("test.rs", NodeType::File);
        node.annotations.memory_complexity = Some(mem.to_string());
        ctx.file_tree.root = node;
        let md = analyzer
            .format_as_comprehensive_markdown_legacy(&ctx)
            .unwrap();
        assert!(md.contains(mem), "Memory complexity '{mem}' not found in output");
    }
}

// ===========================================================================
// Big-O emoji variations tested indirectly
// ===========================================================================

#[test]
fn test_big_o_all_variants_in_tree() {
    let analyzer = make_analyzer();
    let variants = vec![
        "O(1)",
        "O(log n)",
        "O(n)",
        "O(n log n)",
        "O(n\u{00b2})",
        "O(n\u{00b3})",
        "O(2\u{207f})",
        "O(n!)",
        "O(unknown)",
    ];
    for big_o in variants {
        let mut ctx = make_empty_context();
        let mut node = make_annotated_node("test.rs", NodeType::File);
        node.annotations.big_o_complexity = Some(big_o.to_string());
        ctx.file_tree.root = node;
        let md = analyzer
            .format_as_comprehensive_markdown_legacy(&ctx)
            .unwrap();
        assert!(md.contains(big_o), "Big-O '{big_o}' not found in output");
    }
}

// ===========================================================================
// SARIF location structure verified indirectly via complexity results
// ===========================================================================

#[test]
fn test_sarif_location_structure_via_complexity() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.complexity_report = Some(ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: Vec::new(),
        hotspots: Vec::new(),
        files: vec![FileComplexityMetrics {
            path: "src/main.rs".to_string(),
            total_complexity: ComplexityMetrics::default(),
            functions: vec![FunctionComplexity {
                name: "target_func".to_string(),
                line_start: 10,
                line_end: 20,
                metrics: ComplexityMetrics::new(15, 5, 2, 15),
            }],
            classes: Vec::new(),
        }],
    });
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let results = parsed["runs"][0]["results"].as_array().unwrap();
    assert!(!results.is_empty());
    let location = &results[0]["locations"][0]["physicalLocation"];
    assert_eq!(location["artifactLocation"]["uri"], "src/main.rs");
    assert_eq!(location["region"]["startLine"], 10);
    assert_eq!(location["region"]["endLine"], 20);
    assert_eq!(location["region"]["startColumn"], 1);
}

// ===========================================================================
// Edge case: deeply nested tree
// ===========================================================================

#[test]
fn test_legacy_markdown_deeply_nested_tree() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();

    let leaf = make_annotated_node("deep.rs", NodeType::File);
    let mut l3 = make_annotated_node("l3", NodeType::Directory);
    l3.children.push(leaf);
    let mut l2 = make_annotated_node("l2", NodeType::Directory);
    l2.children.push(l3);
    let mut l1 = make_annotated_node("l1", NodeType::Directory);
    l1.children.push(l2);
    let mut root = make_annotated_node("root", NodeType::Directory);
    root.children.push(l1);

    ctx.file_tree = AnnotatedFileTree {
        root,
        total_files: 1,
        total_size_bytes: 256,
    };
    let md = analyzer
        .format_as_comprehensive_markdown_legacy(&ctx)
        .unwrap();
    assert!(md.contains("root/"));
    assert!(md.contains("l1/"));
    assert!(md.contains("l2/"));
    assert!(md.contains("l3/"));
    assert!(md.contains("deep.rs"));
}

// ===========================================================================
// Edge case: SARIF cognitive > 25 produces error level
// ===========================================================================

#[test]
fn test_sarif_cognitive_error_level() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.complexity_report = Some(ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: Vec::new(),
        hotspots: Vec::new(),
        files: vec![FileComplexityMetrics {
            path: "hard.rs".to_string(),
            total_complexity: ComplexityMetrics::default(),
            functions: vec![FunctionComplexity {
                name: "brain_melter".to_string(),
                line_start: 1,
                line_end: 200,
                // cyclomatic <= 10 so no cyclomatic violation, but cognitive > 25 -> error
                metrics: ComplexityMetrics::new(8, 30, 6, 200),
            }],
            classes: Vec::new(),
        }],
    });
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let results = parsed["runs"][0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["level"], "error");
    assert_eq!(results[0]["ruleId"], "complexity/high-cognitive");
}

// ===========================================================================
// Edge case: SARIF cognitive warning (16..=25)
// ===========================================================================

#[test]
fn test_sarif_cognitive_warning_level() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.complexity_report = Some(ComplexityReport {
        summary: ComplexitySummary::default(),
        violations: Vec::new(),
        hotspots: Vec::new(),
        files: vec![FileComplexityMetrics {
            path: "medium.rs".to_string(),
            total_complexity: ComplexityMetrics::default(),
            functions: vec![FunctionComplexity {
                name: "tricky".to_string(),
                line_start: 1,
                line_end: 80,
                metrics: ComplexityMetrics::new(5, 20, 3, 80),
            }],
            classes: Vec::new(),
        }],
    });
    let sarif_str = analyzer.format_as_sarif(&ctx).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&sarif_str).unwrap();
    let results = parsed["runs"][0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["level"], "warning");
}
