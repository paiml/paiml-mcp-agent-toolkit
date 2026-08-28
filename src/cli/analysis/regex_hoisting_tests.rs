//! Per-file extractors must not compile regexes per file.
//!
//! Three extractors built their entire pattern set on entry and were called once
//! per file, so a tree of N files paid for N compilations of the same
//! compile-time literals. Measured on this repository (DEBUG build, warm,
//! `--path src/services`, 1,387 files):
//!
//! | command | before | after | factor |
//! |---------|--------|-------|--------|
//! | `analyze symbol-table`  | 32,247 ms | 2,388 ms | 13.5x |
//! | `analyze graph-metrics` |  6,369 ms |   374 ms | 17.0x |
//!
//! Output is byte-identical across both changes; the symbol-table check was run
//! against the pre-hoist binary and differed only inside the file that was
//! edited (its own symbols moved lines), with zero differences elsewhere.
//!
//! `analyze name-similarity` was hoisted the same way for the same reason, but
//! honestly: no win was measured for it on this tree (18 ms before, 21 ms
//! after), because that invocation does not reach the per-file path. The change
//! is correct and output-identical; it is not a measured speedup, and this note
//! exists so nobody later cites a number that was never taken.
//!
//! This guard is a source-level check because the alternative — a timing
//! assertion — is flaky, and the invariant is genuinely syntactic: the patterns
//! are compile-time constants, so building them inside a per-file function is
//! always wrong regardless of how fast the machine is.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    /// `(file, function, why it is per-file)`
    const PER_FILE_EXTRACTORS: &[(&str, &str, &str)] = &[
        (
            "src/cli/analysis/symbol_table_extraction.rs",
            "fn extract_symbols_simple",
            "called from the `for source in &sources` loop in the same file",
        ),
        (
            "src/cli/analysis/name_similarity_scoring.rs",
            "fn extract_names",
            "called once per file from name_similarity_file_collection.rs",
        ),
        (
            "src/cli/analysis/graph_metrics_handler.rs",
            "fn extract_dependencies",
            "called once per file from build_dependency_graph",
        ),
    ];

    /// The body of `func` in `file`, from its signature to the next top-level
    /// item. Crude on purpose: it only has to be tight enough to catch a
    /// `Regex::new` put back inside the function.
    fn body_of(file: &str, func: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(file);
        let read = std::fs::read_to_string(&path);
        assert!(read.is_ok(), "{} must be readable", path.display());
        let text = read.unwrap_or_default();
        let found = text.find(func);
        assert!(
            found.is_some(),
            "{func} must exist in {file} — was it renamed? If so, update \
             PER_FILE_EXTRACTORS rather than deleting the guard."
        );
        let start = found.unwrap_or_default();
        let rest = &text[start..];
        // The next line that begins at column 0 with `}` closes the fn.
        let end = rest.find("\n}\n").map_or(rest.len(), |i| i + 3);
        rest[..end].to_string()
    }

    /// No per-file extractor may compile a regex.
    ///
    /// RED before the hoist: `extract_symbols_simple` held 12 `Regex::new`
    /// calls and ran once per file.
    #[test]
    fn no_per_file_extractor_compiles_a_regex() {
        let mut offenders = Vec::new();
        for (file, func, why) in PER_FILE_EXTRACTORS {
            let body = body_of(file, func);
            let n = body.matches("Regex::new").count();
            if n > 0 {
                offenders.push(format!(
                    "  {file} :: {func} builds {n} regex(es) per call, and is {why}. \
                     Hoist them into a `LazyLock` static."
                ));
            }
        }
        assert!(
            offenders.is_empty(),
            "regex compilation moved back inside a per-file extractor:\n{}",
            offenders.join("\n")
        );
    }

    /// ...and the hoisted statics still exist, so the check above cannot pass
    /// merely because someone deleted the extractor.
    #[test]
    fn the_hoisted_statics_are_still_there() {
        for (file, needle) in [
            (
                "src/cli/analysis/symbol_table_extraction.rs",
                "static SYMBOL_PATTERNS",
            ),
            (
                "src/cli/analysis/name_similarity_scoring.rs",
                "static NAME_PATTERNS",
            ),
            (
                "src/cli/analysis/graph_metrics_handler.rs",
                "static DEP_PATTERNS",
            ),
        ] {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(file);
            let text = std::fs::read_to_string(&path).expect("readable");
            assert!(
                text.contains(needle),
                "{needle} is gone from {file} — if the patterns moved, update this \
                 guard; if they were inlined again, that is the regression."
            );
        }
    }
}
