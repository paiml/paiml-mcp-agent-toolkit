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
        assert!(
            md.contains(mem),
            "Memory complexity '{mem}' not found in output"
        );
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
