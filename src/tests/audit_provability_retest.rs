#![cfg_attr(coverage_nightly, coverage(off))]
///! Popperian falsification audit retest - Vector 1.1 (Clone Army) and Vector 1.2 (Empty Shell)
use crate::services::lightweight_provability_analyzer::{FunctionId, LightweightProvabilityAnalyzer};

/// Get project root path for reliable test file resolution
fn project_file(relative: &str) -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    format!("{manifest}/{relative}")
}

/// Vector 1.1: Clone Army - verify 5 real project functions produce spread > 0.01
#[tokio::test]
async fn audit_retest_v1_1_clone_army_spread() {
    let analyzer = LightweightProvabilityAnalyzer::new();

    // 5 real project functions with different profiles
    let functions = vec![
        // Pure fn: compute_confidence (no I/O, no unsafe, no unwrap)
        FunctionId {
            file_path: project_file("src/services/lightweight_provability_analyzer.rs"),
            function_name: "compute_confidence".to_string(),
            line_number: 503,
        },
        // I/O fn: read_function_source (reads files)
        FunctionId {
            file_path: project_file("src/services/lightweight_provability_analyzer.rs"),
            function_name: "read_function_source".to_string(),
            line_number: 391,
        },
        // Async fn: start_dashboard_server (async, network I/O)
        FunctionId {
            file_path: project_file("src/tdg/web_dashboard.rs"),
            function_name: "start_dashboard_server".to_string(),
            line_number: 429,
        },
        // Fn with .expect(): normalize_identifiers (uses regex .expect())
        FunctionId {
            file_path: project_file("src/services/similarity.rs"),
            function_name: "normalize_identifiers".to_string(),
            line_number: 360,
        },
        // Constructor: new() on LightweightProvabilityAnalyzer
        FunctionId {
            file_path: project_file("src/services/lightweight_provability_analyzer.rs"),
            function_name: "new".to_string(),
            line_number: 288,
        },
    ];

    let results = analyzer.analyze_incrementally(&functions).await;
    assert_eq!(results.len(), 5, "Expected 5 results");

    let scores: Vec<f64> = results.iter().map(|r| r.provability_score).collect();
    let min_score = scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_score = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let spread = max_score - min_score;

    println!("=== VECTOR 1.1: Clone Army Retest ===");
    for (i, (func, score)) in functions.iter().zip(scores.iter()).enumerate() {
        println!(
            "  [{i}] {:<40} => {:.4}",
            func.function_name, score
        );
    }
    println!("  min={min_score:.4}, max={max_score:.4}, spread={spread:.4}");

    assert!(
        spread > 0.01,
        "FAIL: spread {spread:.4} <= 0.01 -- scores are clone-army flat"
    );
    println!("  PASS: spread {spread:.4} > 0.01");
}

/// Vector 1.2a: Empty Shell - 50 empty fns should NOT score 1.0
#[tokio::test]
async fn audit_retest_v1_2_empty_shell_low_score() {
    let analyzer = LightweightProvabilityAnalyzer::new();

    // Create temp file with 50 empty functions
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let temp_file = temp_dir.path().join("empty_fns.rs");
    let mut content = String::new();
    for i in 0..50 {
        content.push_str(&format!("fn empty_fn_{i}() {{}}\n"));
    }
    std::fs::write(&temp_file, &content).expect("write temp file");

    let functions: Vec<FunctionId> = (0..50)
        .map(|i| FunctionId {
            file_path: temp_file.to_string_lossy().to_string(),
            function_name: format!("empty_fn_{i}"),
            line_number: i + 1,
        })
        .collect();

    let results = analyzer.analyze_incrementally(&functions).await;
    let avg_score: f64 =
        results.iter().map(|r| r.provability_score).sum::<f64>() / results.len() as f64;

    println!("=== VECTOR 1.2a: Empty Shell (50 empty fns) ===");
    println!("  avg_score = {avg_score:.4}");

    assert!(
        avg_score < 0.5,
        "FAIL: empty fns avg {avg_score:.4} >= 0.5 -- empty shell still scores high"
    );
    println!("  PASS: avg {avg_score:.4} < 0.5 (empty fns are penalized)");
}

/// Vector 1.2b: Nonexistent file FunctionIds should score < 0.5
#[tokio::test]
async fn audit_retest_v1_2_nonexistent_file() {
    let analyzer = LightweightProvabilityAnalyzer::new();

    let functions: Vec<FunctionId> = (0..10)
        .map(|i| FunctionId {
            file_path: "/tmp/this_file_does_not_exist_at_all.rs".to_string(),
            function_name: format!("ghost_fn_{i}"),
            line_number: i * 10,
        })
        .collect();

    let results = analyzer.analyze_incrementally(&functions).await;
    let avg_score: f64 =
        results.iter().map(|r| r.provability_score).sum::<f64>() / results.len() as f64;

    println!("=== VECTOR 1.2b: Nonexistent File ===");
    println!("  avg_score = {avg_score:.4}");
    for (i, r) in results.iter().enumerate() {
        println!("  [{i}] score={:.4}", r.provability_score);
    }

    assert!(
        avg_score < 0.5,
        "FAIL: nonexistent file avg {avg_score:.4} >= 0.5 -- phantom functions scored high"
    );
    println!("  PASS: avg {avg_score:.4} < 0.5 (nonexistent file correctly penalized)");
}
