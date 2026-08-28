//! A LIBRARY's exported items are its entry points, not dead code.
//!
//! The cargo engine gets this right because rustc does: `pub fn` in a `--lib`
//! target is reachable by definition. The multi-language engine — the path taken
//! for every non-Rust tree, and for Rust when there is no cargo to ask — had no
//! notion of a target at all. It called every un-called definition dead, so a
//! Python package's whole `__all__` came back as dead code:
//!
//! ```text
//!   mypkg/__init__.py   from .core import public_api, another_export
//!                       __all__ = ["public_api", "another_export"]
//!   mypkg/core.py       def public_api(x): ...      <- reported DEAD
//!                       def another_export(y): ...  <- reported DEAD
//!                       def truly_dead(z): ...      <- reported dead, correctly
//!   dead_percentage: 100.0
//! ```
//!
//! That is the same false positive #1013 removed from the cargo path, relocated.
//! Every test here pins one half of the fix: an export of a determined library is
//! a root, and everything that is *not* an export still gets reported.

use super::*;
use tempfile::TempDir;

// ── Python: `__all__` and `__init__.py` re-exports are the declared API ──────

/// The fixture that reproduced the defect, verbatim.
fn python_package() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("pyproject.toml"),
        "[project]\nname = \"mypkg\"\nversion = \"0.1.0\"\n",
    )
    .expect("write pyproject");
    std::fs::create_dir_all(temp.path().join("mypkg")).expect("mkdir mypkg");
    std::fs::write(
        temp.path().join("mypkg/__init__.py"),
        "from .core import public_api, another_export\n\n\
         __all__ = [\"public_api\", \"another_export\"]\n",
    )
    .expect("write __init__.py");
    std::fs::write(
        temp.path().join("mypkg/core.py"),
        "def public_api(x):\n    return _inner(x)\n\n\
         def another_export(y):\n    return y + 1\n\n\
         def _inner(x):\n    return x * 2\n\n\
         def truly_dead(z):\n    return z - 1\n",
    )
    .expect("write core.py");
    temp
}

fn dead_names(result: &DeadCodeResult) -> Vec<String> {
    result
        .dead_functions
        .iter()
        .map(|f| f.name.clone())
        .collect()
}

#[test]
fn a_python_packages_declared_exports_are_not_dead() {
    let temp = python_package();
    let result = analyze_dead_code_multi_language(temp.path()).expect("analysis runs");
    let dead = dead_names(&result);

    for exported in ["public_api", "another_export"] {
        assert!(
            !dead.contains(&exported.to_string()),
            "`{exported}` is named in `__all__` and re-exported by mypkg/__init__.py — \
             it is this package's public API, which is un-called by construction; got {dead:?}"
        );
    }
}

/// The other half: rescuing the API must not rescue everything. A function that
/// no `__all__` names and no `__init__.py` re-exports is still dead.
#[test]
fn a_python_function_outside_the_declared_api_is_still_dead() {
    let temp = python_package();
    let result = analyze_dead_code_multi_language(temp.path()).expect("analysis runs");
    let dead = dead_names(&result);

    assert!(
        dead.contains(&"truly_dead".to_string()),
        "`truly_dead` is in no `__all__` and is re-exported by nothing; got {dead:?}"
    );
}

/// `__all__` alone is enough: a module that declares its exports has told the
/// analyzer what its API is, whether or not a package `__init__.py` exists.
#[test]
fn dunder_all_alone_declares_the_api() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("api.py"),
        "__all__ = [\"exported\"]\n\n\
         def exported():\n    return 1\n\n\
         def not_exported():\n    return 2\n",
    )
    .expect("write api.py");

    let result = analyze_dead_code_multi_language(temp.path()).expect("analysis runs");
    let dead = dead_names(&result);

    assert!(
        !dead.contains(&"exported".to_string()),
        "`__all__` names it; got {dead:?}"
    );
    assert!(
        dead.contains(&"not_exported".to_string()),
        "`__all__` does not name it; got {dead:?}"
    );
}

// ── Rust with no cargo to ask ───────────────────────────────────────────────

/// A library crate, analysed by the engine that runs when cargo cannot.
///
/// `pmat analyze dead-code` routes Rust to `cargo check`, so this engine is what
/// answers when there is no cargo — and it reported `pub fn public_api` dead,
/// which is the crate's entire reason to exist.
#[test]
fn a_rust_libs_public_api_is_not_dead_without_cargo() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"mylib\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    std::fs::create_dir_all(temp.path().join("src")).expect("mkdir src");
    std::fs::write(
        temp.path().join("src/lib.rs"),
        "pub fn public_api(n: u64) -> u64 {\n    helper(n)\n}\n\n\
         fn helper(n: u64) -> u64 {\n    n + 1\n}\n\n\
         fn private_dead(n: u64) -> u64 {\n    n - 1\n}\n",
    )
    .expect("write lib.rs");

    let result = analyze_dead_code_multi_language(temp.path()).expect("analysis runs");
    assert_eq!(
        result.language, "rust",
        "this fixture must take the Rust path"
    );
    let dead = dead_names(&result);

    assert!(
        !dead.contains(&"public_api".to_string()),
        "a lib crate's `pub fn` IS its entry point; got {dead:?}"
    );
    assert!(
        dead.contains(&"private_dead".to_string()),
        "a private, un-called fn is still dead; got {dead:?}"
    );
}

/// The control that keeps the rule from becoming "no `pub fn` is ever dead": a
/// crate with no library target exports nothing, so its un-called `pub fn` is
/// dead exactly as before.
#[test]
fn a_rust_bin_crates_pub_fn_is_still_dead() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"mybin\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write Cargo.toml");
    std::fs::create_dir_all(temp.path().join("src")).expect("mkdir src");
    std::fs::write(
        temp.path().join("src/main.rs"),
        "fn main() {\n    println!(\"{}\", entry(1));\n}\n\n\
         fn entry(n: u64) -> u64 {\n    n + 1\n}\n\n\
         pub fn never_called_dead_fn() -> u64 {\n    7\n}\n",
    )
    .expect("write main.rs");

    let result = analyze_dead_code_multi_language(temp.path()).expect("analysis runs");
    let dead = dead_names(&result);

    // The verdict, not just the finding: "undetermined" would produce the same
    // list here for the opposite reason — nothing seeded because nothing was
    // known — and a control that cannot tell those apart pins nothing.
    assert_eq!(
        result.library_target.verdict(),
        "not-a-library",
        "a Cargo.toml with no [lib] and no src/lib.rs is a DECIDED verdict, not an \
         absence of one: {:?}",
        result.library_target
    );
    assert_eq!(result.exported_roots, 0);
    assert!(
        dead.contains(&"never_called_dead_fn".to_string()),
        "a bin-only crate has no public API to protect; got {dead:?}"
    );
}

// ── C: determined by the build manifest, undetermined without one ───────────

const C_LIB_SOURCES: &str = "int mylib_init(int n) { return helper(n); }\n\
                             static int helper(int n) { return n + 1; }\n\
                             static int unused_helper(int n) { return n - 1; }\n";

#[test]
fn a_c_library_exports_its_non_static_functions() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.10)\nproject(mylib C)\nadd_library(mylib src.c)\n",
    )
    .expect("write CMakeLists.txt");
    std::fs::write(temp.path().join("src.c"), C_LIB_SOURCES).expect("write src.c");

    let result = analyze_dead_code_multi_language(temp.path()).expect("analysis runs");
    let dead = dead_names(&result);

    assert!(
        !dead.contains(&"mylib_init".to_string()),
        "`add_library` declares a library; a non-`static` function in one has \
         external linkage and is its API; got {dead:?}"
    );
    assert!(
        dead.contains(&"unused_helper".to_string()),
        "`static` is C's own visibility keyword: an un-called `static` function \
         cannot be called from outside its translation unit; got {dead:?}"
    );
}

/// `add_executable` is the opposite declaration, and it must not be read as a
/// licence to keep everything.
#[test]
fn a_c_executable_target_exports_nothing() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.10)\nproject(app C)\nadd_executable(app src.c)\n",
    )
    .expect("write CMakeLists.txt");
    std::fs::write(
        temp.path().join("src.c"),
        "int main(void) { return 0; }\nint never_called(int n) { return n; }\n",
    )
    .expect("write src.c");

    let result = analyze_dead_code_multi_language(temp.path()).expect("analysis runs");
    let dead = dead_names(&result);

    assert_eq!(
        result.library_target.verdict(),
        "not-a-library",
        "`add_executable` with no `add_library` is a DECIDED verdict, not an absence \
         of one: {:?}",
        result.library_target
    );
    assert_eq!(result.exported_roots, 0);
    assert!(
        dead.contains(&"never_called".to_string()),
        "an executable target has no exported API; got {dead:?}"
    );
}

/// The undetermined verdict, pinned on the language where it is the common case:
/// C with no build manifest at all. The finding is still reported — the
/// disclosure is a caveat on the list, not a replacement for it.
#[test]
fn a_c_tree_with_no_build_manifest_is_undetermined() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(temp.path().join("mylib.c"), C_LIB_SOURCES).expect("write mylib.c");

    let result = analyze_dead_code_multi_language(temp.path()).expect("analysis runs");

    assert_eq!(
        result.library_target.verdict(),
        "undetermined",
        "nothing in a bare .c tree declares a library or an executable: {:?}",
        result.library_target
    );
    assert_eq!(
        result.exported_roots, 0,
        "an undetermined verdict must seed nothing"
    );
    assert!(
        result.library_target.detail().contains("external linkage"),
        "the reason must name what could not be decided: {:?}",
        result.library_target
    );
    // `mylib_init` is non-`static` and un-called: exactly the item the verdict
    // could not classify, and it IS in the list.
    assert!(
        dead_names(&result).contains(&"mylib_init".to_string()),
        "{result:?}"
    );
}

// ── the crate is found by walking UP, not by looking where it cannot be ─────

/// A SUBDIRECTORY of a Rust library crate is inside a library.
///
/// The manifest was looked for in the analysed directory alone, and a
/// subdirectory never holds one — so this engine hedged to `undetermined` for
/// every subdirectory of every crate, seeded no roots, and reported the exported
/// API of that subtree as dead. The verdict is decidable; it just lives one
/// directory up.
#[test]
fn a_subdirectory_of_a_rust_library_is_still_inside_a_library() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"subdirlib\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .expect("write Cargo.toml");
    std::fs::create_dir_all(temp.path().join("src/inner")).expect("mkdir src/inner");
    std::fs::write(temp.path().join("src/lib.rs"), "pub mod inner;\n").expect("write lib.rs");
    std::fs::write(
        temp.path().join("src/inner/mod.rs"),
        "pub fn exported_but_uncalled(n: u64) -> u64 {\n    n + 1\n}\n\n\
         fn private_dead(n: u64) -> u64 {\n    n - 1\n}\n",
    )
    .expect("write inner/mod.rs");

    let result =
        analyze_dead_code_multi_language(&temp.path().join("src/inner")).expect("analysis runs");

    assert_eq!(
        result.library_target.verdict(),
        "library",
        "the crate above this subdirectory has a src/lib.rs, so the subdirectory is \
         part of a library: {:?}",
        result.library_target
    );
    let dead = dead_names(&result);
    assert!(
        !dead.contains(&"exported_but_uncalled".to_string()),
        "a library's exported item is un-called by construction; got {dead:?}"
    );
    assert!(
        dead.contains(&"private_dead".to_string()),
        "a private, un-called fn is still dead — the verdict must not rescue \
         everything; got {dead:?}"
    );
}

/// The control, one directory over: a subdirectory of a BIN crate is inside a
/// bin crate, and its un-called `pub fn` is still dead. A fix that answered
/// "library" for every subdirectory would pass the test above and fail here.
#[test]
fn a_subdirectory_of_a_rust_bin_crate_is_still_not_a_library() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"subdirbin\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
    )
    .expect("write Cargo.toml");
    std::fs::create_dir_all(temp.path().join("src/inner")).expect("mkdir src/inner");
    std::fs::write(
        temp.path().join("src/main.rs"),
        "mod inner;\n\nfn main() {\n    println!(\"{}\", inner::used(1));\n}\n",
    )
    .expect("write main.rs");
    std::fs::write(
        temp.path().join("src/inner/mod.rs"),
        "pub fn used(n: u64) -> u64 {\n    n\n}\n\n\
         pub fn never_called_dead_fn() -> u64 {\n    7\n}\n",
    )
    .expect("write inner/mod.rs");

    let result =
        analyze_dead_code_multi_language(&temp.path().join("src/inner")).expect("analysis runs");

    assert_eq!(
        result.library_target.verdict(),
        "not-a-library",
        "the crate above this subdirectory declares no [lib] and has no src/lib.rs: {:?}",
        result.library_target
    );
    assert_eq!(result.exported_roots, 0);
    assert!(
        dead_names(&result).contains(&"never_called_dead_fn".to_string()),
        "a bin-only crate has no public API to protect; got {:?}",
        dead_names(&result)
    );
}

// ── a workspace manifest declares no crate, so it decides nothing ───────────

/// A `Cargo.toml` with `[workspace]` and no `[package]` is not a crate: it
/// declares a workspace. It has no `[lib]` and no `src/lib.rs` beside it
/// because a virtual manifest never does, so reading those absences as evidence
/// produced a `NotLibrary` verdict — "declares no [lib] … which exports nothing
/// to an outside caller" — about a manifest that declares no crate to export
/// anything.
#[test]
fn a_workspace_manifest_decides_nothing_about_a_crate_it_does_not_declare() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crate-a\"]\nresolver = \"2\"\n",
    )
    .expect("write workspace manifest");
    std::fs::create_dir_all(temp.path().join("tools")).expect("mkdir tools");
    std::fs::write(
        temp.path().join("tools/loose.rs"),
        "pub fn exported_but_uncalled(n: u64) -> u64 {\n    n + 1\n}\n",
    )
    .expect("write loose.rs");

    let result =
        analyze_dead_code_multi_language(&temp.path().join("tools")).expect("analysis runs");

    assert_eq!(
        result.library_target.verdict(),
        "undetermined",
        "a manifest that declares no package cannot decide a target shape: {:?}",
        result.library_target
    );
    let detail = result.library_target.detail();
    assert!(
        !detail.contains("declares no [lib]"),
        "the reason asserts a [lib] section is missing from a manifest that \
         declares no package to hold one: {detail}"
    );
    assert!(
        !detail.contains("no Cargo.toml"),
        "there IS a Cargo.toml above this path; the reason must not claim \
         otherwise: {detail}"
    );
    assert!(
        detail.contains(&temp.path().join("Cargo.toml").display().to_string()),
        "the reason must name the manifest it read: {detail}"
    );
    assert!(
        detail.contains("[workspace]"),
        "the reason must say what that manifest actually declares: {detail}"
    );
}

/// COUNTER-TEST, passes before and after: a MEMBER of that workspace is still
/// decided from its own manifest, and its exports are still seeded as roots.
#[test]
fn a_member_of_a_workspace_is_still_decided_from_its_own_manifest() {
    let temp = TempDir::new().expect("tempdir");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crate-a\"]\nresolver = \"2\"\n",
    )
    .expect("write workspace manifest");
    std::fs::create_dir_all(temp.path().join("crate-a/src/inner")).expect("mkdir member");
    std::fs::write(
        temp.path().join("crate-a/Cargo.toml"),
        "[package]\nname = \"crate-a\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write member manifest");
    std::fs::write(temp.path().join("crate-a/src/lib.rs"), "pub mod inner;\n")
        .expect("write lib.rs");
    std::fs::write(
        temp.path().join("crate-a/src/inner/mod.rs"),
        "pub fn exported_but_uncalled(n: u64) -> u64 {\n    n + 1\n}\n\n\
         fn private_dead(n: u64) -> u64 {\n    n - 1\n}\n",
    )
    .expect("write inner/mod.rs");

    let result = analyze_dead_code_multi_language(&temp.path().join("crate-a/src/inner"))
        .expect("analysis runs");

    assert_eq!(
        result.library_target.verdict(),
        "library",
        "crate-a has a src/lib.rs: {:?}",
        result.library_target
    );
    let dead = dead_names(&result);
    assert!(
        !dead.contains(&"exported_but_uncalled".to_string()),
        "a library's exported item is un-called by construction; got {dead:?}"
    );
    assert!(
        dead.contains(&"private_dead".to_string()),
        "a private, un-called fn is still dead; got {dead:?}"
    );
}
