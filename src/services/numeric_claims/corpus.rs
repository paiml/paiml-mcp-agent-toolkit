//! The corpus CB-2104 reads, and the one exclusion that is also a rule.
//!
//! Git-tracked files at HEAD, nothing else. Untracked and gitignored state is
//! invisible on purpose: the research lane's `QUALITY.md:81` false positive came
//! from a predicate reading an untracked `.git/hooks` file that a local
//! installer rewrites, which produces a different verdict per developer, per
//! day.
//!
//! Every exclusion below carries the reason it exists, because an exclusion is
//! a place where a wrong number survives and the census has to be able to say
//! how many:
//!
//! ```text
//! Cargo.lock, vendor/, target/, node_modules/, *.min.js  machine-managed
//! fixtures/ testdata/ golden/ snapshots/ baselines/      planted numbers are the point
//! .pmat/ .pmat-work/ dist/                               generated state trees
//! CHANGELOG*                                             a record of PAST state
//! *.json (for R1)                                        interchange, not authored prose
//! ```
//!
//! `CHANGELOG*` is not a convenience. A changelog is a record of what was true
//! at a past release; flagging it for not re-measuring deleted code was the
//! single largest false-positive source in the research lane. Excluding
//! `*.json` from R1 is the same argument the design already used to discard 93%
//! of its corpus: pulling aprender's 3,318 machine-written `contract.json`
//! files in yields 3.3M numerals and not one human claim.
//!
//! ## G1 — generated files
//!
//! A machine-written number cannot be a false claim: nobody asserted it. G1
//! detects generated files two ways, because each catches files the other
//! misses — a filename convention (`generated_contracts.rs`, `foo.gen.rs`) and
//! a DO-NOT-EDIT marker in the first five lines. Every flagship false positive
//! the cohort lane produced before this guard sat in
//! `crates/*/src/generated_contracts.rs`, whose line 1 reads
//! `// Auto-generated contract assertions from YAML — DO NOT EDIT.`
//!
//! Generated files are **kept in the corpus and stamped**, never dropped here.
//! Suppression is counted where it happens, in [`super::cohort`], so the census
//! can print how many sites the guard hid. A guard that can hide a finding must
//! say how often it did.

use regex::Regex;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

/// File types R1 reads. `*.json` is deliberately absent — see the module note.
pub const R1_PATHSPECS: &[&str] = &["*.md", "*.rs", "*.toml", "*.yaml", "*.yml"];

/// File types R2 reads: everything R1 reads, plus JSON.
///
/// R2 may read interchange formats because a config key is a config key
/// whatever the syntax carrying it, and R2 identifies quantities by name. R1
/// may not, because it identifies them by *sentence*, and a machine-written
/// `contract.json` has no sentences — only 3.3M numerals.
///
/// The two lanes therefore run over different corpora, which is why the census
/// carries a separate `r1_files_scanned`. A single scan collects the union and
/// R1 is handed the subset; nothing runs `git ls-files` twice.
pub const R2_PATHSPECS: &[&str] = &["*.md", "*.rs", "*.toml", "*.yaml", "*.yml", "*.json"];

/// Does this path belong to R1's corpus as well as R2's?
///
/// The one difference is JSON, so this is a suffix test rather than a second
/// pathspec walk.
pub fn in_r1_corpus(path: &str) -> bool {
    !path.ends_with(".json")
}

/// How a file was found to be machine-written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generated {
    /// The path follows a generated-file naming convention.
    ByName,
    /// One of the first five lines carries a DO-NOT-EDIT marker.
    ByMarker,
}

/// Why a path is not in the corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exclusion {
    /// Lockfiles, vendored trees, build output: nobody wrote these by hand.
    MachineManaged,
    /// Fixture and state trees, where a planted number is the point.
    FixtureTree,
    /// A record of past state, not a claim about now.
    Changelog,
}

impl Exclusion {
    /// A short, stable label for the census.
    pub fn label(self) -> &'static str {
        match self {
            Self::MachineManaged => "machine-managed",
            Self::FixtureTree => "fixture-tree",
            Self::Changelog => "changelog",
        }
    }
}

/// The corpus could not be established. Never a zero — an unreadable tree must
/// surface as UNMEASURABLE, never as "clean".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpusError {
    /// `git ls-files` refused: not a work tree, or a broken repository.
    NotAGitWorkTree(String),
    /// `git` itself could not be run.
    GitUnavailable(String),
}

impl std::fmt::Display for CorpusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAGitWorkTree(detail) => {
                write!(f, "not a git work tree: {detail}")
            }
            Self::GitUnavailable(detail) => write!(f, "could not run git: {detail}"),
        }
    }
}

impl std::error::Error for CorpusError {}

/// What the corpus pass saw. Printed, never inferred.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorpusCensus {
    /// Paths `git ls-files` returned for the requested pathspecs.
    pub tracked: usize,
    /// Files that were read and handed on.
    pub scanned: usize,
    /// Dropped as machine-managed.
    pub excluded_machine_managed: usize,
    /// Dropped as a fixture or state tree.
    pub excluded_fixture_tree: usize,
    /// Dropped as a record of past state.
    pub excluded_changelog: usize,
    /// Scanned, and stamped generated by filename.
    pub generated_by_name: usize,
    /// Scanned, and stamped generated by a DO-NOT-EDIT marker.
    pub generated_by_marker: usize,
    /// Tracked but unreadable. Counted, so a vanished corpus cannot read as an
    /// empty one.
    pub unreadable: usize,
}

/// Substring probes for machine-managed paths. Matched against `/{path}` so a
/// top-level `vendor/x` is caught by the same probe as `a/vendor/x`.
const MACHINE_MANAGED: &[&str] = &[
    "/Cargo.lock",
    "/vendor/",
    "/target/",
    "/node_modules/",
    "/package-lock.json",
    ".min.js",
];

/// Fixture and generated-state trees. A planted number in one of these is the
/// point of the tree, not a claim about the repository.
const FIXTURE_TREES: &[&str] = &[
    "/fixtures/",
    "/fixture/",
    "/testdata/",
    "/golden/",
    "/snapshots/",
    "/__snapshots__/",
    "/baselines/",
    "/.pmat/",
    "/.pmat-work/",
    "/dist/",
];

static GENERATED_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(^|/)(generated|autogen)|_generated|generated_|\.gen\.|_pb2?\.")
        .expect("GENERATED_NAME is a compile-time constant pattern")
});

static GENERATED_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)auto[- ]?generated|do not edit|@generated|automatically generated")
        .expect("GENERATED_MARKER is a compile-time constant pattern")
});

/// How many leading lines G1 reads looking for a marker. A DO-NOT-EDIT banner
/// that is not in the header is not a banner.
pub const MARKER_HEADER_LINES: usize = 5;

/// Is this path excluded from the corpus, and why?
///
/// Pure: takes the path as `git ls-files` prints it, repository-relative with
/// no leading slash.
pub fn path_exclusion(path: &str) -> Option<Exclusion> {
    let probe = format!("/{path}");
    if MACHINE_MANAGED.iter().any(|m| probe.contains(m)) {
        return Some(Exclusion::MachineManaged);
    }
    if FIXTURE_TREES.iter().any(|m| probe.contains(m)) {
        return Some(Exclusion::FixtureTree);
    }
    let base = path.rsplit('/').next().unwrap_or(path);
    // Rust test files. Their numerals are FIXTURE DATA, not claims about the
    // repository, and R2's per-line `const` regex cannot tell a declaration
    // from one quoted inside a `let rs = "..."`.
    //
    // The mirror test caught exactly this: the check reported
    // `extract_tests.rs:145` — its own fixture for
    // `rust_contributes_const_and_static_only` — as contradicting
    // `.pmat-metrics.toml`, which would have taken pmat's baseline from 1 to 2
    // on the commit introducing the check. That is `FALSIFY-NC-010`.
    //
    // Truncating at an inline `#[cfg(test)]` (extract.rs) is NOT sufficient on
    // its own: this repository puts the attribute on the `mod` DECLARATION in
    // the parent, so a `*_tests.rs` file contains no `cfg(test)` of its own.
    // 686 files are in that shape. Both rules are needed, and each covers a
    // case the other misses — inline `mod tests` in a production file, and a
    // separate file gated from its parent.
    if base.ends_with("_tests.rs") || base == "tests.rs" {
        return Some(Exclusion::FixtureTree);
    }
    if base.starts_with("CHANGELOG") {
        return Some(Exclusion::Changelog);
    }
    None
}

/// G1, filename half: does the path follow a generated-file convention?
pub fn generated_by_name(path: &str) -> bool {
    GENERATED_NAME.is_match(path)
}

/// G1, marker half: does a DO-NOT-EDIT banner appear in `head`?
pub fn generated_by_marker(head: &str) -> bool {
    GENERATED_MARKER.is_match(head)
}

/// The first [`MARKER_HEADER_LINES`] lines of a file, for G1's marker probe.
pub fn header(text: &str) -> String {
    text.lines()
        .take(MARKER_HEADER_LINES)
        .collect::<Vec<_>>()
        .join("\n")
}

/// G1 in full: name convention first, then the header marker.
pub fn classify_generated(path: &str, text: &str) -> Option<Generated> {
    if generated_by_name(path) {
        return Some(Generated::ByName);
    }
    if generated_by_marker(&header(text)) {
        return Some(Generated::ByMarker);
    }
    None
}

/// Repository-relative paths of tracked files matching `pathspecs`.
///
/// A non-zero exit from `git ls-files` is [`CorpusError::NotAGitWorkTree`], not
/// an empty list. The distinction is the whole vacuity guard: "there is no
/// repository here" and "this repository has no matching files" print the same
/// empty output and must never reach the same verdict.
pub fn tracked_paths(root: &Path, pathspecs: &[&str]) -> Result<Vec<String>, CorpusError> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("ls-files")
        .arg("-z")
        .args(pathspecs)
        .output()
        .map_err(|e| CorpusError::GitUnavailable(e.to_string()))?;
    if !out.status.success() {
        return Err(CorpusError::NotAGitWorkTree(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect())
}

/// Read the corpus: tracked and filtered.
///
/// Generated files are **returned, not dropped** — G1 is applied by the rule
/// that owns it ([`super::cohort`]) so its suppressions land in the census
/// instead of vanishing here. `census.generated_by_*` records how many of the
/// returned files carry the stamp.
pub fn collect(
    root: &Path,
    pathspecs: &[&str],
) -> Result<(Vec<super::CorpusFile>, CorpusCensus), CorpusError> {
    let paths = tracked_paths(root, pathspecs)?;
    let mut census = CorpusCensus {
        tracked: paths.len(),
        ..CorpusCensus::default()
    };
    let mut files = Vec::new();
    for path in paths {
        match path_exclusion(&path) {
            Some(Exclusion::MachineManaged) => {
                census.excluded_machine_managed += 1;
                continue;
            }
            Some(Exclusion::FixtureTree) => {
                census.excluded_fixture_tree += 1;
                continue;
            }
            Some(Exclusion::Changelog) => {
                census.excluded_changelog += 1;
                continue;
            }
            None => {}
        }
        let Ok(bytes) = std::fs::read(root.join(&path)) else {
            census.unreadable += 1;
            continue;
        };
        let text = String::from_utf8_lossy(&bytes).into_owned();
        match classify_generated(&path, &text) {
            Some(Generated::ByName) => census.generated_by_name += 1,
            Some(Generated::ByMarker) => census.generated_by_marker += 1,
            None => {}
        }
        census.scanned += 1;
        files.push(super::CorpusFile { path, text });
    }
    Ok((files, census))
}
