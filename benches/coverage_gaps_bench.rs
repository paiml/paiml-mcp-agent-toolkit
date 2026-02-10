use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use pmat::services::agent_context::{classify_exclusions, CoverageExclusion, QueryResult};
use std::hint::black_box;
use tempfile::TempDir;

/// Create a synthetic QueryResult for benchmarking.
fn make_result(file_path: &str, function_name: &str) -> QueryResult {
    QueryResult {
        file_path: file_path.to_string(),
        function_name: function_name.to_string(),
        signature: format!("fn {}()", function_name),
        definition_type: "function".to_string(),
        doc_comment: None,
        start_line: 1,
        end_line: 10,
        language: "rust".to_string(),
        tdg_score: 5.0,
        tdg_grade: "C".to_string(),
        complexity: 5,
        big_o: "O(n)".to_string(),
        satd_count: 0,
        loc: 10,
        relevance_score: 0.0,
        source: None,
        calls: Vec::new(),
        called_by: Vec::new(),
        pagerank: 0.0,
        in_degree: 0,
        out_degree: 0,
        commit_count: 0,
        churn_score: 0.0,
        clone_count: 0,
        duplication_score: 0.0,
        pattern_diversity: 0.0,
        fault_annotations: Vec::new(),
        line_coverage_pct: 0.0,
        lines_covered: 0,
        lines_total: 10,
        missed_lines: 10,
        impact_score: 0.0,
        coverage_status: "uncovered".to_string(),
        coverage_diff: 0.0,
        coverage_exclusion: CoverageExclusion::None,
        coverage_excluded: false,
        cross_project_callers: 0,
    }
}

/// Set up a temp project dir with N source files (some with coverage(off), some without).
fn setup_project(num_files: usize) -> (TempDir, Vec<QueryResult>) {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();

    // Create a Makefile with COVERAGE_EXCLUDE
    std::fs::write(
        temp.path().join("Makefile"),
        "COVERAGE_EXCLUDE := --ignore-filename-regex='(_tests?\\.rs|/tests/)'\n",
    )
    .unwrap();

    // Create dead-code-cache.json
    std::fs::create_dir_all(temp.path().join(".pmat")).unwrap();
    std::fs::write(
        temp.path().join(".pmat/dead-code-cache.json"),
        r#"{"report":{"files_with_dead_code":[{"file_path":"src/dead.rs","dead_items":[{"name":"unused_fn","kind":"function"}]}]}}"#,
    )
    .unwrap();

    let mut results = Vec::with_capacity(num_files * 15); // ~15 functions per file

    for i in 0..num_files {
        let dir = src.join(format!("mod_{}", i / 50));
        std::fs::create_dir_all(&dir).unwrap();

        let filename = format!("file_{}.rs", i);
        let rel_path = format!("src/mod_{}/{}", i / 50, filename);

        // 10% of files have coverage(off)
        let content = if i % 10 == 0 {
            "#![cfg_attr(coverage_nightly, coverage(off))]\nfn f() {}\n"
        } else {
            "fn f() {}\n"
        };
        std::fs::write(dir.join(&filename), content).unwrap();

        // ~15 functions per file
        for j in 0..15 {
            results.push(make_result(&rel_path, &format!("func_{}_{}", i, j)));
        }
    }

    // Add some test files (should match Makefile exclude)
    let test_dir = src.join("tests");
    std::fs::create_dir_all(&test_dir).unwrap();
    std::fs::write(test_dir.join("test_a.rs"), "fn test_a() {}\n").unwrap();
    for j in 0..10 {
        results.push(make_result("src/tests/test_a.rs", &format!("test_{}", j)));
    }

    (temp, results)
}

fn bench_classify_exclusions(c: &mut Criterion) {
    let mut group = c.benchmark_group("classify_exclusions");
    group.sample_size(50);

    for &num_files in &[100, 500, 1000] {
        let (temp, template_results) = setup_project(num_files);
        let num_funcs = template_results.len();

        group.bench_with_input(
            BenchmarkId::new("files", format!("{}_files_{}_funcs", num_files, num_funcs)),
            &num_files,
            |b, _| {
                b.iter(|| {
                    let mut results = template_results.clone();
                    classify_exclusions(black_box(&mut results), temp.path(), None);
                    black_box(&results);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_classify_exclusions);
criterion_main!(benches);
