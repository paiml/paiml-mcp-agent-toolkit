//! Classification of a comment's TEXT into self-admitted technical debt.
//!
//! ONE rule, in one place: a comment is self-admitted technical debt when its
//! text **opens with a debt marker**. It is not debt because it happens to
//! contain a topic word somewhere.
//!
//! #925: this used to be a `RegexSet` of topic words (`\b(fixme|broken|bug)\b`,
//! `\b(workaround|temp|temporary)\b`, `\b(security|vuln|…)\b`, …) matched
//! ANYWHERE in the line, which reported prose as debt:
//!
//! ```text
//! // Deterministic order: worst score first, ties broken by path.  -> High / Defect
//! // Atomic write: temp file + rename.                             -> Low  / Design
//! // A failing check with no vulnerability count is a bans/…       -> Critical / Security
//! ```
//!
//! 57 of the 62 violations reported on this repository had no marker at all,
//! and both of its two `Critical` findings were prose explaining a completed
//! fix. Every one of those disappears once the marker has to LEAD the comment;
//! the ad-hoc suppression layer that grew next to the old matcher
//! (`contains("bug") && contains("report")`, …) disappears with it.
//!
//! What survives from the old pattern list is the multi-word phrasing people
//! use only when confessing — `code smell`, `temporary workaround`,
//! `performance issue` — kept in [`admission_phrases`] and still matched
//! anywhere in the comment, because debt admitted in prose is still debt. The
//! single topic words are what had to go.

use regex::RegexSet;

use super::types::{AstContext, AstNodeType, DebtCategory, DebtClassifier, DebtPattern, Severity};

/// One canonical debt marker and the debt it admits.
pub(crate) struct MarkerRule {
    /// Canonical (upper-case) spelling of the marker.
    pub(crate) marker: &'static str,
    pub(crate) category: DebtCategory,
    pub(crate) severity: Severity,
}

/// The marker table — the single source of truth for "what is SATD".
///
/// Severity is a pure function of the marker, so `analyze satd`, `quality-gate
/// --checks satd` and `pmat verify`'s satd stage cannot disagree about how bad
/// a given comment is, and a reader can predict the verdict from the comment.
const MARKERS: &[MarkerRule] = &[
    MarkerRule {
        marker: "SECURITY",
        category: DebtCategory::Security,
        severity: Severity::Critical,
    },
    MarkerRule {
        marker: "FIXME",
        category: DebtCategory::Defect,
        severity: Severity::High,
    },
    MarkerRule {
        marker: "BUG",
        category: DebtCategory::Defect,
        severity: Severity::High,
    },
    MarkerRule {
        marker: "BROKEN",
        category: DebtCategory::Defect,
        severity: Severity::High,
    },
    MarkerRule {
        marker: "HACK",
        category: DebtCategory::Design,
        severity: Severity::Medium,
    },
    MarkerRule {
        marker: "KLUDGE",
        category: DebtCategory::Design,
        severity: Severity::Medium,
    },
    MarkerRule {
        marker: "XXX",
        category: DebtCategory::Design,
        severity: Severity::Medium,
    },
    MarkerRule {
        marker: "TECHNICAL DEBT",
        category: DebtCategory::Design,
        severity: Severity::Medium,
    },
    MarkerRule {
        marker: "TECHDEBT",
        category: DebtCategory::Design,
        severity: Severity::Medium,
    },
    MarkerRule {
        marker: "WIP",
        category: DebtCategory::Requirement,
        severity: Severity::Medium,
    },
    MarkerRule {
        marker: "TODO",
        category: DebtCategory::Requirement,
        severity: Severity::Low,
    },
    MarkerRule {
        marker: "REFACTOR",
        category: DebtCategory::Design,
        severity: Severity::Low,
    },
    MarkerRule {
        marker: "OPTIMIZE",
        category: DebtCategory::Performance,
        severity: Severity::Low,
    },
    MarkerRule {
        marker: "WORKAROUND",
        category: DebtCategory::Design,
        severity: Severity::Low,
    },
];

/// Strict mode's markers (issue #651): the five canonical ones, upper case,
/// with a `MARKER: <work item>` shape.
const STRICT_MARKERS: [&str; 5] = ["TODO", "FIXME", "HACK", "XXX", "BUG"];

/// How demanding the marker match is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MarkerMode {
    /// `--strict`: canonical upper-case marker, a colon, and a work item.
    Strict,
    /// Default: any marker in [`MARKERS`], in any case, followed by one of
    /// [`SEPARATORS`] and a work item.
    Standard,
}

/// Characters a comment leader can leave in front of the text (`/// TODO` is
/// handed over as `/ TODO`, a block-comment continuation as `* TODO`).
const RESIDUE: [char; 5] = [' ', '\t', '/', '!', '*'];

/// Punctuation that may follow a marker. `-` is deliberately absent: `BUG-012:`
/// is a tracker id, not an admission.
const SEPARATORS: [char; 4] = [':', '(', '[', '!'];

impl Default for DebtClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl DebtClassifier {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self::with(MarkerMode::Standard, admission_phrases())
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// New strict.
    pub fn new_strict() -> Self {
        Self::with(MarkerMode::Strict, Vec::new())
    }

    /// Extended mode: detects euphemisms like placeholder, stub, "for now"
    /// See issue #149
    ///
    /// Euphemisms are matched ANYWHERE in the comment text on purpose — that is
    /// the point of the mode: debt that deliberately avoids writing a marker.
    /// Extended is therefore a strict superset of the default marker rule, and
    /// opt-in.
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new_extended() -> Self {
        let mut phrases = admission_phrases();
        phrases.extend(euphemism_patterns());
        Self::with(MarkerMode::Standard, phrases)
    }

    fn with(mode: MarkerMode, phrases: Vec<DebtPattern>) -> Self {
        let regex_strings: Vec<&str> = phrases.iter().map(|p| p.regex.as_str()).collect();
        let compiled_phrases =
            RegexSet::new(&regex_strings).expect("Failed to compile SATD phrase patterns");
        Self {
            mode,
            phrases,
            compiled_phrases,
        }
    }

    /// Classify a comment text and return debt information
    ///
    /// The argument is the comment's TEXT (the leader has already been stripped
    /// by `SATDDetector`), though a leftover leader is tolerated so callers that
    /// pass a whole `// TODO: …` line still work.
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::satd_detector::{DebtClassifier, DebtCategory, Severity};
    ///
    /// let classifier = DebtClassifier::new();
    ///
    /// // TODO comments are classified as requirements
    /// let result = classifier.classify_comment("TODO: implement this feature");
    /// assert_eq!(result, Some((DebtCategory::Requirement, Severity::Low)));
    ///
    /// // FIXME comments are defects with higher severity
    /// let result = classifier.classify_comment("FIXME: this crashes sometimes");
    /// assert_eq!(result, Some((DebtCategory::Defect, Severity::High)));
    ///
    /// // Normal comments return None
    /// let result = classifier.classify_comment("This is a regular comment");
    /// assert_eq!(result, None);
    ///
    /// // …including prose that merely mentions a debt topic (#925)
    /// let result = classifier.classify_comment("Deterministic order: ties broken by path.");
    /// assert_eq!(result, None);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn classify_comment(&self, text: &str) -> Option<(DebtCategory, Severity)> {
        if let Some(rule) = self.marker_at_start(text) {
            return Some((rule.category, rule.severity));
        }
        self.classify_phrase(text)
    }

    /// The marker this comment OPENS with, if any.
    pub(crate) fn marker_at_start(&self, text: &str) -> Option<&'static MarkerRule> {
        let text = text.trim_start_matches(|c: char| RESIDUE.contains(&c));
        MARKERS.iter().find(|rule| self.opens_with(text, rule))
    }

    fn opens_with(&self, text: &str, rule: &MarkerRule) -> bool {
        let marker = rule.marker;
        let Some(head) = text.get(..marker.len()) else {
            return false;
        };
        if !head.eq_ignore_ascii_case(marker) {
            return false;
        }
        let rest = &text[marker.len()..];

        match self.mode {
            // `MARKER: <work item>` — canonical spelling only.
            MarkerMode::Strict => {
                STRICT_MARKERS.contains(&marker)
                    && head == marker
                    && rest
                        .strip_prefix(':')
                        .is_some_and(|r| r.starts_with([' ', '\t']) && !r.trim().is_empty())
            }
            // A marker is followed by punctuation and then the admitted work
            // item. Whitespace is deliberately NOT a separator: `TODO the same
            // analysis reports under -p`, `REFACTOR PHASE: CLI integration` and
            // `WIP patterns - work in progress` are sentences that happen to
            // start with the word, and each of them was reported as debt on
            // this repository while that was allowed.
            MarkerMode::Standard => {
                rest.starts_with(|c: char| SEPARATORS.contains(&c))
                    && !work_item_of(rest).is_empty()
            }
        }
    }

    fn classify_phrase(&self, text: &str) -> Option<(DebtCategory, Severity)> {
        if self.mode == MarkerMode::Strict {
            return None;
        }
        self.compiled_phrases
            .matches(text)
            .into_iter()
            .find_map(|idx| self.phrases.get(idx))
            .map(|pattern| (pattern.category, pattern.severity))
    }

    /// Adjust severity based on context
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn adjust_severity(&self, base_severity: Severity, context: &AstContext) -> Severity {
        match context.node_type {
            // Critical paths escalate severity
            AstNodeType::SecurityFunction | AstNodeType::DataValidation => base_severity.escalate(),
            // Test code reduces severity
            AstNodeType::TestFunction | AstNodeType::MockImplementation => base_severity.reduce(),
            // Hot paths (high complexity) escalate performance issues
            AstNodeType::Regular if context.complexity > 20 => base_severity.escalate(),
            _ => base_severity,
        }
    }
}

/// What is left of a comment after its marker and the punctuation that follows
/// it — the admitted work item. Empty means nothing was admitted.
fn work_item_of(rest: &str) -> &str {
    rest.trim_start_matches(|c: char| {
        c.is_whitespace() || SEPARATORS.contains(&c) || matches!(c, ')' | ']')
    })
    .trim_end()
}

/// Debt admitted in prose rather than with a marker, matched ANYWHERE in the
/// comment. Default mode as well as extended: `// this is a temporary
/// workaround we should optimize later` is an admission however it is phrased.
///
/// Only multi-word phrases live here, and that is the whole line between this
/// list and the topic words #925 deleted: `temp`, `broken`, `bug`, `security`
/// and `optimize` appear constantly in prose ABOUT code (`temp file + rename`,
/// `ties broken by path`), while `code smell` and `temporary workaround` are
/// vocabulary people use only when confessing.
///
/// Two old patterns are deliberately NOT here, both measured rather than
/// guessed:
///
/// * `technical debt` — this tool's own domain vocabulary. 12 comments in this
///   repository use it to DESCRIBE debt tooling (`// SATDDefectAnalyzer … and
///   technical debt detection helpers`). The `TECHDEBT:` / `TECHNICAL DEBT:`
///   markers cover the admission.
/// * `test.*(disabled|skipped|failing)` — a wildcard, not a phrase. It matched
///   `// \`#[test]\`-attributed items are skipped the same way, so a test`
///   in this very file on the first run after it was added.
fn admission_phrases() -> Vec<DebtPattern> {
    vec![
        DebtPattern {
            regex: r"(?i)\bcode\s+smell\b".to_string(),
            category: DebtCategory::Design,
            severity: Severity::Medium,
            description: "Code smell".to_string(),
        },
        DebtPattern {
            regex: r"(?i)\btemporary\s+(workaround|hack|fix|solution)\b".to_string(),
            category: DebtCategory::Design,
            severity: Severity::Low,
            description: "Temporary solution".to_string(),
        },
        DebtPattern {
            regex: r"(?i)\bperformance\s+(issue|problem)\b".to_string(),
            category: DebtCategory::Performance,
            severity: Severity::Medium,
            description: "Performance issue".to_string(),
        },
    ]
}

/// EXTENDED PATTERNS: euphemisms that hide technical debt (issue #149).
/// Commonly used by AI coding assistants to bypass SATD detection.
fn euphemism_patterns() -> Vec<DebtPattern> {
    vec![
        DebtPattern {
            regex: r"(?i)\bplaceholder\b".to_string(),
            category: DebtCategory::Requirement,
            severity: Severity::Medium,
            description: "Placeholder - incomplete implementation".to_string(),
        },
        DebtPattern {
            regex: r"(?i)\bstub\b".to_string(),
            category: DebtCategory::Requirement,
            severity: Severity::Medium,
            description: "Stub - missing implementation".to_string(),
        },
        DebtPattern {
            regex: r"(?i)\bsimplified\b".to_string(),
            category: DebtCategory::Design,
            severity: Severity::Low,
            description: "Simplified - corners cut".to_string(),
        },
        DebtPattern {
            regex: r"(?i)\b(for\s+)?demonstrat(e|ion)\b".to_string(),
            category: DebtCategory::Requirement,
            severity: Severity::Low,
            description: "Demo code - not production ready".to_string(),
        },
        DebtPattern {
            regex: r"(?i)\b(mock|dummy|fake)\b".to_string(),
            category: DebtCategory::Test,
            severity: Severity::Low,
            description: "Mock/dummy - fake implementation".to_string(),
        },
        DebtPattern {
            regex: r"(?i)\bhardcoded\b".to_string(),
            category: DebtCategory::Design,
            severity: Severity::Medium,
            description: "Hardcoded - missing configuration".to_string(),
        },
        DebtPattern {
            regex: r"(?i)\bfor\s+now\b".to_string(),
            category: DebtCategory::Design,
            severity: Severity::Medium,
            description: "For now - temporary solution".to_string(),
        },
        DebtPattern {
            regex: r"\bWIP\b".to_string(),
            category: DebtCategory::Requirement,
            severity: Severity::Medium,
            description: "WIP - work in progress".to_string(),
        },
        DebtPattern {
            regex: r"(?i)\b(skip|bypass)\s+(for\s+now|this|validation)\b".to_string(),
            category: DebtCategory::Design,
            severity: Severity::High,
            description: "Skip/bypass - missing validation".to_string(),
        },
    ]
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ── #925: a topic word in prose is not an admission ──

    /// Every one of these was reported as a violation on this repository, at
    /// the severity shown, purely because a topic word appeared somewhere in
    /// the line.
    #[test]
    fn prose_that_merely_mentions_a_debt_topic_is_not_debt() {
        let classifier = DebtClassifier::new();
        for prose in [
            "Deterministic order: worst score first, ties broken by path.",
            "Atomic write: temp file + rename (CB-1334).",
            "vulnerability count. Reporting those as \"0 vulnerabilities\" phrased a",
            "A failing check with no vulnerability count is a bans/licence/source",
            "Calculate Technical Debt Grade (TDG)",
            "Find self-admitted technical debt markers",
            "it was green on a tree whose `src/lib.rs` had just gained a FIXME and a",
            "the workaround is documented in the changelog",
            "this is slow because the index is cold",
        ] {
            assert_eq!(
                classifier.classify_comment(prose),
                None,
                "prose reported as debt: {prose:?}"
            );
        }
    }

    #[test]
    fn a_marker_that_opens_the_comment_is_debt() {
        let classifier = DebtClassifier::new();
        for (text, category, severity) in [
            (
                "TODO: implement error handling",
                DebtCategory::Requirement,
                Severity::Low,
            ),
            (
                "FIXME: this leaks memory",
                DebtCategory::Defect,
                Severity::High,
            ),
            (
                "HACK: works around upstream bug",
                DebtCategory::Design,
                Severity::Medium,
            ),
            ("BUG: off by one", DebtCategory::Defect, Severity::High),
            (
                "XXX: remove before shipping",
                DebtCategory::Design,
                Severity::Medium,
            ),
            (
                "SECURITY: vulnerable to XSS",
                DebtCategory::Security,
                Severity::Critical,
            ),
            (
                "performance issue: O(n^2) complexity",
                DebtCategory::Performance,
                Severity::Medium,
            ),
            (
                "TODO(alice): wire the cache",
                DebtCategory::Requirement,
                Severity::Low,
            ),
            (
                "todo: implement feature",
                DebtCategory::Requirement,
                Severity::Low,
            ),
        ] {
            assert_eq!(
                classifier.classify_comment(text),
                Some((category, severity)),
                "marker comment not classified: {text:?}"
            );
        }
    }

    #[test]
    fn a_bare_marker_admits_no_work_item() {
        let classifier = DebtClassifier::new();
        assert_eq!(classifier.classify_comment("TODO"), None);
        assert_eq!(classifier.classify_comment("TODO:"), None);
        assert_eq!(classifier.classify_comment("FIXME  "), None);
    }

    /// A sentence that merely STARTS with the word is not an admission. Each
    /// of these was reported as debt on this repository while whitespace
    /// counted as a marker separator.
    #[test]
    fn a_marker_needs_punctuation_after_it() {
        let classifier = DebtClassifier::new();
        // A colon (or an owner) makes it an admission…
        assert!(classifier
            .classify_comment("todo: implement feature")
            .is_some());
        assert!(classifier
            .classify_comment("TODO(bob): implement feature")
            .is_some());
        // …a space does not.
        for prose in [
            "todo list for later",
            "TODO the same analysis reports under `-p`.",
            "REFACTOR PHASE: CLI integration",
            "WIP patterns - work in progress",
            "Bug in the parser was fixed",
            "OPTIMIZE this later maybe",
        ] {
            assert_eq!(
                classifier.classify_comment(prose),
                None,
                "prose reported as debt: {prose:?}"
            );
        }
    }

    #[test]
    fn a_tracker_id_is_not_a_marker() {
        let classifier = DebtClassifier::new();
        assert_eq!(
            classifier.classify_comment("BUG-012: single language override"),
            None
        );
        assert_eq!(
            classifier.classify_comment("TODOS are tracked in the issue"),
            None
        );
    }

    #[test]
    fn a_comment_leader_left_in_the_text_is_tolerated() {
        let classifier = DebtClassifier::new();
        assert!(classifier
            .classify_comment("// TODO: implement this")
            .is_some());
        assert!(classifier
            .classify_comment("* FIXME: continuation line")
            .is_some());
    }

    // ── strict mode ──

    #[test]
    fn strict_requires_the_canonical_marker_with_a_work_item() {
        let strict = DebtClassifier::new_strict();
        assert!(strict.classify_comment("TODO: rewrite this loop").is_some());
        assert!(strict
            .classify_comment("// TODO: rewrite this loop")
            .is_some());
        assert_eq!(strict.classify_comment("todo: rewrite this loop"), None);
        assert_eq!(strict.classify_comment("TODO rewrite this loop"), None);
        assert_eq!(strict.classify_comment("this is a todo list"), None);
        // Not one of the five canonical markers.
        assert_eq!(strict.classify_comment("OPTIMIZE: use a bitset"), None);
    }

    #[test]
    fn strict_is_a_subset_of_default() {
        let strict = DebtClassifier::new_strict();
        let default = DebtClassifier::new();
        for text in [
            "TODO: a",
            "FIXME: b",
            "HACK: c",
            "XXX: d",
            "BUG: e",
            "todo: f",
            "OPTIMIZE: g",
            "prose about a bug",
        ] {
            if strict.classify_comment(text).is_some() {
                assert!(
                    default.classify_comment(text).is_some(),
                    "strict matched {text:?} but default did not"
                );
            }
        }
    }

    // ── extended mode ──

    #[test]
    fn extended_matches_markers_and_euphemisms() {
        let extended = DebtClassifier::new_extended();
        assert!(extended.classify_comment("TODO: fix this").is_some());
        for euphemism in [
            "placeholder value",
            "stub implementation",
            "simplified version",
            "for demonstration",
            "mock service",
            "dummy data",
            "fake response",
            "hardcoded path",
            "returns 0 for now",
            "WIP",
            "skip validation",
            "bypass this",
        ] {
            assert!(
                extended.classify_comment(euphemism).is_some(),
                "extended missed euphemism: {euphemism:?}"
            );
        }
    }

    #[test]
    fn extended_is_a_superset_of_default() {
        let default = DebtClassifier::new();
        let extended = DebtClassifier::new_extended();
        for text in ["TODO: a", "SECURITY: b", "placeholder", "ordinary prose"] {
            if default.classify_comment(text).is_some() {
                assert!(
                    extended.classify_comment(text).is_some(),
                    "default matched {text:?} but extended did not"
                );
            }
        }
        assert!(default.classify_comment("placeholder value").is_none());
        assert!(extended.classify_comment("placeholder value").is_some());
    }

    /// Debt confessed in prose is still debt in default mode — and it is the
    /// only thing `--strict` has to narrow. The falsification corpus carries
    /// exactly these two phrasings for that reason: with nothing but canonical
    /// markers in the fixture, `analyze satd --strict` and the default produce
    /// identical output and the flag reads as a no-op.
    #[test]
    fn prose_admissions_are_debt_in_default_but_not_in_strict() {
        let default = DebtClassifier::new();
        let strict = DebtClassifier::new_strict();
        for prose in [
            "this is a temporary workaround we should optimize later",
            "code smell in this module, kept only to avoid a rewrite",
        ] {
            assert!(
                default.classify_comment(prose).is_some(),
                "default missed a prose admission: {prose:?}"
            );
            assert_eq!(
                strict.classify_comment(prose),
                None,
                "strict must only accept canonical markers: {prose:?}"
            );
        }
        // …while the single topic words that produced #925 stay out, and so
        // does this tool's own domain vocabulary.
        for prose in [
            "technical debt lives in the detection helpers below",
            "the workaround is documented in the changelog",
            "temp file + rename",
        ] {
            assert_eq!(
                default.classify_comment(prose),
                None,
                "prose reported as debt: {prose:?}"
            );
        }
    }

    #[test]
    fn extended_does_not_match_normal_code() {
        let extended = DebtClassifier::new_extended();
        assert_eq!(extended.classify_comment("fn process_request()"), None);
        assert_eq!(extended.classify_comment("let result = compute()"), None);
        assert_eq!(extended.classify_comment("return Ok(value)"), None);
    }

    // ── severity is a pure function of the marker ──

    #[test]
    fn every_marker_maps_to_exactly_one_verdict() {
        let classifier = DebtClassifier::new();
        for rule in MARKERS {
            let text = format!("{}: some work item", rule.marker);
            assert_eq!(
                classifier.classify_comment(&text),
                Some((rule.category, rule.severity)),
                "marker {:?} did not classify as its own rule",
                rule.marker
            );
        }
    }

    #[test]
    fn adjust_severity_escalates_and_reduces_by_context() {
        let classifier = DebtClassifier::new();
        let context = |node_type, complexity| AstContext {
            node_type,
            parent_function: "f".to_string(),
            complexity,
            siblings_count: 0,
            nesting_depth: 0,
            surrounding_statements: vec![],
        };
        assert_eq!(
            classifier
                .adjust_severity(Severity::Medium, &context(AstNodeType::SecurityFunction, 1)),
            Severity::High
        );
        assert_eq!(
            classifier.adjust_severity(Severity::High, &context(AstNodeType::TestFunction, 1)),
            Severity::Medium
        );
        assert_eq!(
            classifier.adjust_severity(Severity::Low, &context(AstNodeType::Regular, 25)),
            Severity::Medium
        );
        assert_eq!(
            classifier.adjust_severity(Severity::Medium, &context(AstNodeType::Regular, 5)),
            Severity::Medium
        );
    }
}
