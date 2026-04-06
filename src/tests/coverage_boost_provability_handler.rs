#![cfg_attr(coverage_nightly, coverage(off))]
//! Coverage boost tests for provability_handler and related modules
//! Comprehensive tests for ProvabilityConfig, helpers, and lightweight analyzer
//!
//! This module provides extensive test coverage for:
//! - ProvabilityConfig struct and its fields
//! - Provability helpers (parse_function_spec, filter_summaries, format_*)
//! - LightweightProvabilityAnalyzer and abstract interpretation
//! - Lattice operations (NullabilityLattice, IntervalLattice, AliasLattice, PurityLattice)
//! - PropertyDomain operations (top, join, widen, is_equal)
//! - FunctionId and ProofSummary structs
//! - PropertyType enum variants
//! - VerifiedProperty struct

use crate::cli::enums::ProvabilityOutputFormat;
use crate::cli::handlers::provability_handler::ProvabilityConfig;
use crate::cli::provability_helpers::{
    filter_summaries, format_provability_detailed, format_provability_json,
    format_provability_sarif, format_provability_summary, parse_function_spec,
};
use crate::services::lightweight_provability_analyzer::{
    AliasLattice, FunctionId, IntervalLattice, LightweightProvabilityAnalyzer, NullabilityLattice,
    ProofSummary, PropertyDomain, PropertyType, PurityLattice, VerifiedProperty,
};
use std::path::PathBuf;

fn strip_ansi(s: &str) -> String {
    debug_assert!(!s.is_empty(), "s must not be empty");
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}

// ============================================================
// ProvabilityConfig Tests
// ============================================================

#[test]
fn test_provability_config_all_fields() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/home/user/project"),
        functions: vec!["main".to_string(), "helper".to_string()],
        analysis_depth: 10,
        format: ProvabilityOutputFormat::Json,
        high_confidence_only: true,
        include_evidence: true,
        output: Some(PathBuf::from("/tmp/output.json")),
        top_files: 20,
    };

    assert_eq!(config.project_path, PathBuf::from("/home/user/project"));
    assert_eq!(config.functions.len(), 2);
    assert_eq!(config.analysis_depth, 10);
    assert_eq!(config.format, ProvabilityOutputFormat::Json);
    assert!(config.high_confidence_only);
    assert!(config.include_evidence);
    assert_eq!(config.output, Some(PathBuf::from("/tmp/output.json")));
    assert_eq!(config.top_files, 20);
}

#[test]
fn test_provability_config_minimal() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("."),
        functions: vec![],
        analysis_depth: 1,
        format: ProvabilityOutputFormat::Summary,
        high_confidence_only: false,
        include_evidence: false,
        output: None,
        top_files: 0,
    };

    assert!(config.functions.is_empty());
    assert_eq!(config.analysis_depth, 1);
    assert!(!config.high_confidence_only);
    assert!(!config.include_evidence);
    assert!(config.output.is_none());
}

#[test]
fn test_provability_config_all_formats() {
    let formats = vec![
        ProvabilityOutputFormat::Json,
        ProvabilityOutputFormat::Summary,
        ProvabilityOutputFormat::Full,
        ProvabilityOutputFormat::Sarif,
        ProvabilityOutputFormat::Markdown,
    ];

    for format in formats {
        let config = ProvabilityConfig {
            project_path: PathBuf::from("/test"),
            functions: vec![],
            analysis_depth: 5,
            format: format.clone(),
            high_confidence_only: false,
            include_evidence: false,
            output: None,
            top_files: 5,
        };
        assert_eq!(config.format, format);
    }
}

#[test]
fn test_provability_config_clone() {
    let original = ProvabilityConfig {
        project_path: PathBuf::from("/original"),
        functions: vec!["fn1".to_string(), "fn2".to_string()],
        analysis_depth: 7,
        format: ProvabilityOutputFormat::Sarif,
        high_confidence_only: true,
        include_evidence: true,
        output: Some(PathBuf::from("/output.sarif")),
        top_files: 15,
    };

    let cloned = original.clone();
    assert_eq!(original.project_path, cloned.project_path);
    assert_eq!(original.functions, cloned.functions);
    assert_eq!(original.analysis_depth, cloned.analysis_depth);
    assert_eq!(original.format, cloned.format);
    assert_eq!(original.high_confidence_only, cloned.high_confidence_only);
    assert_eq!(original.include_evidence, cloned.include_evidence);
    assert_eq!(original.output, cloned.output);
    assert_eq!(original.top_files, cloned.top_files);
}

#[test]
fn test_provability_config_debug() {
    let config = ProvabilityConfig {
        project_path: PathBuf::from("/debug/test"),
        functions: vec!["test_fn".to_string()],
        analysis_depth: 3,
        format: ProvabilityOutputFormat::Full,
        high_confidence_only: false,
        include_evidence: true,
        output: None,
        top_files: 10,
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ProvabilityConfig"));
    assert!(debug_str.contains("/debug/test"));
    assert!(debug_str.contains("test_fn"));
}

// ============================================================
// parse_function_spec Tests
// ============================================================

#[test]
fn test_parse_function_spec_simple_name() {
    let result = parse_function_spec("main", &PathBuf::from("/project")).unwrap();
    assert_eq!(result.function_name, "main");
    assert!(result.file_path.is_empty());
    assert_eq!(result.line_number, 0);
}

#[test]
fn test_parse_function_spec_with_path() {
    let result = parse_function_spec("src/lib.rs:helper", &PathBuf::from("/project")).unwrap();
    assert_eq!(result.function_name, "helper");
    assert!(result.file_path.contains("src/lib.rs"));
    assert_eq!(result.line_number, 0);
}

#[test]
fn test_parse_function_spec_nested_path() {
    let result = parse_function_spec(
        "src/services/analyzer.rs:analyze",
        &PathBuf::from("/project"),
    )
    .unwrap();
    assert_eq!(result.function_name, "analyze");
    assert!(result.file_path.contains("services"));
}

#[test]
fn test_parse_function_spec_empty_function_name() {
    let result = parse_function_spec("src/main.rs:", &PathBuf::from("/project")).unwrap();
    assert!(result.function_name.is_empty());
}

#[test]
fn test_parse_function_spec_windows_style_path() {
    // Unix will still process this, just differently
    let result = parse_function_spec("src\\lib.rs:func", &PathBuf::from("/project")).unwrap();
    assert_eq!(result.function_name, "func");
}

// ============================================================
// filter_summaries Tests
// ============================================================

fn create_summary(score: f64) -> ProofSummary {
    debug_assert!(score >= 0.0, "score must be non-negative");
    ProofSummary {
        provability_score: score,
        analysis_time_us: 100,
        verified_properties: vec![],
        version: 1,
    }
}

#[test]
fn test_filter_summaries_no_filter() {
    let summaries = vec![
        create_summary(0.9),
        create_summary(0.5),
        create_summary(0.3),
    ];
    let filtered = filter_summaries(&summaries, false);
    assert_eq!(filtered.len(), 3);
}

#[test]
fn test_filter_summaries_high_confidence_only() {
    let summaries = vec![
        create_summary(0.95),
        create_summary(0.8),
        create_summary(0.79),
        create_summary(0.5),
    ];
    let filtered = filter_summaries(&summaries, true);
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|s| s.provability_score >= 0.8));
}

#[test]
fn test_filter_summaries_empty() {
    let summaries: Vec<ProofSummary> = vec![];
    let filtered = filter_summaries(&summaries, true);
    assert!(filtered.is_empty());
}

#[test]
fn test_filter_summaries_all_below_threshold() {
    let summaries = vec![
        create_summary(0.7),
        create_summary(0.6),
        create_summary(0.5),
    ];
    let filtered = filter_summaries(&summaries, true);
    assert!(filtered.is_empty());
}

#[test]
fn test_filter_summaries_boundary_exact() {
    let summaries = vec![create_summary(0.8), create_summary(0.7999)];
    let filtered = filter_summaries(&summaries, true);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].provability_score, 0.8);
}

// ============================================================
// FunctionId Tests
// ============================================================

fn create_function_id(file: &str, name: &str, line: usize) -> FunctionId {
    debug_assert!(!file.is_empty(), "file must not be empty");
    debug_assert!(!name.is_empty(), "name must not be empty");
    FunctionId {
        file_path: file.to_string(),
        function_name: name.to_string(),
        line_number: line,
    }
}

#[test]
fn test_function_id_creation() {
    let func_id = create_function_id("src/main.rs", "main", 10);
    assert_eq!(func_id.file_path, "src/main.rs");
    assert_eq!(func_id.function_name, "main");
    assert_eq!(func_id.line_number, 10);
}

#[test]
fn test_function_id_clone() {
    let func_id = create_function_id("src/lib.rs", "helper", 42);
    let cloned = func_id.clone();
    assert_eq!(func_id.file_path, cloned.file_path);
    assert_eq!(func_id.function_name, cloned.function_name);
    assert_eq!(func_id.line_number, cloned.line_number);
}

#[test]
fn test_function_id_debug() {
    let func_id = create_function_id("test.rs", "test_fn", 1);
    let debug_str = format!("{:?}", func_id);
    assert!(debug_str.contains("FunctionId"));
    assert!(debug_str.contains("test.rs"));
    assert!(debug_str.contains("test_fn"));
}

#[test]
fn test_function_id_equality() {
    let func_id1 = create_function_id("src/main.rs", "main", 10);
    let func_id2 = create_function_id("src/main.rs", "main", 10);
    let func_id3 = create_function_id("src/main.rs", "main", 20);

    assert_eq!(func_id1, func_id2);
    assert_ne!(func_id1, func_id3);
}

#[test]
fn test_function_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(create_function_id("a.rs", "fn1", 1));
    set.insert(create_function_id("b.rs", "fn2", 2));
    set.insert(create_function_id("a.rs", "fn1", 1)); // duplicate
    assert_eq!(set.len(), 2);
}

// ============================================================
// ProofSummary Tests
// ============================================================

#[test]
fn test_proof_summary_creation() {
    let summary = ProofSummary {
        provability_score: 0.85,
        analysis_time_us: 500,
        verified_properties: vec![],
        version: 1,
    };
    assert_eq!(summary.provability_score, 0.85);
    assert_eq!(summary.analysis_time_us, 500);
    assert!(summary.verified_properties.is_empty());
    assert_eq!(summary.version, 1);
}

#[test]
fn test_proof_summary_with_properties() {
    let summary = ProofSummary {
        provability_score: 0.95,
        analysis_time_us: 200,
        verified_properties: vec![
            VerifiedProperty {
                property_type: PropertyType::NullSafety,
                confidence: 0.9,
                evidence: "No null refs".to_string(),
            },
            VerifiedProperty {
                property_type: PropertyType::BoundsCheck,
                confidence: 0.85,
                evidence: "Bounds verified".to_string(),
            },
        ],
        version: 2,
    };
    assert_eq!(summary.verified_properties.len(), 2);
}

#[test]
fn test_proof_summary_clone() {
    let summary = create_summary(0.75);
    let cloned = summary.clone();
    assert_eq!(summary.provability_score, cloned.provability_score);
}

#[test]
fn test_proof_summary_debug() {
    let summary = create_summary(0.5);
    let debug_str = format!("{:?}", summary);
    assert!(debug_str.contains("ProofSummary"));
    assert!(debug_str.contains("0.5"));
}

// ============================================================
// PropertyType Tests
// ============================================================

#[test]
fn test_property_type_variants() {
    let types = vec![
        PropertyType::NullSafety,
        PropertyType::BoundsCheck,
        PropertyType::NoAliasing,
        PropertyType::PureFunction,
        PropertyType::MemorySafety,
        PropertyType::ThreadSafety,
    ];

    for prop_type in &types {
        let debug_str = format!("{:?}", prop_type);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_property_type_equality() {
    assert_eq!(PropertyType::NullSafety, PropertyType::NullSafety);
    assert_ne!(PropertyType::NullSafety, PropertyType::BoundsCheck);
}

#[test]
fn test_property_type_clone() {
    let prop_type = PropertyType::MemorySafety;
    let cloned = prop_type.clone();
    assert_eq!(prop_type, cloned);
}

// ============================================================
// VerifiedProperty Tests
// ============================================================

#[test]
fn test_verified_property_creation() {
    let prop = VerifiedProperty {
        property_type: PropertyType::PureFunction,
        confidence: 0.95,
        evidence: "No side effects detected".to_string(),
    };
    assert_eq!(prop.property_type, PropertyType::PureFunction);
    assert_eq!(prop.confidence, 0.95);
    assert_eq!(prop.evidence, "No side effects detected");
}

#[test]
fn test_verified_property_clone() {
    let prop = VerifiedProperty {
        property_type: PropertyType::ThreadSafety,
        confidence: 0.88,
        evidence: "Sync + Send bounds verified".to_string(),
    };
    let cloned = prop.clone();
    assert_eq!(prop.property_type, cloned.property_type);
    assert_eq!(prop.confidence, cloned.confidence);
    assert_eq!(prop.evidence, cloned.evidence);
}

#[test]
fn test_verified_property_debug() {
    let prop = VerifiedProperty {
        property_type: PropertyType::NoAliasing,
        confidence: 0.8,
        evidence: "No mutable aliasing".to_string(),
    };
    let debug_str = format!("{:?}", prop);
    assert!(debug_str.contains("VerifiedProperty"));
    assert!(debug_str.contains("NoAliasing"));
}

// ============================================================
// NullabilityLattice Tests
// ============================================================

#[test]
fn test_nullability_lattice_join_not_null() {
    assert_eq!(
        NullabilityLattice::NotNull.join(&NullabilityLattice::NotNull),
        NullabilityLattice::NotNull
    );
}

#[test]
fn test_nullability_lattice_join_with_top() {
    assert_eq!(
        NullabilityLattice::NotNull.join(&NullabilityLattice::Top),
        NullabilityLattice::Top
    );
    assert_eq!(
        NullabilityLattice::Top.join(&NullabilityLattice::Null),
        NullabilityLattice::Top
    );
}

#[test]
fn test_nullability_lattice_join_with_bottom() {
    assert_eq!(
        NullabilityLattice::Bottom.join(&NullabilityLattice::NotNull),
        NullabilityLattice::NotNull
    );
    assert_eq!(
        NullabilityLattice::Null.join(&NullabilityLattice::Bottom),
        NullabilityLattice::Null
    );
}

#[test]
fn test_nullability_lattice_join_contradiction() {
    // NotNull + Null = Bottom (contradiction in original, but join semantics may differ)
    let result = NullabilityLattice::NotNull.join(&NullabilityLattice::Null);
    assert_eq!(result, NullabilityLattice::Bottom);
}

#[test]
fn test_nullability_lattice_join_maybe_null() {
    assert_eq!(
        NullabilityLattice::NotNull.join(&NullabilityLattice::MaybeNull),
        NullabilityLattice::MaybeNull
    );
}

#[test]
fn test_nullability_lattice_equality() {
    assert_eq!(NullabilityLattice::NotNull, NullabilityLattice::NotNull);
    assert_ne!(NullabilityLattice::NotNull, NullabilityLattice::Null);
}

// ============================================================
// IntervalLattice Tests
// ============================================================

#[test]
fn test_interval_lattice_widen_both_bounds() {
    let interval1 = IntervalLattice {
        lower: Some(0),
        upper: Some(100),
    };
    let interval2 = IntervalLattice {
        lower: Some(0),
        upper: Some(100),
    };
    let result = interval1.widen(&interval2);
    assert_eq!(result.lower, Some(0));
    assert_eq!(result.upper, Some(100));
}

#[test]
fn test_interval_lattice_widen_to_infinity_upper() {
    let interval1 = IntervalLattice {
        lower: Some(0),
        upper: Some(100),
    };
    let interval2 = IntervalLattice {
        lower: Some(0),
        upper: Some(200),
    }; // upper increased
    let result = interval1.widen(&interval2);
    assert_eq!(result.lower, Some(0));
    assert_eq!(result.upper, None); // Widened to +inf
}

#[test]
fn test_interval_lattice_widen_to_infinity_lower() {
    let interval1 = IntervalLattice {
        lower: Some(10),
        upper: Some(100),
    };
    let interval2 = IntervalLattice {
        lower: Some(5),
        upper: Some(100),
    }; // lower decreased
    let result = interval1.widen(&interval2);
    assert_eq!(result.lower, None); // Widened to -inf
    assert_eq!(result.upper, Some(100));
}

#[test]
fn test_interval_lattice_is_equal() {
    let interval1 = IntervalLattice {
        lower: Some(0),
        upper: Some(100),
    };
    let interval2 = IntervalLattice {
        lower: Some(0),
        upper: Some(100),
    };
    let interval3 = IntervalLattice {
        lower: Some(0),
        upper: Some(50),
    };

    assert!(interval1.is_equal(&interval2));
    assert!(!interval1.is_equal(&interval3));
}

#[test]
fn test_interval_lattice_none_bounds() {
    let interval1 = IntervalLattice {
        lower: None,
        upper: None,
    };
    let interval2 = IntervalLattice {
        lower: Some(0),
        upper: Some(100),
    };
    let result = interval1.widen(&interval2);
    assert_eq!(result.lower, None);
    assert_eq!(result.upper, None);
}

// ============================================================
// AliasLattice Tests
// ============================================================

#[test]
fn test_alias_lattice_join_no_alias() {
    assert_eq!(
        AliasLattice::NoAlias.join(&AliasLattice::NoAlias),
        AliasLattice::NoAlias
    );
}

#[test]
fn test_alias_lattice_join_with_top() {
    assert_eq!(
        AliasLattice::NoAlias.join(&AliasLattice::Top),
        AliasLattice::Top
    );
}

#[test]
fn test_alias_lattice_join_with_bottom() {
    assert_eq!(
        AliasLattice::Bottom.join(&AliasLattice::NoAlias),
        AliasLattice::NoAlias
    );
}

#[test]
fn test_alias_lattice_join_must_alias() {
    assert_eq!(
        AliasLattice::MustAlias.join(&AliasLattice::MustAlias),
        AliasLattice::MustAlias
    );
}

#[test]
fn test_alias_lattice_join_mixed() {
    assert_eq!(
        AliasLattice::NoAlias.join(&AliasLattice::MustAlias),
        AliasLattice::MayAlias
    );
}

#[test]
fn test_alias_lattice_equality() {
    assert_eq!(AliasLattice::MayAlias, AliasLattice::MayAlias);
    assert_ne!(AliasLattice::NoAlias, AliasLattice::MayAlias);
}

// ============================================================
// PurityLattice Tests
// ============================================================

#[test]
fn test_purity_lattice_meet_pure() {
    assert_eq!(
        PurityLattice::Pure.meet(&PurityLattice::Pure),
        PurityLattice::Pure
    );
}

#[test]
fn test_purity_lattice_meet_readonly() {
    assert_eq!(
        PurityLattice::ReadOnly.meet(&PurityLattice::ReadOnly),
        PurityLattice::ReadOnly
    );
}

#[test]
fn test_purity_lattice_meet_write_local() {
    assert_eq!(
        PurityLattice::Pure.meet(&PurityLattice::WriteLocal),
        PurityLattice::WriteLocal
    );
}

#[test]
fn test_purity_lattice_meet_write_global() {
    assert_eq!(
        PurityLattice::Pure.meet(&PurityLattice::WriteGlobal),
        PurityLattice::WriteGlobal
    );
}

#[test]
fn test_purity_lattice_meet_bottom() {
    assert_eq!(
        PurityLattice::Bottom.meet(&PurityLattice::Pure),
        PurityLattice::Bottom
    );
}

#[test]
fn test_purity_lattice_meet_top() {
    // Pure + Top = Top (conservative)
    let result = PurityLattice::Pure.meet(&PurityLattice::Top);
    assert_eq!(result, PurityLattice::Top);
}

#[test]
fn test_purity_lattice_equality() {
    assert_eq!(PurityLattice::WriteLocal, PurityLattice::WriteLocal);
    assert_ne!(PurityLattice::ReadOnly, PurityLattice::WriteLocal);
}

// ============================================================
// PropertyDomain Tests
// ============================================================

#[test]
fn test_property_domain_top() {
    let domain = PropertyDomain::top();
    assert_eq!(domain.nullability, NullabilityLattice::Top);
    assert_eq!(domain.aliasing, AliasLattice::Top);
    assert_eq!(domain.purity, PurityLattice::Top);
    assert!(domain.bounds.lower.is_none());
    assert!(domain.bounds.upper.is_none());
}

#[test]
fn test_property_domain_join() {
    let domain1 = PropertyDomain::top();
    let domain2 = PropertyDomain {
        nullability: NullabilityLattice::NotNull,
        bounds: IntervalLattice {
            lower: Some(0),
            upper: Some(100),
        },
        aliasing: AliasLattice::NoAlias,
        purity: PurityLattice::Pure,
    };

    let joined = domain1.join(&domain2);
    assert_eq!(joined.nullability, NullabilityLattice::Top);
}

#[test]
fn test_property_domain_widen() {
    let domain1 = PropertyDomain {
        nullability: NullabilityLattice::NotNull,
        bounds: IntervalLattice {
            lower: Some(0),
            upper: Some(50),
        },
        aliasing: AliasLattice::NoAlias,
        purity: PurityLattice::Pure,
    };
    let domain2 = PropertyDomain {
        nullability: NullabilityLattice::NotNull,
        bounds: IntervalLattice {
            lower: Some(0),
            upper: Some(100),
        },
        aliasing: AliasLattice::NoAlias,
        purity: PurityLattice::Pure,
    };

    let widened = domain1.widen(&domain2);
    // Bounds should be widened
    assert_eq!(widened.bounds.upper, None);
}

#[test]
fn test_property_domain_is_equal() {
    let domain1 = PropertyDomain {
        nullability: NullabilityLattice::NotNull,
        bounds: IntervalLattice {
            lower: Some(0),
            upper: Some(100),
        },
        aliasing: AliasLattice::NoAlias,
        purity: PurityLattice::Pure,
    };
    let domain2 = domain1.clone();
    assert!(domain1.is_equal(&domain2));
}

#[test]
fn test_property_domain_is_not_equal() {
    let domain1 = PropertyDomain::top();
    let domain2 = PropertyDomain {
        nullability: NullabilityLattice::NotNull,
        bounds: IntervalLattice {
            lower: Some(0),
            upper: Some(100),
        },
        aliasing: AliasLattice::NoAlias,
        purity: PurityLattice::Pure,
    };
    assert!(!domain1.is_equal(&domain2));
}

#[test]
fn test_property_domain_clone() {
    let domain = PropertyDomain {
        nullability: NullabilityLattice::MaybeNull,
        bounds: IntervalLattice {
            lower: Some(10),
            upper: Some(50),
        },
        aliasing: AliasLattice::MayAlias,
        purity: PurityLattice::ReadOnly,
    };
    let cloned = domain.clone();
    assert!(domain.is_equal(&cloned));
}

// ============================================================
// LightweightProvabilityAnalyzer Tests
// ============================================================

#[test]
fn test_analyzer_new() {
    let analyzer = LightweightProvabilityAnalyzer::new();
    // Just verify it can be constructed
    let _ = format!("{:p}", &analyzer);
}

#[test]
fn test_analyzer_default() {
    let analyzer = LightweightProvabilityAnalyzer::default();
    let _ = format!("{:p}", &analyzer);
}

#[tokio::test]
async fn test_analyzer_analyze_incrementally_empty() {
    let analyzer = LightweightProvabilityAnalyzer::new();
    let functions: Vec<FunctionId> = vec![];
    let summaries = analyzer.analyze_incrementally(&functions).await;
    assert!(summaries.is_empty());
}

#[tokio::test]
async fn test_analyzer_analyze_incrementally_single() {
    let analyzer = LightweightProvabilityAnalyzer::new();
    let functions = vec![create_function_id("nonexistent.rs", "test_fn", 1)];
    let summaries = analyzer.analyze_incrementally(&functions).await;
    assert_eq!(summaries.len(), 1);
    // Non-existent file should have low confidence
    assert!(summaries[0].provability_score <= 1.0);
}

#[tokio::test]
async fn test_analyzer_analyze_incrementally_multiple() {
    let analyzer = LightweightProvabilityAnalyzer::new();
    let functions = vec![
        create_function_id("a.rs", "fn1", 1),
        create_function_id("b.rs", "fn2", 2),
        create_function_id("c.rs", "fn3", 3),
    ];
    let summaries = analyzer.analyze_incrementally(&functions).await;
    assert_eq!(summaries.len(), 3);
}

#[test]
fn test_analyzer_calculate_provability_factor_high_score() {
    let analyzer = LightweightProvabilityAnalyzer::new();
    let summary = ProofSummary {
        provability_score: 0.9,
        analysis_time_us: 100,
        verified_properties: vec![],
        version: 1,
    };
    let factor = analyzer.calculate_provability_factor(&summary);
    // High provability = low TDG factor
    assert!(factor < 1.0);
}

#[test]
fn test_analyzer_calculate_provability_factor_low_score() {
    let analyzer = LightweightProvabilityAnalyzer::new();
    let summary = ProofSummary {
        provability_score: 0.1,
        analysis_time_us: 100,
        verified_properties: vec![],
        version: 1,
    };
    let factor = analyzer.calculate_provability_factor(&summary);
    // Low provability = high TDG factor
    assert!(factor > 4.0);
}

#[test]
fn test_analyzer_calculate_provability_factor_with_critical_properties() {
    let analyzer = LightweightProvabilityAnalyzer::new();
    let summary = ProofSummary {
        provability_score: 0.5,
        analysis_time_us: 100,
        verified_properties: vec![VerifiedProperty {
            property_type: PropertyType::MemorySafety,
            confidence: 0.9,
            evidence: "Memory safe".to_string(),
        }],
        version: 1,
    };
    let factor = analyzer.calculate_provability_factor(&summary);
    // Should be reduced due to critical property
    assert!(factor < 2.5 * 0.8); // Less than base factor
}

#[test]
fn test_analyzer_calculate_provability_factor_with_thread_safety() {
    let analyzer = LightweightProvabilityAnalyzer::new();
    let summary = ProofSummary {
        provability_score: 0.5,
        analysis_time_us: 100,
        verified_properties: vec![VerifiedProperty {
            property_type: PropertyType::ThreadSafety,
            confidence: 0.95,
            evidence: "Thread safe".to_string(),
        }],
        version: 1,
    };
    let factor = analyzer.calculate_provability_factor(&summary);
    // Should be reduced due to critical property
    assert!(factor < 2.5);
}

// ============================================================
// Format Functions Tests
// ============================================================

#[test]
fn test_format_provability_json_basic() {
    let function_ids = vec![create_function_id("src/main.rs", "main", 10)];
    let summaries = vec![create_summary(0.85)];

    let result = format_provability_json(&function_ids, &summaries, false).unwrap();
    assert!(result.contains("provability_analysis"));
    assert!(result.contains("\"total_functions\": 1"));
    assert!(result.contains("main"));
}

#[test]
fn test_format_provability_json_with_evidence() {
    let function_ids = vec![create_function_id("test.rs", "test_fn", 5)];
    let summaries = vec![ProofSummary {
        provability_score: 0.9,
        analysis_time_us: 200,
        verified_properties: vec![VerifiedProperty {
            property_type: PropertyType::NullSafety,
            confidence: 0.95,
            evidence: "No null refs".to_string(),
        }],
        version: 1,
    }];

    let result = format_provability_json(&function_ids, &summaries, true).unwrap();
    assert!(result.contains("properties"));
    assert!(result.contains("NullSafety"));
    assert!(result.contains("No null refs"));
}

#[test]
fn test_format_provability_json_empty() {
    let function_ids: Vec<FunctionId> = vec![];
    let summaries: Vec<ProofSummary> = vec![];

    let result = format_provability_json(&function_ids, &summaries, false).unwrap();
    assert!(result.contains("\"total_functions\": 0"));
}

#[test]
fn test_format_provability_summary_basic() {
    let function_ids = vec![
        create_function_id("src/main.rs", "main", 10),
        create_function_id("src/lib.rs", "helper", 20),
    ];
    let summaries = vec![create_summary(0.9), create_summary(0.6)];

    let result = strip_ansi(&format_provability_summary(&function_ids, &summaries, 5).unwrap());
    assert!(result.contains("Provability Analysis Summary"));
    assert!(result.contains("Total functions analyzed: 2"));
    assert!(result.contains("Score Distribution"));
}

#[test]
fn test_format_provability_summary_empty() {
    let function_ids: Vec<FunctionId> = vec![];
    let summaries: Vec<ProofSummary> = vec![];

    let result = strip_ansi(&format_provability_summary(&function_ids, &summaries, 5).unwrap());
    assert!(result.contains("Total functions analyzed: 0"));
}

#[test]
fn test_format_provability_summary_score_distribution() {
    let function_ids = vec![
        create_function_id("a.rs", "high", 1),
        create_function_id("b.rs", "medium", 2),
        create_function_id("c.rs", "low", 3),
    ];
    let summaries = vec![
        create_summary(0.9),  // High
        create_summary(0.65), // Medium
        create_summary(0.3),  // Low
    ];

    let result = format_provability_summary(&function_ids, &summaries, 5).unwrap();
    assert!(result.contains("High"));
    assert!(result.contains("Medium"));
    assert!(result.contains("Low"));
}

#[test]
fn test_format_provability_detailed_basic() {
    let function_ids = vec![create_function_id("src/main.rs", "main", 10)];
    let summaries = vec![create_summary(0.85)];

    let result = format_provability_detailed(&function_ids, &summaries, false).unwrap();
    assert!(result.contains("Detailed Provability Analysis"));
    assert!(result.contains("main"));
}

#[test]
fn test_format_provability_detailed_with_evidence() {
    let function_ids = vec![create_function_id("test.rs", "secure_fn", 42)];
    let summaries = vec![ProofSummary {
        provability_score: 0.95,
        analysis_time_us: 150,
        verified_properties: vec![
            VerifiedProperty {
                property_type: PropertyType::MemorySafety,
                confidence: 0.98,
                evidence: "All memory safe".to_string(),
            },
            VerifiedProperty {
                property_type: PropertyType::ThreadSafety,
                confidence: 0.92,
                evidence: "No data races".to_string(),
            },
        ],
        version: 1,
    }];

    let result = format_provability_detailed(&function_ids, &summaries, true).unwrap();
    assert!(result.contains("Verified Properties"));
    assert!(result.contains("MemorySafety"));
    assert!(result.contains("ThreadSafety"));
}

#[test]
fn test_format_provability_detailed_empty() {
    let function_ids: Vec<FunctionId> = vec![];
    let summaries: Vec<ProofSummary> = vec![];

    let result = format_provability_detailed(&function_ids, &summaries, true).unwrap();
    assert!(result.contains("Detailed Provability Analysis"));
}

#[test]
fn test_format_provability_sarif_basic() {
    let function_ids = vec![create_function_id("src/main.rs", "main", 10)];
    let summaries = vec![create_summary(0.85)];

    let result = format_provability_sarif(&function_ids, &summaries).unwrap();
    assert!(result.contains("sarif-schema-2.1.0"));
    assert!(result.contains("paiml-provability-analyzer"));
}

#[test]
fn test_format_provability_sarif_all_levels() {
    let function_ids = vec![
        create_function_id("a.rs", "high_fn", 1),
        create_function_id("b.rs", "medium_fn", 2),
        create_function_id("c.rs", "low_fn", 3),
    ];
    let summaries = vec![
        create_summary(0.9),  // High - level: none
        create_summary(0.65), // Medium - level: note
        create_summary(0.3),  // Low - level: warning
    ];

    let result = format_provability_sarif(&function_ids, &summaries).unwrap();
    assert!(result.contains("high-provability"));
    assert!(result.contains("medium-provability"));
    assert!(result.contains("low-provability"));
    assert!(result.contains("\"level\": \"none\""));
    assert!(result.contains("\"level\": \"note\""));
    assert!(result.contains("\"level\": \"warning\""));
}

#[test]
fn test_format_provability_sarif_empty() {
    let function_ids: Vec<FunctionId> = vec![];
    let summaries: Vec<ProofSummary> = vec![];

    let result = format_provability_sarif(&function_ids, &summaries).unwrap();
    assert!(result.contains("sarif-schema"));
    assert!(result.contains("\"results\": []"));
}

// ============================================================
// Edge Case Tests
// ============================================================

#[test]
fn test_boundary_scores() {
    // Test exact boundary values
    let summaries = vec![
        create_summary(0.0),
        create_summary(0.5),
        create_summary(0.8),
        create_summary(1.0),
    ];

    let filtered_all = filter_summaries(&summaries, false);
    assert_eq!(filtered_all.len(), 4);

    let filtered_high = filter_summaries(&summaries, true);
    assert_eq!(filtered_high.len(), 2); // 0.8 and 1.0
}

#[test]
fn test_large_function_list() {
    let function_ids: Vec<FunctionId> = (0..100)
        .map(|i| create_function_id(&format!("file{}.rs", i), &format!("fn{}", i), i))
        .collect();
    let summaries: Vec<ProofSummary> = (0..100).map(|i| create_summary(i as f64 / 100.0)).collect();

    let result = strip_ansi(&format_provability_summary(&function_ids, &summaries, 10).unwrap());
    assert!(result.contains("Total functions analyzed: 100"));
}

#[test]
fn test_special_characters_in_function_names() {
    let function_ids = vec![
        create_function_id("test.rs", "test_fn_with_underscore", 1),
        create_function_id("test.rs", "fn123", 2),
        create_function_id("test.rs", "__internal__", 3),
    ];
    let summaries = vec![
        create_summary(0.8),
        create_summary(0.7),
        create_summary(0.6),
    ];

    let result = format_provability_json(&function_ids, &summaries, false).unwrap();
    assert!(result.contains("test_fn_with_underscore"));
    assert!(result.contains("fn123"));
    assert!(result.contains("__internal__"));
}

#[test]
fn test_unicode_paths() {
    let function_ids = vec![create_function_id("src/\u{00e9}dition.rs", "main", 1)];
    let summaries = vec![create_summary(0.75)];

    let result = format_provability_json(&function_ids, &summaries, false).unwrap();
    // Should handle unicode paths
    assert!(result.contains("provability_analysis"));
}

#[test]
fn test_very_long_evidence() {
    let long_evidence = "a".repeat(10000);
    let summaries = vec![ProofSummary {
        provability_score: 0.9,
        analysis_time_us: 100,
        verified_properties: vec![VerifiedProperty {
            property_type: PropertyType::PureFunction,
            confidence: 0.95,
            evidence: long_evidence.clone(),
        }],
        version: 1,
    }];
    let function_ids = vec![create_function_id("test.rs", "fn", 1)];

    let result = format_provability_json(&function_ids, &summaries, true).unwrap();
    assert!(result.len() > 10000);
}

#[test]
fn test_zero_analysis_time() {
    let summary = ProofSummary {
        provability_score: 0.5,
        analysis_time_us: 0,
        verified_properties: vec![],
        version: 0,
    };
    assert_eq!(summary.analysis_time_us, 0);
}

#[test]
fn test_max_version() {
    let summary = ProofSummary {
        provability_score: 0.5,
        analysis_time_us: 100,
        verified_properties: vec![],
        version: u64::MAX,
    };
    assert_eq!(summary.version, u64::MAX);
}
