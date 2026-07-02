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

    /// Total migrating parse (MACS-004): strict, then lenient, then lenient
    /// on the first whitespace token (annotated legacy variants like
    /// `"L4 (kani_proof)"`), else `L0`. This is the read-path semantics of
    /// `WorkContract::verification_level`; `pmat work migrate --levels`
    /// rewrites stored values through the same function.
    pub fn parse_migrating(s: &str) -> Self {
        let first_token = s.split_whitespace().next().unwrap_or("");
        Self::parse_strict(s)
            .or_else(|| Self::parse_lenient(s))
            .or_else(|| Self::parse_lenient(first_token))
            .unwrap_or(Self::L0)
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

    /// Evidenced level from a bound YAML, computed **bottom-up**: each rung
    /// requires every lower rung's evidence to be present, so a YAML that
    /// declares `kani_harnesses` but no `falsification_tests` caps at L2 (not
    /// L4). `kani_ran` is the out-of-band kani execution proof (L4 needs a
    /// real run, not just declared harnesses). Unlike `max_attainable_from_yaml`
    /// (a ceiling), this is what a ticket has actually *earned* (MACS F2 —
    /// closes the ladder_evidence gate-bypass found in adversarial review).
    pub fn evidenced_level_from_yaml(yaml: &str, kani_ran: bool) -> Self {
        // L2: bound to an equation.
        if !yaml_section_non_empty(yaml, "equations") {
            return Self::L1;
        }
        // L3: falsification_tests present.
        if !yaml_section_non_empty(yaml, "falsification_tests") {
            return Self::L2;
        }
        // L4: kani harnesses declared AND actually executed.
        if !(yaml_section_non_empty(yaml, "kani_harnesses") && kani_ran) {
            return Self::L3;
        }
        // L5: lean theorem proved.
        if !yaml_lean_theorem_proved(yaml) {
            return Self::L4;
        }
        Self::L5
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
    for line in yaml.lines() {
        match section_line_action(line, &header, in_section) {
            LineAction::Skip => {}
            LineAction::OpenSection => in_section = true,
            LineAction::Decide(non_empty) => return non_empty,
        }
    }
    false
}

/// What one line means to the section scan, given whether a section is open.
enum LineAction {
    /// Blank/comment/unrelated — carry on.
    Skip,
    /// `section:` header with a block body to follow.
    OpenSection,
    /// A terminal decision: the section is (non-)empty.
    Decide(bool),
}

/// Classify a single line for `yaml_section_non_empty` (branch-free loop body).
fn section_line_action(line: &str, header: &str, in_section: bool) -> LineAction {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return LineAction::Skip;
    }
    if line.starts_with(' ') {
        // Indented body line: the section is non-empty iff one is open.
        return if in_section {
            LineAction::Decide(true)
        } else {
            LineAction::Skip
        };
    }
    // Top-level line: an already-open section reached here with no body.
    if in_section {
        return LineAction::Decide(false);
    }
    match yaml_header_kind(trimmed, header) {
        HeaderKind::Other => LineAction::Skip,
        HeaderKind::OpenBlock => LineAction::OpenSection,
        HeaderKind::Inline(non_empty) => LineAction::Decide(non_empty),
    }
}

enum HeaderKind {
    /// A line that is not the section header.
    Other,
    /// `section:` with no inline value — a block body may follow.
    OpenBlock,
    /// `section: <value>` inline; carries whether the value is non-empty.
    Inline(bool),
}

/// Classify a top-level `trimmed` line relative to the target `header`.
fn yaml_header_kind(trimmed: &str, header: &str) -> HeaderKind {
    if !trimmed.starts_with(header) {
        return HeaderKind::Other;
    }
    let rest = trimmed[header.len()..].trim();
    if rest.is_empty() {
        HeaderKind::OpenBlock
    } else if rest.starts_with('[') && rest.ends_with(']') {
        // Flow sequence: non-empty iff anything sits between the brackets.
        HeaderKind::Inline(!rest[1..rest.len() - 1].trim().is_empty())
    } else if rest.starts_with('{') {
        // Flow mapping: non-empty if non-trivial (`{}` is empty).
        HeaderKind::Inline(rest.len() > 2)
    } else {
        // Scalar value — present, hence non-empty.
        HeaderKind::Inline(true)
    }
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
        if in_lean && line_asserts_status_proved(trimmed) {
            return true;
        }
    }
    false
}

/// True iff a `status:` field on this line reads `proved` (quotes optional).
fn line_asserts_status_proved(trimmed: &str) -> bool {
    let Some(idx) = trimmed.find("status:") else {
        return false;
    };
    trimmed[idx + "status:".len()..]
        .trim()
        .trim_matches(|c: char| c == '"' || c == '\'')
        .trim()
        .eq_ignore_ascii_case("proved")
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

    #[test]
    fn ord_matches_numeric() {
        // Ord agrees with the numeric ladder index (fieldless enum cast).
        for pair in VerificationLevel::ALL.windows(2) {
            assert!(pair[0] < pair[1]);
            assert!((pair[0] as u8) < (pair[1] as u8));
        }
        assert_eq!(VerificationLevel::L0 as u8, 0);
        assert_eq!(VerificationLevel::L5 as u8, 5);
    }

    #[test]
    fn serde_string_repr() {
        // Wire format is the display string — no wire break vs legacy files.
        for v in VerificationLevel::ALL {
            let json = serde_json::to_string(&v).expect("serialize");
            assert_eq!(json, format!("\"{}\"", v.as_str()));
            let back: VerificationLevel = serde_json::from_str(&json).expect("parse");
            assert_eq!(back, v);
        }
    }

    #[test]
    fn display_parse_id() {
        for v in VerificationLevel::ALL {
            assert_eq!(v.as_str().parse::<VerificationLevel>().ok(), Some(v));
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]

        /// parse is total (never panics) and strict: accepts exactly "L0".."L5".
        #[test]
        fn parse_total_strict_prop(s in "\\PC{0,8}") {
            let expected = matches!(s.as_str(), "L0" | "L1" | "L2" | "L3" | "L4" | "L5");
            proptest::prop_assert_eq!(VerificationLevel::parse_strict(&s).is_some(), expected);
            proptest::prop_assert_eq!(s.parse::<VerificationLevel>().is_ok(), expected);
        }

        /// Generated corruptions of valid levels (case, padding, suffix words)
        /// must be rejected by the strict parse.
        #[test]
        fn parse_strict_rejects_generated_corruptions(
            base in 0usize..6,
            mode in 0usize..4,
            pad in "[ \\t]{1,3}",
            word in "[a-z]{3,8}",
        ) {
            let valid = VerificationLevel::ALL[base].as_str().to_string();
            let corrupted = match mode {
                0 => valid.to_lowercase(),
                1 => format!("{valid}{pad}"),
                2 => format!("{pad}{valid}"),
                _ => word.clone(),
            };
            if corrupted != valid {
                proptest::prop_assert!(
                    VerificationLevel::parse_strict(&corrupted).is_none(),
                    "corruption '{}' must be rejected", corrupted
                );
            }
        }
    }
}
