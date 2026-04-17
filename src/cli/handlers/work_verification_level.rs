//! Typed verification ladder for `pmat work` tickets.
//!
//! Sub-spec: `docs/specifications/components/pmat-work-verification-ladder.md` (Component 28).
//!
//! Today `WorkContract::verification_level` is a free-form `String`.
//! Component 28's migration plan keeps the string field in place while
//! introducing the typed [`VerificationLevel`] enum used by the
//! CB-1610..1619 checks and `pmat work verify` gating. A follow-up
//! commit will swap the `String` for this type once callers are audited.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Ordered verification ladder. `Ord` matters: `L5 > L4 > … > L0` so the
/// completion gate can compare achieved vs. target and reject regressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum VerificationLevel {
    /// Documentation/review only — no executable check.
    L0,
    /// `debug_assert!` contract macros compile and run during `cargo test`.
    L1,
    /// `#[contract]` attribute bound; trait-based equations instantiated.
    L2,
    /// Bound equation's `falsification_tests[]` execute and pass.
    L3,
    /// Bound equation's `kani_harnesses[]` verified (zero counterexamples).
    L4,
    /// Bound equation's `lean_theorem.status == "proved"`, zero `sorry`.
    L5,
}

impl VerificationLevel {
    /// All six variants in ascending order.
    pub const ALL: [VerificationLevel; 6] = [
        VerificationLevel::L0,
        VerificationLevel::L1,
        VerificationLevel::L2,
        VerificationLevel::L3,
        VerificationLevel::L4,
        VerificationLevel::L5,
    ];

    /// Parse a level string strictly. Rejects whitespace, case variants,
    /// and anything outside `L0..=L5`. Used by CB-1610 to flag typos like
    /// `"L3 "`, `"l4"`, or `"strong"`.
    pub fn parse_strict(s: &str) -> Option<Self> {
        match s {
            "L0" => Some(Self::L0),
            "L1" => Some(Self::L1),
            "L2" => Some(Self::L2),
            "L3" => Some(Self::L3),
            "L4" => Some(Self::L4),
            "L5" => Some(Self::L5),
            _ => None,
        }
    }

    /// Migration-friendly parse: trims, uppercases, accepts `l3`, `L3 `.
    /// Returns `None` for values outside the ladder so downstream code can
    /// record `MIGRATION-LEVEL-UNKNOWN`.
    pub fn parse_lenient(s: &str) -> Option<Self> {
        let canonical = s.trim().to_ascii_uppercase();
        Self::parse_strict(&canonical)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::L0 => "L0",
            Self::L1 => "L1",
            Self::L2 => "L2",
            Self::L3 => "L3",
            Self::L4 => "L4",
            Self::L5 => "L5",
        }
    }

    /// Compute the max attainable level from YAML body text by scanning for
    /// the per-level evidence markers listed in the spec §Max-Attainable
    /// Level Computation.
    ///
    /// The scan is intentionally shape-tolerant: it peeks for the *presence*
    /// of these top-level keys rather than parsing their values. A full YAML
    /// loader is a cross-cutting concern that pv-yaml-loader (Component 29)
    /// will deliver — until then line-scan suffices for binding-scope checks.
    pub fn max_attainable_from_yaml(yaml: &str) -> Self {
        let has_lean_proved = yaml_lean_theorem_proved(yaml);
        let has_kani = yaml_section_non_empty(yaml, "kani_harnesses");
        let has_fals = yaml_section_non_empty(yaml, "falsification_tests");
        let has_eqs = yaml_section_non_empty(yaml, "equations");

        if has_lean_proved {
            Self::L5
        } else if has_kani {
            Self::L4
        } else if has_fals {
            Self::L3
        } else if has_eqs {
            Self::L2
        } else {
            Self::L1
        }
    }
}

impl fmt::Display for VerificationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for VerificationLevel {
    type Err = ParseLevelError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_strict(s).ok_or_else(|| ParseLevelError(s.to_string()))
    }
}

/// Error returned when a string cannot be parsed strictly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLevelError(pub String);

impl fmt::Display for ParseLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid verification level '{}': expected one of L0, L1, L2, L3, L4, L5",
            self.0
        )
    }
}

impl std::error::Error for ParseLevelError {}

// ─── YAML scanners (line-wise, deliberately minimal) ─────────────────────────

/// True iff a top-level key `<section>:` is present AND the block body under
/// it contains at least one two-space-indented key or list element. Blank
/// sections (e.g. `kani_harnesses: []` or `kani_harnesses:` with no body)
/// return false — a declaration without content does not raise the ceiling.
fn yaml_section_non_empty(yaml: &str, section: &str) -> bool {
    let header = format!("{}:", section);
    let mut in_section = false;
    let mut saw_body = false;
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            if in_section && !saw_body {
                return false;
            }
            // Detect "section: []" flow-style empty
            if trimmed.starts_with(&header) {
                let rest = trimmed[header.len()..].trim();
                if rest.is_empty() {
                    in_section = true;
                    saw_body = false;
                } else if rest.starts_with('[') && rest.ends_with(']') {
                    // Flow style: non-empty if there's anything between the brackets
                    let inner = &rest[1..rest.len() - 1];
                    return !inner.trim().is_empty();
                } else if rest.starts_with('{') {
                    // Flow mapping — treat as non-empty if non-trivial
                    return rest.len() > 2;
                } else {
                    return true; // scalar value — not the shape we want but still non-empty
                }
            } else {
                in_section = false;
            }
            continue;
        }
        if in_section {
            // Any indented non-comment line is a body entry
            return true;
        }
    }
    saw_body
}

/// True iff the YAML contains a `lean_theorem:` block with
/// `status: proved`. Minimal scanner — ignores ordering, comments.
fn yaml_lean_theorem_proved(yaml: &str) -> bool {
    let mut in_lean = false;
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            in_lean = trimmed.starts_with("lean_theorem:");
            continue;
        }
        if !in_lean {
            continue;
        }
        // Accept `status: proved`, `status: "proved"`, `status: 'proved'`
        if let Some(idx) = trimmed.find("status:") {
            let rest = trimmed[idx + "status:".len()..]
                .trim()
                .trim_matches(|c: char| c == '"' || c == '\'')
                .trim();
            if rest.eq_ignore_ascii_case("proved") {
                return true;
            }
        }
    }
    false
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_strict() {
        assert!(VerificationLevel::L0 < VerificationLevel::L1);
        assert!(VerificationLevel::L3 < VerificationLevel::L4);
        assert!(VerificationLevel::L5 > VerificationLevel::L3);
    }

    #[test]
    fn parse_strict_accepts_exact() {
        for (s, want) in [
            ("L0", VerificationLevel::L0),
            ("L1", VerificationLevel::L1),
            ("L5", VerificationLevel::L5),
        ] {
            assert_eq!(VerificationLevel::parse_strict(s), Some(want));
        }
    }

    #[test]
    fn parse_strict_rejects_typos() {
        for bad in ["l3", "L3 ", " L3", "strong", "L6", ""] {
            assert!(
                VerificationLevel::parse_strict(bad).is_none(),
                "should reject '{}'",
                bad
            );
        }
    }

    #[test]
    fn parse_lenient_accepts_whitespace_and_case() {
        assert_eq!(
            VerificationLevel::parse_lenient("l3"),
            Some(VerificationLevel::L3)
        );
        assert_eq!(
            VerificationLevel::parse_lenient("  L4  "),
            Some(VerificationLevel::L4)
        );
        assert!(VerificationLevel::parse_lenient("strong").is_none());
    }

    #[test]
    fn display_round_trips() {
        for v in VerificationLevel::ALL {
            assert_eq!(VerificationLevel::parse_strict(v.as_str()), Some(v));
            assert_eq!(format!("{}", v), v.as_str());
        }
    }

    #[test]
    fn from_str_errors_on_bad_input() {
        let err = "strong".parse::<VerificationLevel>().unwrap_err();
        assert!(err.to_string().contains("invalid verification level"));
    }

    #[test]
    fn max_attainable_picks_lean_first() {
        let y = "equations:\n  rope: {}\nkani_harnesses:\n  - name: h\nfalsification_tests:\n  - id: t\nlean_theorem:\n  status: proved\n";
        assert_eq!(
            VerificationLevel::max_attainable_from_yaml(y),
            VerificationLevel::L5
        );
    }

    #[test]
    fn max_attainable_falls_back_to_kani() {
        let y = "equations:\n  rope: {}\nkani_harnesses:\n  - name: h\n";
        assert_eq!(
            VerificationLevel::max_attainable_from_yaml(y),
            VerificationLevel::L4
        );
    }

    #[test]
    fn max_attainable_falls_back_to_falsification() {
        let y = "equations:\n  rope: {}\nfalsification_tests:\n  - id: t\n";
        assert_eq!(
            VerificationLevel::max_attainable_from_yaml(y),
            VerificationLevel::L3
        );
    }

    #[test]
    fn max_attainable_l2_when_only_equations() {
        let y = "equations:\n  rope: {}\n";
        assert_eq!(
            VerificationLevel::max_attainable_from_yaml(y),
            VerificationLevel::L2
        );
    }

    #[test]
    fn max_attainable_l1_on_empty_yaml() {
        let y = "version: 1\ncreated: 2026-01-01\n";
        assert_eq!(
            VerificationLevel::max_attainable_from_yaml(y),
            VerificationLevel::L1
        );
    }

    #[test]
    fn max_attainable_ignores_empty_kani_list() {
        // `kani_harnesses: []` should not raise ceiling to L4
        let y = "equations:\n  rope: {}\nkani_harnesses: []\n";
        assert_eq!(
            VerificationLevel::max_attainable_from_yaml(y),
            VerificationLevel::L2
        );
    }

    #[test]
    fn lean_status_detects_proved_variants() {
        assert!(yaml_lean_theorem_proved(
            "lean_theorem:\n  status: proved\n"
        ));
        assert!(yaml_lean_theorem_proved(
            "lean_theorem:\n  status: \"proved\"\n"
        ));
        assert!(yaml_lean_theorem_proved(
            "lean_theorem:\n  status: PROVED\n"
        ));
        assert!(!yaml_lean_theorem_proved(
            "lean_theorem:\n  status: pending\n"
        ));
    }
}
