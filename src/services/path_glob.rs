//! Shell-style glob expansion and source-tree walking helpers.
//!
//! **R22-2 / D102**: Two MCP tool dispatchers exist in parallel —
//! `src/mcp_pmcp/tool_functions/` and `src/handlers/tools/`. R21-4 (#370)
//! patched glob handling only in the former. This module hoists the fix
//! into `src/services/` so both trees share one implementation, closing
//! the parity gap that the Phase 3 v3.15.0 dogfood NO-GO exposed.
//!
//! The public API is:
//! - [`path_has_glob_meta`] — cheap test for `*`, `?`, `[`.
//! - [`resolve_paths_with_globs`] — shell-like expansion; non-globs pass
//!   through unchanged so composition with plain-path walkers is free.
//! - [`expand_paths_to_source_files`] — globs + walkdir over a default
//!   allow-list of source extensions (R17-2 D82 directory-walk behavior).
//! - [`expand_project_path`] — convenience for the live `handlers/tools`
//!   tree which accepts a single `project_path: String` that may contain
//!   glob metacharacters. Returns a flat file list; callers fail loud when
//!   the list is empty.
//!
//! The pure-string [`path_has_glob_meta`] check is intentionally loose
//! (matches brackets inside filenames too). The contract is "if the input
//! looks glob-ish, route it through `glob::glob_with`; otherwise, treat as
//! literal". False positives are handled by `glob_with` returning no
//! matches — which is identical to what a POSIX shell would do.

use std::path::{Path, PathBuf};

/// Source-file extensions considered by [`expand_paths_to_source_files`].
///
/// Kept in sync with the mirror list in
/// `src/mcp_pmcp/tool_functions/analysis_tools.rs` (R17-2 / R21-4).
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "tsx", "jsx", "java", "cpp", "cc", "cxx", "c", "h", "hpp", "go", "rb",
    "php", "swift", "kt", "lua", "sh",
];

/// Returns `true` when `p` contains glob metacharacters (`*`, `?`, `[`).
///
/// Used to decide whether a raw input path should be routed through
/// [`glob::glob_with`] before any filesystem walk.
#[must_use]
pub fn path_has_glob_meta(p: &Path) -> bool {
    p.to_str()
        .is_some_and(|s| s.contains('*') || s.contains('?') || s.contains('['))
}

/// Expand shell-style glob patterns into concrete filesystem entries.
///
/// Non-glob paths pass through unchanged. Glob entries are matched with
/// `require_literal_separator = false`, letting `**` (and a bare `*`) cross
/// directory boundaries so that `**/*.rs`, `src/**`, and `src/**/*.rs` all
/// produce the results users expect. Patterns that match nothing yield an
/// empty set (no literal fallback, mirroring POSIX shell behavior).
///
/// # Rationale
///
/// R21-4 (D98): MCP analysis tools receive paths as raw strings converted to
/// `PathBuf`. A pattern like `"**/*.rs"` becomes a non-existent literal path;
/// `path.exists()` is false and the downstream walk yields authoritative zeros.
/// Expanding globs here makes the pipeline match the documented CLI
/// semantics (`rust-docs/cli-reference.md` lines 403-404).
///
/// R22-2 (D102): The live `handlers/tools` dispatcher inherits the same bug
/// through its `project_path: Option<String>` argument — a value like
/// `"src/**"` was analyzed as a non-existent literal directory. Sharing this
/// helper closes the parity gap.
#[must_use]
pub fn resolve_paths_with_globs(paths: &[PathBuf]) -> Vec<PathBuf> {
    use glob::{glob_with, MatchOptions};

    // `require_literal_separator = false` is the critical flag: it lets `**`
    // (and a bare `*`) match across `/` boundaries, which is what every user
    // expects from `**/*.rs`.
    let opts = MatchOptions {
        case_sensitive: true,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        if !path_has_glob_meta(path) {
            out.push(path.clone());
            continue;
        }
        let Some(pattern) = path.to_str() else {
            // Non-UTF8 globs are unsupported; preserve the literal PathBuf so
            // existence checks yield a clean zero rather than an error.
            out.push(path.clone());
            continue;
        };
        match glob_with(pattern, opts) {
            Ok(iter) => out.extend(iter.flatten()),
            Err(_) => {
                // Malformed pattern: fall back to literal (will miss later).
                out.push(path.clone());
            }
        }
    }
    out
}

/// Expand a list of paths into a flat list of source files.
///
/// When a path contains glob metacharacters (`*`, `?`, `[`) it is first
/// expanded via [`resolve_paths_with_globs`], then each resolved entry is
/// processed: files are included verbatim; directories are walked (via
/// `walkdir`) to enumerate all source files beneath them (see
/// [`SOURCE_EXTENSIONS`]). Hidden dirs, `target/`, and `node_modules/` are
/// skipped.
///
/// # Rationale
///
/// R17-2 (D82): MCP handlers previously only accepted files, returning zeros
/// for directory inputs ("authoritative zeros" regression).
/// R21-4 (D98): The same handlers also silently ignored glob-style inputs
/// like `**/*.rs` because the raw `PathBuf` was a non-existent literal.
/// R22-2 (D102): Live `handlers/tools` dispatcher now shares this helper.
#[must_use]
pub fn expand_paths_to_source_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let resolved = resolve_paths_with_globs(paths);
    let mut out = Vec::new();
    for path in &resolved {
        if !path.exists() {
            continue;
        }
        if path.is_file() {
            out.push(path.clone());
            continue;
        }
        if path.is_dir() {
            // ONE ignore policy, ONE implementation.
            //
            // This was a raw `WalkDir` filtering only hidden entries, `target`
            // and `node_modules`. It never read .gitignore/.ignore/.pmatignore
            // and never excluded minified or vendored assets, so the MCP tools
            // that reach files through here analysed a different population
            // than the CLI did for the same directory — and the two surfaces
            // then disagreed about the same repository.
            //
            // Measured on ~/src/cohete (10 .rs files) before this change:
            //
            // ```text
            //   pmat analyze complexity   ->  11 files, total cyclomatic    54
            //   mcp analyze_complexity    ->  27 files, total cyclomatic  2098
            // ```
            //
            // The extra 16 were generated mdbook output — `book/book/*.js` and
            // `book/out/*.js`, including a 137,537-byte `highlight.js` across
            // 53 lines. Minified vendor code has enormous branch counts, which
            // is where a 39x complexity gap comes from. They are untracked but
            // NOT gitignored, so this is the vendor/minified exclusion doing
            // the work, not .gitignore alone.
            //
            // `project_files`'s doc comment already records four analyzers that
            // hand-rolled this same walk (`cuda-tdg`, `validate-docs`,
            // `analyze assembly-script`, `analyze web-assembly`), each of which
            // descended into gitignored trees. This was the fifth, and the one
            // that made two TRANSPORTS disagree rather than two commands.
            //
            // A directory now goes through the discovery the CLI uses. An
            // explicitly named FILE is still passed straight through above:
            // asking for a specific file is an instruction, not a search, and
            // the caller may legitimately name something the walk would skip.
            match crate::services::file_discovery::ProjectFileDiscovery::new(path.clone())
                .discover_files()
            {
                Ok(found) => out.extend(found),
                // Discovery failing is not a licence to fall back to a walk
                // with a different policy — that would reintroduce the split
                // silently and only under error conditions, which is the worst
                // place for it to live. Yield nothing for this root; the caller
                // sees an empty population rather than a wrong one.
                Err(_) => {}
            }
        }
    }
    out
}

/// Expand a single MCP `project_path` argument (the shape the live
/// `handlers/tools` dispatcher uses) into a flat list of source files.
///
/// Semantics:
/// * Plain literal path → passed through
///   [`expand_paths_to_source_files`] (file or directory walk).
/// * Path containing glob metacharacters (`*`, `?`, `[`) → expanded via
///   [`resolve_paths_with_globs`] then each match is file-or-directory walked.
/// * Empty expansion → returns an empty `Vec`. Callers are expected to
///   fail loud with JSON-RPC `-32602` (see [`ResolvedProjectPath`]).
#[must_use]
pub fn expand_project_path(project_path: &str) -> Vec<PathBuf> {
    let buf = PathBuf::from(project_path);
    expand_paths_to_source_files(std::slice::from_ref(&buf))
}

/// Result of attempting to resolve a potentially-glob `project_path`.
///
/// R22-2 / D102: This is the single shared return type for both MCP
/// dispatcher trees (`src/handlers/tools/` and `src/mcp_pmcp/tool_functions/`)
/// when they adapt the shared glob helpers to their `project_path: String`
/// argument shape.
#[derive(Debug)]
pub enum ResolvedProjectPath {
    /// Literal or successfully-expanded concrete path (directory or file).
    Concrete(PathBuf),
    /// Glob pattern that matched nothing on disk. Callers should emit a
    /// JSON-RPC `-32602` error using [`Self::into_error_message`].
    EmptyGlob(String),
}

impl ResolvedProjectPath {
    /// Format a `-32602` error message for the "paths provided but resolve
    /// to zero files" contract from R22-2 / D102.
    #[must_use]
    pub fn into_error_message(self) -> String {
        match self {
            Self::EmptyGlob(pattern) => format!(
                "'project_path' contains a glob pattern ({pattern:?}) that matched \
zero files or directories — R22-2/D102 requires non-empty expansion"
            ),
            Self::Concrete(_) => String::new(),
        }
    }
}

/// Resolve a raw `project_path` string, expanding globs via
/// [`resolve_paths_with_globs`].
///
/// If the input contains no glob metacharacters the returned
/// [`ResolvedProjectPath::Concrete`] simply wraps `PathBuf::from(raw)` — no
/// filesystem I/O is performed, matching the pre-fix behavior for literal
/// inputs.
///
/// If the input contains `*`, `?`, or `[` the value is fed through
/// [`resolve_paths_with_globs`]. The first matching entry that is a
/// directory is preferred; otherwise the common parent of matching files is
/// used. Zero matches yields [`ResolvedProjectPath::EmptyGlob`] so the caller
/// can return JSON-RPC `-32602`.
#[must_use]
pub fn resolve_project_path_with_globs(raw: &str) -> ResolvedProjectPath {
    let path = PathBuf::from(raw);
    if !path_has_glob_meta(&path) {
        return ResolvedProjectPath::Concrete(path);
    }

    let resolved = resolve_paths_with_globs(std::slice::from_ref(&path));
    if resolved.is_empty() {
        return ResolvedProjectPath::EmptyGlob(raw.to_string());
    }

    // Prefer the first directory; this is the most natural "project root".
    if let Some(dir) = resolved.iter().find(|p| p.is_dir()) {
        return ResolvedProjectPath::Concrete(dir.clone());
    }

    // Otherwise collapse to the longest shared parent of matching files so
    // downstream services (which require a directory) get a usable root.
    let common = longest_common_parent(&resolved).unwrap_or_else(|| resolved[0].clone());
    ResolvedProjectPath::Concrete(common)
}

/// Longest shared ancestor directory of a list of paths.
///
/// Returns `None` when `paths` is empty. For a single path, returns its
/// parent (or the path itself if it has none).
fn longest_common_parent(paths: &[PathBuf]) -> Option<PathBuf> {
    let first = paths.first()?;
    let parent_to_components = |p: &PathBuf| -> Vec<std::ffi::OsString> {
        let parent = p.parent().unwrap_or(p.as_path());
        parent
            .components()
            .map(|c| c.as_os_str().to_os_string())
            .collect()
    };

    let mut common = parent_to_components(first);
    for p in paths.iter().skip(1) {
        let theirs = parent_to_components(p);
        let new_len = common
            .iter()
            .zip(theirs.iter())
            .take_while(|(a, b)| a == b)
            .count();
        common.truncate(new_len);
        if common.is_empty() {
            break;
        }
    }

    if common.is_empty() {
        None
    } else {
        let mut out = PathBuf::new();
        for c in common {
            out.push(c);
        }
        Some(out)
    }
}

#[cfg(test)]
mod parity_tests {
    use super::*;

    /// A directory walk must honour the project's ignore policy, so the MCP
    /// tools and the CLI describe the same population.
    ///
    /// RED: restore the raw `WalkDir` (hidden / `target` / `node_modules` only)
    /// and this fails — the minified file is admitted.
    ///
    /// The live case, measured on ~/src/cohete (10 .rs files):
    ///
    /// ```text
    ///   pmat analyze complexity   ->  11 files, total cyclomatic    54
    ///   mcp analyze_complexity    ->  27 files, total cyclomatic  2098
    /// ```
    ///
    /// The extra files were generated mdbook output — `book/book/*.js`,
    /// `book/out/*.js` — including a 137,537-byte `highlight.js` across 53
    /// lines. Minified vendor code carries enormous branch counts, which is
    /// where a 39x complexity gap comes from. After the fix: 19 files, 364.
    #[test]
    fn a_directory_walk_excludes_what_the_cli_excludes() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).expect("mkdir src");
        std::fs::create_dir_all(root.join("book/out")).expect("mkdir book");
        std::fs::write(root.join("src/lib.rs"), "pub fn f() -> i32 { 1 }\n").expect("lib.rs");

        // A minified vendor asset: one very long line, the shape that carries a
        // huge branch count and no meaning for a quality report.
        let minified = format!("var a={};{}", 1, "if(a){a++}else{a--};".repeat(2000));
        std::fs::write(root.join("book/out/highlight.js"), &minified).expect("highlight.js");

        let found = expand_paths_to_source_files(&[root.to_path_buf()]);
        let names: Vec<String> = found
            .iter()
            .map(|p| p.strip_prefix(root).unwrap_or(p).display().to_string())
            .collect();

        assert!(
            names.iter().any(|n| n.ends_with("lib.rs")),
            "the real source file must still be found, got {names:?}"
        );
        assert!(
            !names.iter().any(|n| n.contains("highlight.js")),
            "a minified vendor asset under book/ must not be analysed as source \
             — this is what made MCP report 27 files where the CLI reported 11. \
             got {names:?}"
        );
    }

    /// ...and an explicitly NAMED file is still passed straight through.
    ///
    /// The counter-test. Without it, "exclude everything" satisfies the
    /// assertion above. Naming a file is an instruction, not a search: a caller
    /// may legitimately ask about something a directory walk would skip.
    #[test]
    fn an_explicitly_named_file_is_never_filtered_out() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let odd = dir.path().join("vendor.min.js");
        std::fs::write(&odd, "var a=1;\n").expect("write");

        let found = expand_paths_to_source_files(std::slice::from_ref(&odd));
        assert_eq!(
            found,
            vec![odd],
            "a path named explicitly is an instruction, not a search"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper: build a small on-disk Rust project with root + nested files.
    fn make_rust_project(temp: &TempDir) -> std::io::Result<(PathBuf, PathBuf, PathBuf)> {
        let root = temp.path().join("root.rs");
        fs::write(&root, "fn root_fn() { let _ = 1 + 1; }")?;

        let src = temp.path().join("src");
        fs::create_dir_all(&src)?;
        let lib = src.join("lib.rs");
        fs::write(&lib, "pub fn lib_fn() { if true { 1 } else { 2 }; }")?;

        let nested = src.join("nested");
        fs::create_dir_all(&nested)?;
        let deep = nested.join("deep.rs");
        fs::write(&deep, "pub fn deep_fn() { for i in 0..3 { let _ = i; } }")?;

        Ok((root, lib, deep))
    }

    // ---- path_has_glob_meta --------------------------------------------

    #[test]
    fn path_has_glob_meta_star() {
        assert!(path_has_glob_meta(Path::new("src/**/*.rs")));
        assert!(path_has_glob_meta(Path::new("src/*.rs")));
    }

    #[test]
    fn path_has_glob_meta_question() {
        assert!(path_has_glob_meta(Path::new("src/lib?.rs")));
    }

    #[test]
    fn path_has_glob_meta_bracket() {
        assert!(path_has_glob_meta(Path::new("src/[ab].rs")));
    }

    #[test]
    fn path_has_glob_meta_plain() {
        assert!(!path_has_glob_meta(Path::new("src/lib.rs")));
        assert!(!path_has_glob_meta(Path::new("src")));
    }

    // ---- resolve_paths_with_globs --------------------------------------

    /// (a) literal path — passes through unchanged.
    #[test]
    fn resolve_literal_passthrough() {
        let temp = TempDir::new().unwrap();
        let (root, _, _) = make_rust_project(&temp).unwrap();
        let resolved = resolve_paths_with_globs(std::slice::from_ref(&root));
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0], root);
    }

    /// (b) `**` prefix glob — crosses directory boundaries.
    #[test]
    fn resolve_recursive_star_star() {
        let temp = TempDir::new().unwrap();
        make_rust_project(&temp).unwrap();

        let pattern = temp.path().join("**/*.rs");
        let resolved = resolve_paths_with_globs(&[pattern]);

        let rs_count = resolved
            .iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .count();
        assert_eq!(
            rs_count, 3,
            "expected 3 .rs files from **/*.rs, got {rs_count}: {resolved:?}"
        );
    }

    /// (c) nested glob — `src/**/*.rs` → 2 matches.
    #[test]
    fn resolve_dir_star_star_ext() {
        let temp = TempDir::new().unwrap();
        make_rust_project(&temp).unwrap();

        let pattern = temp.path().join("src").join("**").join("*.rs");
        let resolved = resolve_paths_with_globs(&[pattern]);

        let rs_files: Vec<_> = resolved
            .iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        assert_eq!(
            rs_files.len(),
            2,
            "expected 2 .rs files from src/**/*.rs, got {}: {resolved:?}",
            rs_files.len()
        );
    }

    /// (d) empty expansion — no matches yields empty Vec, not a literal.
    #[test]
    fn resolve_no_match_is_empty() {
        let pattern = PathBuf::from("/tmp/pmat-r22-2-d102-no-match-definitely-absent-*.rs");
        let resolved = resolve_paths_with_globs(&[pattern]);
        assert!(resolved.is_empty(), "expected empty, got {resolved:?}");
    }

    /// Shallow `*.rs` — top-level only.
    #[test]
    fn resolve_shallow_star() {
        let temp = TempDir::new().unwrap();
        make_rust_project(&temp).unwrap();

        let pattern = temp.path().join("*.rs");
        let resolved = resolve_paths_with_globs(&[pattern]);

        let rs: Vec<_> = resolved
            .iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        assert!(
            !rs.is_empty(),
            "expected at least 1 .rs file from *.rs, got 0: {resolved:?}"
        );
    }

    // ---- expand_paths_to_source_files ----------------------------------

    #[test]
    fn expand_paths_literal_directory_walks() {
        let temp = TempDir::new().unwrap();
        make_rust_project(&temp).unwrap();

        let files = expand_paths_to_source_files(&[temp.path().to_path_buf()]);
        let rs: Vec<_> = files
            .iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        assert_eq!(rs.len(), 3, "walked tree should yield 3 .rs: {files:?}");
    }

    #[test]
    fn expand_paths_glob_composes_with_walker() {
        let temp = TempDir::new().unwrap();
        make_rust_project(&temp).unwrap();

        // `src/**` resolves via the glob crate (0.3) to nested entries; the
        // walker then flattens any directories into their .rs members. The
        // exact count depends on how the crate interprets trailing `**` —
        // R21-4's acceptance contract only required "at least one match",
        // which is the behavior users care about in practice.
        let pattern = temp.path().join("src").join("**");
        let files = expand_paths_to_source_files(&[pattern]);
        let rs: Vec<_> = files
            .iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
            .collect();
        assert!(
            !rs.is_empty(),
            "src/** should yield at least one .rs file, got {files:?}"
        );
    }

    // ---- expand_project_path (the live-tree hook) ---------------------

    /// (a) live-tree: a literal project_path walks the directory.
    #[test]
    fn expand_project_path_literal_directory() {
        let temp = TempDir::new().unwrap();
        make_rust_project(&temp).unwrap();

        let files = expand_project_path(temp.path().to_str().unwrap());
        assert!(files.iter().any(|p| p.ends_with("root.rs")));
        assert!(files.iter().any(|p| p.ends_with("lib.rs")));
    }

    /// (b) live-tree: `**` prefix glob.
    #[test]
    fn expand_project_path_double_star_glob() {
        let temp = TempDir::new().unwrap();
        make_rust_project(&temp).unwrap();

        let pattern = temp.path().join("**/*.rs");
        let files = expand_project_path(pattern.to_str().unwrap());
        assert!(
            files.len() >= 3,
            "expected >=3 files from **/*.rs, got {files:?}"
        );
    }

    /// (c) live-tree: nested `src/**/*.rs`.
    #[test]
    fn expand_project_path_nested_glob() {
        let temp = TempDir::new().unwrap();
        make_rust_project(&temp).unwrap();

        let pattern = temp.path().join("src").join("**").join("*.rs");
        let files = expand_project_path(pattern.to_str().unwrap());
        assert!(
            files.len() >= 2,
            "expected >=2 files from src/**/*.rs, got {files:?}"
        );
    }

    /// (d) live-tree: empty expansion so callers can fail loud.
    #[test]
    fn expand_project_path_empty_expansion() {
        // An unmatched glob under a tmp path yields zero files.
        let bogus = "/tmp/pmat-r22-2-d102-empty-*.rs";
        let files = expand_project_path(bogus);
        assert!(files.is_empty(), "expected empty expansion, got {files:?}");
    }
}
