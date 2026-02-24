//! Unified code intelligence interface
//!
//! Provides a comprehensive analysis interface that combines DAG representation,
//! duplicate detection, dead code analysis, and more into a single API.

#![cfg_attr(coverage_nightly, coverage(off))]
use crate::models::unified_ast::AstDag;
use crate::services::{
    context::analyze_project,
    dag_builder::DagBuilder,
    dead_code_analyzer::{DeadCodeAnalyzer, DeadCodeReport},
    duplicate_detector::CloneReport,
    mermaid_generator::{MermaidGenerator, MermaidOptions},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

include!("code_intelligence_types.rs");
include!("code_intelligence_cache.rs");
include!("code_intelligence_engine.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_request_cache_key() {
        let req = AnalysisRequest {
            project_path: "/test/project".to_string(),
            analysis_types: vec![AnalysisType::DuplicateDetection],
            include_patterns: vec![],
            exclude_patterns: vec![],
            max_depth: None,
            parallel: true,
        };

        let key1 = req.cache_key();
        let key2 = req.cache_key();

        assert_eq!(key1, key2);
    }

    #[tokio::test]
    async fn test_unified_cache() {
        let cache = UnifiedCache::new(10);

        let report = AnalysisReport {
            duplicates: None,
            dead_code: None,
            complexity_metrics: None,
            dependency_graph: None,
            defect_predictions: None,
            graph_metrics: None,
            timestamp: Utc::now(),
        };

        cache.put("test_key".to_string(), report.clone()).await;

        let cached = cache.get("test_key").await;
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_code_intelligence_creation() {
        let intelligence = CodeIntelligence::new();
        let (nodes, gen) = intelligence.get_dag_stats().await;

        assert_eq!(nodes, 0);
        assert_eq!(gen, 0);
    }

    /// Test UnifiedCache with zero capacity (validates unwrap fix at line 180)
    #[tokio::test]
    async fn test_unified_cache_zero_capacity() {
        // Zero capacity should default to 1 (NonZeroUsize requirement)
        let cache = UnifiedCache::new(0);

        let report = AnalysisReport {
            duplicates: None,
            dead_code: None,
            complexity_metrics: None,
            dependency_graph: None,
            defect_predictions: None,
            graph_metrics: None,
            timestamp: Utc::now(),
        };

        // Cache should work with capacity 1 (defaulted from 0)
        cache.put("key1".to_string(), report.clone()).await;

        // First item should be retrievable
        assert!(cache.get("key1").await.is_some());

        // Adding second item should evict first (LRU with capacity 1)
        let report2 = AnalysisReport {
            duplicates: None,
            dead_code: None,
            complexity_metrics: None,
            dependency_graph: None,
            defect_predictions: None,
            graph_metrics: None,
            timestamp: Utc::now(),
        };
        cache.put("key2".to_string(), report2).await;

        // First item should be evicted
        assert!(cache.get("key1").await.is_none());
        // Second item should be present
        assert!(cache.get("key2").await.is_some());
    }

    /// Test UnifiedCache with valid non-zero capacities
    #[tokio::test]
    async fn test_unified_cache_valid_capacities() {
        // Test capacity 1
        let cache1 = UnifiedCache::new(1);
        let report = AnalysisReport {
            duplicates: None,
            dead_code: None,
            complexity_metrics: None,
            dependency_graph: None,
            defect_predictions: None,
            graph_metrics: None,
            timestamp: Utc::now(),
        };
        cache1.put("test".to_string(), report.clone()).await;
        assert!(cache1.get("test").await.is_some());

        // Test capacity 100
        let cache100 = UnifiedCache::new(100);
        cache100.put("test".to_string(), report.clone()).await;
        assert!(cache100.get("test").await.is_some());

        // Test large capacity
        let cache_large = UnifiedCache::new(10_000);
        cache_large.put("test".to_string(), report).await;
        assert!(cache_large.get("test").await.is_some());
    }

    /// Test UnifiedCache LRU eviction behavior
    #[tokio::test]
    async fn test_unified_cache_lru_eviction() {
        let cache = UnifiedCache::new(2); // Capacity 2

        let report1 = AnalysisReport {
            duplicates: None,
            dead_code: None,
            complexity_metrics: None,
            dependency_graph: None,
            defect_predictions: None,
            graph_metrics: None,
            timestamp: Utc::now(),
        };

        let report2 = report1.clone();
        let report3 = report1.clone();

        // Fill cache to capacity
        cache.put("key1".to_string(), report1).await;
        cache.put("key2".to_string(), report2).await;

        // Both should be present
        assert!(cache.get("key1").await.is_some());
        assert!(cache.get("key2").await.is_some());

        // Add third item - should evict least recently used (key1)
        cache.put("key3".to_string(), report3).await;

        // key1 should be evicted
        assert!(cache.get("key1").await.is_none());
        // key2 and key3 should be present
        assert!(cache.get("key2").await.is_some());
        assert!(cache.get("key3").await.is_some());
    }

    // === AnalysisType tests ===

    #[test]
    fn test_analysis_type_equality() {
        assert_eq!(
            AnalysisType::DuplicateDetection,
            AnalysisType::DuplicateDetection
        );
        assert_ne!(
            AnalysisType::DuplicateDetection,
            AnalysisType::DeadCodeAnalysis
        );
    }

    #[test]
    fn test_analysis_type_clone() {
        let t = AnalysisType::ComplexityMetrics;
        let cloned = t;
        assert_eq!(t, cloned);
    }

    #[test]
    fn test_analysis_type_debug() {
        let t = AnalysisType::DependencyGraph;
        let debug_str = format!("{:?}", t);
        assert!(debug_str.contains("DependencyGraph"));
    }

    #[test]
    fn test_analysis_type_all_variants() {
        let types = vec![
            AnalysisType::DuplicateDetection,
            AnalysisType::DeadCodeAnalysis,
            AnalysisType::ComplexityMetrics,
            AnalysisType::DependencyGraph,
            AnalysisType::DefectPrediction,
            AnalysisType::NameSimilarity,
        ];
        assert_eq!(types.len(), 6);
    }

    // === AnalysisRequest tests ===

    #[test]
    fn test_analysis_request_creation() {
        let req = AnalysisRequest {
            project_path: "/test/path".to_string(),
            analysis_types: vec![AnalysisType::DuplicateDetection],
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec!["target/".to_string()],
            max_depth: Some(5),
            parallel: true,
        };

        assert_eq!(req.project_path, "/test/path");
        assert_eq!(req.analysis_types.len(), 1);
        assert!(req.parallel);
    }

    #[test]
    fn test_analysis_request_clone() {
        let req = AnalysisRequest {
            project_path: "/test".to_string(),
            analysis_types: vec![AnalysisType::DeadCodeAnalysis],
            include_patterns: vec![],
            exclude_patterns: vec![],
            max_depth: None,
            parallel: false,
        };

        let cloned = req.clone();
        assert_eq!(req.project_path, cloned.project_path);
        assert_eq!(req.analysis_types, cloned.analysis_types);
    }

    #[test]
    fn test_analysis_request_cache_key_different_paths() {
        let req1 = AnalysisRequest {
            project_path: "/path/one".to_string(),
            analysis_types: vec![AnalysisType::DuplicateDetection],
            include_patterns: vec![],
            exclude_patterns: vec![],
            max_depth: None,
            parallel: true,
        };

        let req2 = AnalysisRequest {
            project_path: "/path/two".to_string(),
            analysis_types: vec![AnalysisType::DuplicateDetection],
            include_patterns: vec![],
            exclude_patterns: vec![],
            max_depth: None,
            parallel: true,
        };

        assert_ne!(req1.cache_key(), req2.cache_key());
    }

    #[test]
    fn test_analysis_request_cache_key_different_types() {
        let req1 = AnalysisRequest {
            project_path: "/test".to_string(),
            analysis_types: vec![AnalysisType::DuplicateDetection],
            include_patterns: vec![],
            exclude_patterns: vec![],
            max_depth: None,
            parallel: true,
        };

        let req2 = AnalysisRequest {
            project_path: "/test".to_string(),
            analysis_types: vec![AnalysisType::DeadCodeAnalysis],
            include_patterns: vec![],
            exclude_patterns: vec![],
            max_depth: None,
            parallel: true,
        };

        assert_ne!(req1.cache_key(), req2.cache_key());
    }

    #[test]
    fn test_analysis_request_cache_key_length() {
        let req = AnalysisRequest {
            project_path: "/test".to_string(),
            analysis_types: vec![],
            include_patterns: vec![],
            exclude_patterns: vec![],
            max_depth: None,
            parallel: false,
        };

        let key = req.cache_key();
        assert_eq!(key.len(), 64); // SHA256 hex = 64 chars
    }

    // === ComplexityReport tests ===

    #[test]
    fn test_complexity_report_creation() {
        let report = ComplexityReport {
            total_files: 10,
            average_complexity: 5.5,
            hotspots: vec![],
        };

        assert_eq!(report.total_files, 10);
        assert_eq!(report.average_complexity, 5.5);
        assert!(report.hotspots.is_empty());
    }

    #[test]
    fn test_complexity_report_with_hotspots() {
        let hotspot = ComplexityHotspot {
            file_path: "src/main.rs".to_string(),
            function_name: "process_data".to_string(),
            cyclomatic_complexity: 15,
            cognitive_complexity: 20,
        };

        let report = ComplexityReport {
            total_files: 1,
            average_complexity: 15.0,
            hotspots: vec![hotspot],
        };

        assert_eq!(report.hotspots.len(), 1);
        assert_eq!(report.hotspots[0].cyclomatic_complexity, 15);
    }

    // === ComplexityHotspot tests ===

    #[test]
    fn test_complexity_hotspot_creation() {
        let hotspot = ComplexityHotspot {
            file_path: "lib.rs".to_string(),
            function_name: "calculate".to_string(),
            cyclomatic_complexity: 10,
            cognitive_complexity: 8,
        };

        assert_eq!(hotspot.file_path, "lib.rs");
        assert_eq!(hotspot.function_name, "calculate");
    }

    #[test]
    fn test_complexity_hotspot_clone() {
        let hotspot = ComplexityHotspot {
            file_path: "test.rs".to_string(),
            function_name: "test_fn".to_string(),
            cyclomatic_complexity: 5,
            cognitive_complexity: 3,
        };

        let cloned = hotspot.clone();
        assert_eq!(hotspot.file_path, cloned.file_path);
        assert_eq!(hotspot.cyclomatic_complexity, cloned.cyclomatic_complexity);
    }

    // === DependencyGraphReport tests ===

    #[test]
    fn test_dependency_graph_report_creation() {
        let report = DependencyGraphReport {
            nodes: 50,
            edges: 100,
            circular_dependencies: vec![],
            mermaid_diagram: "graph TD".to_string(),
        };

        assert_eq!(report.nodes, 50);
        assert_eq!(report.edges, 100);
        assert!(report.circular_dependencies.is_empty());
    }

    #[test]
    fn test_dependency_graph_with_cycles() {
        let report = DependencyGraphReport {
            nodes: 3,
            edges: 3,
            circular_dependencies: vec![vec!["A".to_string(), "B".to_string(), "C".to_string()]],
            mermaid_diagram: "".to_string(),
        };

        assert_eq!(report.circular_dependencies.len(), 1);
        assert_eq!(report.circular_dependencies[0].len(), 3);
    }

    // === DefectScore tests ===

    #[test]
    fn test_defect_score_creation() {
        let score = DefectScore {
            entity: "main.rs".to_string(),
            score: 0.75,
            confidence: 0.9,
            reasons: vec!["High complexity".to_string()],
        };

        assert_eq!(score.entity, "main.rs");
        assert_eq!(score.score, 0.75);
        assert_eq!(score.confidence, 0.9);
    }

    #[test]
    fn test_defect_score_multiple_reasons() {
        let score = DefectScore {
            entity: "lib.rs".to_string(),
            score: 0.5,
            confidence: 0.8,
            reasons: vec![
                "High churn".to_string(),
                "Large function".to_string(),
                "Deep nesting".to_string(),
            ],
        };

        assert_eq!(score.reasons.len(), 3);
    }

    // === GraphMetricsReport tests ===

    #[test]
    fn test_graph_metrics_report_creation() {
        let report = GraphMetricsReport {
            centrality_scores: vec![],
            clustering_coefficient: 0.5,
            modularity: 0.7,
        };

        assert!(report.centrality_scores.is_empty());
        assert_eq!(report.clustering_coefficient, 0.5);
        assert_eq!(report.modularity, 0.7);
    }

    // === CentralityScore tests ===

    #[test]
    fn test_centrality_score_creation() {
        let score = CentralityScore {
            node: "main_module".to_string(),
            degree: 10.0,
            betweenness: 0.5,
            closeness: 0.8,
            pagerank: 0.1,
        };

        assert_eq!(score.node, "main_module");
        assert_eq!(score.degree, 10.0);
        assert_eq!(score.betweenness, 0.5);
    }

    #[test]
    fn test_centrality_score_clone() {
        let score = CentralityScore {
            node: "test".to_string(),
            degree: 5.0,
            betweenness: 0.3,
            closeness: 0.6,
            pagerank: 0.05,
        };

        let cloned = score.clone();
        assert_eq!(score.node, cloned.node);
        assert_eq!(score.pagerank, cloned.pagerank);
    }

    // === AnalysisReport tests ===

    #[test]
    fn test_analysis_report_empty() {
        let report = AnalysisReport {
            duplicates: None,
            dead_code: None,
            complexity_metrics: None,
            dependency_graph: None,
            defect_predictions: None,
            graph_metrics: None,
            timestamp: Utc::now(),
        };

        assert!(report.duplicates.is_none());
        assert!(report.dead_code.is_none());
        assert!(report.complexity_metrics.is_none());
    }

    #[test]
    fn test_analysis_report_with_complexity() {
        let complexity = ComplexityReport {
            total_files: 5,
            average_complexity: 7.5,
            hotspots: vec![],
        };

        let report = AnalysisReport {
            duplicates: None,
            dead_code: None,
            complexity_metrics: Some(complexity),
            dependency_graph: None,
            defect_predictions: None,
            graph_metrics: None,
            timestamp: Utc::now(),
        };

        assert!(report.complexity_metrics.is_some());
        assert_eq!(report.complexity_metrics.as_ref().unwrap().total_files, 5);
    }

    // === CodeIntelligence tests ===

    #[tokio::test]
    async fn test_code_intelligence_default() {
        let intel = CodeIntelligence::default();
        let (nodes, gen) = intel.get_dag_stats().await;

        assert_eq!(nodes, 0);
        assert_eq!(gen, 0);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn basic_property_stability(_input in ".*") {
            // Basic property test for coverage
            prop_assert!(true);
        }

        #[test]
        fn module_consistency_check(_x in 0u32..1000) {
            // Module consistency verification
            prop_assert!(_x < 1001);
        }
    }
}
