//! R18 — `DeepContextConfig::include_patterns` must actually narrow the run.
//!
//! Before this wiring, `pmat analyze deep-context --format sarif
//! --include-pattern '**/*.py'` produced output byte-identical (bar the
//! duration line) to the unfiltered run and still reported `.rs` findings:
//! the CLI set `DeepContextConfig::include_patterns` and nothing in
//! `analyzer_core` ever read it. These tests drive `DeepContextAnalyzer`
//! directly — the same object the CLI's SARIF path uses — so both the file
//! count and the findings are pinned.

use super::{AnalysisType, DeepContextAnalyzer, DeepContextConfig};
use std::path::PathBuf;

const RUST_SOURCE: &str = r#"
// FIXME: this needs rework
pub fn tangled(a: i32, b: i32, c: i32) -> i32 {
    let mut t = 0;
    for i in 0..a {
        if i % 2 == 0 {
            for j in 0..b {
                if j % 3 == 0 {
                    while t < c {
                        match t % 4 {
                            0 => t += 1,
                            1 => t += 2,
                            2 => { if a > b { t += 3 } else { t += 4 } }
                            _ => t += 5,
                        }
                    }
                } else if j % 5 == 0 {
                    t -= 1;
                } else {
                    t += 1;
                }
            }
        } else if i % 7 == 0 {
            t *= 2;
        } else {
            t += 1;
        }
    }
    t
}
"#;

const PYTHON_SOURCE: &str = r#"
# TODO: refactor this mess
def tangled(a, b, c):
    t = 0
    for i in range(a):
        if i % 2 == 0:
            for j in range(b):
                if j % 3 == 0:
                    while t < c:
                        if t % 4 == 0:
                            t += 1
                        elif t % 4 == 1:
                            t += 2
                        elif a > b:
                            t += 3
                        else:
                            t += 5
                elif j % 5 == 0:
                    t -= 1
                else:
                    t += 1
        elif i % 7 == 0:
            t *= 2
        else:
            t += 1
    return t
"#;

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("hot.rs"), RUST_SOURCE).expect("write rs");
    std::fs::write(dir.path().join("hot.py"), PYTHON_SOURCE).expect("write py");
    dir
}

fn config(include_patterns: Vec<String>) -> DeepContextConfig {
    DeepContextConfig {
        include_analyses: vec![AnalysisType::Complexity, AnalysisType::Satd],
        include_patterns,
        ..DeepContextConfig::default()
    }
}

/// Every file path named anywhere in the SARIF `results` array.
fn sarif_files(sarif: &str) -> Vec<String> {
    let doc: serde_json::Value = serde_json::from_str(sarif).expect("SARIF is JSON");
    doc["runs"][0]["results"]
        .as_array()
        .expect("SARIF results array")
        .iter()
        .map(|r| {
            r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                .as_str()
                .expect("finding location")
                .to_string()
        })
        .collect()
}

#[tokio::test]
async fn include_patterns_narrow_the_file_count_and_the_findings() {
    let dir = fixture();
    let path: PathBuf = dir.path().to_path_buf();

    // Baseline: both files are seen, and both produce findings. Without this
    // the filtered assertions below could pass vacuously.
    let unfiltered = DeepContextAnalyzer::new(config(vec![]))
        .analyze_project(&path)
        .await
        .expect("unfiltered analysis");
    assert_eq!(
        unfiltered.file_tree.total_files, 2,
        "fixture should contribute exactly hot.rs and hot.py"
    );
    let unfiltered_sarif = DeepContextAnalyzer::new(config(vec![]))
        .format_as_sarif(&unfiltered)
        .expect("unfiltered sarif");
    let unfiltered_files = sarif_files(&unfiltered_sarif);
    assert!(
        unfiltered_files.iter().any(|f| f.ends_with("hot.rs")),
        "baseline must report hot.rs findings, got {unfiltered_files:?}"
    );
    assert!(
        unfiltered_files.iter().any(|f| f.ends_with("hot.py")),
        "baseline must report hot.py findings, got {unfiltered_files:?}"
    );

    // Filtered: `**/*.py` must drop hot.rs from BOTH the count and the findings.
    let analyzer = DeepContextAnalyzer::new(config(vec!["**/*.py".to_string()]));
    let filtered = analyzer
        .analyze_project(&path)
        .await
        .expect("filtered analysis");
    assert_eq!(
        filtered.file_tree.total_files, 1,
        "--include-pattern '**/*.py' must shrink file_count"
    );

    let filtered_sarif = analyzer.format_as_sarif(&filtered).expect("filtered sarif");
    let filtered_files = sarif_files(&filtered_sarif);
    assert!(
        !filtered_files.iter().any(|f| f.ends_with("hot.rs")),
        "a '**/*.py' filter must not report .rs findings, got {filtered_files:?}"
    );
    assert!(
        filtered_files.iter().any(|f| f.ends_with("hot.py")),
        "a '**/*.py' filter must still report .py findings, got {filtered_files:?}"
    );
}

#[tokio::test]
async fn an_empty_include_pattern_list_still_means_every_file() {
    // An empty collection is "no filter", never "no files".
    let dir = fixture();
    let context = DeepContextAnalyzer::new(config(vec![]))
        .analyze_project(&dir.path().to_path_buf())
        .await
        .expect("analysis");
    assert_eq!(context.file_tree.total_files, 2);
}
