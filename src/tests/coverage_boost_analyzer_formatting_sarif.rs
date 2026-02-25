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
    let levels: Vec<&str> = results
        .iter()
        .map(|r| r["level"].as_str().unwrap())
        .collect();
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
