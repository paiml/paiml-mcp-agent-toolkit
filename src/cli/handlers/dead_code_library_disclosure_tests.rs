//! The report must say whether it decided the target was a library.
//!
//! Seeding a library's exports as reachability roots changes which findings
//! exist, so it is not an implementation detail: the same tree yields a
//! different list depending on a verdict the reader never sees. And where the
//! verdict could not be reached — C with no CMakeLists, a Python tree with no
//! `__all__` — an un-called export IS still listed as dead, which a reader can
//! only weigh if the report admits it could not tell an export from a corpse.
//!
//! This is the disclosure pattern the rest of the release uses: name what the
//! analyzer could not decide, and why, rather than letting a silent default
//! stand in for a measurement.

use super::{format_dead_code_as_json, format_dead_code_as_summary, DeadCodeAnalysisFilters};
use tempfile::TempDir;

fn filters() -> DeadCodeAnalysisFilters {
    DeadCodeAnalysisFilters {
        include_unreachable: false,
        include_tests: false,
        min_dead_lines: 0,
        top_files: None,
        include: Vec::new(),
        exclude: Vec::new(),
        max_depth: 10,
        no_cache: false,
    }
}

/// A package that declares its API: the verdict is `library`, and the payload
/// says which items it kept alive.
#[test]
fn a_determined_library_publishes_its_verdict_and_root_count() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("pyproject.toml"),
        "[project]\nname = \"mypkg\"\n",
    )
    .expect("write pyproject");
    std::fs::create_dir_all(temp.path().join("mypkg")).expect("mkdir");
    std::fs::write(
        temp.path().join("mypkg/__init__.py"),
        "from .core import public_api\n\n__all__ = [\"public_api\"]\n",
    )
    .expect("write __init__.py");
    std::fs::write(
        temp.path().join("mypkg/core.py"),
        "def public_api(x):\n    return x\n\ndef truly_dead(z):\n    return z\n",
    )
    .expect("write core.py");

    let outcome = super::run_multi_language_dead_code(temp.path(), &filters(), "python")
        .expect("analysis runs");
    let json = format_dead_code_as_json(&outcome.report).expect("json renders");

    assert!(
        json.contains("\"library_target\""),
        "the report does not say whether it treated this target as a library, and \
         that decision changed the finding list: {json}"
    );
    assert!(
        json.contains("\"verdict\": \"library\""),
        "an `__all__` naming an `__init__.py` re-export is a declared API: {json}"
    );
    assert!(
        json.contains("\"exported_roots\": 1"),
        "one export was seeded as a root and the payload must count it: {json}"
    );
}

/// C with no build manifest: the verdict is `undetermined`, the reason names
/// what could not be decided, and the un-called non-`static` function is still
/// listed — the point of the disclosure being that the reader can now tell why.
#[test]
fn an_undetermined_target_names_what_it_could_not_decide() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("mylib.c"),
        "int mylib_init(int n) { return n; }\n",
    )
    .expect("write mylib.c");

    let outcome =
        super::run_multi_language_dead_code(temp.path(), &filters(), "c").expect("analysis runs");
    let json = format_dead_code_as_json(&outcome.report).expect("json renders");

    assert!(
        json.contains("\"verdict\": \"undetermined\""),
        "nothing in a bare .c tree declares a library or an executable: {json}"
    );
    assert!(
        json.contains("no CMakeLists.txt"),
        "the reason must name what was missing, not just that something was: {json}"
    );
    assert!(
        json.contains("\"exported_roots\": 0"),
        "an undetermined verdict must seed nothing, or the disclosure would \
         describe a decision already taken: {json}"
    );
    // …and the finding is still reported, so the disclosure is a caveat on a
    // list rather than a replacement for one.
    assert!(
        json.contains("mylib_init"),
        "the un-called function must still be listed: {json}"
    );
}

/// The text summary carries the same disclosure. It is the surface a human
/// reads, and it is where "100% dead" over a library's whole API was printed
/// with nothing to qualify it.
#[test]
fn the_text_summary_discloses_an_undetermined_verdict() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("mylib.c"),
        "int mylib_init(int n) { return n; }\n",
    )
    .expect("write mylib.c");

    let outcome =
        super::run_multi_language_dead_code(temp.path(), &filters(), "c").expect("analysis runs");
    let rendered = format_dead_code_as_summary(&outcome.report).expect("summary renders");

    assert!(
        rendered.contains("Library target:"),
        "the summary does not state the verdict that shaped its list: {rendered}"
    );
    assert!(rendered.contains("undetermined"), "{rendered}");
    assert!(
        rendered.contains("external linkage"),
        "the reason must reach the human-readable surface too: {rendered}"
    );
}

/// EVERY output format, not three of four.
///
/// A disclosure that one renderer drops is a disclosure the consumer who chose
/// that renderer never sees — and the two formats a CI pipeline actually
/// consumes are `json` and `sarif`. `markdown` is the one a human files.
#[test]
fn every_output_format_carries_the_verdict() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("mylib.c"),
        "int mylib_init(int n) { return n; }\n",
    )
    .expect("write mylib.c");

    let outcome =
        super::run_multi_language_dead_code(temp.path(), &filters(), "c").expect("analysis runs");

    for format in [
        crate::cli::DeadCodeOutputFormat::Json,
        crate::cli::DeadCodeOutputFormat::Sarif,
        crate::cli::DeadCodeOutputFormat::Summary,
        crate::cli::DeadCodeOutputFormat::Markdown,
    ] {
        let rendered = super::format_dead_code_result(
            &outcome.report,
            &format,
            super::DeadCodeReportScope::default(),
        )
        .expect("renders");
        assert!(
            rendered.contains("undetermined"),
            "`--format {format:?}` does not say the library verdict was undetermined, \
             so a consumer of that format cannot tell an export from a corpse:\n{rendered}"
        );
    }
}

/// MCP carries the same disclosure, per analysed path.
///
/// This is the surface an agent reads, and an agent has no summary text to fall
/// back on: without the verdict it cannot tell "these are dead" from "these
/// might be a library's API and I could not tell".
#[tokio::test]
async fn the_mcp_payload_carries_the_verdict_per_path() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("mylib.c"),
        "int mylib_init(int n) { return n; }\n",
    )
    .expect("write mylib.c");

    let mcp =
        crate::mcp_pmcp::tool_functions::analyze_dead_code(&[temp.path().to_path_buf()], false)
            .await
            .expect("MCP dead-code analysis returns a payload");
    let scope = &mcp["results"]["paths"][0];

    assert_eq!(
        scope["library_target"]["verdict"].as_str(),
        Some("undetermined"),
        "the MCP payload does not say whether these findings could be a library's \
         public API: {mcp}"
    );
    assert!(
        scope["library_target"]["detail"]
            .as_str()
            .is_some_and(|d| d.contains("external linkage")),
        "the reason must travel with the verdict: {mcp}"
    );
}

/// The cargo engine does not make this decision — rustc does, and it gets it
/// right — so the report says that rather than leaving the field blank for the
/// reader to interpret.
#[tokio::test]
async fn the_cargo_engine_attributes_the_decision_to_rustc() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"discl\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
         [workspace]\n",
    )
    .expect("write Cargo.toml");
    std::fs::create_dir_all(temp.path().join("src")).expect("mkdir");
    std::fs::write(
        temp.path().join("src/lib.rs"),
        "pub fn entry(n: u64) -> u64 {\n    n + 1\n}\n\nfn dead_one() -> u64 {\n    1\n}\n",
    )
    .expect("write lib.rs");

    let outcome = super::run_dead_code_analysis_with_filters(
        temp.path(),
        filters(),
        std::time::Duration::from_secs(600),
    )
    .await
    .expect("cargo analysis runs");
    let json = format_dead_code_as_json(&outcome.report).expect("json renders");

    assert!(
        json.contains("\"verdict\": \"library\""),
        "src/lib.rs is a library target: {json}"
    );
    assert!(
        json.contains("\"exported_roots\": null"),
        "the cargo engine seeds no roots of its own — rustc's dead-code pass \
         already treats a library's public API as reachable — and a `0` here \
         would read as \"it found none\": {json}"
    );
}
