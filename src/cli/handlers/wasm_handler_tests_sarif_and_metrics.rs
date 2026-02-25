// ==================== SARIF Format Tests ====================

#[test]
fn test_format_sarif_empty() {
    let vulns: Vec<VulnerabilityMatch> = vec![];
    let result = format_sarif(&vulns);

    assert!(result.is_ok());
    let output = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(parsed["$schema"].as_str().unwrap().contains("sarif-schema"));
    assert_eq!(parsed["version"].as_str().unwrap(), "2.1.0");
}

#[test]
fn test_format_sarif_with_vulnerabilities() {
    let vulns = mock_security_results_with_critical();
    let result = format_sarif(&vulns);

    assert!(result.is_ok());
    let output = result.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    let runs = parsed["runs"].as_array().unwrap();
    assert!(!runs.is_empty());

    let results = runs[0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_create_sarif_output() {
    let vulns = mock_security_results_no_critical();
    let sarif = create_sarif_output(&vulns);

    assert!(sarif["$schema"].is_string());
    assert_eq!(sarif["version"].as_str().unwrap(), "2.1.0");
    assert!(sarif["runs"].is_array());
}

#[test]
fn test_create_sarif_rules() {
    let vulns = mock_security_results_no_critical();
    let rules = create_sarif_rules(&vulns);

    // Should have 2 unique patterns
    assert_eq!(rules.len(), 2);
}

#[test]
fn test_create_sarif_rules_deduplication() {
    let vulns = vec![
        VulnerabilityMatch {
            pattern: "same-pattern".to_string(),
            location: 0..10,
            severity: Severity::High,
            operator_index: 1,
        },
        VulnerabilityMatch {
            pattern: "same-pattern".to_string(),
            location: 10..20,
            severity: Severity::High,
            operator_index: 2,
        },
    ];
    let rules = create_sarif_rules(&vulns);

    // Should deduplicate to 1 rule
    assert_eq!(rules.len(), 1);
}

#[test]
fn test_create_sarif_rule() {
    let rule = create_sarif_rule("test-pattern");

    assert_eq!(rule["id"].as_str().unwrap(), "test-pattern");
    assert_eq!(rule["name"].as_str().unwrap(), "test-pattern");
    assert!(rule["shortDescription"]["text"]
        .as_str()
        .unwrap()
        .contains("test-pattern"));
}

#[test]
fn test_create_sarif_results() {
    let vulns = mock_security_results_no_critical();
    let results = create_sarif_results(&vulns);

    assert_eq!(results.len(), 2);
}

#[test]
fn test_create_sarif_result_critical() {
    let vuln = VulnerabilityMatch {
        pattern: "critical-vuln".to_string(),
        location: 0..100,
        severity: Severity::Critical,
        operator_index: 42,
    };
    let result = create_sarif_result(&vuln);

    assert_eq!(result["ruleId"].as_str().unwrap(), "critical-vuln");
    assert_eq!(result["level"].as_str().unwrap(), "error");
}

#[test]
fn test_create_sarif_result_high() {
    let vuln = VulnerabilityMatch {
        pattern: "high-vuln".to_string(),
        location: 0..100,
        severity: Severity::High,
        operator_index: 42,
    };
    let result = create_sarif_result(&vuln);

    assert_eq!(result["level"].as_str().unwrap(), "error");
}

#[test]
fn test_create_sarif_result_medium() {
    let vuln = VulnerabilityMatch {
        pattern: "medium-vuln".to_string(),
        location: 0..100,
        severity: Severity::Medium,
        operator_index: 42,
    };
    let result = create_sarif_result(&vuln);

    assert_eq!(result["level"].as_str().unwrap(), "warning");
}

#[test]
fn test_create_sarif_result_low() {
    let vuln = VulnerabilityMatch {
        pattern: "low-vuln".to_string(),
        location: 0..100,
        severity: Severity::Low,
        operator_index: 42,
    };
    let result = create_sarif_result(&vuln);

    assert_eq!(result["level"].as_str().unwrap(), "note");
}

// ==================== create_metrics_from_analysis Tests ====================

#[test]
fn test_create_metrics_from_analysis() {
    let analysis = mock_analysis_result();
    let metrics = create_metrics_from_analysis(&analysis);

    assert_eq!(metrics.function_count, 10);
    assert_eq!(metrics.instruction_count, 500);
    assert_eq!(metrics.binary_size, 2048);
    // complexity_p90 = max_complexity - 2 = 15 - 2 = 13
    assert_eq!(metrics.complexity_p90, 13);
    // complexity_p95 = max_complexity = 15
    assert_eq!(metrics.complexity_p95, 15);
    // complexity_p99 = max_complexity + 2 = 17
    assert_eq!(metrics.complexity_p99, 17);
    // memory_usage_mb = (memory_pages * 64) / 1024 = (2 * 64) / 1024 = 0
    assert_eq!(metrics.memory_usage_mb, 0);
    assert_eq!(metrics.init_time_ms, 10); // Default estimate
}

#[test]
fn test_create_metrics_from_analysis_edge_cases() {
    let analysis = AnalysisResult {
        function_count: 0,
        instruction_count: 0,
        binary_size: 0,
        memory_pages: 0,
        max_complexity: 0,
    };
    let metrics = create_metrics_from_analysis(&analysis);

    // Test saturating subtraction: 0 - 2 should be 0 not underflow
    assert_eq!(metrics.complexity_p90, 0);
    assert_eq!(metrics.complexity_p95, 0);
    assert_eq!(metrics.complexity_p99, 2);
}

#[test]
fn test_create_metrics_from_analysis_large_memory() {
    let analysis = AnalysisResult {
        function_count: 100,
        instruction_count: 10000,
        binary_size: 1_000_000,
        memory_pages: 256, // 256 pages = 16 MB
        max_complexity: 50,
    };
    let metrics = create_metrics_from_analysis(&analysis);

    // memory_usage_mb = (256 * 64) / 1024 = 16384 / 1024 = 16
    assert_eq!(metrics.memory_usage_mb, 16);
}
