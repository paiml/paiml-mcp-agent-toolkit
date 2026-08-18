// =============================================================================
// Library targets: an exported item of a LIBRARY is a reachability ROOT
// =============================================================================
//
// Reachability analysis needs roots. This engine had exactly one rule for
// finding them — "something called it" — so an item nothing in the tree calls
// was dead, full stop. That rule is right for a program and wrong for a library:
// a library's exported API is un-called *by construction*, because its callers
// are not in the tree. Applied to a Python package it reported the whole of
// `__all__` as dead code at 100%:
//
//   mypkg/__init__.py  from .core import public_api, another_export
//                      __all__ = ["public_api", "another_export"]
//   mypkg/core.py      def public_api(x): ...      -> "never called"
//                      def another_export(y): ...  -> "never called"
//
// #1013 removed the same false positive from the Rust path by routing Rust to
// cargo, where rustc knows the crate's targets. This is that decision made
// explicitly, for the engine that has no compiler to ask: decide whether the
// target is a library, and if it is, seed the roots with its exports.
//
// The decision is only ever made from something the tree DECLARES — a manifest,
// an `__all__`, a `static` keyword. Where nothing declares it, the analyzer says
// so ([`LibraryTarget::Undetermined`]) and the verdict is published in the
// report; it does not guess in either direction, because both guesses are
// expensive. Guessing "library" hides real dead code behind a rule the user
// cannot see; guessing "program" is the false positive above.

/// What the analyzer could decide about the analysed target being a LIBRARY,
/// and hence about whether its exported items are entry points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryTarget {
    /// A library, on the evidence named. Its exported items were enumerated and
    /// seeded as reachability roots, so an un-called export is not dead.
    Library { evidence: String },
    /// Not a library, on the evidence named: nothing here is reachable from
    /// outside the tree, so an un-called item is dead.
    NotLibrary { evidence: String },
    /// Could not be decided. Exports were NOT seeded as roots — so an un-called
    /// export IS listed as dead — and `reason` says why the analyzer could not
    /// tell one from the other. Stated rather than assumed: this is the case
    /// where a reader has to apply their own knowledge of the project, and they
    /// can only do that if the report admits the gap.
    Undetermined { reason: String },
}

impl LibraryTarget {
    /// The one-word verdict published in the report.
    pub fn verdict(&self) -> &'static str {
        match self {
            Self::Library { .. } => "library",
            Self::NotLibrary { .. } => "not-a-library",
            Self::Undetermined { .. } => "undetermined",
        }
    }

    /// The evidence behind the verdict, or — when undetermined — what could not
    /// be decided and why.
    pub fn detail(&self) -> &str {
        match self {
            Self::Library { evidence } | Self::NotLibrary { evidence } => evidence,
            Self::Undetermined { reason } => reason,
        }
    }

    /// Are exported items seeded as reachability roots?
    pub fn seeds_exports_as_roots(&self) -> bool {
        matches!(self, Self::Library { .. })
    }
}

/// Seed `called` with every exported definition, so a library's API is not
/// reported dead. Returns how many exported definitions became roots.
///
/// A no-op unless the target was DETERMINED to be a library: an undetermined
/// verdict must not quietly behave like a library one, or the disclosure would
/// describe a decision the analyzer had in fact already made.
fn seed_exported_roots(
    target: &LibraryTarget,
    defined: &[FunctionInfo],
    called: &mut HashSet<String>,
) -> usize {
    if !target.seeds_exports_as_roots() {
        return 0;
    }
    let mut roots = 0;
    for func in defined.iter().filter(|f| f.exported) {
        called.insert(func.name.clone());
        roots += 1;
    }
    roots
}

// ── Rust ────────────────────────────────────────────────────────────────────

/// Is the Rust tree at `path` a library?
///
/// The same rule `CargoDeadCodeAnalyzer` uses to decide `--lib` vs `--bins`, so
/// the two engines cannot disagree about what the crate is; it is re-exported
/// from there rather than reimplemented here.
///
/// The manifest is looked for by walking UP from `path`, because a path inside
/// a crate is a view of that crate and not a crate of its own. Looked for in
/// `path` alone, this answered `NotLibrary` for every subdirectory of every
/// library on earth — a verdict the enclosing `Cargo.toml` flatly contradicts,
/// stated as fact in the published report.
///
/// The walk stops at the nearest `[package]`, not at the nearest `Cargo.toml`.
/// A workspace manifest declares no package, so it has no `[lib]` and no
/// `src/lib.rs` beside it whatever its members are — and reading those absences
/// as evidence produced exactly the same contradicted verdict one level up:
/// "declares no [lib] … which exports nothing to an outside caller", about a
/// file that declares no crate to export anything.
fn detect_rust_library_target(path: &Path) -> LibraryTarget {
    use crate::services::cargo_dead_code_analyzer::CrateRootResolution;

    // A `src/lib.rs` sitting right here is a declaration in its own right, and
    // it is the one this engine can read without a manifest at all.
    if path.join("src/lib.rs").exists() {
        return LibraryTarget::Library {
            evidence: "cargo: this crate has a library target (src/lib.rs), whose `pub` \
                       items are its public API"
                .to_string(),
        };
    }
    let resolution = crate::services::cargo_dead_code_analyzer::resolve_crate_root(path);
    let CrateRootResolution::Package {
        root,
        manifest,
        name,
    } = &resolution
    else {
        return LibraryTarget::Undetermined {
            reason: format!(
                "{}, and no src/lib.rs under it: with no crate declared, whether these \
                 `pub` items are a crate's public API or a binary's internals cannot be \
                 decided from the source alone",
                resolution
                    .no_package_reason(path)
                    .unwrap_or_else(|| "no package encloses this path".to_string())
            ),
        };
    };
    let package = name
        .as_ref()
        .map_or_else(|| "a package".to_string(), |n| format!("package `{n}`"));
    if crate::services::cargo_dead_code_analyzer::project_has_library(root) {
        LibraryTarget::Library {
            evidence: format!(
                "cargo: {} declares {package}, which has a library target (src/lib.rs or \
                 an explicit [lib] section), whose `pub` items are its public API",
                manifest.display()
            ),
        }
    } else {
        LibraryTarget::NotLibrary {
            evidence: format!(
                "cargo: {} declares {package}, which declares no [lib] and has no \
                 src/lib.rs beside it — a binary-only crate, which exports nothing to an \
                 outside caller",
                manifest.display()
            ),
        }
    }
}

// ── Python ──────────────────────────────────────────────────────────────────

/// The names a Python tree DECLARES as its public API.
///
/// Two declarations, both explicit, both machine-readable:
///
/// * `__all__ = [...]` — the module's own statement of what it exports.
/// * `from .core import public_api` inside a package `__init__.py` — a
///   re-export, which is how a package presents a submodule's function as its
///   own.
///
/// Deliberately NOT "every module-level `def` whose name lacks a leading
/// underscore". That is the language's convention for *privacy*, not a
/// declaration of an API, and treating it as one would make almost every
/// function in every package a root — an analyzer that reports nothing is no
/// more useful than one that reports everything.
fn python_declared_exports(files: &[std::path::PathBuf]) -> HashSet<String> {
    let mut exports = HashSet::new();
    for file in files {
        let Ok(content) = std::fs::read_to_string(file) else {
            continue;
        };
        collect_dunder_all_names(&content, &mut exports);
        if file.file_name().and_then(|n| n.to_str()) == Some("__init__.py") {
            collect_init_reexports(&content, &mut exports);
        }
    }
    exports
}

/// Names inside an `__all__` list, across however many lines it spans.
fn collect_dunder_all_names(content: &str, out: &mut HashSet<String>) {
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        if !PY_DUNDER_ALL_REGEX.is_match(line) {
            continue;
        }
        // Accumulate until the list closes, so a multi-line `__all__` is read
        // whole rather than yielding only whatever sat on the first line.
        let mut block = line.to_string();
        while !block.contains(']') && !block.contains(')') {
            match lines.next() {
                Some(next) => {
                    block.push('\n');
                    block.push_str(next);
                }
                None => break,
            }
        }
        for cap in PY_NAME_LITERAL_REGEX.captures_iter(&block) {
            if let Some(name) = cap.get(1) {
                out.insert(name.as_str().to_string());
            }
        }
    }
}

/// Names an `__init__.py` re-exports with `from … import a, b as c`.
///
/// The name kept is the one on the LEFT of an `as`: that is the name the
/// function is defined under, and therefore the name the reachability set is
/// keyed by.
fn collect_init_reexports(content: &str, out: &mut HashSet<String>) {
    for line in content.lines() {
        let Some(cap) = PY_FROM_IMPORT_REGEX.captures(line) else {
            continue;
        };
        let Some(imported) = cap.get(1) else { continue };
        for part in imported.as_str().split(',') {
            let name = part
                .trim()
                .trim_start_matches('(')
                .trim_end_matches(')')
                .trim_end_matches('\\')
                // No `.trim()` here: `split_whitespace` already skips leading
                // whitespace, and clippy's `trim_split_whitespace` denies the
                // pair. The earlier `.trim()` calls are load-bearing — they feed
                // the `trim_*_matches` below, which do not skip whitespace.
                .split_whitespace()
                .next()
                .unwrap_or_default();
            if !name.is_empty() && name != "*" {
                out.insert(name.to_string());
            }
        }
    }
}

/// Is the Python tree at `path` a library whose exports this analyzer can name?
///
/// `declared` is the output of [`python_declared_exports`]; the verdict turns on
/// it because Python has no other place to look. There is no `pub`, no export
/// table and no linker: every module is importable, so "is this a library" and
/// "which of these functions are its API" are the same question, and only the
/// source can answer it.
fn detect_python_library_target(path: &Path, declared: &HashSet<String>) -> LibraryTarget {
    if !declared.is_empty() {
        return LibraryTarget::Library {
            evidence: format!(
                "{} name(s) declared as this package's public API by `__all__` \
                 and/or re-exported by an `__init__.py`",
                declared.len()
            ),
        };
    }

    let packaged = ["pyproject.toml", "setup.py", "setup.cfg"]
        .iter()
        .find(|manifest| path.join(manifest).exists());
    match packaged {
        Some(manifest) => LibraryTarget::Undetermined {
            reason: format!(
                "{manifest} declares a distributable package here, but no `__all__` and \
                 no `from … import` re-export names its public API. Python has no export \
                 keyword, so an exported-but-un-called function cannot be told from a dead \
                 one: the functions below are reported as dead because nothing calls them, \
                 not because they are known to be private"
            ),
        },
        None => LibraryTarget::Undetermined {
            reason: "no packaging manifest (pyproject.toml / setup.py / setup.cfg) and no \
                     `__all__` under this path. Every Python module is importable, so a \
                     loose .py tree may equally be a script or a library; the functions \
                     below are reported as dead because nothing calls them, not because \
                     they are known to be private"
                .to_string(),
        },
    }
}

// ── C and C++ ───────────────────────────────────────────────────────────────

/// Is the C/C++ tree at `path` a library?
///
/// C and C++ have no manifest of their own, so the only declaration available is
/// the build system's. `add_library`/`library()` says the artefact is linked
/// INTO something else — its non-`static` functions are called from outside the
/// tree — and `add_executable` says the opposite. When neither is present
/// nothing in the tree states which it is, and the verdict is undetermined.
///
/// `static` is then the export rule, and it is the language's own: a function
/// declared `static` has internal linkage and cannot be called from another
/// translation unit, so an un-called one is genuinely dead even in a library.
fn detect_c_family_library_target(path: &Path) -> LibraryTarget {
    for (manifest, library_markers, program_marker) in [
        (
            "CMakeLists.txt",
            &["add_library("][..],
            Some("add_executable("),
        ),
        (
            "meson.build",
            &["library(", "shared_library(", "static_library("][..],
            Some("executable("),
        ),
    ] {
        let Ok(content) = std::fs::read_to_string(path.join(manifest)) else {
            continue;
        };
        let stripped = strip_whitespace_before_parens(&content);
        if library_markers.iter().any(|m| stripped.contains(m)) {
            return LibraryTarget::Library {
                evidence: format!(
                    "{manifest} declares a library target; a non-`static` function in one \
                     has external linkage and is part of its API"
                ),
            };
        }
        if program_marker.is_some_and(|m| stripped.contains(m)) {
            return LibraryTarget::NotLibrary {
                evidence: format!(
                    "{manifest} declares an executable target and no library target, so \
                     nothing here is linked into an outside caller"
                ),
            };
        }
    }

    LibraryTarget::Undetermined {
        reason: "C and C++ declare no library target of their own, and no CMakeLists.txt \
                 or meson.build under this path declares one either. A non-`static` \
                 function has external linkage and may be called from a translation unit \
                 outside this tree, so it cannot be told from a dead one: the functions \
                 below are reported as dead because nothing in this tree calls them"
            .to_string(),
    }
}

/// Normalise `add_library (foo)` to `add_library(foo)` so the marker search does
/// not miss the spelling CMake and meson both allow.
fn strip_whitespace_before_parens(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for ch in content.chars() {
        if ch == '(' {
            while out.ends_with(' ') || out.ends_with('\t') {
                out.pop();
            }
        }
        out.push(ch);
    }
    out
}

// ── Lua ─────────────────────────────────────────────────────────────────────

/// Lua's export declaration is the module return: a file ending in `return M`
/// hands `M`'s fields to `require`, which makes them the module's API.
///
/// [`analyze_lua_files`] already reads it — this names the verdict so the report
/// can publish it, and states the gap where there is no such declaration.
fn detect_lua_library_target(modules_returned: usize) -> LibraryTarget {
    if modules_returned > 0 {
        LibraryTarget::Library {
            evidence: format!(
                "{modules_returned} Lua file(s) end in `return <table>`; the functions on \
                 those tables are handed to `require` and are the module's API"
            ),
        }
    } else {
        LibraryTarget::Undetermined {
            reason: "no Lua file under this path ends in `return <table>`, and Lua has no \
                     manifest declaring a library. A bare `function f()` is a GLOBAL and \
                     may be called from any file loaded alongside this tree, so it cannot \
                     be told from a dead one: the functions below are reported as dead \
                     because nothing in this tree calls them"
                .to_string(),
        }
    }
}
