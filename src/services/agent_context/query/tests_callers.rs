// Caller summarization and production caller cap tests.
// Lines 786–906 of the original tests.rs.

#[test]
fn test_called_by_test_summarization() {
    let mut index = build_test_index();
    // Simulate many test callers for function 0
    let mut callers = vec![1usize]; // one production caller
    for i in 10..25 {
        // Add fake test function indices
        index.functions.push(FunctionEntry {
            file_path: "tests/t.rs".to_string(),
            function_name: format!("test_case_{i}"),
            signature: format!("fn test_case_{i}()"),
            doc_comment: None,
            source: "fn test() {}".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: format!("t{i}"),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        });
        callers.push(index.functions.len() - 1);
    }
    index.called_by.insert(0, callers);
    // Rebuild name_index for the new functions
    for (i, f) in index.functions.iter().enumerate() {
        index
            .name_index
            .entry(f.function_name.clone())
            .or_default()
            .push(i);
    }

    let result = QueryResult::from_entry_with_context(&index.functions[0], 0, &index, 0.9, false);
    // Should have production caller + test summary
    assert!(result.called_by.iter().any(|s| s.contains("tests)")));
    // Should not list individual test_case_N names
    assert!(!result.called_by.iter().any(|s| s.starts_with("test_case_")));
}

#[test]
fn test_called_by_production_cap() {
    let mut index = build_test_index();
    // Simulate >10 production callers for function 0
    let mut callers = Vec::new();
    for i in 10..25 {
        index.functions.push(FunctionEntry {
            file_path: "src/callers.rs".to_string(),
            function_name: format!("caller_{i}"),
            signature: format!("fn caller_{i}()"),
            doc_comment: None,
            source: "fn caller() {}".to_string(),
            start_line: 1,
            end_line: 1,
            language: "Rust".to_string(),
            quality: QualityMetrics::default(),
            checksum: format!("c{i}"),
            definition_type: DefinitionType::default(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: Vec::new(),
        });
        callers.push(index.functions.len() - 1);
        index
            .name_index
            .entry(format!("caller_{i}"))
            .or_default()
            .push(index.functions.len() - 1);
    }
    index.called_by.insert(0, callers);

    let result = QueryResult::from_entry_with_context(&index.functions[0], 0, &index, 0.9, false);
    // Should cap at 10 + "(+N more)"
    assert!(result.called_by.iter().any(|s| s.contains("more)")));
    // Total entries should be 10 visible + 1 summary = 11
    assert!(result.called_by.len() <= 12);
}
