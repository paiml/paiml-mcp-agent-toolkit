//! Which crate a requested path belongs to.
//!
//! Every question this analyzer asks about a crate — is it a library, which
//! targets does it have, where does `cargo check` run — used to be asked of the
//! directory the caller named. A directory inside a crate answers all three
//! wrongly, because it holds no manifest of its own, and the wrong answers
//! compounded: no `[lib]` found meant `--lib` was dropped, and `cargo check
//! --bins` on a library-only crate matched no target and compiled nothing.

use super::{absolutize, enclosing_crate_root, CargoDeadCodeAnalyzer};

/// A crate root, a subdirectory two levels down, and a file.
fn crate_fixture() -> tempfile::TempDir {
    let tmp = tempfile::Builder::new()
        .prefix("croot")
        .tempdir()
        .expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("src/inner")).expect("mkdir");
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[package]\nname = \"croot\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .expect("write manifest");
    std::fs::write(tmp.path().join("src/lib.rs"), "pub mod inner;\n").expect("write lib");
    std::fs::write(tmp.path().join("src/inner/mod.rs"), "fn dead() {}\n").expect("write inner");
    tmp
}

#[test]
fn a_crate_root_resolves_to_itself() {
    let tmp = crate_fixture();
    assert_eq!(
        enclosing_crate_root(tmp.path()),
        Some(absolutize(tmp.path())),
        "a directory holding a Cargo.toml is its own crate root"
    );
}

/// THE regression. `--path <crate>/src/inner` must resolve to `<crate>`.
#[test]
fn a_subdirectory_resolves_to_the_crate_above_it() {
    let tmp = crate_fixture();
    assert_eq!(
        enclosing_crate_root(&tmp.path().join("src/inner")),
        Some(absolutize(tmp.path())),
        "a subdirectory of a crate is a VIEW of that crate, not a crate of its own"
    );
}

#[test]
fn a_file_resolves_to_the_crate_that_contains_it() {
    let tmp = crate_fixture();
    assert_eq!(
        enclosing_crate_root(&tmp.path().join("src/inner/mod.rs")),
        Some(absolutize(tmp.path()))
    );
}

/// The NEAREST manifest wins: a package inside a workspace is compiled as that
/// package, not as the workspace.
#[test]
fn a_workspace_member_resolves_to_the_member_not_the_workspace() {
    let tmp = tempfile::Builder::new()
        .prefix("cws")
        .tempdir()
        .expect("tempdir");
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\"]\nresolver = \"2\"\n",
    )
    .expect("write workspace manifest");
    std::fs::create_dir_all(tmp.path().join("crates/member/src")).expect("mkdir");
    std::fs::write(
        tmp.path().join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write member manifest");
    std::fs::write(
        tmp.path().join("crates/member/src/lib.rs"),
        "pub fn a() {}\n",
    )
    .expect("write lib");

    assert_eq!(
        enclosing_crate_root(&tmp.path().join("crates/member/src")),
        Some(absolutize(&tmp.path().join("crates/member")))
    );
}

/// No manifest anywhere above: there is no crate, and saying so is the only
/// honest answer. Guessing "not a library" here is what published a verdict
/// about a crate that did not exist.
#[test]
fn a_tree_with_no_manifest_anywhere_resolves_to_nothing() {
    let tmp = tempfile::Builder::new()
        .prefix("cnone")
        .tempdir()
        .expect("tempdir");
    std::fs::write(tmp.path().join("stray.rs"), "fn dead() {}\n").expect("write stray");
    assert_eq!(enclosing_crate_root(tmp.path()), None);
}

/// The analyzer resolves it once, at construction, so the cargo invocation and
/// every target-shape decision are taken from the same directory.
#[test]
fn the_analyzer_compiles_the_crate_that_encloses_the_requested_path() {
    let tmp = crate_fixture();
    let analyzer = CargoDeadCodeAnalyzer::new(tmp.path().join("src/inner"));
    assert_eq!(analyzer.cargo_root(), absolutize(tmp.path()));
}

/// A real directory of this repo: `src/services/satd_detector` is inside a
/// crate whose `src/lib.rs` exists, and nothing may report otherwise.
#[test]
fn this_repos_service_directory_resolves_to_this_repo() {
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let service = repo.join("src/services/satd_detector");
    assert!(
        service.is_dir(),
        "fixture path moved: {}",
        service.display()
    );
    assert_eq!(enclosing_crate_root(&service), Some(absolutize(&repo)));
    assert!(
        repo.join("src/lib.rs").is_file(),
        "this repo has a library target, so any verdict derived from the crate \
         above src/services/satd_detector must say so"
    );
}

// ── a workspace manifest declares no crate ──────────────────────────────────

/// A VIRTUAL workspace root, a member crate, and a directory under the root
/// that belongs to no member.
fn virtual_workspace_fixture() -> tempfile::TempDir {
    let tmp = tempfile::Builder::new()
        .prefix("cvws")
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
    std::fs::write(tmp.path().join("crate-a/src/lib.rs"), "pub mod inner;\n").expect("write lib");
    std::fs::write(
        tmp.path().join("crate-a/src/inner/mod.rs"),
        "fn dead() {}\n",
    )
    .expect("write inner");
    std::fs::create_dir_all(tmp.path().join("tools")).expect("mkdir tools");
    std::fs::write(tmp.path().join("tools/loose.rs"), "fn stray() {}\n").expect("write loose");
    tmp
}

/// THE regression. A `Cargo.toml` holding `[workspace]` and no `[package]`
/// declares a workspace, not a crate: it has no targets, no `[lib]` and no
/// `src/lib.rs`, because a virtual manifest never does. Stopping the walk there
/// answered every target-shape question from a manifest that declares nothing
/// to answer them about.
#[test]
fn a_virtual_workspace_root_is_not_a_crate_root() {
    let tmp = virtual_workspace_fixture();
    assert_eq!(
        enclosing_crate_root(tmp.path()),
        None,
        "a [workspace] table with no [package] declares no crate for rustc to compile"
    );
}

/// …and neither is anything under it that no member contains. There is no
/// enclosing package at all, which is a fact the caller must be told rather
/// than have a verdict invented from.
#[test]
fn a_path_under_a_virtual_workspace_and_inside_no_member_has_no_crate_root() {
    let tmp = virtual_workspace_fixture();
    assert_eq!(enclosing_crate_root(&tmp.path().join("tools")), None);
    assert_eq!(
        enclosing_crate_root(&tmp.path().join("tools/loose.rs")),
        None
    );
}

/// COUNTER-TEST, passes before and after: a path inside a MEMBER still resolves
/// to that member — the walk must skip the workspace manifest, not stop dead at
/// the first `Cargo.toml` and not refuse everything inside a workspace.
#[test]
fn a_subdirectory_of_a_workspace_member_still_resolves_to_the_member() {
    let tmp = virtual_workspace_fixture();
    let member = absolutize(&tmp.path().join("crate-a"));
    assert_eq!(
        enclosing_crate_root(&tmp.path().join("crate-a/src/inner")),
        Some(member.clone())
    );
    assert_eq!(
        enclosing_crate_root(&tmp.path().join("crate-a/src/inner/mod.rs")),
        Some(member)
    );
}

/// COUNTER-TEST, passes before and after: a manifest carrying BOTH `[package]`
/// and `[workspace]` — this repo's own root — is a crate. A fix that rejected
/// every manifest mentioning `[workspace]` would break every crate in the tree.
#[test]
fn a_manifest_with_both_package_and_workspace_is_still_a_crate_root() {
    let tmp = crate_fixture();
    assert_eq!(
        enclosing_crate_root(&tmp.path().join("src/inner")),
        Some(absolutize(tmp.path())),
        "the fixture manifest has [package] AND [workspace]; it declares a crate"
    );
}

/// A REAL virtual workspace root, not a two-line one: `[workspace.package]` and
/// `[workspace.dependencies]` are how every modern monorepo shares versions, and
/// their table names begin with the word `package`'s neighbour, not with
/// `package`. Read by their first segment they are workspace tables; read
/// sloppily, `[workspace.package]` makes every monorepo root a crate again and
/// restores the verdict this fix removes.
#[test]
fn workspace_inheritance_tables_do_not_make_a_workspace_root_a_crate() {
    let tmp = tempfile::Builder::new()
        .prefix("cvwsinherit")
        .tempdir()
        .expect("tempdir");
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crate-a\"]\nresolver = \"2\"\n\n\
         [workspace.package]\nversion = \"1.2.3\"\nedition = \"2021\"\n\n\
         [workspace.dependencies]\nserde = \"1\"\n",
    )
    .expect("write workspace manifest");
    std::fs::create_dir_all(tmp.path().join("crate-a/src")).expect("mkdir member");
    std::fs::write(
        tmp.path().join("crate-a/Cargo.toml"),
        "[package]\nname = \"crate-a\"\nversion.workspace = true\nedition.workspace = true\n",
    )
    .expect("write member manifest");
    std::fs::write(tmp.path().join("crate-a/src/lib.rs"), "pub fn a() {}\n").expect("write lib");

    assert_eq!(
        enclosing_crate_root(tmp.path()),
        None,
        "[workspace.package] is a workspace table; it declares no package"
    );
    assert_eq!(
        enclosing_crate_root(&tmp.path().join("crate-a/src")),
        Some(absolutize(&tmp.path().join("crate-a"))),
        "the member is still a crate, and still the one that governs its own src/"
    );
}

/// What the manifest DECLARES, read straight from its text — the rule the walk
/// turns on, pinned without a filesystem so each shape is visible at once.
#[test]
fn manifest_kinds_are_read_from_the_tables_the_manifest_declares() {
    use super::{classify_manifest_content, ManifestKind};

    let kind = classify_manifest_content("[package]\nname = \"solo\"\nversion = \"0.1.0\"\n");
    match kind {
        ManifestKind::Package { name } => assert_eq!(name.as_deref(), Some("solo")),
        _ => panic!("a [package] table declares a package"),
    }

    // This repo's own root, and every fixture that keeps a tempdir out of an
    // enclosing workspace: BOTH tables, and it is a crate.
    let kind =
        classify_manifest_content("[package]\nname = \"both\"\nversion = \"0.1.0\"\n[workspace]\n");
    match kind {
        ManifestKind::Package { name } => assert_eq!(name.as_deref(), Some("both")),
        _ => panic!("[package] + [workspace] is still a package"),
    }

    let kind = classify_manifest_content(
        "[workspace]\nmembers = [\n  \"crates/a\",\n  \"crates/b\",\n]\nresolver = \"2\"\n",
    );
    match kind {
        ManifestKind::WorkspaceOnly { members } => {
            assert_eq!(members, vec!["crates/a", "crates/b"])
        }
        _ => panic!("[workspace] with no [package] declares no crate"),
    }

    // `default-members` is a different key and must not be mistaken for the
    // member list, or a refusal would name crates the caller cannot point at.
    let kind =
        classify_manifest_content("[workspace]\nmembers = [\"a\"]\ndefault-members = [\"b\"]\n");
    match kind {
        ManifestKind::WorkspaceOnly { members } => assert_eq!(members, vec!["a"]),
        _ => panic!("still a workspace"),
    }

    // Cargo rejects a manifest with neither table; it declares no crate either,
    // and it is not a workspace boundary, so the walk may pass through it.
    assert!(matches!(
        classify_manifest_content("[dependencies]\nserde = \"1\"\n"),
        ManifestKind::Neither
    ));
}
