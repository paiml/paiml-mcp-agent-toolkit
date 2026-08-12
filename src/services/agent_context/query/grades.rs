#![cfg_attr(coverage_nightly, coverage(off))]
//! Letter-grade handling for query results.
//!
//! The index stores `tdg_grade` as a string. Every place that has to *reason*
//! about that string — the `--min-grade` filter, the terminal colouring, the
//! git-history decay heuristic — used to carry its OWN five-letter table
//! (`["A","B","C","D","F"]`). Those tables silently dropped `A+`, `A-`, `B+`,
//! `B-`, `C+`, `C-`: an unmatched grade compared as "not worse", so a filter
//! could pass a function it never actually graded.
//!
//! There is one grade type in this codebase — `crate::tdg::Grade` — and this
//! module is the only bridge from the stored string to it.

use crate::tdg::Grade;

/// Parse a stored/CLI grade string into `crate::tdg::Grade`.
///
/// Reuses `Grade`'s own `Deserialize`, which already accepts both spellings
/// (`"A+"` and `"APlus"`) case-insensitively, rather than adding a second
/// parser that could drift from it.
#[must_use]
pub(crate) fn parse_grade(s: &str) -> Option<Grade> {
    use serde::Deserialize;
    let trimmed = s.trim();
    Grade::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(trimmed))
        .ok()
}

/// Whether `grade` is at least as good as `threshold`, both given as strings.
///
/// Returns `false` when either side does not parse. An ungradeable entry is
/// NOT quietly admitted: `--min-grade B` asks for functions known to be B or
/// better, and "we could not tell" is not that.
#[must_use]
pub(crate) fn grade_meets_threshold(grade: &str, threshold: &str) -> bool {
    match (parse_grade(grade), parse_grade(threshold)) {
        (Some(g), Some(t)) => g.meets_threshold(t),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_grade_accepts_all_eleven_symbolic_forms() {
        for g in Grade::all() {
            let s = g.to_string();
            assert_eq!(parse_grade(&s), Some(g), "round-trip failed for {s}");
        }
    }

    #[test]
    fn test_parse_grade_rejects_garbage() {
        assert_eq!(parse_grade("Z"), None);
        assert_eq!(parse_grade(""), None);
    }

    /// R30 regression: the old five-letter tables had no entry for the
    /// modifier grades, and an unmatched grade was treated as passing.
    #[test]
    fn test_modifier_grades_are_ordered_not_ignored() {
        assert!(!grade_meets_threshold("A-", "A"));
        assert!(!grade_meets_threshold("B+", "A"));
        assert!(grade_meets_threshold("A+", "A"));
        assert!(grade_meets_threshold("A-", "B+"));
        assert!(!grade_meets_threshold("C-", "C"));
    }

    #[test]
    fn test_unparseable_grade_never_passes_a_threshold() {
        assert!(!grade_meets_threshold("", "F"));
        assert!(!grade_meets_threshold("Z", "F"));
    }
}
