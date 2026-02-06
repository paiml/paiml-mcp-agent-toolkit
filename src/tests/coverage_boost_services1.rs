#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services - batch 1
//! Tests for DeepContext configuration and related types

use crate::services::deep_context::{
    AnalysisType, CacheStrategy, DagType, DeepContextConfig,
    DeepContext, NodeType, ComplexityThresholds,
};

#[test]
fn test_deep_context_config_default() {
    let config = DeepContextConfig::default();
    assert!(!config.include_analyses.is_empty());
    assert!(config.period_days > 0);
}

#[test]
fn test_deep_context_config_analysis_types() {
    let config = DeepContextConfig::default();
    // Default should include common analyses
    assert!(config.include_analyses.contains(&AnalysisType::Ast));
}

#[test]
fn test_analysis_type_variants() {
    let types = vec![
        AnalysisType::Ast,
        AnalysisType::Complexity,
        AnalysisType::Churn,
        AnalysisType::Dag,
        AnalysisType::DeadCode,
        AnalysisType::DuplicateCode,
        AnalysisType::Satd,
        AnalysisType::Provability,
        AnalysisType::TechnicalDebtGradient,
        AnalysisType::BigO,
    ];
    for t in types {
        let _ = format!("{:?}", t);
    }
}

#[test]
fn test_dag_type_variants() {
    let types = vec![
        DagType::CallGraph,
        DagType::ImportGraph,
        DagType::Inheritance,
        DagType::FullDependency,
    ];
    for t in types {
        let _ = format!("{:?}", t);
    }
}

#[test]
fn test_cache_strategy_variants() {
    let strategies = vec![
        CacheStrategy::Normal,
        CacheStrategy::ForceRefresh,
        CacheStrategy::Offline,
    ];
    for s in strategies {
        let _ = format!("{:?}", s);
    }
}

#[test]
fn test_node_type_variants() {
    let types = vec![NodeType::Directory, NodeType::File];
    for t in types {
        let _ = format!("{:?}", t);
    }
    assert_eq!(NodeType::default(), NodeType::File);
}

#[test]
fn test_deep_context_default() {
    let ctx = DeepContext::default();
    assert!(ctx.recommendations.is_empty());
    assert!(ctx.hotspots.is_empty());
}

#[test]
fn test_complexity_thresholds() {
    let thresholds = ComplexityThresholds {
        max_cyclomatic: 20,
        max_cognitive: 30,
    };
    assert_eq!(thresholds.max_cyclomatic, 20);
    assert_eq!(thresholds.max_cognitive, 30);
}

#[test]
fn test_analysis_type_equality() {
    assert_eq!(AnalysisType::Ast, AnalysisType::Ast);
    assert_ne!(AnalysisType::Ast, AnalysisType::Complexity);
}

#[test]
fn test_dag_type_equality() {
    assert_eq!(DagType::CallGraph, DagType::CallGraph);
    assert_ne!(DagType::CallGraph, DagType::ImportGraph);
}

#[test]
fn test_cache_strategy_equality() {
    assert_eq!(CacheStrategy::Normal, CacheStrategy::Normal);
    assert_ne!(CacheStrategy::Normal, CacheStrategy::ForceRefresh);
}

#[test]
fn test_node_type_equality() {
    assert_eq!(NodeType::File, NodeType::File);
    assert_ne!(NodeType::File, NodeType::Directory);
}

#[test]
fn test_deep_context_config_clone() {
    let config = DeepContextConfig::default();
    let cloned = config.clone();
    assert_eq!(config.period_days, cloned.period_days);
}

#[test]
fn test_analysis_type_clone() {
    let t = AnalysisType::Complexity;
    let cloned = t.clone();
    assert_eq!(t, cloned);
}
