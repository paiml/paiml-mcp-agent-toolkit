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
    root.children
        .push(make_annotated_node("src", NodeType::Directory));
    root.children
        .push(make_annotated_node("README.md", NodeType::File));
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

