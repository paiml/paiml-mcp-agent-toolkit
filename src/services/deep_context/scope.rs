//! The single owner of "is this file inside the analysis scope?" for deep
//! context.
//!
//! # Why this module exists (R18)
//!
//! `DeepContextConfig::include_patterns` was a field that every caller set and
//! nobody read:
//!
//! * `pmat analyze deep-context --format sarif --include-pattern '**/*.py'`
//!   (`cli/handlers/advanced_analysis_handlers.rs::deep_context_sarif`) produced
//!   output byte-identical — bar the duration line — to the unfiltered run, and
//!   still reported `.rs` findings.
//! * The MCP `analyze_deep_context` tool was patched to *refuse* the argument
//!   instead of ignoring it, which left two policies for one rule: refuse on one
//!   surface, silently ignore on the other.
//!
//! `FileScope` is the one implementation of that rule. `analyzer_core` consults
//! it in exactly two places — when walking the file tree (so `file_count`
//! shrinks) and when pruning the parallel analysis results (so the findings
//! shrink) — which keeps the count and the findings describing the same set of
//! files.
//!
//! The include matcher deliberately reproduces
//! `SimpleDeepContext::matches_include_patterns` verbatim, because the CLI's
//! text/json path runs through `SimpleDeepContext` and the SARIF path runs
//! through `DeepContextAnalyzer`: two matchers that disagree would recreate the
//! contradiction this module exists to remove. `simple_deep_context` should be
//! switched to call `FileScope` and delete its private copy.

use std::path::Path;

use super::DeepContextConfig;

/// Membership test for the set of files a deep-context run is about.
///
/// A file is in scope when it is not excluded **and** (there is no include
/// filter **or** it matches the include filter).
#[derive(Debug, Clone, Default)]
pub struct FileScope {
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl FileScope {
    /// Build the scope from a `DeepContextConfig`.
    #[must_use]
    pub fn from_config(config: &DeepContextConfig) -> Self {
        Self::new(&config.include_patterns, &config.exclude_patterns)
    }

    /// Build the scope from raw pattern lists.
    #[must_use]
    pub fn new(include_patterns: &[String], exclude_patterns: &[String]) -> Self {
        Self {
            include_patterns: include_patterns
                .iter()
                .filter(|p| !p.trim().is_empty())
                .cloned()
                .collect(),
            exclude_patterns: exclude_patterns
                .iter()
                .filter(|p| !p.trim().is_empty())
                .cloned()
                .collect(),
        }
    }

    /// Whether the user narrowed the scope with `--include-pattern`.
    ///
    /// An empty include list means "every file", never "no files": an empty
    /// collection must not be read as a verdict.
    #[must_use]
    pub fn has_include_filter(&self) -> bool {
        !self.include_patterns.is_empty()
    }

    /// Whether any pattern at all narrows the scope.
    #[must_use]
    pub fn is_unrestricted(&self) -> bool {
        self.include_patterns.is_empty() && self.exclude_patterns.is_empty()
    }

    /// Whether `path` is excluded.
    ///
    /// This is the rule `DeepContextAnalyzer::should_exclude_path` has always
    /// applied — substring containment against the pattern with its `*`
    /// wrapping trimmed — kept verbatim so wiring the include half in does not
    /// silently change the exclude half.
    #[must_use]
    pub fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.exclude_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern.trim_matches('*')))
    }

    /// Whether `path` satisfies the include filter (vacuously true when there
    /// is no include filter).
    #[must_use]
    pub fn matches_include(&self, path: &Path) -> bool {
        if self.include_patterns.is_empty() {
            return true;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let path_str = path.to_string_lossy();
        self.include_patterns.iter().any(|pattern| {
            // Glob shorthand: "**/*.rs" selects by extension.
            if let Some(ext_from_pattern) = pattern
                .strip_prefix("**/")
                .and_then(|p| p.strip_prefix("*."))
            {
                return ext == ext_from_pattern;
            }
            // Plain pattern: substring of the file name or of the full path.
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.contains(pattern.as_str()))
                || path_str.contains(pattern.as_str())
        })
    }

    /// Whether a **file** is in scope.
    #[must_use]
    pub fn contains_file(&self, path: &Path) -> bool {
        !self.is_excluded(path) && self.matches_include(path)
    }

    /// Whether a **file named by a string** is in scope.
    ///
    /// Analysis results carry their paths as `String` as often as `PathBuf`;
    /// routing both through one predicate is the point of this type.
    #[must_use]
    pub fn contains_file_str(&self, path: &str) -> bool {
        self.contains_file(Path::new(path))
    }

    /// Whether a **directory** may still contain in-scope files.
    ///
    /// Include patterns select files, never directories, so a directory is
    /// pruned only by the exclude rule.
    #[must_use]
    pub fn may_contain_files(&self, path: &Path) -> bool {
        !self.is_excluded(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scope(include: &[&str], exclude: &[&str]) -> FileScope {
        FileScope::new(
            &include.iter().map(|s| (*s).to_string()).collect::<Vec<_>>(),
            &exclude.iter().map(|s| (*s).to_string()).collect::<Vec<_>>(),
        )
    }

    #[test]
    fn no_include_filter_admits_everything() {
        let s = scope(&[], &[]);
        assert!(!s.has_include_filter());
        assert!(s.contains_file(&PathBuf::from("/p/hot.rs")));
        assert!(s.contains_file(&PathBuf::from("/p/hot.py")));
    }

    #[test]
    fn extension_shorthand_selects_only_that_extension() {
        let s = scope(&["**/*.py"], &[]);
        assert!(s.has_include_filter());
        assert!(s.contains_file(&PathBuf::from("/p/hot.py")));
        assert!(!s.contains_file(&PathBuf::from("/p/hot.rs")));
    }

    #[test]
    fn exclude_still_wins_over_include() {
        let s = scope(&["**/*.rs"], &["**/target/**"]);
        assert!(s.contains_file(&PathBuf::from("/p/src/lib.rs")));
        assert!(!s.contains_file(&PathBuf::from("/p/target/debug/build.rs")));
    }

    #[test]
    fn whitespace_only_patterns_do_not_narrow_the_scope() {
        let s = scope(&["   "], &[]);
        assert!(!s.has_include_filter());
        assert!(s.contains_file(&PathBuf::from("/p/hot.rs")));
    }

    #[test]
    fn string_and_path_predicates_agree() {
        let s = scope(&["**/*.py"], &[]);
        assert_eq!(
            s.contains_file(&PathBuf::from("/p/hot.rs")),
            s.contains_file_str("/p/hot.rs")
        );
        assert_eq!(
            s.contains_file(&PathBuf::from("/p/hot.py")),
            s.contains_file_str("/p/hot.py")
        );
    }
}
