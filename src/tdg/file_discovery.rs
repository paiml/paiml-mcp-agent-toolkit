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
//!
//! ## Skip-or-grade is decided here, once
//!
//! "Can TDG grade this path at all?" had two implementations and they gave two
//! answers for the same bytes. `pmat tdg <repo>/tests/bad.rs` answered
//!
//! ```text
//! {"analyzed":false,"skipped":true,
//!  "skip_reason":"test-or-bench file: TDG does not grade test sources",
//!  "score":null,"not_measured":["score","grade"]}
//! ```
//!
//! while the MCP `quality_gate` tool, whose analyzer had no skip rule at all,
//! answered `{"passed":true,"score":90.0,"grade":"A","not_measured":[],
//! "files_analyzed":1}` — the untouched component caps — for the identical file.
//! One surface called it unmeasurable, the other called it an A.
//!
//! [`refusal`] is now the only answer to that question, and every entry point
//! asks it: the walk above, `analyzer_simple::analyze_file`,
//! `TdgAnalyzerAst::analyze_file`, and `crate::tdg::grades_source` (which the
//! MCP gate consults before it grades anything). A path this module refuses
//! comes back refused everywhere — never as a score.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::tdg::language_simple::Language;

/// Said by every surface that declines to grade test or bench source.
///
/// The wording is `pmat tdg`'s, verbatim, because the CLI already published it
/// in `skip_reason` and two surfaces must not describe one rule two ways.
pub(crate) const TEST_SOURCE_SKIP_REASON: &str =
    "test-or-bench file: TDG does not grade test sources";

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
/// Markdown and YAML are the ONLY thing the two analyzers may differ on, and
/// the difference is declared here rather than re-implemented as a second walk:
/// the AST analyzer has Markdown and YAML heuristics and grades those files,
/// while `analyzer_simple::grades_source` refuses them. Everything else — the
/// walk, the ignore rules, the skip list, the test-source rule, the
/// ungraded-source rule — is shared.
///
/// There used to be a second knob, `skip_tests`, set for the AST walk and clear
/// for the heuristic one. That is not a policy, it is a contradiction with a
/// field name: one build answered "TDG does not grade test sources" and
/// "90.0/A" for the same file depending on which analyzer the caller reached.
/// Test source is out of the population for both, decided by
/// [`is_test_or_bench_source`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct Policy {
    /// Grade Markdown and YAML rather than treating them as non-source.
    grades_markup: bool,
}

impl Policy {
    /// `TdgAnalyzerAst`: grades Markdown and YAML.
    pub(crate) const fn ast() -> Self {
        Self {
            grades_markup: true,
        }
    }

    /// `analyzer_simple::TdgAnalyzer`: refuses non-source.
    pub(crate) const fn heuristic() -> Self {
        Self {
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
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            if entry.file_type().is_some_and(|t| t.is_dir()) {
                return !is_skipped_directory(entry.path());
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
            Scope::OutOfPopulation(_) => {}
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
    /// Never part of a source average to begin with: not source code, or
    /// deliberately excluded (test and bench source). Carries the reason
    /// anyway, because a caller that named this ONE file still has to be told
    /// why it got no grade rather than being handed one.
    OutOfPopulation(String),
}

/// Does this build have a TDG analyzer for `path`, under `policy`?
pub(crate) fn is_gradable_path(path: &Path, policy: Policy) -> bool {
    refusal(path, policy).is_none()
}

/// Why TDG will not grade `path`, or `None` if it will. THE skip-or-grade rule.
///
/// Every surface that can decline a single file asks this one function:
/// `pmat tdg` (via the CLI handler), `analyzer_simple::analyze_file`,
/// `TdgAnalyzerAst::analyze_file`, and `crate::tdg::grades_source`, which the
/// MCP `quality_gate` tool consults before it grades anything. The refusal used
/// to exist only in the CLI handler, so the identical file came back
/// `skipped: true, score: null` from `pmat tdg` and `90.0/"A"` from MCP.
///
/// A refusal is a distinct state, not a bad grade: callers must report it as
/// unmeasured and must not let a gate pass on it.
pub(crate) fn refusal(path: &Path, policy: Policy) -> Option<String> {
    match classify(path, policy) {
        Scope::Gradable => None,
        Scope::UngradedSource(reason) | Scope::OutOfPopulation(reason) => Some(reason),
    }
}

fn classify(path: &Path, policy: Policy) -> Scope {
    // First, because it is the only rule that outranks the language: a
    // `tests/bad.rs` is perfectly gradable Rust and is still not graded.
    if is_test_or_bench_source(path) {
        return Scope::OutOfPopulation(TEST_SOURCE_SKIP_REASON.to_string());
    }

    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        // R13: this used to be an unconditional "not in the population", so a
        // shebang script named `deploy`, `configure` or `run` was dropped in
        // silence while a byte-identical `deploy.sh` was disclosed as a hole.
        // The extension is a hint about the language; when it is missing, the
        // file says what it is on its own first line.
        return match shebang_language(path) {
            Some(language) => Scope::UngradedSource(format!(
                "{language} script: this build grades by file extension and this file has \
                 none (its interpreter is named on the shebang line), so it is not part \
                 of the score"
            )),
            None => Scope::OutOfPopulation(NOT_SOURCE_REASON.to_string()),
        };
    };

    if is_gradable_extension(ext, policy) {
        return Scope::Gradable;
    }

    if let Some(language) = ungradable_source_language(path) {
        return Scope::UngradedSource(format!(
            "{language} source: this build has no TDG analyzer for .{ext}, so the file \
             is not part of the score"
        ));
    }

    Scope::OutOfPopulation(format!(
        "TDG grades source files and .{ext} is not one, so score and grade are not measured"
    ))
}

/// Said of a path that is not source code at all.
const NOT_SOURCE_REASON: &str =
    "TDG grades source files and this file is not one, so score and grade are not measured";

/// Is `path` test, bench, example or fuzz source rather than production code?
///
/// THE rule, and the reason it is spelled this way: the identical predicate
/// existed in `cli/handlers/tdg_handlers` (directories `tests`/`benches` only),
/// in `cli/handlers/new_tdg_handler` (directories plus file-name patterns) and
/// in `services::defect_detector_rust` (path substrings), and the MCP surface
/// had no copy at all. The widest of them wins, because the narrow ones are how
/// `src/widget_tests.rs` got graded 100.0/A+ while the byte-identical
/// `src/widget.rs` got 25.164/F: the defect detector excluded the file from
/// detection, and the grader graded it anyway, so the exclusion turned into a
/// perfect score instead of no score.
///
/// Directory components are matched on the absolute path (as `pmat tdg` already
/// did) so `./bad.rs` and `/abs/tests/bad.rs` cannot disagree by cwd.
pub(crate) fn is_test_or_bench_source(path: &Path) -> bool {
    use std::path::Component;

    let resolved = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());

    let in_test_directory = resolved
        .parent()
        .into_iter()
        .flat_map(Path::components)
        .any(|component| {
            matches!(component, Component::Normal(name)
                if name.to_str().is_some_and(is_test_directory_name))
        });

    in_test_directory || has_test_file_name(path)
}

fn is_test_directory_name(name: &str) -> bool {
    matches!(name, "tests" | "benches" | "examples" | "fuzz")
}

/// The file-name half of [`is_test_or_bench_source`], kept identical to
/// `new_tdg_handler::is_test_source` — this repo splits test modules into
/// `hygiene_scorer/tests.rs` and `lang_analyzer_tests_part4.rs`, which no
/// directory rule catches.
fn has_test_file_name(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
    stem == "tests"
        || stem == "test"
        || stem.starts_with("test_")
        || stem.starts_with("tests_")
        || stem.ends_with("_test")
        || stem.ends_with("_tests")
        || stem.contains("_test_")
        || stem.contains("_tests_")
}

/// The language named on `path`'s shebang line, if it has one.
///
/// Only consulted for files with no extension, so the cost is one short read
/// per extensionless file — `LICENSE`, `Makefile` and friends read their first
/// bytes and answer `None`.
fn shebang_language(path: &Path) -> Option<&'static str> {
    use std::io::Read;

    let mut head = [0u8; 128];
    let read = std::fs::File::open(path)
        .and_then(|mut f| f.read(&mut head))
        .ok()?;
    let first_line = String::from_utf8_lossy(&head[..read]);
    let first_line = first_line.lines().next()?;
    let rest = first_line.trim().strip_prefix("#!")?;

    // `#!/usr/bin/env -S python3 -u` and `#!/bin/sh` both name their
    // interpreter; skip `env`, its flags, and the interpreter's own flags.
    rest.split_whitespace()
        .map(|word| word.rsplit('/').next().unwrap_or(word))
        .filter(|word| !word.starts_with('-') && *word != "env")
        .find_map(interpreter_language)
}

fn interpreter_language(interpreter: &str) -> Option<&'static str> {
    // `python3.12` / `ruby2.7`: the version suffix is not part of the name.
    let base = interpreter.trim_end_matches(|c: char| c.is_ascii_digit() || c == '.');
    Some(match base {
        "sh" | "bash" | "dash" | "ash" | "ksh" | "zsh" => "Shell",
        "fish" => "Fish shell",
        "python" => "Python",
        "perl" => "Perl",
        "ruby" => "Ruby",
        "node" | "deno" | "bun" => "JavaScript",
        "lua" => "Lua",
        "awk" | "gawk" | "mawk" => "AWK",
        "php" => "PHP",
        "Rscript" => "R",
        "julia" => "Julia",
        "tclsh" | "wish" => "Tcl",
        "pwsh" | "powershell" => "PowerShell",
        "groovy" => "Groovy",
        "scala" => "Scala",
        "escript" => "Erlang",
        "elixir" => "Elixir",
        _ => return None,
    })
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
/// This was a second hand-maintained extension table — 100-odd extensions in a
/// `match`, sitting next to the gradable one, drifting on its own schedule.
/// pmat already owns exactly one authority on what a file extension *is*:
/// `crate::services::language_registry::Language`, a data table of 55
/// languages. That table decides here, and the classification of a language as
/// code-or-not is an exhaustive `match` on the enum, so adding a language to
/// the registry fails to compile until someone says which side it is on.
///
/// Only *programming languages* are reported: `.json`, `.toml`, `.lock`, `.png`
/// are not code and are not holes in a source average (see the module note).
fn ungradable_source_language(path: &Path) -> Option<&'static str> {
    use crate::services::language_registry::Language as Known;

    match Known::from_path(path) {
        Known::Unknown => registry_gap_language(path),
        known if is_programming_language(known) => Some(known.name()),
        _ => None,
    }
}

/// Is this registry language code, as opposed to data, markup or build config?
///
/// Exhaustive on purpose — no `_` arm. A new `Language` variant must be
/// classified here rather than silently defaulting to "not code", which is how
/// files vanish from a verdict without being disclosed.
fn is_programming_language(language: crate::services::language_registry::Language) -> bool {
    use crate::services::language_registry::Language as L;
    match language {
        L::Rust
        | L::C
        | L::Cpp
        | L::Go
        | L::Zig
        | L::Java
        | L::Kotlin
        | L::Scala
        | L::Groovy
        | L::Clojure
        | L::CSharp
        | L::FSharp
        | L::VisualBasic
        | L::Python
        | L::JavaScript
        | L::TypeScript
        | L::Ruby
        | L::PHP
        | L::Perl
        | L::Lua
        | L::Haskell
        | L::Elixir
        | L::Erlang
        | L::OCaml
        | L::ReasonML
        | L::Elm
        | L::PureScript
        | L::Lean
        | L::Swift
        | L::ObjectiveC
        | L::Dart
        | L::Bash
        | L::Zsh
        | L::Fish
        | L::PowerShell
        | L::SQL
        | L::Solidity
        | L::VHDL
        | L::Verilog
        | L::R
        | L::Julia
        | L::Matlab
        | L::Assembly
        | L::PTX => true,
        // Data, markup, documentation and build description. Absent from a
        // *source* average by definition, not holes in it.
        L::HCL
        | L::YAML
        | L::TOML
        | L::JSON
        | L::XML
        | L::Markdown
        | L::LaTeX
        | L::AsciiDoc
        | L::Makefile
        | L::CMake
        | L::Bazel
        | L::Gradle
        | L::Maven
        | L::Unknown => false,
    }
}

/// Languages the canonical registry does not know yet.
///
/// This is NOT a second authority: it is a list of gaps, and
/// `registry_gap_is_still_a_gap` fails the moment the registry learns one, so
/// the entry has to be deleted rather than left to drift. Deleting these
/// outright would have quietly stopped disclosing Fortran, COBOL, Pascal, Ada,
/// Tcl, Nim, Crystal, D, AWK, Vue and Svelte files — turning a named hole back
/// into a silent one, which is the defect this module exists to prevent.
const REGISTRY_GAP_LANGUAGES: &[(&str, &str)] = &[
    ("ksh", "Shell"),
    ("bat", "Batch"),
    ("cmd", "Batch"),
    ("csx", "C#"),
    ("nim", "Nim"),
    ("cr", "Crystal"),
    ("d", "D"),
    ("rmd", "R"),
    ("tcl", "Tcl"),
    ("lisp", "Lisp"),
    ("lsp", "Lisp"),
    ("el", "Lisp"),
    ("scm", "Lisp"),
    ("rkt", "Lisp"),
    ("pas", "Pascal"),
    ("pp", "Pascal"),
    ("adb", "Ada"),
    ("ads", "Ada"),
    ("f", "Fortran"),
    ("f90", "Fortran"),
    ("f95", "Fortran"),
    ("for", "Fortran"),
    ("cob", "COBOL"),
    ("cbl", "COBOL"),
    ("vala", "Vala"),
    ("coffee", "CoffeeScript"),
    ("vue", "Web component"),
    ("svelte", "Web component"),
    ("awk", "AWK"),
    ("sml", "Standard ML"),
];

fn registry_gap_language(path: &Path) -> Option<&'static str> {
    let ext = path.extension().and_then(|e| e.to_str())?.to_lowercase();
    REGISTRY_GAP_LANGUAGES
        .iter()
        .find(|(gap, _)| *gap == ext)
        .map(|(_, language)| *language)
}

/// Directories the walk does not descend into.
///
/// `tests`, `benches`, `examples` and `fuzz` are here for both analyzers now:
/// the `skip_tests` parameter that used to make this answer depend on WHICH
/// analyzer asked is exactly how one build graded a test file 90.0/A on one
/// surface and refused to grade it on the other.
pub(crate) fn is_skipped_directory(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    if is_test_directory_name(name) {
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
                        Scope::OutOfPopulation(_)
                    ),
                    ".{ext} is not part of a source average's population"
                );
            }
        }
    }

    /// The gap list must shrink, never compete: once the canonical registry
    /// learns an extension, the local entry is a second authority and has to go.
    #[test]
    fn registry_gap_is_still_a_gap() {
        use crate::services::language_registry::Language as Known;
        for (ext, language) in REGISTRY_GAP_LANGUAGES {
            let probe = PathBuf::from(format!("a.{ext}"));
            assert_eq!(
                Known::from_path(&probe),
                Known::Unknown,
                ".{ext} ({language}) is in the canonical registry now — delete it from \
                 REGISTRY_GAP_LANGUAGES so there is one authority again"
            );
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
