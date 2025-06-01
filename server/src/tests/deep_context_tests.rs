use crate::services::deep_context::*;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio::fs;

#[tokio::test]
async fn test_deep_context_config_default_values() {
    let config = DeepContextConfig::default();
    
    assert_eq!(config.period_days, 30);
    assert!(config.include_analyses.contains(&AnalysisType::Ast));
    assert!(config.include_analyses.contains(&AnalysisType::Complexity));
    assert!(config.include_analyses.contains(&AnalysisType::Churn));
    assert!(matches!(config.dag_type, DagType::CallGraph));
    assert!(matches!(config.cache_strategy, CacheStrategy::Normal));
    assert_eq!(config.max_depth, Some(10));
    assert!(config.exclude_patterns.contains(&"**/node_modules/**".to_string()));
    assert!(config.exclude_patterns.contains(&"**/target/**".to_string()));
}

#[tokio::test]
async fn test_deep_context_analyzer_creation() {
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config.clone());
    
    // Test that analyzer is created successfully (private fields can't be accessed directly)
    assert!(std::mem::size_of_val(&analyzer) > 0);
}

// Note: should_exclude_path is private method, so we test it indirectly through discovery

#[tokio::test]
async fn test_discovery_simple_project() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Create a simple project structure
    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).await.unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").await.unwrap();
    fs::write(project_path.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"").await.unwrap();
    
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);
    
    let file_tree = analyzer.discover_project_structure(&project_path).await.unwrap();
    
    assert!(file_tree.total_files >= 2); // main.rs and Cargo.toml
    assert!(file_tree.total_size_bytes > 0);
    assert_eq!(file_tree.root.name, project_path.file_name().unwrap().to_string_lossy());
    assert!(matches!(file_tree.root.node_type, NodeType::Directory));
}

#[tokio::test]
async fn test_discovery_with_excludes() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    
    // Create project with excluded directories
    let target_dir = project_path.join("target");
    fs::create_dir(&target_dir).await.unwrap();
    fs::write(target_dir.join("binary"), "binary content").await.unwrap();
    
    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).await.unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").await.unwrap();
    
    let config = DeepContextConfig::default();
    let analyzer = DeepContextAnalyzer::new(config);
    
    let file_tree = analyzer.discover_project_structure(&project_path).await.unwrap();
    
    // Should find src/main.rs but not target/binary
    let has_target = file_tree.root.children.iter().any(|child| child.name == "target");
    let has_src = file_tree.root.children.iter().any(|child| child.name == "src");
    
    assert!(!has_target); // target should be excluded
    assert!(has_src);     // src should be included
}

#[tokio::test]
async fn test_metadata_creation() {
    let metadata = ContextMetadata {
        generated_at: Utc::now(),
        tool_version: "0.18.3".to_string(),
        project_root: PathBuf::from("/test/project"),
        cache_stats: CacheStats {
            hit_rate: 0.85,
            memory_efficiency: 0.72,
            time_saved_ms: 1500,
        },
        analysis_duration: Duration::from_millis(2500),
    };
    
    assert_eq!(metadata.tool_version, "0.18.3");
    assert_eq!(metadata.project_root, PathBuf::from("/test/project"));
    assert_eq!(metadata.cache_stats.hit_rate, 0.85);
    assert_eq!(metadata.cache_stats.memory_efficiency, 0.72);
    assert_eq!(metadata.cache_stats.time_saved_ms, 1500);
    assert_eq!(metadata.analysis_duration, Duration::from_millis(2500));
}

#[tokio::test]
async fn test_quality_scorecard_calculations() {
    let scorecard = QualityScorecard {
        overall_health: 85.5,
        complexity_score: 78.0,
        maintainability_index: 92.0,
        modularity_score: 88.5,
        test_coverage: Some(75.0),
        technical_debt_hours: 12.5,
    };
    
    assert_eq!(scorecard.overall_health, 85.5);
    assert_eq!(scorecard.complexity_score, 78.0);
    assert_eq!(scorecard.maintainability_index, 92.0);
    assert_eq!(scorecard.modularity_score, 88.5);
    assert_eq!(scorecard.test_coverage, Some(75.0));
    assert_eq!(scorecard.technical_debt_hours, 12.5);
}

#[tokio::test]
async fn test_defect_summary_aggregation() {
    let mut by_severity = HashMap::new();
    by_severity.insert("high".to_string(), 5);
    by_severity.insert("medium".to_string(), 10);
    by_severity.insert("low".to_string(), 15);
    
    let mut by_type = HashMap::new();
    by_type.insert("dead_code".to_string(), 8);
    by_type.insert("technical_debt".to_string(), 12);
    by_type.insert("complexity".to_string(), 10);
    
    let defect_summary = DefectSummary {
        total_defects: 30,
        by_severity,
        by_type,
        defect_density: 0.25,
    };
    
    assert_eq!(defect_summary.total_defects, 30);
    assert_eq!(defect_summary.defect_density, 0.25);
    assert_eq!(defect_summary.by_severity.get("high"), Some(&5));
    assert_eq!(defect_summary.by_severity.get("medium"), Some(&10));
    assert_eq!(defect_summary.by_severity.get("low"), Some(&15));
    assert_eq!(defect_summary.by_type.get("dead_code"), Some(&8));
    assert_eq!(defect_summary.by_type.get("technical_debt"), Some(&12));
    assert_eq!(defect_summary.by_type.get("complexity"), Some(&10));
}

#[tokio::test]
async fn test_defect_hotspot_ranking() {
    let hotspot = DefectHotspot {
        location: FileLocation {
            file: PathBuf::from("/src/complex.rs"),
            line: 42,
            column: 10,
        },
        composite_score: 0.85,
        contributing_factors: vec![
            DefectFactor::Complexity {
                cyclomatic: 15,
                cognitive: 12,
                violations: vec!["Exceeds threshold".to_string()],
            },
            DefectFactor::TechnicalDebt {
                category: TechnicalDebtCategory::Implementation,
                severity: TechnicalDebtSeverity::High,
                age_days: 60,
            },
        ],
        refactoring_effort: RefactoringEstimate {
            estimated_hours: 8.0,
            priority: Priority::High,
            impact: Impact::High,
            suggested_actions: vec![
                "Extract method".to_string(),
                "Reduce complexity".to_string(),
            ],
        },
    };
    
    assert_eq!(hotspot.location.line, 42);
    assert_eq!(hotspot.location.column, 10);
    assert_eq!(hotspot.composite_score, 0.85);
    assert_eq!(hotspot.contributing_factors.len(), 2);
    assert_eq!(hotspot.refactoring_effort.estimated_hours, 8.0);
    assert!(matches!(hotspot.refactoring_effort.priority, Priority::High));
    assert!(matches!(hotspot.refactoring_effort.impact, Impact::High));
    assert_eq!(hotspot.refactoring_effort.suggested_actions.len(), 2);
}

#[tokio::test]
async fn test_prioritized_recommendations() {
    let recommendation = PrioritizedRecommendation {
        title: "Reduce Cyclomatic Complexity".to_string(),
        description: "Functions exceed complexity thresholds and should be refactored".to_string(),
        priority: Priority::Critical,
        estimated_effort: Duration::from_secs(6 * 3600), // 6 hours
        impact: Impact::High,
        prerequisites: vec![
            "Identify complex functions".to_string(),
            "Write unit tests".to_string(),
        ],
    };
    
    assert_eq!(recommendation.title, "Reduce Cyclomatic Complexity");
    assert!(matches!(recommendation.priority, Priority::Critical));
    assert_eq!(recommendation.estimated_effort, Duration::from_secs(6 * 3600));
    assert!(matches!(recommendation.impact, Impact::High));
    assert_eq!(recommendation.prerequisites.len(), 2);
}

#[tokio::test]
async fn test_cross_language_references() {
    let cross_ref = CrossLangReference {
        source_file: PathBuf::from("/src/rust_module.rs"),
        target_file: PathBuf::from("/bindings/python_wrapper.py"),
        reference_type: CrossLangReferenceType::PythonBinding,
        confidence: 0.92,
    };
    
    assert_eq!(cross_ref.source_file, PathBuf::from("/src/rust_module.rs"));
    assert_eq!(cross_ref.target_file, PathBuf::from("/bindings/python_wrapper.py"));
    assert!(matches!(cross_ref.reference_type, CrossLangReferenceType::PythonBinding));
    assert_eq!(cross_ref.confidence, 0.92);
}

#[tokio::test]
async fn test_template_provenance_tracking() {
    use serde_json::json;
    
    let mut parameters = HashMap::new();
    parameters.insert("project_name".to_string(), json!("test-project"));
    parameters.insert("language".to_string(), json!("rust"));
    
    let provenance = TemplateProvenance {
        scaffold_timestamp: Utc::now(),
        templates_used: vec![
            "rust/cargo.toml".to_string(),
            "rust/main.rs".to_string(),
            "gitignore/rust".to_string(),
        ],
        parameters,
        drift_analysis: DriftAnalysis {
            added_files: vec![
                PathBuf::from("/src/new_module.rs"),
                PathBuf::from("/tests/integration.rs"),
            ],
            modified_files: vec![PathBuf::from("/Cargo.toml")],
            deleted_files: vec![],
            drift_score: 0.15,
        },
    };
    
    assert_eq!(provenance.templates_used.len(), 3);
    assert!(provenance.templates_used.contains(&"rust/cargo.toml".to_string()));
    assert_eq!(provenance.parameters.len(), 2);
    assert_eq!(provenance.parameters.get("project_name").unwrap(), &json!("test-project"));
    assert_eq!(provenance.drift_analysis.added_files.len(), 2);
    assert_eq!(provenance.drift_analysis.modified_files.len(), 1);
    assert_eq!(provenance.drift_analysis.deleted_files.len(), 0);
    assert_eq!(provenance.drift_analysis.drift_score, 0.15);
}

#[tokio::test]
async fn test_analysis_type_equality() {
    assert_eq!(AnalysisType::Ast, AnalysisType::Ast);
    assert_eq!(AnalysisType::Complexity, AnalysisType::Complexity);
    assert_eq!(AnalysisType::Churn, AnalysisType::Churn);
    assert_eq!(AnalysisType::Dag, AnalysisType::Dag);
    assert_eq!(AnalysisType::DeadCode, AnalysisType::DeadCode);
    assert_eq!(AnalysisType::Satd, AnalysisType::Satd);
    assert_eq!(AnalysisType::DefectProbability, AnalysisType::DefectProbability);
    
    assert_ne!(AnalysisType::Ast, AnalysisType::Complexity);
}

#[tokio::test]
async fn test_enum_variants_complete() {
    // Test all enum variants exist and can be matched
    match DagType::CallGraph {
        DagType::CallGraph => (),
        DagType::ImportGraph => (),
        DagType::Inheritance => (),
        DagType::FullDependency => (),
    }
    
    match CacheStrategy::Normal {
        CacheStrategy::Normal => (),
        CacheStrategy::ForceRefresh => (),
        CacheStrategy::Offline => (),
    }
    
    match ConfidenceLevel::High {
        ConfidenceLevel::High => (),
        ConfidenceLevel::Medium => (),
        ConfidenceLevel::Low => (),
    }
}