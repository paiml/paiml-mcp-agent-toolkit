#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for services/polyglot_analyzer.rs
//! Tests: LanguageInfo, PolyglotAnalysis, LanguageStats, CrossLanguageDependency,
//! DependencyType, ArchitecturePattern, IntegrationPoint, IntegrationType, RiskLevel

use crate::services::polyglot_analyzer::{
    ArchitecturePattern, CrossLanguageDependency, DependencyType, IntegrationPoint,
    IntegrationType, LanguageInfo, LanguageStats, PolyglotAnalysis, RiskLevel,
};

// ============ LanguageInfo Tests ============

#[test]
fn test_language_info_serde() {
    let info = LanguageInfo {
        name: "Rust".to_string(),
        file_count: 150,
        line_count: 45000,
        frameworks: vec!["tokio".to_string(), "serde".to_string()],
    };
    let json = serde_json::to_string(&info).unwrap();
    let back: LanguageInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "Rust");
    assert_eq!(back.file_count, 150);
    assert_eq!(back.frameworks.len(), 2);
}

#[test]
fn test_language_info_clone() {
    let info = LanguageInfo {
        name: "Python".to_string(),
        file_count: 50,
        line_count: 10000,
        frameworks: vec!["flask".to_string()],
    };
    let cloned = info.clone();
    assert_eq!(cloned.name, info.name);
}

// ============ LanguageStats Tests ============

#[test]
fn test_language_stats_serde() {
    let stats = LanguageStats {
        language: "TypeScript".to_string(),
        file_count: 200,
        line_count: 60000,
        complexity_score: 2.5,
        test_coverage: 0.85,
        primary_frameworks: vec!["React".to_string(), "Next.js".to_string()],
    };
    let json = serde_json::to_string(&stats).unwrap();
    let back: LanguageStats = serde_json::from_str(&json).unwrap();
    assert_eq!(back.language, "TypeScript");
    assert_eq!(back.file_count, 200);
    assert!((back.test_coverage - 0.85).abs() < f64::EPSILON);
}

// ============ DependencyType Tests ============

#[test]
fn test_dependency_type_serde() {
    let types = vec![
        DependencyType::FFI,
        DependencyType::ProcessCommunication,
        DependencyType::SharedDataStructure,
        DependencyType::ConfigurationFile,
        DependencyType::BuildSystem,
        DependencyType::Testing,
    ];
    for dt in &types {
        let json = serde_json::to_string(dt).unwrap();
        let back: DependencyType = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", dt), format!("{:?}", back));
    }
}

// ============ ArchitecturePattern Tests ============

#[test]
fn test_architecture_pattern_serde() {
    let patterns = vec![
        ArchitecturePattern::Microservices,
        ArchitecturePattern::Monolithic,
        ArchitecturePattern::LayeredArchitecture,
        ArchitecturePattern::EventDriven,
        ArchitecturePattern::PluginArchitecture,
        ArchitecturePattern::ClientServer,
        ArchitecturePattern::Mixed,
    ];
    for p in &patterns {
        let json = serde_json::to_string(p).unwrap();
        let back: ArchitecturePattern = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", p), format!("{:?}", back));
    }
}

// ============ IntegrationType Tests ============

#[test]
fn test_integration_type_serde() {
    let types = vec![
        IntegrationType::API,
        IntegrationType::Database,
        IntegrationType::FileSystem,
        IntegrationType::Memory,
        IntegrationType::Network,
        IntegrationType::Configuration,
    ];
    for it in &types {
        let json = serde_json::to_string(it).unwrap();
        let back: IntegrationType = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", it), format!("{:?}", back));
    }
}

// ============ RiskLevel Tests ============

#[test]
fn test_risk_level_serde() {
    let levels = vec![
        RiskLevel::Low,
        RiskLevel::Medium,
        RiskLevel::High,
        RiskLevel::Critical,
    ];
    for rl in &levels {
        let json = serde_json::to_string(rl).unwrap();
        let back: RiskLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{:?}", rl), format!("{:?}", back));
    }
}

// ============ CrossLanguageDependency Tests ============

#[test]
fn test_cross_language_dependency_serde() {
    let dep = CrossLanguageDependency {
        from_language: "Rust".to_string(),
        to_language: "C".to_string(),
        dependency_type: DependencyType::FFI,
        coupling_strength: 0.8,
        files_involved: vec!["src/ffi.rs".to_string(), "include/lib.h".to_string()],
    };
    let json = serde_json::to_string(&dep).unwrap();
    let back: CrossLanguageDependency = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from_language, "Rust");
    assert_eq!(back.to_language, "C");
    assert!((back.coupling_strength - 0.8).abs() < f64::EPSILON);
}

// ============ IntegrationPoint Tests ============

#[test]
fn test_integration_point_serde() {
    let point = IntegrationPoint {
        name: "REST API".to_string(),
        languages: vec!["Rust".to_string(), "TypeScript".to_string()],
        integration_type: IntegrationType::API,
        risk_level: RiskLevel::Medium,
        description: "Backend-frontend communication".to_string(),
    };
    let json = serde_json::to_string(&point).unwrap();
    let back: IntegrationPoint = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "REST API");
    assert_eq!(back.languages.len(), 2);
}

// ============ PolyglotAnalysis Tests ============

#[test]
fn test_polyglot_analysis_serde() {
    let analysis = PolyglotAnalysis {
        languages: vec![LanguageStats {
            language: "Rust".to_string(),
            file_count: 100,
            line_count: 30000,
            complexity_score: 1.8,
            test_coverage: 0.9,
            primary_frameworks: vec!["tokio".to_string()],
        }],
        cross_language_dependencies: vec![],
        architecture_pattern: Some(ArchitecturePattern::Monolithic),
        integration_points: vec![],
        recommendation_score: 8.5,
    };
    let json = serde_json::to_string(&analysis).unwrap();
    let back: PolyglotAnalysis = serde_json::from_str(&json).unwrap();
    assert_eq!(back.languages.len(), 1);
    assert!(back.architecture_pattern.is_some());
    assert!((back.recommendation_score - 8.5).abs() < f64::EPSILON);
}

#[test]
fn test_polyglot_analysis_no_architecture() {
    let analysis = PolyglotAnalysis {
        languages: vec![],
        cross_language_dependencies: vec![],
        architecture_pattern: None,
        integration_points: vec![],
        recommendation_score: 0.0,
    };
    let json = serde_json::to_string(&analysis).unwrap();
    let back: PolyglotAnalysis = serde_json::from_str(&json).unwrap();
    assert!(back.architecture_pattern.is_none());
}
