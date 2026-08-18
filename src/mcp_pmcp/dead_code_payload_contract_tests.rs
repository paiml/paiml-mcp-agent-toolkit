//! `analyze_dead_code`'s MCP payload is a CONTRACT, and it lost two keys.
//!
//! Moving this tool onto the shared analyzer rewrote its payload, and two keys
//! master published stopped being emitted with nothing recording it:
//!
//! ```text
//!   master  results -> total_dead_code total_functions languages files
//!                      analyzer analyzer_note include_tests
//!   here    results -> total_dead_code by_kind files_analyzed files
//!                      analyzer engines analyzer_note include_tests
//!                      paths paths_not_analyzed
//! ```
//!
//! `total_functions` is the DENOMINATOR for the dead-function count, and a
//! count without a denominator is the defect this whole release is about: an
//! agent reading `dead_functions: 3` cannot tell a 3-of-4 tree from a 3-of-900
//! tree. `languages` is which language was actually READ — the multi-language
//! engine reads ONE language per project and skips the rest, so on a mixed tree
//! `engine: multi-language-reachability` alone does not say what was looked at.
//! Neither is meaningless under the one-analyzer design, so neither may be
//! dropped, and a client that read them broke silently rather than loudly.
//!
//! These tests assert PRESENCE of every key the payload is contracted to carry,
//! by name, against the results object itself — not with `json["k"].is_null()`,
//! which serde_json answers `true` for a key that was never emitted and which
//! would therefore have passed on the payload that dropped them.

use serde_json::Value;
use tempfile::TempDir;

/// Two Python functions, one dead. The multi-language engine COUNTS the
/// functions it walked, so this is the case where a denominator exists and must
/// be published.
fn python_fixture() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("used.py"),
        "def used():\n    return 1\n\nused()\n",
    )
    .expect("write used.py");
    std::fs::write(
        temp.path().join("dead.py"),
        "def never_called():\n    return 2\n",
    )
    .expect("write dead.py");
    temp
}

/// A one-crate library with one dead private function. Goes to the cargo
/// engine, which reports dead items and never counts live ones.
fn rust_fixture() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"dcpayload\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .expect("write Cargo.toml");
    std::fs::create_dir_all(temp.path().join("src")).expect("mkdir src");
    std::fs::write(
        temp.path().join("src/lib.rs"),
        "pub fn used() {}\n\nfn unused_helper() {\n    let _ = 1;\n}\n",
    )
    .expect("write lib.rs");
    temp
}

async fn analyze(dir: &TempDir) -> Value {
    crate::mcp_pmcp::tool_functions::analyze_dead_code(&[dir.path().to_path_buf()], false)
        .await
        .expect("MCP dead-code analysis returns a payload")
}

fn results_of(payload: &Value) -> &serde_json::Map<String, Value> {
    payload["results"]
        .as_object()
        .expect("`results` is an object")
}

/// Every key `results` is contracted to carry.
///
/// The first two are the ones that were dropped; the rest are this release's
/// own additions, listed here so the next rewrite of this payload cannot drop
/// one of them the same silent way.
const CONTRACTED_RESULT_KEYS: &[&str] = &[
    "total_functions",
    "languages",
    "total_dead_code",
    "by_kind",
    "files",
    "files_analyzed",
    "analyzer",
    "engines",
    "analyzer_note",
    "include_tests",
    "paths",
    "paths_not_analyzed",
];

/// Every key each row of `results.files` is contracted to carry: a counter and
/// a list for each of the six kinds, so nothing listed can be uncounted.
const CONTRACTED_FILE_KEYS: &[&str] = &[
    "file",
    "dead_code_count",
    "counts",
    "dead_functions",
    "dead_classes",
    "dead_variables",
    "dead_modules",
    "unreachable_blocks",
    "other",
];

/// Every key each entry of `results.paths` is contracted to carry.
const CONTRACTED_PATH_KEYS: &[&str] = &[
    "language",
    "total_functions",
    "requested",
    "analysis_root",
    "engine",
    "files_analyzed",
    "files_listed",
    "library_target",
    "findings_outside_requested_path",
];

#[tokio::test]
async fn the_payload_carries_every_key_it_is_contracted_to_carry() {
    let fixture = python_fixture();
    let payload = analyze(&fixture).await;
    let results = results_of(&payload);

    let missing: Vec<&str> = CONTRACTED_RESULT_KEYS
        .iter()
        .copied()
        .filter(|key| !results.contains_key(*key))
        .collect();
    assert!(
        missing.is_empty(),
        "`results` dropped {missing:?}; any client reading one of those keys \
         now reads nothing, and nothing said so: {payload:#}"
    );

    let path_entry = payload["results"]["paths"][0]
        .as_object()
        .expect("`results.paths[0]` is an object");
    let missing: Vec<&str> = CONTRACTED_PATH_KEYS
        .iter()
        .copied()
        .filter(|key| !path_entry.contains_key(*key))
        .collect();
    assert!(
        missing.is_empty(),
        "`results.paths[0]` dropped {missing:?}: {payload:#}"
    );

    let file_row = payload["results"]["files"][0]
        .as_object()
        .expect("`results.files[0]` is an object");
    let missing: Vec<&str> = CONTRACTED_FILE_KEYS
        .iter()
        .copied()
        .filter(|key| !file_row.contains_key(*key))
        .collect();
    assert!(
        missing.is_empty(),
        "`results.files[0]` dropped {missing:?}; the per-kind lists and the \
         `counts` beside them are what keep a listed item from being uncounted: \
         {payload:#}"
    );
    for &key in CONTRACTED_FILE_KEYS {
        if let Some(count) = file_row["counts"].get(key) {
            assert_eq!(
                count.as_u64(),
                Some(file_row[key].as_array().expect("a list of items").len() as u64),
                "`counts.{key}` does not agree with the list beside it: {payload:#}"
            );
        }
    }
}

/// The denominator, where the engine measures one.
#[tokio::test]
async fn the_dead_function_count_is_published_with_its_denominator() {
    let fixture = python_fixture();
    let payload = analyze(&fixture).await;
    let results = results_of(&payload);

    assert!(
        results.contains_key("total_functions"),
        "the multi-language engine counts every function it walked, and the \
         payload publishes the dead ones without it: {payload:#}"
    );
    let total = results["total_functions"]
        .as_u64()
        .unwrap_or_else(|| panic!("`total_functions` is not a number: {payload:#}"));
    let dead = payload["results"]["by_kind"]["dead_functions"]
        .as_u64()
        .expect("by_kind.dead_functions");

    assert_eq!(
        total, 2,
        "the fixture has exactly two functions, `used` and `never_called`: {payload:#}"
    );
    assert!(
        dead <= total,
        "more functions are reported dead ({dead}) than the analyzer says exist \
         ({total}): {payload:#}"
    );
    assert_eq!(
        payload["results"]["paths"][0]["total_functions"].as_u64(),
        Some(total),
        "the total must be attributable to the path it was measured over: {payload:#}"
    );
}

/// …and where it does NOT measure one, it says so, in the way this release
/// says everything else it could not measure: `null`, never `0`.
///
/// `0` would read as "this crate has no functions", which is both false and
/// exactly the silent-zero defect the rest of the payload was fixed for. The
/// number cannot be invented here either: counting Rust functions with the
/// reachability analyzer's walk would measure a different file set than the
/// dead-code pass whose findings it would head — `cargo check` skips the test,
/// example and bench targets — so the ratio would be quietly wrong rather than
/// absent.
#[tokio::test]
async fn an_engine_that_counts_no_functions_says_so_rather_than_zero() {
    let fixture = rust_fixture();
    let payload = analyze(&fixture).await;
    let results = results_of(&payload);

    assert!(
        results.contains_key("total_functions"),
        "the key must be present even when the engine has no figure for it — an \
         absent key and an unmeasurable one are different facts: {payload:#}"
    );
    assert!(
        results["total_functions"].is_null(),
        "the cargo engine does not count live functions; any number here is \
         invented: {payload:#}"
    );

    let path_entry = payload["results"]["paths"][0]
        .as_object()
        .expect("`results.paths[0]` is an object");
    assert!(
        path_entry.contains_key("total_functions"),
        "the per-path entry must name which path had no count: {payload:#}"
    );
    assert!(
        path_entry["total_functions"].is_null(),
        "a `0` here reads as an empty crate: {payload:#}"
    );
}

/// Which language was actually read.
///
/// The multi-language engine reads ONE language per project and skips every
/// other source file in the tree, so this is not cosmetic: it is the difference
/// between "no dead Python" and "the Python was never opened".
#[tokio::test]
async fn the_payload_names_the_language_that_was_read() {
    let fixture = python_fixture();
    // Source in a language this run will NOT read. Without `languages` the
    // payload cannot distinguish it from source that was read and found clean.
    std::fs::write(
        fixture.path().join("skipped.lua"),
        "local function never_called() return 1 end\n",
    )
    .expect("write skipped.lua");

    let payload = analyze(&fixture).await;
    let results = results_of(&payload);

    assert!(
        results.contains_key("languages"),
        "the payload does not say which language it read: {payload:#}"
    );
    assert_eq!(
        results["languages"]
            .as_array()
            .expect("`languages` is an array")
            .iter()
            .map(|v| v.as_str().expect("a language name").to_string())
            .collect::<Vec<_>>(),
        vec!["python".to_string()],
        "the analyzer read Python and skipped the Lua beside it: {payload:#}"
    );
    assert_eq!(
        payload["results"]["paths"][0]["language"].as_str(),
        Some("python"),
        "the language must be attributable to the path it was detected for: {payload:#}"
    );
}

/// The cargo engine names its language too, so a client never has to infer one
/// from an engine name.
#[tokio::test]
async fn the_cargo_engine_names_its_language_too() {
    let fixture = rust_fixture();
    let payload = analyze(&fixture).await;

    assert_eq!(
        payload["results"]["languages"]
            .as_array()
            .expect("`languages` is an array")
            .iter()
            .map(|v| v.as_str().expect("a language name").to_string())
            .collect::<Vec<_>>(),
        vec!["rust".to_string()],
        "{payload:#}"
    );
    assert_eq!(
        payload["results"]["paths"][0]["language"].as_str(),
        Some("rust"),
        "{payload:#}"
    );
}

/// Two paths, one countable and one not: the total is `null`, not the half of
/// it that happened to be measurable.
///
/// A sum over the subset of paths that could be counted is not a denominator
/// for a numerator drawn from all of them — `total_functions: 2` beside a dead
/// count that also covers the Rust crate would understate the tree by however
/// much the uncounted path holds, which is the same silent-zero failure in a
/// subtler form. `paths[]` still carries each path's own figure, so the reader
/// can see which one had none.
#[tokio::test]
async fn a_total_that_covers_only_some_of_the_paths_is_not_published() {
    let python = python_fixture();
    let rust = rust_fixture();
    let payload = crate::mcp_pmcp::tool_functions::analyze_dead_code(
        &[python.path().to_path_buf(), rust.path().to_path_buf()],
        false,
    )
    .await
    .expect("MCP dead-code analysis returns a payload");
    let results = results_of(&payload);

    assert!(results.contains_key("total_functions"), "{payload:#}");
    assert!(
        results["total_functions"].is_null(),
        "one of the two paths was never counted, so this total covers neither \
         the tree nor the dead count beside it: {payload:#}"
    );

    let paths = payload["results"]["paths"]
        .as_array()
        .expect("`results.paths` is an array");
    assert_eq!(paths.len(), 2, "{payload:#}");
    let per_path: Vec<Option<u64>> = paths
        .iter()
        .map(|p| p["total_functions"].as_u64())
        .collect();
    assert!(
        per_path.contains(&Some(2)) && per_path.contains(&None),
        "each path must still carry its own figure so the null above can be \
         attributed: {per_path:?} in {payload:#}"
    );
    assert_eq!(
        results["languages"]
            .as_array()
            .expect("`languages` is an array")
            .iter()
            .map(|v| v.as_str().expect("a language name").to_string())
            .collect::<Vec<_>>(),
        vec!["python".to_string(), "rust".to_string()],
        "both languages were read: {payload:#}"
    );
}

// ── Counter-tests ──────────────────────────────────────────────────────────
//
// Green BEFORE and AFTER. A "fix" that satisfies the assertions above by
// refusing to analyse anything — an empty file list, an error payload, a
// path parked in `paths_not_analyzed` — fails here.

#[tokio::test]
async fn counter_the_findings_are_unchanged_on_the_multi_language_engine() {
    let fixture = python_fixture();
    let payload = analyze(&fixture).await;

    assert_eq!(payload["status"], "completed", "{payload:#}");
    assert_eq!(
        payload["results"]["paths_not_analyzed"]
            .as_array()
            .expect("paths_not_analyzed")
            .len(),
        0,
        "the fixture was not analysed: {payload:#}"
    );

    let named: Vec<String> = payload["results"]["files"]
        .as_array()
        .expect("files array")
        .iter()
        .flat_map(|f| {
            f["dead_functions"]
                .as_array()
                .expect("dead_functions")
                .iter()
        })
        .map(|i| i["name"].as_str().expect("name").to_string())
        .collect();
    assert_eq!(
        named,
        vec!["never_called".to_string()],
        "the dead function this tool must find: {payload:#}"
    );
    assert_eq!(
        payload["results"]["total_dead_code"].as_u64(),
        Some(1),
        "{payload:#}"
    );
}

#[tokio::test]
async fn counter_the_findings_are_unchanged_on_the_cargo_engine() {
    let fixture = rust_fixture();
    let payload = analyze(&fixture).await;

    assert_eq!(payload["status"], "completed", "{payload:#}");
    assert_eq!(
        payload["results"]["paths_not_analyzed"]
            .as_array()
            .expect("paths_not_analyzed")
            .len(),
        0,
        "the fixture was not analysed: {payload:#}"
    );

    let named: Vec<String> = payload["results"]["files"]
        .as_array()
        .expect("files array")
        .iter()
        .flat_map(|f| {
            f["dead_functions"]
                .as_array()
                .expect("dead_functions")
                .iter()
        })
        .map(|i| i["name"].as_str().expect("name").to_string())
        .collect();
    assert_eq!(
        named,
        vec!["unused_helper".to_string()],
        "`used` is this LIBRARY's public API and `unused_helper` is dead: {payload:#}"
    );
    assert_eq!(
        payload["results"]["total_dead_code"].as_u64(),
        Some(1),
        "{payload:#}"
    );
}
