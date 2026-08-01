// Tests for `extract_ruchy_pipeline_patterns` in pattern_extractor_ruchy.rs —
// isolated into its own file so `pattern_extractor_tests.rs` stays under the
// 500-line pre-commit gate (it's already at 982; any growth is blocked).
//
// Attached via `#[path]` + `mod` from pattern_extractor.rs, so this file IS
// the module body — use `super::*;` like pattern_extractor_tests.rs does.

use super::*;
use std::path::PathBuf;

fn run_pipeline(content: &str) -> PatternCollection {
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let file_path = PathBuf::from("pipeline.ruchy");
    let mut collection = PatternCollection::new();
    extractor
        .extract_ruchy_pipeline_patterns(&file_path, content, &mut collection)
        .expect("extract_ruchy_pipeline_patterns should not error");
    collection
}

#[test]
fn four_pipeline_ops_create_data_transformation_pattern() {
    // matches.len() > 3 branch: exactly 4 operators must fire the extractor.
    let content = "x |> a() |> b() |> c() |> d()";
    let collection = run_pipeline(content);
    assert_eq!(
        collection.patterns.len(),
        1,
        "4 pipeline ops should produce exactly one pattern"
    );
    let pattern = collection.patterns.values().next().unwrap();
    assert_eq!(pattern.pattern_type, PatternType::DataTransformation);
    assert_eq!(pattern.frequency, 4);
    // UPDATED (round 3): asserted `estimated_loc == 8` from
    // `matches.len() * 2`, an unmeasured "each pipeline operation is ~2 lines".
    // All four operations are on one line, so one line is what they occupy.
    assert_eq!(
        pattern.estimated_loc, 1,
        "estimated_loc is the measured source lines occupied"
    );
    assert_eq!(
        pattern.locations.len(),
        1,
        "four operations on one line are one location"
    );
    for loc in &pattern.locations {
        assert_eq!(loc.file, PathBuf::from("pipeline.ruchy"));
        assert_eq!(loc.column, 1);
    }
}

#[test]
fn three_pipeline_ops_produce_no_pattern() {
    // matches.len() == 3 is NOT > 3 → early return, no pattern added.
    let content = "x |> a() |> b() |> c()";
    let collection = run_pipeline(content);
    assert!(
        collection.patterns.is_empty(),
        "3 pipeline ops should not trigger pattern extraction"
    );
}

#[test]
fn zero_pipeline_ops_produce_no_pattern() {
    let content = "let x = foo(bar, baz); // no pipelines here";
    let collection = run_pipeline(content);
    assert!(collection.patterns.is_empty());
}

/// UPDATED (round 3): this test was named `..._truncate_locations_at_sixteen`
/// and asserted `locations.len() == 16` for 20 operations — it pinned a
/// truncated list presented as the pattern's locations, and an `estimated_loc`
/// of 40 that came from the constant `20 * 2` rather than from any line count.
#[test]
fn twenty_pipeline_ops_record_every_line() {
    // One operation per line, so lines and operations coincide.
    let ops = (0..20)
        .map(|i| format!("  |> f{}()\n", i))
        .collect::<String>();
    let content = format!("seed\n{}", ops);
    let collection = run_pipeline(&content);
    assert_eq!(collection.patterns.len(), 1);
    let pattern = collection.patterns.values().next().unwrap();
    assert_eq!(
        pattern.frequency, 20,
        "frequency = total matches, not capped"
    );
    assert_eq!(
        pattern.locations.len(),
        20,
        "every occurrence is recorded, not the first sixteen"
    );
    assert_eq!(
        pattern.estimated_loc, 20,
        "estimated_loc is the measured line count"
    );
}

#[test]
fn pattern_hash_is_file_path_scoped() {
    // Same content in two different files should produce different hashes
    // because hash_pattern is called with `ruchy_pipeline_{file_path}`.
    let extractor = PatternExtractor::new(EntropyConfig::default());
    let content = "x |> a() |> b() |> c() |> d()";
    let mut c1 = PatternCollection::new();
    let mut c2 = PatternCollection::new();
    extractor
        .extract_ruchy_pipeline_patterns(&PathBuf::from("alpha.ruchy"), content, &mut c1)
        .unwrap();
    extractor
        .extract_ruchy_pipeline_patterns(&PathBuf::from("beta.ruchy"), content, &mut c2)
        .unwrap();
    let h1 = c1.patterns.keys().next().unwrap().clone();
    let h2 = c2.patterns.keys().next().unwrap().clone();
    assert_ne!(h1, h2, "pattern hash should vary by file path");
}

#[test]
fn example_code_captured_around_first_match() {
    // example_code slices content[start.saturating_sub(20) .. end.min(start+100)].
    let content = "leading_padding_here x |> transform_alpha() |> b() |> c() |> d()";
    let collection = run_pipeline(content);
    let pattern = collection.patterns.values().next().unwrap();
    assert!(
        pattern.example_code.contains("|> transform_alpha("),
        "example_code should include first match: got {:?}",
        pattern.example_code
    );
}

#[test]
fn location_line_numbers_reflect_newline_positions() {
    // Put each pipeline op on its own line so line numbers are deterministic.
    let content = "seed\n |> a()\n |> b()\n |> c()\n |> d()";
    let collection = run_pipeline(content);
    let pattern = collection.patterns.values().next().unwrap();
    let lines: Vec<usize> = pattern.locations.iter().map(|l| l.line).collect();
    assert_eq!(lines, vec![2, 3, 4, 5]);
}

// calculate_pipeline_variation_score — only len<2 branch was covered.
// The multi-match branches (unique-set, identical ops, mixed) are below.

#[test]
fn test_calculate_pipeline_variation_score_all_distinct() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    // Three distinct operators → unique_operations.len() == matches.len() → 1.0.
    let content = "|>a |>b |>c";
    let pattern = Regex::new(r"\|>\w").unwrap();
    let matches: Vec<_> = pattern.find_iter(content).collect();
    assert_eq!(matches.len(), 3);
    let score = extractor.calculate_pipeline_variation_score(&matches, content);
    assert!((score - 1.0).abs() < f64::EPSILON, "got {score}");
}

#[test]
fn test_calculate_pipeline_variation_score_all_identical() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    // All same operator text → unique_operations has one entry → 1/3 ≈ 0.333.
    let content = "|>a |>a |>a";
    let pattern = Regex::new(r"\|>\w").unwrap();
    let matches: Vec<_> = pattern.find_iter(content).collect();
    assert_eq!(matches.len(), 3);
    let score = extractor.calculate_pipeline_variation_score(&matches, content);
    assert!((score - (1.0 / 3.0)).abs() < 1e-9, "got {score}");
}

#[test]
fn test_calculate_pipeline_variation_score_partial() {
    use regex::Regex;
    let extractor = PatternExtractor::new(EntropyConfig::default());
    // Two distinct ops out of four matches → 0.5.
    let content = "|>a |>b |>a |>b";
    let pattern = Regex::new(r"\|>\w").unwrap();
    let matches: Vec<_> = pattern.find_iter(content).collect();
    assert_eq!(matches.len(), 4);
    let score = extractor.calculate_pipeline_variation_score(&matches, content);
    assert!((score - 0.5).abs() < 1e-9, "got {score}");
}
