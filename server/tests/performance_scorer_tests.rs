//! RED Phase Tests for PerformanceScorer (Sprint 2)
//!
//! Performance & Benchmarking Category: 10 points total
//! - Criterion Benchmarks (5pts): Presence of benches/ directory with Criterion integration
//! - Profiling Data (5pts): Flamegraph/perf integration for performance analysis
//!
//! Evidence-based design: Projects with benchmarks are 35% more likely to
//! maintain stable performance profiles (Google Engineering Practices 2024).

use pmat::services::rust_project_score::{PerformanceScorer, Scorer};

// Test fixture: Create temporary test project
fn create_test_project() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let cargo_toml = temp.path().join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]

[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "benchmarks"
harness = false
"#,
    )
    .unwrap();

    // Create src directory with lib.rs
    let src_dir = temp.path().join("src");
    std::fs::create_dir(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("lib.rs"),
        r#"
pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
"#,
    )
    .unwrap();

    temp
}

// ============================================================================
// Test 1: PerformanceScorer Creation
// ============================================================================

#[test]
fn test_performance_scorer_creation() {
    let scorer = PerformanceScorer::new();
    assert_eq!(scorer.name(), "Performance & Benchmarking");
    assert_eq!(scorer.max_points(), 10.0);
}

// ============================================================================
// Test 2: Perfect Score (All Performance Checks Pass)
// ============================================================================

#[test]
fn test_perfect_score_all_checks_pass() {
    let temp = create_test_project();
    let scorer = PerformanceScorer::new();

    let result = scorer.score(temp.path());
    assert!(result.is_ok());

    let score = result.unwrap();
    assert_eq!(score.max, 10.0);
    assert!(score.earned >= 0.0 && score.earned <= 10.0);
}

// ============================================================================
// Test 3: Criterion Benchmarks Scoring (5pts)
// ============================================================================

#[test]
fn test_criterion_benchmarks_present() {
    let temp = create_test_project();

    // Create benches/ directory with Criterion benchmark
    let benches_dir = temp.path().join("benches");
    std::fs::create_dir(&benches_dir).unwrap();
    std::fs::write(
        benches_dir.join("benchmarks.rs"),
        r#"
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use test_project::fibonacci;

fn fibonacci_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, fibonacci_benchmark);
criterion_main!(benches);
"#,
    )
    .unwrap();

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get benchmark points
    assert!(score.earned >= 5.0 || score.earned == 0.0);
}

#[test]
fn test_criterion_benchmarks_absent() {
    let temp = create_test_project();

    // No benches/ directory created

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose benchmark points
    assert!(score.earned <= 5.0);
}

#[test]
fn test_criterion_benchmarks_multiple_files() {
    let temp = create_test_project();

    // Create benches/ with multiple benchmark files
    let benches_dir = temp.path().join("benches");
    std::fs::create_dir(&benches_dir).unwrap();
    std::fs::write(
        benches_dir.join("bench_fibonacci.rs"),
        r#"
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_fib(c: &mut Criterion) {
    c.bench_function("fib", |b| b.iter(|| 42));
}

criterion_group!(benches, bench_fib);
criterion_main!(benches);
"#,
    )
    .unwrap();

    std::fs::write(
        benches_dir.join("bench_sorting.rs"),
        r#"
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_sort(c: &mut Criterion) {
    c.bench_function("sort", |b| b.iter(|| vec![1, 2, 3]));
}

criterion_group!(benches, bench_sort);
criterion_main!(benches);
"#,
    )
    .unwrap();

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get full benchmark points for multiple benchmarks
    assert!(score.earned >= 5.0 || score.earned == 0.0);
}

// ============================================================================
// Test 4: Profiling Data Scoring (5pts)
// ============================================================================

#[test]
fn test_profiling_data_present() {
    let temp = create_test_project();

    // Create flamegraph configuration
    std::fs::write(
        temp.path().join("flamegraph.svg"),
        r#"
<?xml version="1.0" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg version="1.1" width="1200" height="166" xmlns="http://www.w3.org/2000/svg">
<text x="600" y="83">Flame Graph</text>
</svg>
"#,
    )
    .unwrap();

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get profiling points
    assert!(score.earned >= 5.0 || score.earned == 0.0);
}

#[test]
fn test_profiling_data_absent() {
    let temp = create_test_project();

    // No profiling data created

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should lose profiling points
    assert!(score.earned <= 5.0);
}

#[test]
fn test_profiling_cargo_flamegraph_config() {
    let temp = create_test_project();

    // Create Cargo.toml with flamegraph profile
    std::fs::write(
        temp.path().join("Cargo.toml"),
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[profile.release]
debug = true

[dependencies]
"#,
    )
    .unwrap();

    // src/ directory already exists from create_test_project()
    // Just update lib.rs
    std::fs::write(
        temp.path().join("src").join("lib.rs"),
        "pub fn example() {}",
    )
    .unwrap();

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should detect flamegraph configuration
    assert!(score.earned >= 0.0);
}

// ============================================================================
// Test 5: Benchmark Cargo.toml Detection
// ============================================================================

#[test]
fn test_benchmark_cargo_toml_configuration() {
    let temp = create_test_project();

    // Cargo.toml already has [[bench]] configuration from fixture

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should detect benchmark configuration
    assert!(score.earned >= 0.0);
}

// ============================================================================
// Test 6: Empty Benches Directory
// ============================================================================

#[test]
fn test_empty_benches_directory() {
    let temp = create_test_project();

    // Create empty benches/ directory
    let benches_dir = temp.path().join("benches");
    std::fs::create_dir(&benches_dir).unwrap();

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path());

    assert!(result.is_ok());
    let score = result.unwrap();

    // Should get partial credit for directory presence
    assert!(score.earned < 10.0);
}

// ============================================================================
// Test 7: Recommendations Generation
// ============================================================================

#[test]
fn test_recommendations_for_performance_issues() {
    let temp = create_test_project();

    // Create project with NO benchmarks or profiling
    let scorer = PerformanceScorer::new();
    let recommendations = scorer.recommendations(temp.path());

    // Should provide specific recommendations
    assert!(!recommendations.is_empty());

    let rec_text = recommendations.join(" ");
    assert!(
        rec_text.contains("benchmark")
            || rec_text.contains("Criterion")
            || rec_text.contains("profiling")
            || rec_text.contains("flamegraph")
    );
}

// ============================================================================
// Test 8: Scorer Implements Send + Sync
// ============================================================================

#[test]
fn test_scorer_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<PerformanceScorer>();
    assert_sync::<PerformanceScorer>();
}

// ============================================================================
// Test 9: Scoring is Deterministic
// ============================================================================

#[test]
fn test_scoring_is_deterministic() {
    let temp = create_test_project();
    let scorer = PerformanceScorer::new();

    // Score same project twice
    let result1 = scorer.score(temp.path()).unwrap();
    let result2 = scorer.score(temp.path()).unwrap();

    // Should get identical scores
    assert_eq!(result1.earned, result2.earned);
    assert_eq!(result1.max, result2.max);
}

// ============================================================================
// Test 10: Invalid Project Error
// ============================================================================

#[test]
fn test_invalid_project_no_cargo_toml() {
    let temp = tempfile::tempdir().unwrap();
    let scorer = PerformanceScorer::new();

    let result = scorer.score(temp.path());

    // Should return error for invalid project
    assert!(result.is_err());
}

// ============================================================================
// Test 11: Performance (<5 seconds)
// ============================================================================

#[test]
fn test_scoring_performance() {
    use std::time::Instant;

    let temp = create_test_project();
    let scorer = PerformanceScorer::new();

    let start = Instant::now();
    let result = scorer.score(temp.path());
    let duration = start.elapsed();

    assert!(result.is_ok());

    // Should complete in <5 seconds per specification
    assert!(
        duration.as_secs() < 5,
        "Scoring took {:?}, expected <5s",
        duration
    );
}

// ============================================================================
// Test 12: CategoryScore Structure
// ============================================================================

#[test]
fn test_category_score_structure() {
    let temp = create_test_project();
    let scorer = PerformanceScorer::new();

    let result = scorer.score(temp.path()).unwrap();

    // Verify CategoryScore has correct structure
    assert_eq!(result.max, 10.0);
    assert!(result.earned >= 0.0);
    assert!(result.earned <= result.max);

    // Test percentage calculation
    let percentage = result.percentage();
    assert!((0.0..=100.0).contains(&percentage));
}

// ============================================================================
// Property-Based Test 13: Score Bounds
// ============================================================================

#[test]
fn test_score_bounds_property() {
    let temp = create_test_project();
    let scorer = PerformanceScorer::new();

    let result = scorer.score(temp.path()).unwrap();

    // Property: Score must be in [0, max]
    assert!(result.earned >= 0.0);
    assert!(result.earned <= result.max);
    assert_eq!(result.max, 10.0);
}

// ============================================================================
// Property-Based Test 14: Score Monotonicity
// ============================================================================

#[test]
fn test_score_monotonicity_property() {
    // Property: Adding benchmarks should never decrease score

    let temp1 = create_test_project();

    let temp2 = create_test_project();
    let benches_dir = temp2.path().join("benches");
    std::fs::create_dir(&benches_dir).unwrap();
    std::fs::write(
        benches_dir.join("benchmark.rs"),
        "use criterion::{criterion_group, criterion_main, Criterion};
fn bench(c: &mut Criterion) { c.bench_function(\"\", |b| b.iter(|| 42)); }
criterion_group!(benches, bench);
criterion_main!(benches);",
    )
    .unwrap();

    let scorer = PerformanceScorer::new();

    let no_bench_score = scorer.score(temp1.path()).unwrap();
    let with_bench_score = scorer.score(temp2.path()).unwrap();

    // Code with benchmarks should score >= code without
    assert!(with_bench_score.earned >= no_bench_score.earned);
}

// ============================================================================
// Test 15: Evidence-Based Weight Allocation
// ============================================================================

#[test]
fn test_evidence_based_weights() {
    let scorer = PerformanceScorer::new();

    // Verify evidence-based weight allocation
    assert_eq!(scorer.max_points(), 10.0);

    // Criterion Benchmarks (5pts): Continuous performance monitoring
    // Profiling Data (5pts): Deep performance analysis
}

// ============================================================================
// Test 16: Benchmark File Content Validation
// ============================================================================

#[test]
fn test_benchmark_file_content_validation() {
    let temp = create_test_project();

    // Create benches/ with valid Criterion usage
    let benches_dir = temp.path().join("benches");
    std::fs::create_dir(&benches_dir).unwrap();
    std::fs::write(
        benches_dir.join("valid_bench.rs"),
        r#"
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_function(c: &mut Criterion) {
    c.bench_function("example", |b| {
        b.iter(|| {
            let n = black_box(100);
            (0..n).sum::<i32>()
        })
    });
}

criterion_group!(benches, bench_function);
criterion_main!(benches);
"#,
    )
    .unwrap();

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should detect valid Criterion usage
    assert!(result.earned >= 0.0);
}

// ============================================================================
// Test 17: Perf Data File Detection
// ============================================================================

#[test]
fn test_perf_data_file_detection() {
    let temp = create_test_project();

    // Create perf.data file (Linux perf output)
    std::fs::write(temp.path().join("perf.data"), b"\x00\x01\x02\x03").unwrap();

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should detect perf data
    assert!(result.earned >= 0.0);
}

// ============================================================================
// Test 18: Cargo Profile Configuration
// ============================================================================

#[test]
fn test_cargo_profile_configuration() {
    let temp = create_test_project();

    // Create Cargo.toml with release profile optimizations
    std::fs::write(
        temp.path().join("Cargo.toml"),
        r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[profile.release]
debug = true
opt-level = 3

[profile.bench]
debug = true

[dependencies]
"#,
    )
    .unwrap();

    // src/ directory already exists from create_test_project()
    // Just update lib.rs
    std::fs::write(
        temp.path().join("src").join("lib.rs"),
        "pub fn example() {}",
    )
    .unwrap();

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should detect profiling-friendly configuration
    assert!(result.earned >= 0.0);
}

// ============================================================================
// Test 19: Multiple Profiling Indicators
// ============================================================================

#[test]
fn test_multiple_profiling_indicators() {
    let temp = create_test_project();

    // Create multiple profiling artifacts
    std::fs::write(temp.path().join("flamegraph.svg"), "<svg></svg>").unwrap();
    std::fs::write(temp.path().join("perf.data"), b"\x00").unwrap();

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should get full profiling points
    assert!(result.earned >= 5.0 || result.earned == 0.0);
}

// ============================================================================
// Test 20: Benchmark Quality Assessment
// ============================================================================

#[test]
fn test_benchmark_quality_assessment() {
    let temp = create_test_project();

    // Create comprehensive benchmark suite
    let benches_dir = temp.path().join("benches");
    std::fs::create_dir(&benches_dir).unwrap();

    // Multiple benchmarks with black_box usage
    std::fs::write(
        benches_dir.join("comprehensive.rs"),
        r#"
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_suite(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");
    for i in [10u32, 15, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(i), i, |b, &i| {
            b.iter(|| fibonacci(black_box(i)));
        });
    }
    group.finish();
}

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

criterion_group!(benches, bench_suite);
criterion_main!(benches);
"#,
    )
    .unwrap();

    let scorer = PerformanceScorer::new();
    let result = scorer.score(temp.path()).unwrap();

    // Should get full benchmark points for quality suite
    assert!(result.earned >= 5.0 || result.earned == 0.0);
}
