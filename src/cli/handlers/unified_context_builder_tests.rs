// Sync tests for UnifiedContextBuilder
// Included via include!() in unified_context_builder.rs mod tests

// ============================================================================
// UnifiedContextBuilder basic tests
// ============================================================================

#[test]
fn test_builder_new() {
    let temp = create_temp_project();
    let builder = UnifiedContextBuilder::new(temp.path());
    assert!(builder.output.is_empty());
}

#[test]
fn test_builder_add_basic_structure() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_basic_structure();

    let output = builder.build();
    assert!(output.contains("# Project Context"));
    assert!(output.contains("## Project Structure"));
    assert!(output.contains("**Language**"));
    assert!(output.contains("**Total Files**"));
    assert!(output.contains("**Total Functions**"));
    assert!(output.contains("**Total Structs**"));
    assert!(output.contains("**Total Enums**"));
    assert!(output.contains("**Total Traits**"));
}

#[test]
fn test_builder_add_big_o_analysis() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_big_o_analysis();

    let output = builder.build();
    assert!(output.contains("## Big-O Complexity Analysis"));
    assert!(output.contains("O(n)"));
    assert!(output.contains("O(n log n)"));
    assert!(output.contains("O(n\u{b2})"));
}

#[test]
fn test_builder_add_entropy_analysis() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_entropy_analysis();

    let output = builder.build();
    assert!(output.contains("## Entropy Analysis"));
    assert!(output.contains("Pattern Entropy"));
    assert!(output.contains("Code Duplication"));
    assert!(output.contains("Structural Entropy"));
    assert!(output.contains("Actionable Improvements"));
}

#[test]
fn test_builder_add_tdg_analysis() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_tdg_analysis();

    let output = builder.build();
    assert!(output.contains("## Technical Debt Gradient (TDG)"));
    assert!(output.contains("Overall TDG Score"));
    assert!(output.contains("File-level TDG"));
    assert!(output.contains("Debt Hotspots"));
    assert!(output.contains("Refactoring Priority"));
}

#[test]
fn test_builder_chaining() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder
        .add_basic_structure()
        .add_big_o_analysis()
        .add_entropy_analysis()
        .add_tdg_analysis();
    let output = builder.build();

    assert!(output.contains("# Project Context"));
    assert!(output.contains("## Big-O Complexity Analysis"));
    assert!(output.contains("## Entropy Analysis"));
    assert!(output.contains("## Technical Debt Gradient"));
}

#[test]
fn test_builder_display() {
    let temp = create_temp_project();
    let mut builder = UnifiedContextBuilder::new(temp.path());
    builder.add_basic_structure();

    let display_output = format!("{}", builder);
    assert!(display_output.contains("# Project Context"));
}

// ============================================================================
// DeadCodeAnalysis tests
// ============================================================================

#[test]
fn test_dead_code_analysis_is_empty_when_all_empty() {
    let analysis = DeadCodeAnalysis {
        unreachable_functions: vec![],
        unused_variables: vec![],
        unused_imports: vec![],
        dead_branches: vec![],
    };
    assert!(analysis.is_empty());
}

#[test]
fn test_dead_code_analysis_is_not_empty_with_unreachable_functions() {
    let analysis = DeadCodeAnalysis {
        unreachable_functions: vec!["unused_fn".to_string()],
        unused_variables: vec![],
        unused_imports: vec![],
        dead_branches: vec![],
    };
    assert!(!analysis.is_empty());
}

#[test]
fn test_dead_code_analysis_is_not_empty_with_unused_variables() {
    let analysis = DeadCodeAnalysis {
        unreachable_functions: vec![],
        unused_variables: vec!["x".to_string()],
        unused_imports: vec![],
        dead_branches: vec![],
    };
    assert!(!analysis.is_empty());
}

#[test]
fn test_dead_code_analysis_is_not_empty_with_unused_imports() {
    let analysis = DeadCodeAnalysis {
        unreachable_functions: vec![],
        unused_variables: vec![],
        unused_imports: vec!["std::io".to_string()],
        dead_branches: vec![],
    };
    assert!(!analysis.is_empty());
}

#[test]
fn test_dead_code_analysis_is_not_empty_with_dead_branches() {
    let analysis = DeadCodeAnalysis {
        unreachable_functions: vec![],
        unused_variables: vec![],
        unused_imports: vec![],
        dead_branches: vec!["line 42".to_string()],
    };
    assert!(!analysis.is_empty());
}

// ============================================================================
// Analysis struct tests
// ============================================================================

#[test]
fn test_big_o_analysis_creation() {
    let mut complexities = HashMap::new();
    complexities.insert("sort".to_string(), "O(n log n)".to_string());
    let analysis = BigOAnalysis { complexities };
    assert_eq!(analysis.complexities.len(), 1);
}

#[test]
fn test_entropy_analysis_creation() {
    let analysis = EntropyAnalysis {
        pattern_entropy: 0.75,
        duplication_percentage: 15.0,
        structural_entropy: 0.65,
        actionable_improvements: vec!["Improve code".to_string()],
    };
    assert_eq!(analysis.pattern_entropy, 0.75);
    assert_eq!(analysis.duplication_percentage, 15.0);
    assert_eq!(analysis.structural_entropy, 0.65);
    assert_eq!(analysis.actionable_improvements.len(), 1);
}

#[test]
fn test_provability_analysis_creation() {
    let analysis = ProvabilityAnalysis {
        invariants: vec!["x > 0".to_string()],
        preconditions: vec!["input != null".to_string()],
        postconditions: vec!["result >= 0".to_string()],
        is_sound: true,
        is_complete: false,
    };
    assert!(analysis.is_sound);
    assert!(!analysis.is_complete);
    assert_eq!(analysis.invariants.len(), 1);
}

#[test]
fn test_graph_metrics_analysis_creation() {
    let analysis = GraphMetricsAnalysis {
        betweenness: 0.5,
        closeness: 0.7,
        degree: 0.3,
        node_count: 100,
        edge_count: 200,
        cyclomatic: 15,
        critical_paths: vec!["A -> B -> C".to_string()],
    };
    assert_eq!(analysis.node_count, 100);
    assert_eq!(analysis.edge_count, 200);
    assert_eq!(analysis.cyclomatic, 15);
}

#[test]
fn test_tdg_analysis_creation() {
    let analysis = TdgAnalysis {
        overall_score: 3.5,
        file_scores: HashMap::new(),
        hotspots: vec![],
        priorities: vec!["Refactor utils.rs".to_string()],
    };
    assert_eq!(analysis.overall_score, 3.5);
    assert!(analysis.hotspots.is_empty());
}

#[test]
fn test_tdg_hotspot_creation() {
    let hotspot = TdgHotspot {
        location: "main.rs:42".to_string(),
        score: 4.5,
    };
    assert_eq!(hotspot.location, "main.rs:42");
    assert_eq!(hotspot.score, 4.5);
}

#[test]
fn test_satd_comment_creation() {
    let comment = SatdComment {
        location: "src/lib.rs:10".to_string(),
        comment: "TODO: fix this".to_string(),
    };
    assert_eq!(comment.location, "src/lib.rs:10");
    assert!(comment.comment.contains("TODO"));
}

#[test]
fn test_satd_analysis_creation() {
    let analysis = SatdAnalysis {
        todos: vec![SatdComment {
            location: "file.rs:1".to_string(),
            comment: "TODO".to_string(),
        }],
        fixmes: vec![],
        hacks: vec![],
        tech_debt: vec![],
        design_debt_count: 1,
        code_debt_count: 2,
        test_debt_count: 3,
        doc_debt_count: 0,
    };
    assert_eq!(analysis.todos.len(), 1);
    assert_eq!(analysis.design_debt_count, 1);
    assert_eq!(analysis.code_debt_count, 2);
}

// ============================================================================
// Error enum tests
// ============================================================================

#[test]
fn test_error_not_implemented() {
    let err = Error::NotImplemented;
    assert!(matches!(err, Error::NotImplemented));
}

#[test]
fn test_error_analysis_failed() {
    let err = Error::AnalysisFailed("test error".to_string());
    if let Error::AnalysisFailed(msg) = err {
        assert_eq!(msg, "test error");
    } else {
        panic!("Expected AnalysisFailed variant");
    }
}
