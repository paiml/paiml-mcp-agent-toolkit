//! Which tracked `.rs` files does the build actually compile?
//!
//! pmat's unit of analysis has always been the FILE — discovery is
//! `git ls-files` filtered by extension, and every analyzer inherits that
//! universe. Rust's unit of compilation is the TARGET, reached through a module
//! graph. Nothing reconciled the two, and rustc emits no diagnostic for a `.rs`
//! file that no `mod`, `#[path]` or `include!` reaches.
//!
//! The gap is not theoretical. A stack-wide audit found ~475 tracked files,
//! over 320,000 lines and ~8,900 `#[test]` functions that no compilation unit
//! reaches (depyler 109 files, bashrs 239, aprender 105, whisper.apr 121).
//! pmat had its own: `src/transport/`, 1434 lines and 26 tests, where
//! `cargo test <name>` printed "0 passed" and exited 0.
//!
//! Worse than missing them, pmat **graded** them: 79 of aprender's orphans
//! appear as scored keys in its baseline, and pepita's orphaned
//! `verification_specs.rs` is recorded AMinus / 97.27 / confidence 1.0.
//!
//! This module answers the question directly — walk the module graph from every
//! target root `cargo metadata` declares, and report the tracked files it never
//! arrives at.

use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

/// A tracked `.rs` file no compilation unit reaches.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Orphan {
    /// Path relative to the project root.
    pub path: String,
    /// Lines in the file — orphaned volume is the headline number.
    pub lines: usize,
    /// `#[test]` functions that therefore never run.
    pub tests: usize,
}

/// What the walk found, including what it could not do.
///
/// `unresolved` is not cosmetic. A `mod` whose file cannot be located means the
/// graph is incomplete, so an "orphan" may simply be something this walker
/// failed to follow. Reporting a count of orphans without it would be the same
/// defect this module exists to fix — a number with no denominator.
#[derive(Debug, Clone, Default)]
pub struct Report {
    /// Tracked `.rs` files reached from some target root.
    pub reachable: usize,
    /// Tracked `.rs` files reached from nothing.
    pub orphans: Vec<Orphan>,
    /// Target roots enumerated from the manifest.
    pub roots: usize,
    /// `mod` declarations whose file could not be found on disk.
    pub unresolved: Vec<String>,
}

impl Report {
    #[must_use]
    pub fn orphan_lines(&self) -> usize {
        self.orphans.iter().map(|o| o.lines).sum()
    }

    #[must_use]
    pub fn orphan_tests(&self) -> usize {
        self.orphans.iter().map(|o| o.tests).sum()
    }

    /// A one-line verdict that always states the scope it was measured over.
    #[must_use]
    pub fn summary(&self) -> String {
        let scanned = self.reachable + self.orphans.len();
        if self.orphans.is_empty() {
            return format!(
                "all {scanned} tracked .rs file(s) are reachable from {} target root(s)",
                self.roots
            );
        }
        let mut s = format!(
            "{} of {scanned} tracked .rs file(s) are reachable from {} target root(s); \
             {} unreachable ({} lines, {} #[test] fns that never run)",
            self.reachable,
            self.roots,
            self.orphans.len(),
            self.orphan_lines(),
            self.orphan_tests()
        );
        if !self.unresolved.is_empty() {
            s.push_str(&format!(
                " — {} `mod` declaration(s) could not be resolved, so this is a FLOOR, not a total",
                self.unresolved.len()
            ));
        }
        s
    }
}

/// Module declarations in one file: `mod x;`, `#[path = "y.rs"] mod x;`,
/// `include!("z.rs")`.
///
/// Deliberately a scanner rather than a full parser. It must not follow a `mod`
/// that appears inside a string literal or a comment, because over-reporting
/// reachability is the dangerous direction: it would mark an orphan as live and
/// hide exactly what this module looks for.
fn declarations(src: &str) -> (Vec<String>, Vec<String>) {
    let mut mods = Vec::new();
    let mut paths = Vec::new();
    let mut pending_path: Option<String> = None;

    for raw in src.lines() {
        // Strip a trailing line comment BEFORE parsing. `pub mod agent; // …`
        // is extremely common, and matching on a bare `;` suffix silently
        // dropped every such declaration — which orphaned the whole subtree
        // beneath it and produced 2421 false positives on this repo alone.
        let no_comment = match raw.find("//") {
            Some(i) => &raw[..i],
            None => raw,
        };
        let line = no_comment.trim();
        if line.is_empty() {
            continue;
        }
        // `#[path = "…"]` applies to the NEXT mod declaration.
        if let Some(rest) = line.strip_prefix("#[path") {
            if let Some(v) = rest.split('"').nth(1) {
                pending_path = Some(v.to_string());
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("include!") {
            if let Some(v) = rest.split('"').nth(1) {
                paths.push(v.to_string());
            }
            continue;
        }
        // `mod x;` / `pub mod x;` / `pub(crate) mod x;` — a declaration, not an
        // inline `mod x {` block, which needs no file.
        let after_vis = line
            .strip_prefix("pub(crate) ")
            .or_else(|| line.strip_prefix("pub(super) "))
            .or_else(|| line.strip_prefix("pub "))
            .unwrap_or(line);
        if let Some(rest) = after_vis.strip_prefix("mod ") {
            if let Some(name) = rest.strip_suffix(';') {
                let name = name.trim();
                if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    match pending_path.take() {
                        Some(p) => paths.push(p),
                        None => mods.push(name.to_string()),
                    }
                }
            } else {
                // `mod x {` — inline, no file, but a `#[path]` before it is spent.
                pending_path = None;
            }
        }
    }
    (mods, paths)
}

/// Where `mod name;` inside `owner` resolves to: `dir/name.rs` or
/// `dir/name/mod.rs`.
///
/// `is_root` is load-bearing. A CRATE ROOT — lib.rs, main.rs, and equally every
/// bin, test, example and bench entry point — resolves its modules from its own
/// directory, exactly like `mod.rs`. Only a non-root `foo.rs` owns a `foo/`
/// subdirectory. Treating `tests/all.rs` as a non-root looked for
/// `tests/all/modules/` instead of `tests/modules/`, failed, and orphaned all
/// 233 files under `tests/` in one go.
fn resolve_mod(owner: &Path, name: &str, is_root: bool) -> Option<PathBuf> {
    let dir = if is_root
        || owner.file_name().and_then(|s| s.to_str()) == Some("mod.rs")
        || owner.file_stem().and_then(|s| s.to_str()) == Some("lib")
        || owner.file_stem().and_then(|s| s.to_str()) == Some("main")
    {
        owner.parent()?.to_path_buf()
    } else {
        owner.parent()?.join(owner.file_stem()?)
    };
    [dir.join(format!("{name}.rs")), dir.join(name).join("mod.rs")]
        .into_iter()
        .find(|c| c.is_file())
}

fn count_tests(src: &str) -> usize {
    src.lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("#[test]") || t.starts_with("#[tokio::test")
        })
        .count()
}

/// Walk every target root and report tracked files nothing reaches.
///
/// `roots` are the target source paths (lib.rs, main.rs, each bin/example/bench)
/// and `tracked` the repository's tracked `.rs` files, both supplied by the
/// caller so this stays pure and testable — no cargo invocation, no git.
#[must_use]
pub fn analyze(project_root: &Path, roots: &[PathBuf], tracked: &[PathBuf]) -> Report {
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    let mut unresolved = Vec::new();
    let root_set: BTreeSet<PathBuf> = roots.iter().filter_map(|p| p.canonicalize().ok()).collect();
    let mut queue: VecDeque<PathBuf> = roots.iter().filter(|p| p.is_file()).cloned().collect();

    while let Some(file) = queue.pop_front() {
        let canonical = file.canonicalize().unwrap_or_else(|_| file.clone());
        if !seen.insert(canonical) {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(&file) else {
            continue;
        };
        let is_root = file
            .canonicalize()
            .map(|c| root_set.contains(&c))
            .unwrap_or(false);
        let (mods, paths) = declarations(&src);
        for name in mods {
            match resolve_mod(&file, &name, is_root) {
                Some(p) => queue.push_back(p),
                None => unresolved.push(format!(
                    "{}: mod {name};",
                    file.strip_prefix(project_root).unwrap_or(&file).display()
                )),
            }
        }
        for rel in paths {
            if let Some(parent) = file.parent() {
                let p = parent.join(&rel);
                if p.is_file() {
                    queue.push_back(p);
                }
            }
        }
    }

    let mut orphans = Vec::new();
    let mut reachable = 0usize;
    for t in tracked {
        let abs = if t.is_absolute() {
            t.clone()
        } else {
            project_root.join(t)
        };
        let canonical = abs.canonicalize().unwrap_or_else(|_| abs.clone());
        if seen.contains(&canonical) {
            reachable += 1;
            continue;
        }
        let src = std::fs::read_to_string(&abs).unwrap_or_default();
        orphans.push(Orphan {
            path: t.display().to_string(),
            lines: src.lines().count(),
            tests: count_tests(&src),
        });
    }
    orphans.sort();

    Report {
        reachable,
        orphans,
        roots: roots.len(),
        unresolved,
    }
}

/// Target roots and tracked files for a cargo project, for CLI use.
///
/// Split from [`analyze`] so the walk itself stays pure: this is the only part
/// that shells out, and it is the part that cannot be unit-tested without a
/// real repository.
pub fn discover(project_root: &Path) -> std::io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    use std::process::Command;

    let meta = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(project_root)
        .output()?;
    let json: serde_json::Value = serde_json::from_slice(&meta.stdout).unwrap_or_default();
    let mut roots = Vec::new();
    if let Some(pkgs) = json.get("packages").and_then(|p| p.as_array()) {
        for p in pkgs {
            for t in p
                .get("targets")
                .and_then(|t| t.as_array())
                .into_iter()
                .flatten()
            {
                if let Some(src) = t.get("src_path").and_then(|s| s.as_str()) {
                    roots.push(PathBuf::from(src));
                }
            }
        }
    }

    let ls = Command::new("git")
        .args(["ls-files", "-z", "--", "*.rs"])
        .current_dir(project_root)
        .output()?;
    let tracked = String::from_utf8_lossy(&ls.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();

    Ok((roots, tracked))
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, body: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    /// The shape that cost pmat 1434 lines and 26 tests: a directory on disk
    /// that no `mod` declaration reaches. `cargo test <name>` printed
    /// "0 passed" and exited 0 for three years.
    #[test]
    fn an_undeclared_module_is_reported_as_unreachable() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "src/lib.rs", "pub mod live;\n");
        write(root, "src/live.rs", "pub fn f() {}\n");
        write(
            root,
            "src/orphan/mod.rs",
            "pub fn g() {}\n#[test]\nfn t1() {}\n#[test]\nfn t2() {}\n",
        );

        let report = analyze(
            root,
            &[root.join("src/lib.rs")],
            &[
                PathBuf::from("src/lib.rs"),
                PathBuf::from("src/live.rs"),
                PathBuf::from("src/orphan/mod.rs"),
            ],
        );

        assert_eq!(report.reachable, 2, "lib.rs and live.rs are reachable");
        assert_eq!(report.orphans.len(), 1, "{:?}", report.orphans);
        assert_eq!(report.orphans[0].path, "src/orphan/mod.rs");
        assert_eq!(
            report.orphans[0].tests, 2,
            "the tests in an unreachable file never run, and that is the number that matters"
        );
        assert!(
            report.summary().contains("never run"),
            "{}",
            report.summary()
        );
    }

    /// `#[path]` and `include!` are how this repo actually wires several
    /// modules, so a walker that only understands `mod x;` would report live
    /// files as orphans — the dangerous direction, since it manufactures work.
    #[test]
    fn path_attribute_and_include_are_followed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(
            root,
            "src/lib.rs",
            "#[path = \"elsewhere/thing.rs\"]\nmod thing;\ninclude!(\"inlined.rs\");\n",
        );
        write(root, "src/elsewhere/thing.rs", "pub fn a() {}\n");
        write(root, "src/inlined.rs", "pub fn b() {}\n");

        let report = analyze(
            root,
            &[root.join("src/lib.rs")],
            &[
                PathBuf::from("src/lib.rs"),
                PathBuf::from("src/elsewhere/thing.rs"),
                PathBuf::from("src/inlined.rs"),
            ],
        );
        assert!(
            report.orphans.is_empty(),
            "#[path] and include! targets are reachable: {:?}",
            report.orphans
        );
    }

    /// A `mod` inside a comment must not create reachability. Over-reporting
    /// reachability hides orphans, which is the failure this module exists to
    /// prevent — so it is the direction to be strict about.
    #[test]
    fn commented_out_mod_does_not_make_a_file_reachable() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "src/lib.rs", "// mod ghost;\npub fn f() {}\n");
        write(root, "src/ghost.rs", "pub fn g() {}\n");

        let report = analyze(
            root,
            &[root.join("src/lib.rs")],
            &[PathBuf::from("src/lib.rs"), PathBuf::from("src/ghost.rs")],
        );
        assert_eq!(report.orphans.len(), 1, "{:?}", report.orphans);
        assert_eq!(report.orphans[0].path, "src/ghost.rs");
    }

    /// An unresolvable `mod` means the graph is incomplete, so the orphan count
    /// is a floor. Saying otherwise would repeat the defect this module fixes.
    #[test]
    fn an_unresolvable_mod_is_disclosed_and_downgrades_the_claim() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "src/lib.rs", "mod missing;\n");

        let report = analyze(
            root,
            &[root.join("src/lib.rs")],
            &[PathBuf::from("src/lib.rs")],
        );
        assert_eq!(report.unresolved.len(), 1, "{:?}", report.unresolved);
        assert!(report.unresolved[0].contains("mod missing"));
    }

    #[test]
    fn a_fully_wired_tree_reports_clean_and_says_what_it_scanned() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write(root, "src/lib.rs", "pub mod a;\n");
        write(root, "src/a.rs", "pub fn f() {}\n");

        let report = analyze(
            root,
            &[root.join("src/lib.rs")],
            &[PathBuf::from("src/lib.rs"), PathBuf::from("src/a.rs")],
        );
        assert!(report.orphans.is_empty());
        assert!(
            report.summary().contains("all 2 tracked"),
            "{}",
            report.summary()
        );
    }
}

