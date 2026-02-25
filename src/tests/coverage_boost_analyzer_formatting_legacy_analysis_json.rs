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
    ctx.analyses.cross_language_refs = vec![make_cross_lang_ref("binding.rs", "lib.py", 0.85)];
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
        make_recommendation(
            "Fix critical bug",
            Priority::Critical,
            Impact::High,
            vec!["Hotfix"],
        ),
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
