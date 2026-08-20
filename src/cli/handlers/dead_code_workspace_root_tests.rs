//! `analyze dead-code` pointed at a path governed by a WORKSPACE-ONLY manifest.
//!
//! "The nearest `Cargo.toml` wins" fixed the subdirectory case for ordinary
//! crates and left one manifest shape behind: a workspace root declares a
//! `[workspace]` table and NO `[package]`, so it is a `Cargo.toml` that declares
//! no crate at all. The walk stopped there anyway, found no `[lib]` and no
//! `src/lib.rs` beside it — because a virtual manifest has neither, by
//! definition — and published the same two falsehoods the subdirectory fix was
//! about:
//!
//! ```text
//!   ws/                     Cargo.toml = [workspace] members = ["crate-a"]
//!   ws/crate-a/             a library, with dead code in src/inner/
//!   ws/tools/loose.rs       inside NO package
//!
//!   analyze dead-code --path ws/tools   dead_functions:0 … exit 0
//!     library_target: {"verdict":"not-a-library",
//!       "detail":"cargo: the Cargo.toml at ws declares no [lib] and there is
//!                 no src/lib.rs beside it — a binary-only crate …"}
//!   analyze dead-code --path ws         3 files analyzed, 0 with dead code
//! ```
//!
//! `ws/Cargo.toml` declares no package, so it is neither a binary-only crate nor
//! any other kind; and `cargo check --bins` in a virtual workspace whose members
//! are libraries matches no target, compiles NOTHING and exits 0, so the zero is
//! measured over nothing while `ws/crate-a/src/inner` holds a dead private
//! function and a never-constructed struct.
//!
//! A workspace manifest is not a crate. The path is answered by the PACKAGE that
//! encloses it, and where no package does, there is nothing for rustc to
//! compile and the command refuses instead of publishing a clean zero.

use super::{cargo_library_target, run_dead_code_analysis_with_filters, DeadCodeAnalysisFilters};
use std::path::Path;
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
    }
}

/// A VIRTUAL workspace: the root manifest has `[workspace]` and no `[package]`.
///
/// * `crate-a` is a member, a library, and its only dead code is in
///   `src/inner/` — the subdirectory case, one level further in.
/// * `tools/loose.rs` sits under the workspace root and inside no member, which
///   is the path no package governs.
fn virtual_workspace() -> TempDir {
    let tmp = tempfile::Builder::new()
        .prefix("dcws")
        .tempdir()
        .expect("tempdir");
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crate-a\"]\nresolver = \"2\"\n",
    )
    .expect("write workspace manifest");

    std::fs::create_dir_all(tmp.path().join("crate-a/src/inner")).expect("mkdir member");
    std::fs::write(
        tmp.path().join("crate-a/Cargo.toml"),
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write member manifest");
    std::fs::write(
        tmp.path().join("crate-a/src/lib.rs"),
        "pub mod inner;\n\npub fn public_api() -> i32 {\n    42\n}\n",
    )
    .expect("write member lib.rs");
    std::fs::write(
        tmp.path().join("crate-a/src/inner/mod.rs"),
        "fn dead_private_helper() -> i32 {\n    7\n}\n\n\
         struct NeverConstructed {\n    field: i32,\n}\n",
    )
    .expect("write member inner/mod.rs");

    std::fs::create_dir_all(tmp.path().join("tools")).expect("mkdir tools");
    std::fs::write(
        tmp.path().join("tools/loose.rs"),
        "fn stray_dead() -> i32 {\n    3\n}\n",
    )
    .expect("write tools/loose.rs");
    tmp
}

/// Every finding as `(file, item)`, for a run that is expected to succeed.
async fn findings(path: &Path) -> Vec<(String, String)> {
    let outcome =
        run_dead_code_analysis_with_filters(path, filters(), std::time::Duration::from_secs(600))
            .await
            .expect("the analysis runs");
    let mut rows: Vec<(String, String)> = outcome
        .report
        .files
        .iter()
        .flat_map(|file| {
            file.items
                .iter()
                .map(move |item| (file.path.clone(), item.name.clone()))
        })
        .collect();
    rows.sort();
    rows
}

// ── the member package answers for its own subtree ──────────────────────────

/// A subdirectory of a MEMBER crate resolves to the member, not to the
/// workspace root above it, and reports the member's dead code.
#[tokio::test]
async fn a_member_crates_subdirectory_reports_the_dead_code_it_holds() {
    let ws = virtual_workspace();

    let subtree = findings(&ws.path().join("crate-a/src/inner")).await;

    assert!(
        subtree
            .iter()
            .any(|(_, name)| name == "dead_private_helper"),
        "the dead private function in a workspace member's subdirectory is \
         invisible: {subtree:?}"
    );
    assert!(
        subtree.iter().any(|(_, name)| name == "NeverConstructed"),
        "the never-constructed struct in a workspace member's subdirectory is \
         invisible: {subtree:?}"
    );
}

// ── a path inside no package: refuse, do not publish a zero ─────────────────

/// `ws/tools` is under the workspace root and inside no member. There is no
/// package for rustc to compile, so there is no measurement — and an unmeasured
/// tree must not be published as a clean one.
#[tokio::test]
async fn a_path_inside_no_package_of_a_workspace_is_refused_not_reported_as_clean() {
    let ws = virtual_workspace();
    let loose = ws.path().join("tools");

    let outcome =
        run_dead_code_analysis_with_filters(&loose, filters(), std::time::Duration::from_secs(600))
            .await;

    let error = match outcome {
        Ok(run) => panic!(
            "a path inside no package cannot be measured, so it must not be \
             reported; got a report claiming {} file(s) with dead code and \
             library_target {:?}",
            run.report.summary.files_with_dead_code, run.report.library_target
        ),
        Err(e) => e.to_string(),
    };

    assert!(
        error.contains("no dead-code measurement was taken"),
        "the refusal must say that nothing was measured: {error}"
    );
    assert!(
        error.contains("This is not a clean result"),
        "the refusal must say what the absence of a result is NOT: {error}"
    );
    assert!(
        error.contains(&loose.display().to_string()),
        "the refusal must name the path it could find no package for: {error}"
    );
    assert!(
        error.contains(&ws.path().join("Cargo.toml").display().to_string()),
        "the refusal must name the manifest it stopped at, so the reader can \
         check the claim: {error}"
    );
    assert!(
        error.contains("[workspace]") && error.contains("[package]"),
        "the refusal must say WHY that manifest is not a crate — it declares a \
         [workspace] and no [package] — rather than merely reporting an \
         absence: {error}"
    );
    assert!(
        error.contains("crate-a"),
        "the refusal must name a member the caller could point at instead: {error}"
    );
    assert!(
        !error.contains("no Cargo.toml"),
        "there IS a Cargo.toml above this path; the refusal must not claim \
         otherwise: {error}"
    );
}

/// The workspace ROOT itself. Its members hold real dead code — the run above
/// finds it — and `cargo check --bins` on a virtual manifest whose members are
/// libraries matches no target and compiles nothing, so the report was a zero
/// measured over an empty compile.
#[tokio::test]
async fn a_workspace_root_is_refused_rather_than_reported_clean_over_its_members() {
    let ws = virtual_workspace();

    // The dead code is really there: the member reports it.
    let member = findings(&ws.path().join("crate-a")).await;
    assert!(
        member.iter().any(|(_, name)| name == "dead_private_helper"),
        "fixture check: the member crate must hold dead code for this test to \
         prove anything; got {member:?}"
    );

    let outcome = run_dead_code_analysis_with_filters(
        ws.path(),
        filters(),
        std::time::Duration::from_secs(600),
    )
    .await;

    let error = match outcome {
        Ok(run) => panic!(
            "the workspace root is no package: cargo compiled nothing, so the \
             report is a zero measured over an empty compile. Got {} file(s) \
             with dead code over {} analysed, while its member reports {member:?}",
            run.report.summary.files_with_dead_code, run.report.analyzed_files
        ),
        Err(e) => e.to_string(),
    };
    assert!(
        error.contains("no dead-code measurement was taken"),
        "the refusal must say that nothing was measured: {error}"
    );
    assert!(
        error.contains("crate-a"),
        "the refusal must name the member that CAN be measured: {error}"
    );
}

// ── the verdict must not contradict the manifest that governs the path ──────

/// The published `detail` asserted that `ws/Cargo.toml` "declares no [lib] …
/// a binary-only crate". That manifest declares no package at all, so it
/// declares no crate of any kind, and "not-a-library" is a decision the
/// evidence does not support.
#[test]
fn a_workspace_manifest_yields_no_verdict_about_a_crate_it_does_not_declare() {
    let ws = virtual_workspace();

    let verdict = cargo_library_target(&ws.path().join("tools"));

    assert_eq!(
        verdict.verdict, "undetermined",
        "a manifest that declares no package cannot decide a crate's target \
         shape: {verdict:?}"
    );
    assert!(
        !verdict.detail.contains("declares no [lib]"),
        "the detail asserts a [lib] section is missing from a manifest that \
         declares no package to hold one: {verdict:?}"
    );
    assert!(
        !verdict.detail.contains("binary-only crate"),
        "a virtual workspace manifest is not a binary-only crate: {verdict:?}"
    );
    assert!(
        verdict
            .detail
            .contains(&ws.path().join("Cargo.toml").display().to_string()),
        "the detail must name the manifest the verdict was taken from: {verdict:?}"
    );
    assert!(
        verdict.detail.contains("[workspace]"),
        "the detail must say what that manifest actually declares: {verdict:?}"
    );
}

/// …and where a package DOES govern the path, the verdict names that package's
/// manifest — the file a reader would open to check it — not merely a directory.
#[test]
fn the_verdict_names_the_manifest_that_governs_the_path() {
    let ws = virtual_workspace();

    let verdict = cargo_library_target(&ws.path().join("crate-a/src/inner"));

    assert_eq!(
        verdict.verdict, "library",
        "crate-a has a src/lib.rs, so its subdirectory is inside a library: {verdict:?}"
    );
    assert!(
        verdict
            .detail
            .contains(&ws.path().join("crate-a/Cargo.toml").display().to_string()),
        "the detail must name the manifest it read the target shape from: {verdict:?}"
    );
    assert!(
        !verdict
            .detail
            .contains(&format!("{}/Cargo.toml", ws.path().display())),
        "the verdict must not be attributed to the workspace manifest, which \
         declares no package: {verdict:?}"
    );
}

// ── COUNTER-TESTS: the ordinary single-crate cases are unchanged ────────────

/// Must pass BEFORE and AFTER. A crate manifest carrying BOTH `[package]` and
/// `[workspace]` — the shape of this repo's own root, and of every crate a test
/// fixture writes to keep a tempdir out of an enclosing workspace — is still a
/// crate, and still answers for its own subdirectories.
#[test]
fn a_manifest_with_both_package_and_workspace_is_still_a_crate() {
    let tmp = tempfile::Builder::new()
        .prefix("dcboth")
        .tempdir()
        .expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("src/inner")).expect("mkdir");
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"dcboth\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .expect("write manifest");
    std::fs::write(tmp.path().join("src/lib.rs"), "pub mod inner;\n").expect("write lib.rs");
    std::fs::write(tmp.path().join("src/inner/mod.rs"), "fn dead() {}\n").expect("write inner");

    assert_eq!(cargo_library_target(tmp.path()).verdict, "library");
    assert_eq!(
        cargo_library_target(&tmp.path().join("src/inner")).verdict,
        "library",
        "a subdirectory of that crate is inside a library"
    );
}

/// Must pass BEFORE and AFTER. A bin-only crate is still a DECIDED
/// "not-a-library" — a fix that answers "undetermined" for everything with no
/// `[lib]` would pass every test above and fail here.
#[test]
fn a_bin_only_crate_is_still_a_decided_not_a_library() {
    let tmp = tempfile::Builder::new()
        .prefix("dcbinonly")
        .tempdir()
        .expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("src/inner")).expect("mkdir");
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"dcbinonly\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .expect("write manifest");
    std::fs::write(
        tmp.path().join("src/main.rs"),
        "mod inner;\n\nfn main() {\n    println!(\"{}\", inner::used());\n}\n",
    )
    .expect("write main.rs");
    std::fs::write(
        tmp.path().join("src/inner/mod.rs"),
        "pub fn used() -> i32 {\n    1\n}\n",
    )
    .expect("write inner");

    assert_eq!(cargo_library_target(tmp.path()).verdict, "not-a-library");
    assert_eq!(
        cargo_library_target(&tmp.path().join("src/inner")).verdict,
        "not-a-library",
        "a subdirectory of a bin crate is inside a bin crate"
    );
}

/// Must pass BEFORE and AFTER. No manifest ANYWHERE above the path is a
/// different fact from "the manifest above it declares no package", and the two
/// refusals must stay distinguishable.
#[tokio::test]
async fn a_tree_with_no_manifest_at_all_still_says_there_is_no_cargo_toml() {
    let tmp = tempfile::Builder::new()
        .prefix("dcnomanifest")
        .tempdir()
        .expect("tempdir");
    std::fs::write(
        tmp.path().join("stray.rs"),
        "fn dead_helper() -> i32 {\n    3\n}\n",
    )
    .expect("write stray.rs");

    let error = match run_dead_code_analysis_with_filters(
        tmp.path(),
        filters(),
        std::time::Duration::from_secs(600),
    )
    .await
    {
        Ok(run) => panic!(
            "a tree with no crate cannot be measured; got a report claiming {} \
             file(s) with dead code",
            run.report.summary.files_with_dead_code
        ),
        Err(e) => e.to_string(),
    };

    assert!(
        error.contains("no Cargo.toml"),
        "the refusal must name what was missing: {error}"
    );
    assert_eq!(
        cargo_library_target(tmp.path()).verdict,
        "undetermined",
        "no manifest at all is undetermined, not a decision"
    );
}

// ── the agent-facing surface reports the refusal, not a zero ────────────────

/// MCP is where a zero does the most damage: an agent reading
/// `dead_functions: 0` has no summary text to fall back on and no reason to
/// look twice. The workspace root must appear in `paths_not_analyzed` with the
/// reason, never in `paths` with a clean count.
#[tokio::test]
async fn the_mcp_payload_reports_a_workspace_root_as_not_analyzed() {
    let ws = virtual_workspace();

    let mcp = crate::mcp_pmcp::tool_functions::analyze_dead_code(&[ws.path().to_path_buf()], false)
        .await
        .expect("MCP dead-code analysis returns a payload");
    let results = &mcp["results"];

    assert_eq!(
        results["paths"].as_array().map(Vec::len),
        Some(0),
        "a path that could not be measured must not appear as an analysed one: {mcp}"
    );
    let not_analyzed = results["paths_not_analyzed"]
        .as_array()
        .expect("paths_not_analyzed is a list");
    assert_eq!(not_analyzed.len(), 1, "{mcp}");
    let reason = not_analyzed[0]["reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("no dead-code measurement was taken") && reason.contains("crate-a"),
        "the reason must travel with the refusal, and name the member that CAN be \
         measured: {mcp}"
    );
    assert_eq!(
        results["files_analyzed"].as_u64(),
        Some(0),
        "nothing was compiled, so nothing may be counted as analysed — a \
         `files_analyzed` above zero next to `total_dead_code: 0` is the clean \
         bill of health this refusal exists to prevent: {mcp}"
    );
}
