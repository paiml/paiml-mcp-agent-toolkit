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
    /// Tracked `.rs` files reached from some target root by an edge that
    /// COMPILES — i.e. not through the quarantine feature.
    pub reachable: usize,
    /// Tracked `.rs` files reached from nothing.
    pub orphans: Vec<Orphan>,
    /// Tracked `.rs` files reached ONLY through a [`QUARANTINE_FEATURE`] edge.
    ///
    /// Not orphans — something declares them — and not reachable either, because
    /// the declaration is behind a feature that is in no bundle and does not
    /// compile. Counting them as reachable is what hid them: 47 modules whose
    /// tests exist, are tracked, are counted by every file-based metric, and run
    /// in no build. See #1023.
    pub quarantined: Vec<Orphan>,
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

    /// `#[test]` functions inside quarantined modules — declared, tracked, and
    /// executed by no build.
    #[must_use]
    pub fn quarantined_tests(&self) -> usize {
        self.quarantined.iter().map(|o| o.tests).sum()
    }

    /// Lines inside quarantined modules.
    #[must_use]
    pub fn quarantined_lines(&self) -> usize {
        self.quarantined.iter().map(|o| o.lines).sum()
    }

    /// A one-line verdict that always states the scope it was measured over.
    ///
    /// Quarantined files are reported SEPARATELY from orphans and never folded
    /// into `reachable`. They are a third state, and collapsing them into either
    /// of the other two loses the distinction that matters: an orphan is
    /// declared by nothing, a quarantined module is declared by something that
    /// does not compile, and both run exactly as often.
    #[must_use]
    pub fn summary(&self) -> String {
        let scanned = self.reachable + self.orphans.len() + self.quarantined.len();
        let mut s = if self.orphans.is_empty() {
            format!(
                "all {scanned} tracked .rs file(s) are reachable from {} target root(s)",
                self.roots
            )
        } else {
            format!(
                "{} of {scanned} tracked .rs file(s) are reachable from {} target root(s); \
                 {} unreachable ({} lines, {} #[test] fns that never run)",
                self.reachable,
                self.roots,
                self.orphans.len(),
                self.orphan_lines(),
                self.orphan_tests()
            )
        };
        if !self.quarantined.is_empty() {
            s.push_str(&format!(
                "; {} quarantined behind `pmat_broken_tests` ({} lines, {} #[test] fns that \
                 no build compiles) — declared, so not orphans, but they run exactly as often",
                self.quarantined.len(),
                self.quarantined_lines(),
                self.quarantined_tests()
            ));
        }
        if !self.unresolved.is_empty() {
            s.push_str(&format!(
                " — {} `mod` declaration(s) could not be resolved, so this is a FLOOR, not a total",
                self.unresolved.len()
            ));
        }
        s
    }
}

/// The feature that marks a deliberately non-compiling quarantine (#1023).
///
/// `pmat_broken_tests` is set by no build, so a `mod` behind it is compiled out of every
/// build anybody runs. The declaration still exists, which is the problem: a
/// file reached only through one of these edges looked REACHABLE to this walker
/// while its tests ran in no build at all — the same "looks measured and is not"
/// shape this module was written to expose, one level down.
const QUARANTINE_FEATURE: &str = "pmat_broken_tests";

/// One outgoing edge: the module name or `#[path]`/`include!` target, and
/// whether the declaration that produced it sits behind [`QUARANTINE_FEATURE`].
type Edge = (String, bool);

/// Module declarations in one file: `mod x;`, `#[path = "y.rs"] mod x;`,
/// `include!("z.rs")`.
///
/// Each edge carries whether it is QUARANTINED — declared behind
/// [`QUARANTINE_FEATURE`], so rustc never follows it in any build that ships.
///
/// Deliberately a scanner rather than a full parser. It must not follow a `mod`
/// that appears inside a string literal or a comment, because over-reporting
/// reachability is the dangerous direction: it would mark an orphan as live and
/// hide exactly what this module looks for.
fn declarations(src: &str) -> (Vec<Edge>, Vec<Edge>) {
    let mut mods = Vec::new();
    let mut paths = Vec::new();
    let mut pending_path: Option<String> = None;
    let mut pending_quarantine = false;

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
        // A `#[cfg(...)]` naming the quarantine feature applies to the NEXT
        // declaration, exactly like `#[path]`. Both spellings occur in the tree:
        // `#[cfg(all(test, pmat_broken_tests))]` and the bare
        // `#[cfg(pmat_broken_tests)]`.
        if line.starts_with("#[cfg(") && line.contains(QUARANTINE_FEATURE) {
            pending_quarantine = true;
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
                paths.push((v.to_string(), pending_quarantine));
            }
            pending_quarantine = false;
            continue;
        }
        // This line CONSUMES the pending marker, whatever it turns out to be —
        // taken, not merely read, exactly like `pending_path` below.
        //
        // Taking it is what makes the marker apply to the next declaration and
        // nothing further. Clearing it with a bare `if !line.starts_with("#[")`
        // instead cleared it on the very `mod tests;` line it was meant to
        // describe, so every quarantined edge read as live and the whole feature
        // reported zero. Leaving it set until some later line clears it has the
        // opposite failure: a `#[cfg(pmat_broken_tests)]` on a FUNCTION
        // would leak onto an unrelated `mod` further down and report a live
        // module as quarantined.
        let quarantined = std::mem::take(&mut pending_quarantine);
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
                        Some(p) => paths.push((p, quarantined)),
                        None => mods.push((name.to_string(), quarantined)),
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
    [
        dir.join(format!("{name}.rs")),
        dir.join(name).join("mod.rs"),
    ]
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
/// One breadth-first walk from the target roots.
///
/// `follow_quarantined` selects which graph is walked. Running it twice — once
/// refusing quarantined edges, once accepting them — is what separates the three
/// states, and it is done this way rather than by threading a flag through the
/// queue because a file can be reached by BOTH a live and a quarantined edge.
/// Only the set difference answers "reached ONLY through the quarantine"; a
/// per-edge flag would answer "the last edge that reached it", which is
/// whichever the queue happened to pop first.
fn walk(
    project_root: &Path,
    roots: &[PathBuf],
    follow_quarantined: bool,
) -> (BTreeSet<PathBuf>, Vec<String>) {
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
        for (name, quarantined) in mods {
            if quarantined && !follow_quarantined {
                continue;
            }
            match resolve_mod(&file, &name, is_root) {
                Some(p) => queue.push_back(p),
                None => unresolved.push(format!(
                    "{}: mod {name};",
                    file.strip_prefix(project_root).unwrap_or(&file).display()
                )),
            }
        }
        for (rel, quarantined) in paths {
            if quarantined && !follow_quarantined {
                continue;
            }
            if let Some(parent) = file.parent() {
                let p = parent.join(&rel);
                if p.is_file() {
                    queue.push_back(p);
                }
            }
        }
    }
    (seen, unresolved)
}

#[must_use]
pub fn analyze(project_root: &Path, roots: &[PathBuf], tracked: &[PathBuf]) -> Report {
    // Live graph: what rustc actually compiles. Quarantined graph: that plus the
    // edges behind `pmat_broken_tests`. `unresolved` comes from the FULL walk so an
    // unfollowable quarantined `mod` still degrades the answer to a floor rather
    // than disappearing.
    let (live_seen, _) = walk(project_root, roots, false);
    let (all_seen, unresolved) = walk(project_root, roots, true);

    let mut orphans = Vec::new();
    let mut quarantined = Vec::new();
    let mut reachable = 0usize;
    for t in tracked {
        let abs = if t.is_absolute() {
            t.clone()
        } else {
            project_root.join(t)
        };
        let canonical = abs.canonicalize().unwrap_or_else(|_| abs.clone());
        if live_seen.contains(&canonical) {
            reachable += 1;
            continue;
        }
        let src = std::fs::read_to_string(&abs).unwrap_or_default();
        let entry = Orphan {
            path: t.display().to_string(),
            lines: src.lines().count(),
            tests: count_tests(&src),
        };
        if all_seen.contains(&canonical) {
            quarantined.push(entry);
        } else {
            orphans.push(entry);
        }
    }
    orphans.sort();
    quarantined.sort();

    Report {
        reachable,
        orphans,
        quarantined,
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

    /// The quarantine attribute, ASSEMBLED rather than written out.
    ///
    /// `broken_tests_quarantine_tests.rs` censuses the tree by looking for a
    /// line that starts with `#[` and names the marker, and caps the count at a
    /// ceiling that may only fall. A fixture written as a literal
    /// `"#[cfg(all(test, pmat_broken_tests))]\n"` inside a raw-string block is
    /// indistinguishable from a real declaration at that level — it added two
    /// phantom sites and pushed the census over its ceiling. Building the string
    /// from this const keeps the fixtures honest without weakening the census
    /// with a path exclusion, which is the #923 mistake.
    const Q_ATTR: &str = "#[cfg(all(test, pmat_broken_tests))]";

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

    /// #1023: a module declared ONLY behind `pmat_broken_tests` is a third state.
    ///
    /// It is not an orphan — something declares it, so a search for "who
    /// references this file" finds an answer — and it is not reachable, because
    /// the feature is in no bundle and rustc never follows the edge. Before this
    /// distinction existed the walker counted it as REACHABLE, which is how 82
    /// files and 2,021 `#[test]` functions in this repository read as measured
    /// while running in no build at all.
    #[test]
    fn a_module_declared_only_behind_broken_tests_is_quarantined_not_reachable() {
        let tmp = tempfile::tempdir().expect("tempdir for the reachability fixture");
        let root = tmp.path();
        write(
            root,
            "src/lib.rs",
            &format!("pub mod live;\n{Q_ATTR}\n#[path = \"quarantined.rs\"]\nmod quarantined;\n"),
        );
        write(root, "src/live.rs", "pub fn f() {}\n");
        write(
            root,
            "src/quarantined.rs",
            "pub fn g() {}\n#[test]\nfn t1() {}\n#[test]\nfn t2() {}\n",
        );

        let report = analyze(
            root,
            &[root.join("src/lib.rs")],
            &[
                PathBuf::from("src/lib.rs"),
                PathBuf::from("src/live.rs"),
                PathBuf::from("src/quarantined.rs"),
            ],
        );

        assert!(
            report.orphans.is_empty(),
            "a declared module is not an orphan: {:?}",
            report.orphans
        );
        assert_eq!(
            report.quarantined.len(),
            1,
            "expected exactly the quarantined file, got {:?}",
            report.quarantined
        );
        assert_eq!(report.quarantined[0].path, "src/quarantined.rs");
        assert_eq!(
            report.quarantined_tests(),
            2,
            "the two #[test] fns behind the quarantine must be counted"
        );
        assert_eq!(
            report.reachable, 2,
            "the quarantined file must NOT be counted as reachable — that is the defect"
        );
        assert!(
            report
                .summary()
                .contains("quarantined behind `pmat_broken_tests`"),
            "the summary must name the third state: {}",
            report.summary()
        );
    }

    /// The control for the test above. Change ONLY the cfg, and the same file
    /// must come back reachable.
    ///
    /// Without this, a bug that marked every `#[path]` edge quarantined would
    /// satisfy the assertions above and look like a working feature.
    #[test]
    fn the_same_module_declared_without_the_feature_is_plain_reachable() {
        let tmp = tempfile::tempdir().expect("tempdir for the reachability fixture");
        let root = tmp.path();
        write(
            root,
            "src/lib.rs",
            "pub mod live;\n\
             #[cfg(test)]\n\
             #[path = \"quarantined.rs\"]\n\
             mod quarantined;\n",
        );
        write(root, "src/live.rs", "pub fn f() {}\n");
        write(
            root,
            "src/quarantined.rs",
            "pub fn g() {}\n#[test]\nfn t1() {}\n",
        );

        let report = analyze(
            root,
            &[root.join("src/lib.rs")],
            &[
                PathBuf::from("src/lib.rs"),
                PathBuf::from("src/live.rs"),
                PathBuf::from("src/quarantined.rs"),
            ],
        );
        assert!(
            report.quarantined.is_empty(),
            "only `pmat_broken_tests` marks a quarantine, not any cfg: {:?}",
            report.quarantined
        );
        assert_eq!(report.reachable, 3);
    }

    /// A file reached by BOTH a live and a quarantined edge is REACHABLE.
    ///
    /// This is why the implementation walks twice and takes a set difference
    /// rather than tagging each visited file with the flag of the edge that
    /// reached it: which edge arrives first is queue order, so a per-edge tag
    /// would report this file quarantined about half the time.
    #[test]
    fn a_file_reached_by_both_a_live_and_a_quarantined_edge_is_reachable() {
        let tmp = tempfile::tempdir().expect("tempdir for the reachability fixture");
        let root = tmp.path();
        write(
            root,
            "src/lib.rs",
            &format!("pub mod a;\n{Q_ATTR}\n#[path = \"shared.rs\"]\nmod shared_q;\n"),
        );
        write(
            root,
            "src/a.rs",
            "#[path = \"shared.rs\"]\nmod shared_live;\n",
        );
        write(root, "src/shared.rs", "pub fn g() {}\n");

        let report = analyze(
            root,
            &[root.join("src/lib.rs")],
            &[
                PathBuf::from("src/lib.rs"),
                PathBuf::from("src/a.rs"),
                PathBuf::from("src/shared.rs"),
            ],
        );
        assert!(
            report.quarantined.is_empty(),
            "a file with any live edge is reachable: {:?}",
            report.quarantined
        );
        assert_eq!(report.reachable, 3);
    }

    /// The marker applies to the NEXT declaration and nothing further.
    ///
    /// A `#[cfg(pmat_broken_tests)]` on a function must not leak onto a
    /// `mod` declared later in the same file. The first implementation cleared
    /// the flag on any non-attribute line, which cleared it on the very
    /// `mod tests;` line it described and reported ZERO quarantined files on a
    /// tree with 82 of them — a feature that silently did nothing.
    #[test]
    fn the_marker_does_not_leak_onto_a_later_declaration() {
        let src =
            format!("#[cfg({QUARANTINE_FEATURE})]\nfn disabled_helper() {{}}\npub mod later;\n");
        let src = src.as_str();
        let (mods, _) = declarations(src);
        assert_eq!(
            mods,
            vec![("later".to_string(), false)],
            "the marker leaked past the function it was attached to"
        );
    }
}
