//! The ONE walk that decides which files a TDG project score covers — and which
//! it does not, by name and with a reason.
//!
//! `quality_gate` over a directory of twelve source files (`a.rs a.py a.go a.ts
//! a.js a.c a.sh a.php a.md a.lua a.cs a.zig`) answered
//! `{"passed":true,"score":90.0,"grade":"A","not_measured":[],"files_analyzed":6}`.
//! Six of the twelve were graded. The other six never reached the analyzer at
//! all: the walk filtered on a hardcoded extension whitelist and dropped
//! everything else without a word, so `not_measured: []` — the field a reader
//! consults to learn what a verdict does NOT cover — asserted full coverage of a
//! run that covered half the tree.
//!
//! Two rules follow, and this module exists to keep them in one place:
//!
//! 1. A source file that TDG cannot grade is a HOLE in the verdict and is
//!    returned in [`Discovery::ungraded`] with the reason it was refused.
//! 2. Files that are not source code at all (documentation, configuration,
//!    data, binaries) were never in a *source* average's population, so their
//!    absence is not a hole and they are not listed. Reporting the 34,000
//!    Markdown files under this repository as "not measured" would drown the
//!    signal that rule 1 exists to raise.
//!
//! The extension whitelist was also narrower than TDG's own language table —
//! `.lua`, `.sql`, `.scala` and `.ruchy` all have analyzers and were skipped
//! anyway — so those are now discovered rather than silently dropped.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::tdg::language_simple::Language;

/// What a walk of a project tree found.
pub(crate) struct Discovery {
    /// Files TDG will attempt to grade.
    pub(crate) gradable: Vec<PathBuf>,
    /// Source files TDG will NOT grade, each with the reason. Never silently
    /// dropped, because a shrinking denominator is invisible in an average.
    pub(crate) ungraded: Vec<(PathBuf, String)>,
}

/// Which population a caller is walking.
///
/// The two analyzers genuinely differ, and the difference is declared here
/// rather than re-implemented as a second walk: the AST analyzer has Markdown
/// and YAML heuristics and grades those files, while
/// `analyzer_simple::grades_source` refuses them. Everything else — the walk,
/// the ignore rules, the skip list, the ungraded-source rule — is shared.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Policy {
    /// Exclude `tests/` directories from the population.
    skip_tests: bool,
    /// Grade Markdown and YAML rather than treating them as non-source.
    grades_markup: bool,
}

impl Policy {
    /// `TdgAnalyzerAst`: skips `tests/`, grades Markdown and YAML.
    pub(crate) const fn ast() -> Self {
        Self {
            skip_tests: true,
            grades_markup: true,
        }
    }

    /// `analyzer_simple::TdgAnalyzer`: keeps `tests/`, refuses non-source.
    pub(crate) const fn heuristic() -> Self {
        Self {
            skip_tests: false,
            grades_markup: false,
        }
    }
}

/// Walk `dir` and partition it.
pub(crate) fn discover(dir: &Path, policy: Policy) -> Result<Discovery> {
    use ignore::WalkBuilder;

    let mut gradable = Vec::new();
    let mut ungraded = Vec::new();

    if !dir.is_dir() {
        return Ok(Discovery { gradable, ungraded });
    }

    for entry in WalkBuilder::new(dir)
        .follow_links(false)
        .hidden(true)
        .parents(true)
        .ignore(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        // Honour a .gitignore even when the tree is not itself a checkout: an
        // ignored directory is ignored because it is not source, and whether
        // `.git` happens to sit above it does not change that.
        .require_git(false)
        .filter_entry(move |entry| {
            if entry.depth() == 0 {
                return true;
            }
            if entry.file_type().is_some_and(|t| t.is_dir()) {
                return !is_skipped_directory(entry.path(), policy.skip_tests);
            }
            true
        })
        .build()
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let path = entry.path();
        match classify(path, policy) {
            Scope::Gradable => gradable.push(path.to_path_buf()),
            Scope::UngradedSource(reason) => ungraded.push((path.to_path_buf(), reason)),
            Scope::OutOfPopulation => {}
        }
    }

    // The walk order is the filesystem's; both lists are serialised verbatim in
    // JSON payloads, so identical input must serialise identically.
    gradable.sort();
    ungraded.sort();

    Ok(Discovery { gradable, ungraded })
}

/// Where a single file stands with respect to a TDG project score.
enum Scope {
    /// TDG has an analyzer for it: it belongs in the average.
    Gradable,
    /// It is source code, and TDG cannot grade it. A hole, and disclosed.
    UngradedSource(String),
    /// Not source code — never part of a source average to begin with.
    OutOfPopulation,
}

/// Does this build have a TDG analyzer for `path`, under `policy`?
pub(crate) fn is_gradable_path(path: &Path, policy: Policy) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| is_gradable_extension(ext, policy))
}

fn classify(path: &Path, policy: Policy) -> Scope {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return Scope::OutOfPopulation;
    };

    if is_gradable_extension(ext, policy) {
        return Scope::Gradable;
    }

    if let Some(language) = ungradable_source_language(ext) {
        return Scope::UngradedSource(format!(
            "{language} source: this build has no TDG analyzer for .{ext}, so the file \
             is not part of the score"
        ));
    }

    Scope::OutOfPopulation
}

/// Extensions TDG has an analyzer for, under `policy`.
///
/// Derived from `Language::from_extension` rather than restated, so a language
/// added there cannot be silently skipped by the walk again — the old
/// whitelists missed `.lua`, `.sql` and `.scala`, all of which have analyzers.
fn is_gradable_extension(ext: &str, policy: Policy) -> bool {
    // `from_extension` keys on the path's extension only.
    let probe = PathBuf::from(format!("probe.{ext}"));
    match Language::from_extension(&probe) {
        Language::Unknown => false,
        Language::Markdown | Language::Yaml => policy.grades_markup,
        _ => true,
    }
}

/// Source code in a language this build cannot grade.
///
/// Deliberately a list of *programming languages*: `.json`, `.toml`, `.lock`,
/// `.png` and friends are not code and are not reported (see the module note).
fn ungradable_source_language(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "sh" | "bash" | "zsh" | "ksh" | "fish" => "Shell",
        "ps1" | "psm1" => "PowerShell",
        "bat" | "cmd" => "Batch",
        "php" | "phtml" => "PHP",
        "cs" | "csx" => "C#",
        "vb" => "Visual Basic",
        "fs" | "fsx" | "fsi" => "F#",
        "zig" => "Zig",
        "dart" => "Dart",
        "ex" | "exs" => "Elixir",
        "erl" | "hrl" => "Erlang",
        "clj" | "cljs" | "cljc" => "Clojure",
        "hs" | "lhs" => "Haskell",
        "ml" | "mli" => "OCaml",
        "elm" => "Elm",
        "purs" => "PureScript",
        "nim" => "Nim",
        "cr" => "Crystal",
        "d" => "D",
        "jl" => "Julia",
        "r" | "rmd" => "R",
        "pl" | "pm" | "t" => "Perl",
        "tcl" => "Tcl",
        "lisp" | "lsp" | "el" | "scm" | "rkt" => "Lisp",
        "groovy" | "gradle" => "Groovy",
        "pas" | "pp" => "Pascal",
        "adb" | "ads" => "Ada",
        "f" | "f90" | "f95" | "for" => "Fortran",
        "cob" | "cbl" => "COBOL",
        "asm" | "s" => "Assembly",
        "sol" => "Solidity",
        "vala" => "Vala",
        "coffee" => "CoffeeScript",
        "vue" | "svelte" => "Web component",
        "m" | "mm" => "Objective-C",
        "awk" => "AWK",
        "sml" => "Standard ML",
        "v" => "V",
        _ => return None,
    })
}

pub(crate) fn is_skipped_directory(path: &Path, skip_tests: bool) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    if skip_tests && name == "tests" {
        return true;
    }
    matches!(
        name,
        "node_modules"
            | "target"
            | "build"
            | "dist"
            | ".git"
            | "__pycache__"
            | ".pytest_cache"
            | "venv"
            | ".venv"
            | "vendor"
            | ".idea"
            | ".vscode"
            | ".lake"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_language_with_an_analyzer_is_gradable() {
        // `.lua`, `.sql` and `.scala` have analyzers and were missing from BOTH
        // hand-maintained whitelists, so their files were dropped in silence.
        for ext in ["rs", "py", "go", "ts", "js", "c", "lua", "sql", "scala"] {
            for policy in [Policy::ast(), Policy::heuristic()] {
                assert!(
                    is_gradable_extension(ext, policy),
                    ".{ext} has a TDG analyzer"
                );
            }
        }
    }

    #[test]
    fn documentation_and_configuration_are_not_the_source_population() {
        for ext in ["json", "toml", "lock", "png", "txt"] {
            for policy in [Policy::ast(), Policy::heuristic()] {
                assert!(
                    matches!(
                        classify(&PathBuf::from(format!("a.{ext}")), policy),
                        Scope::OutOfPopulation
                    ),
                    ".{ext} is not part of a source average's population"
                );
            }
        }
    }

    /// The one place the analyzers legitimately differ, declared instead of
    /// re-implemented: the AST analyzer has Markdown/YAML heuristics, the
    /// heuristic analyzer refuses them via `grades_source`.
    #[test]
    fn markup_follows_the_policy_of_the_analyzer_that_asked() {
        for ext in ["md", "yaml", "yml"] {
            assert!(is_gradable_extension(ext, Policy::ast()));
            assert!(!is_gradable_extension(ext, Policy::heuristic()));
        }
    }

    #[test]
    fn source_this_build_cannot_grade_is_named_not_dropped() {
        for ext in ["sh", "php", "cs", "zig"] {
            for policy in [Policy::ast(), Policy::heuristic()] {
                let scope = classify(&PathBuf::from(format!("a.{ext}")), policy);
                let Scope::UngradedSource(reason) = scope else {
                    panic!(".{ext} is source code and must be reported, not skipped");
                };
                assert!(reason.contains(&format!(".{ext}")), "{reason}");
            }
        }
    }
}
