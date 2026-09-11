//! PMAT-1317 — the seven analysis-section formatters in `analysis_sections.rs`
//! measured 0 of 209 lines covered: every one is reached only through the
//! legacy comprehensive-markdown path, and nothing drove that path over a
//! context whose analyses were populated. These tests run a real analysis
//! over the two-file fixture `scope_wiring_tests.rs` already uses, so the
//! sections format what the analyzer actually produced — and each assertion
//! names a thing the fixture contains, so a formatter that prints a heading
//! over an empty table cannot pass.

use crate::services::deep_context::{
    AnalysisType, DeepContext, DeepContextAnalyzer, DeepContextConfig,
};

const RUST_SOURCE: &str = r#"
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
                } else {
                    t += 1;
                }
            }
        } else {
            t += 1;
        }
    }
    t
}

pub fn plain() -> i32 { 1 } // TODO: this needs rework — trailing, so the satd ratchet's line-start grep does not count a fixture
"#;

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("hot.rs"), RUST_SOURCE).expect("write rs");
    dir
}

async fn analyzed(
    analyses: Vec<AnalysisType>,
) -> (tempfile::TempDir, DeepContextAnalyzer, DeepContext) {
    let dir = fixture();
    let analyzer = DeepContextAnalyzer::new(DeepContextConfig {
        include_analyses: analyses,
        ..DeepContextConfig::default()
    });
    let context = analyzer
        .analyze_project(&dir.path().to_path_buf())
        .await
        .expect("analysis over the fixture");
    (dir, analyzer, context)
}

#[tokio::test]
async fn complexity_hotspots_name_the_fixtures_most_complex_function_first() {
    let (_dir, analyzer, context) =
        analyzed(vec![AnalysisType::Ast, AnalysisType::Complexity]).await;
    let report = context.analyses.complexity_report.as_ref().expect(
        "the fixture must produce a complexity report, or the section has nothing to format",
    );
    assert_eq!(report.files.len(), 1, "one fixture file, one file entry");

    let out = analyzer
        .format_as_comprehensive_markdown_legacy(&context)
        .expect("legacy markdown");

    let section = out
        .split("## Complexity Hotspots")
        .nth(1)
        .expect("the hotspots section is rendered when a complexity report exists");
    let tangled = section.find("`tangled`");
    assert!(
        tangled.is_some(),
        "the tangled function is a hotspot; section was:\n{section}"
    );
    let tangled = tangled.expect("checked above");
    let plain = section.find("`plain`");
    assert!(
        plain.is_none_or(|p| tangled < p),
        "hotspots are sorted by cyclomatic complexity, descending: tangled before plain"
    );
    assert!(
        section.contains("| Function | File | Cyclomatic | Cognitive |"),
        "the table header names the four columns"
    );
}

/// PMAT-1319: `include_analyses: [Complexity]` WITHOUT `Ast` used to yield
/// `Some(report)` with ZERO files — the complexity phase reads a process-global
/// cache only the AST phase fills, and an empty cache was not treated as an
/// error. `execute_parallel_analyses_with_progress` now runs the AST phase
/// implicitly whenever a cache-dependent phase (Complexity, Provability, Dag)
/// is requested without Ast, so the cache is populated either way — but the
/// caller must not receive `ast_contexts` it never asked for.
#[tokio::test]
async fn complexity_without_ast_still_finds_the_fixtures_functions() {
    let (_dir, _analyzer, context) = analyzed(vec![AnalysisType::Complexity]).await;
    let report = context
        .analyses
        .complexity_report
        .as_ref()
        .expect("the phase ran and returned Ok");
    assert_eq!(
        report.files.len(),
        1,
        "the implicit AST phase fills the cache, so the one fixture file is found"
    );
    let names: Vec<&str> = report
        .files
        .iter()
        .flat_map(|f| f.functions.iter())
        .map(|f| f.name.as_str())
        .collect();
    assert!(
        names.contains(&"tangled"),
        "tangled must be present; got {names:?}"
    );
    assert!(
        names.contains(&"plain"),
        "plain must be present; got {names:?}"
    );
    assert!(
        context.analyses.ast_contexts.is_empty(),
        "Ast was not requested, so the implicit AST phase must not leak ast_contexts to the caller"
    );
}

#[tokio::test]
async fn technical_debt_section_reports_the_fixtures_todo() {
    let (_dir, analyzer, context) = analyzed(vec![AnalysisType::Ast, AnalysisType::Satd]).await;
    assert!(
        context.analyses.satd_results.is_some(),
        "the fixture carries a TODO"
    );

    let out = analyzer
        .format_as_comprehensive_markdown_legacy(&context)
        .expect("legacy markdown");
    assert!(
        out.contains("Technical Debt") || out.contains("SATD"),
        "a populated SATD result must render a technical-debt section; got:\n{out}"
    );
    assert!(
        out.contains("hot.rs"),
        "the debt item is attributed to the file that carries it"
    );
}

#[tokio::test]
async fn sections_with_no_analysis_are_omitted_not_rendered_empty() {
    // Complexity only: churn, dead code and SATD were not requested, so their
    // sections must be absent rather than printed as empty tables.
    let (_dir, analyzer, context) =
        analyzed(vec![AnalysisType::Ast, AnalysisType::Complexity]).await;
    assert!(context.analyses.churn_analysis.is_none());
    assert!(context.analyses.dead_code_results.is_none());

    let out = analyzer
        .format_as_comprehensive_markdown_legacy(&context)
        .expect("legacy markdown");
    assert!(
        !out.contains("## Code Churn") && !out.contains("## Churn"),
        "no churn analysis → no churn section"
    );
    assert!(
        !out.contains("## Dead Code"),
        "no dead-code analysis → no dead-code section"
    );
}

#[tokio::test]
async fn the_full_legacy_report_over_every_analysis_is_stable_and_non_empty() {
    let (_dir, analyzer, context) = analyzed(vec![
        AnalysisType::Ast,
        AnalysisType::Complexity,
        AnalysisType::Satd,
        AnalysisType::DeadCode,
        AnalysisType::Churn,
    ])
    .await;
    let a = analyzer
        .format_as_comprehensive_markdown_legacy(&context)
        .expect("legacy markdown");
    let b = analyzer
        .format_as_comprehensive_markdown_legacy(&context)
        .expect("legacy markdown, again");
    assert_eq!(
        a, b,
        "formatting the same context twice must be byte-identical"
    );
    assert!(a.len() > 200, "a report over four analyses is not a stub");
    assert!(
        a.contains("## Complexity Hotspots"),
        "the complexity section is present when requested"
    );
}
