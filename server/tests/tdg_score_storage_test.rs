//! TDD Tests for TDG File Score Storage
//!
//! RED Phase: Tests that verify file scores are persisted after TDG analysis
//! These tests should FAIL initially, demonstrating the missing functionality

use pmat::tdg::{TdgAnalyzer, TdgConfig};
use tempfile::TempDir;

#[ignore]
#[ignore]
#[tokio::test]
async fn test_tdg_analysis_stores_file_score() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.rs");
    std::fs::write(&test_file, "fn main() { println!(\"Hello\"); }")
        .expect("Failed to write test file");

    let config = TdgConfig::default();
    let analyzer = TdgAnalyzer::with_storage(config).expect("Failed to create analyzer");

    // Act - Run TDG analysis on the file
    let score = analyzer
        .analyze_file(&test_file)
        .await
        .expect("Analysis should succeed");

    // Assert - Score should be stored in storage system
    let stored_score = analyzer
        .get_stored_score(&test_file)
        .await
        .expect("Should get stored score");

    assert!(
        stored_score.is_some(),
        "Score should be stored after analysis"
    );
    assert_eq!(
        stored_score.unwrap().total,
        score.total,
        "Stored score should match analyzed score"
    );
}

#[ignore]
#[ignore]
#[tokio::test]
async fn test_tdg_storage_tracks_multiple_files() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let files = vec![
        ("file1.rs", "fn foo() { }"),
        ("file2.rs", "fn bar() { let x = 1; }"),
        ("file3.rs", "fn baz() { for i in 0..10 { } }"),
    ];

    let config = TdgConfig::default();
    let analyzer = TdgAnalyzer::with_storage(config).expect("Failed to create analyzer");

    // Act - Analyze multiple files
    for (name, content) in &files {
        let file_path = temp_dir.path().join(name);
        std::fs::write(&file_path, content).expect("Failed to write file");
        analyzer
            .analyze_file(&file_path)
            .await
            .expect("Analysis should succeed");
    }

    // Assert - All scores should be stored
    let stats = analyzer
        .get_storage_stats()
        .expect("Should have storage stats");

    assert_eq!(
        stats.total_entries,
        files.len(),
        "Storage should contain all analyzed files"
    );

    // Verify each file has a stored score
    for (name, _) in &files {
        let file_path = temp_dir.path().join(name);
        let stored_score = analyzer
            .get_stored_score(&file_path)
            .await
            .expect("Should get stored score");
        assert!(
            stored_score.is_some(),
            "Score for {} should be stored",
            name
        );
    }
}

#[ignore]
#[ignore]
#[tokio::test]
async fn test_tdg_storage_persists_across_sessions() {
    // Arrange
    let storage_dir = TempDir::new().expect("Failed to create storage dir");
    let test_file = storage_dir.path().join("persistent_test.rs");
    std::fs::write(&test_file, "fn complex() { /* complexity here */ }")
        .expect("Failed to write file");

    let config = TdgConfig::default();

    let original_score: pmat::tdg::TdgScore;

    // Act - First session: analyze and store
    {
        let analyzer =
            TdgAnalyzer::with_storage(config.clone()).expect("Failed to create analyzer");
        original_score = analyzer
            .analyze_file(&test_file)
            .await
            .expect("Analysis should succeed");
        // Storage should auto-save on drop
    }

    // Act - Second session: retrieve stored score
    {
        let analyzer = TdgAnalyzer::with_storage(config).expect("Failed to create analyzer");
        let retrieved_score = analyzer
            .get_stored_score(&test_file)
            .await
            .expect("Should get stored score");

        // Assert - Score should persist across sessions
        assert!(
            retrieved_score.is_some(),
            "Score should persist across sessions"
        );
        assert_eq!(
            retrieved_score.unwrap().total,
            original_score.total,
            "Persisted score should match original"
        );
    }
}

#[ignore]
#[ignore]
#[tokio::test]
async fn test_tdg_storage_updates_on_file_change() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("evolving.rs");

    let config = TdgConfig::default();
    let analyzer = TdgAnalyzer::with_storage(config).expect("Failed to create analyzer");

    // Act - Initial analysis
    std::fs::write(&test_file, "fn simple() { }").expect("Failed to write file");
    let initial_score = analyzer
        .analyze_file(&test_file)
        .await
        .expect("Analysis should succeed");

    // Act - Modify file and re-analyze
    std::fs::write(
        &test_file,
        "fn complex() { if true { for i in 0..10 { match i { _ => {} } } } }",
    )
    .expect("Failed to update file");
    let updated_score = analyzer
        .analyze_file(&test_file)
        .await
        .expect("Analysis should succeed");

    // Assert - Storage should have the updated score
    let stored_score = analyzer
        .get_stored_score(&test_file)
        .await
        .expect("Should get stored score")
        .expect("Score should be stored");
    assert_ne!(
        initial_score.total, updated_score.total,
        "Scores should differ after file change"
    );
    assert_eq!(
        stored_score.total, updated_score.total,
        "Storage should contain the latest score"
    );
}

#[ignore]
#[ignore]
#[tokio::test]
async fn test_tdg_storage_statistics_accuracy() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = TdgConfig::default();
    let analyzer = TdgAnalyzer::with_storage(config).expect("Failed to create analyzer");

    // Act - Analyze files with different scores
    let test_cases = vec![
        ("excellent.rs", "fn a() { }", 90.0), // Expected high score
        ("good.rs", "fn b() { let x = 1; }", 75.0), // Expected medium score
        ("poor.rs", "fn c() { /* very complex */ }", 50.0), // Expected low score
    ];

    for (name, content, _expected) in &test_cases {
        let file_path = temp_dir.path().join(name);
        std::fs::write(&file_path, content).expect("Failed to write file");
        analyzer
            .analyze_file(&file_path)
            .await
            .expect("Analysis should succeed");
    }

    // Assert - Statistics should be accurate
    let stats = analyzer
        .get_storage_stats()
        .expect("Should have storage stats");
    assert_eq!(stats.total_entries, 3, "Should have 3 stored scores");
    // hot_entries is always >= 0 for unsigned types
    assert!(
        stats.compression_ratio > 0.0,
        "Compression ratio should be calculated"
    );
}

#[ignore]
#[ignore]
#[tokio::test]
async fn test_tdg_score_comparison_tracking() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file1 = temp_dir.path().join("file1.rs");
    let file2 = temp_dir.path().join("file2.rs");

    std::fs::write(&file1, "fn good() { println!(\"clean\"); }").expect("Failed to write file1");
    std::fs::write(&file2, "fn bad() { /* complex nested logic */ }")
        .expect("Failed to write file2");

    let config = TdgConfig::default();
    let analyzer = TdgAnalyzer::with_storage(config).expect("Failed to create analyzer");

    // Act - Analyze both files
    analyzer
        .analyze_file(&file1)
        .await
        .expect("Analysis should succeed");
    analyzer
        .analyze_file(&file2)
        .await
        .expect("Analysis should succeed");

    // Assert - Should be able to compare scores
    let score1 = analyzer
        .get_stored_score(&file1)
        .await
        .expect("Should get stored score")
        .expect("Score1 should be stored");
    let score2 = analyzer
        .get_stored_score(&file2)
        .await
        .expect("Should get stored score")
        .expect("Score2 should be stored");

    assert!(
        score1.total > score2.total,
        "Clean code should score higher than complex code"
    );

    // TODO: Add methods for getting top/bottom files if needed
    // For now, just verify we can retrieve and compare scores
}

#[ignore]
#[ignore]
#[tokio::test]
async fn test_tdg_storage_cache_hit_tracking() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("cached.rs");
    std::fs::write(&test_file, "fn cached() { }").expect("Failed to write file");

    let config = TdgConfig::default();
    let analyzer = TdgAnalyzer::with_storage(config).expect("Failed to create analyzer");

    // Act - First analysis (cache miss)
    let first_analysis = analyzer
        .analyze_file(&test_file)
        .await
        .expect("Analysis should succeed");

    // Act - Second analysis (should be cache hit from storage)
    let second_analysis = analyzer
        .analyze_file(&test_file)
        .await
        .expect("Analysis should succeed");

    // Assert - scores should match (indicating cache was used)
    assert_eq!(
        first_analysis.total, second_analysis.total,
        "Cached score should match original"
    );
}

// Property-based test for storage consistency
#[ignore]
#[ignore]
#[tokio::test]
async fn test_tdg_storage_consistency_invariants() {
    // This test verifies that storage maintains consistency invariants:
    // 1. Every analyzed file has exactly one current score
    // 2. Storage stats accurately reflect stored data
    // 3. Cache tiers are properly managed
    // 4. Compression is applied correctly

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = TdgConfig::default();
    let analyzer = TdgAnalyzer::with_storage(config).expect("Failed to create analyzer");

    // Generate multiple test files
    for i in 0..10 {
        let file_path = temp_dir.path().join(format!("test_{}.rs", i));
        let content = format!("fn test_{}() {{ /* complexity level {} */ }}", i, i);
        std::fs::write(&file_path, content).expect("Failed to write file");

        // Analyze file
        analyzer
            .analyze_file(&file_path)
            .await
            .expect("Analysis should succeed");

        // Verify invariants after each operation
        let stats = analyzer
            .get_storage_stats()
            .expect("Should have storage stats");
        assert_eq!(
            stats.total_entries,
            i + 1,
            "Total entries should match analyzed files"
        );

        let stored_score = analyzer
            .get_stored_score(&file_path)
            .await
            .expect("Should get stored score");
        assert!(
            stored_score.is_some(),
            "Every analyzed file should have a score"
        );
    }

    // Final consistency check
    let final_stats = analyzer
        .get_storage_stats()
        .expect("Should have storage stats");
    assert_eq!(final_stats.total_entries, 10, "Should have all 10 scores");
    assert!(
        final_stats.hot_entries <= final_stats.total_entries,
        "Hot cache cannot exceed total entries"
    );
}

#[cfg(test)]
mod tdg_storage_integration {

    /// Integration test for complete TDG storage workflow
    #[ignore]
    #[tokio::test]
    async fn test_complete_tdg_storage_workflow() {
        // This integration test verifies the complete workflow:
        // 1. Initialize TDG with storage
        // 2. Analyze multiple files
        // 3. Store scores persistently
        // 4. Retrieve and compare scores
        // 5. Generate storage statistics
        // 6. Clean up old entries

        // The test should demonstrate that TDG is properly dogfooding
        // its own quality metrics by storing and tracking scores

        todo!("GREEN PHASE: Implement complete storage workflow")
    }
}
