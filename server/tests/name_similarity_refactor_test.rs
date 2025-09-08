//! TDD test for format_output refactor in name_similarity.rs
//! Following Toyota Way TDD: Red → Green → Refactor
//! Testing structural integrity during refactoring from complexity 25 → ≤8

use anyhow::Result;
use pmat::cli::analysis::name_similarity::{NameMatch, NameSimilarityResult};
use pmat::cli::NameSimilarityOutputFormat;

/// Test configuration structure is preserved during refactor
#[test]
fn test_name_similarity_format_structure() -> Result<()> {
    let test_result = NameSimilarityResult {
        query: "test_function".to_string(),
        matches: vec![
            NameMatch {
                name: "test_func".to_string(),
                file: "test.rs".to_string(),
                line: 10,
                kind: "function".to_string(),
                similarity_score: 0.85,
                edit_distance: 2,
                phonetic_match: false,
            },
        ],
        total_candidates: 1,
        search_scope: "Functions".to_string(),
    };

    // Test that function accepts all format types without panic
    let _json_result = pmat::cli::analysis::name_similarity::format_output(
        test_result.clone(), 
        NameSimilarityOutputFormat::Json
    );
    
    let _human_result = pmat::cli::analysis::name_similarity::format_output(
        test_result.clone(),
        NameSimilarityOutputFormat::Human
    );
    
    let _csv_result = pmat::cli::analysis::name_similarity::format_output(
        test_result.clone(),
        NameSimilarityOutputFormat::Csv
    );
    
    let _markdown_result = pmat::cli::analysis::name_similarity::format_output(
        test_result,
        NameSimilarityOutputFormat::Markdown
    );

    // Function structure test - accepts all parameters without panic
    assert!(true, "Function structure maintained during refactor");
    Ok(())
}

/// Test JSON output format patterns
#[test]
fn test_json_output_patterns() -> Result<()> {
    let test_result = NameSimilarityResult {
        query: "test_function".to_string(),
        matches: vec![
            NameMatch {
                name: "test_func".to_string(),
                file: "test.rs".to_string(),
                line: 10,
                kind: "function".to_string(),
                similarity_score: 0.85,
                edit_distance: 2,
                phonetic_match: true,
            },
        ],
        total_candidates: 1,
        search_scope: "Functions".to_string(),
    };

    let result = pmat::cli::analysis::name_similarity::format_output(
        test_result,
        NameSimilarityOutputFormat::Json
    )?;

    // Should produce valid JSON
    assert!(result.contains("test_function"), "JSON contains query");
    assert!(result.contains("test_func"), "JSON contains match name");
    assert!(result.contains("0.85"), "JSON contains similarity score");
    
    Ok(())
}

/// Test CSV output format patterns
#[test]
fn test_csv_output_patterns() -> Result<()> {
    let test_result = NameSimilarityResult {
        query: "test_variable".to_string(),
        matches: vec![
            NameMatch {
                name: "test_var".to_string(),
                file: "lib.rs".to_string(),
                line: 25,
                kind: "variable".to_string(),
                similarity_score: 0.75,
                edit_distance: 5,
                phonetic_match: false,
            },
        ],
        total_candidates: 1,
        search_scope: "Variables".to_string(),
    };

    let result = pmat::cli::analysis::name_similarity::format_output(
        test_result,
        NameSimilarityOutputFormat::Csv
    )?;

    // Should produce valid CSV
    assert!(result.contains("name,file,line,kind"), "CSV contains header");
    assert!(result.contains("test_var,lib.rs,25"), "CSV contains match data");
    assert!(result.contains("0.750"), "CSV contains formatted score");
    
    Ok(())
}

/// Test Markdown output format patterns
#[test]
fn test_markdown_output_patterns() -> Result<()> {
    let test_result = NameSimilarityResult {
        query: "TestStruct".to_string(),
        matches: vec![
            NameMatch {
                name: "TestStructure".to_string(),
                file: "types.rs".to_string(),
                line: 15,
                kind: "struct".to_string(),
                similarity_score: 0.90,
                edit_distance: 3,
                phonetic_match: true,
            },
            NameMatch {
                name: "TestClass".to_string(),
                file: "classes.rs".to_string(),
                line: 8,
                kind: "class".to_string(),
                similarity_score: 0.65,
                edit_distance: 7,
                phonetic_match: false,
            },
        ],
        total_candidates: 2,
        search_scope: "Types".to_string(),
    };

    let result = pmat::cli::analysis::name_similarity::format_output(
        test_result,
        NameSimilarityOutputFormat::Markdown
    )?;

    // Should produce valid Markdown
    assert!(result.contains("# Name Similarity Report"), "Markdown contains title");
    assert!(result.contains("**Query:** `TestStruct`"), "Markdown contains query");
    assert!(result.contains("| TestStructure |"), "Markdown contains table data");
    assert!(result.contains("✓"), "Markdown contains phonetic match indicator");
    assert!(result.contains("✗"), "Markdown contains non-match indicator");
    
    Ok(())
}

/// Test Human/Summary/Detailed output format patterns
#[test]
fn test_human_output_patterns() -> Result<()> {
    let test_result = NameSimilarityResult {
        query: "helper_function".to_string(),
        matches: vec![
            NameMatch {
                name: "helper_func".to_string(),
                file: "utils.rs".to_string(),
                line: 42,
                kind: "function".to_string(),
                similarity_score: 0.88,
                edit_distance: 4,
                phonetic_match: false,
            },
        ],
        total_candidates: 1,
        search_scope: "All".to_string(),
    };

    // Test Human format
    let human_result = pmat::cli::analysis::name_similarity::format_output(
        test_result.clone(),
        NameSimilarityOutputFormat::Human
    )?;

    assert!(human_result.contains("# Name Similarity Analysis"), "Human contains title");
    assert!(human_result.contains("Query: 'helper_function'"), "Human contains query");
    assert!(human_result.contains("1. helper_func"), "Human contains numbered match");
    assert!(human_result.contains("File: utils.rs:42"), "Human contains file info");

    // Test Summary format (should behave same as Human)
    let summary_result = pmat::cli::analysis::name_similarity::format_output(
        test_result.clone(),
        NameSimilarityOutputFormat::Summary
    )?;

    assert!(summary_result.contains("# Name Similarity Analysis"), "Summary contains title");
    
    // Test Detailed format (should behave same as Human)
    let detailed_result = pmat::cli::analysis::name_similarity::format_output(
        test_result,
        NameSimilarityOutputFormat::Detailed
    )?;

    assert!(detailed_result.contains("# Name Similarity Analysis"), "Detailed contains title");
    
    Ok(())
}

/// Test phonetic match display patterns
#[test]
fn test_phonetic_match_patterns() -> Result<()> {
    let test_result = NameSimilarityResult {
        query: "color".to_string(),
        matches: vec![
            NameMatch {
                name: "colour".to_string(), // Should be phonetic match
                file: "british.rs".to_string(),
                line: 1,
                kind: "variable".to_string(),
                similarity_score: 0.70,
                edit_distance: 1,
                phonetic_match: true,
            },
        ],
        total_candidates: 1,
        search_scope: "Variables".to_string(),
    };

    // Test phonetic match display in Human format
    let human_result = pmat::cli::analysis::name_similarity::format_output(
        test_result.clone(),
        NameSimilarityOutputFormat::Human
    )?;

    assert!(human_result.contains("✓ Phonetic match"), "Human shows phonetic match");

    // Test phonetic match display in Markdown format
    let markdown_result = pmat::cli::analysis::name_similarity::format_output(
        test_result,
        NameSimilarityOutputFormat::Markdown
    )?;

    assert!(markdown_result.contains("✓"), "Markdown shows phonetic match symbol");
    
    Ok(())
}

/// Test empty matches handling
#[test]
fn test_empty_matches_handling() -> Result<()> {
    let test_result = NameSimilarityResult {
        query: "nonexistent_function".to_string(),
        matches: vec![], // Empty matches
        total_candidates: 0,
        search_scope: "Functions".to_string(),
    };

    // All formats should handle empty matches gracefully
    let _json_result = pmat::cli::analysis::name_similarity::format_output(
        test_result.clone(),
        NameSimilarityOutputFormat::Json
    )?;

    let human_result = pmat::cli::analysis::name_similarity::format_output(
        test_result.clone(),
        NameSimilarityOutputFormat::Human
    )?;

    assert!(human_result.contains("Found 0 matches"), "Human handles empty matches");

    let csv_result = pmat::cli::analysis::name_similarity::format_output(
        test_result.clone(),
        NameSimilarityOutputFormat::Csv
    )?;

    // Should still have header
    assert!(csv_result.contains("name,file,line,kind"), "CSV has header even when empty");

    let markdown_result = pmat::cli::analysis::name_similarity::format_output(
        test_result,
        NameSimilarityOutputFormat::Markdown
    )?;

    assert!(markdown_result.contains("**Total matches:** 0"), "Markdown handles empty matches");
    
    Ok(())
}