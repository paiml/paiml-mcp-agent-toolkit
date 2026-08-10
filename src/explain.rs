//! Centralized check/metric explanation registry.
//!
//! Provides `--explain <ID>` for all scoring commands (comply, score, tdg,
//! infra-score, rust-project-score). Each entry has what/why/fail/fix/see_also.

/// A single check explanation entry.
pub struct CheckExplanation {
    pub id: &'static str,
    pub name: &'static str,
    pub what: &'static str,
    pub why: &'static str,
    pub fail_when: &'static [&'static str],
    pub how_to_fix: &'static str,
    pub see_also: &'static [&'static str],
}

/// Lookup explanations by exact match, prefix match, or fuzzy match.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn lookup(pattern: &str) -> Vec<&'static CheckExplanation> {
    let pattern_upper = pattern.to_uppercase();
    let pattern_lower = pattern.to_lowercase();

    // Exact match
    let exact: Vec<_> = EXPLANATIONS
        .iter()
        .filter(|e| e.id.eq_ignore_ascii_case(&pattern_upper))
        .collect();
    if !exact.is_empty() {
        return exact;
    }

    // Prefix match
    let prefix: Vec<_> = EXPLANATIONS
        .iter()
        .filter(|e| e.id.to_uppercase().starts_with(&pattern_upper))
        .collect();
    if !prefix.is_empty() {
        return prefix;
    }

    // Fuzzy: search in name and what fields
    EXPLANATIONS
        .iter()
        .filter(|e| {
            e.name.to_lowercase().contains(&pattern_lower)
                || e.what.to_lowercase().contains(&pattern_lower)
        })
        .collect()
}

/// Does `pattern` have the shape of an ID the scoring commands print
/// (`CB-030`, `PV-04`, `RT-12`)?
///
/// Used to tell "you typed something that is not an ID" apart from "that IS an
/// ID pmat prints, but nobody has written an explanation for it yet".
#[must_use]
pub fn looks_like_check_id(pattern: &str) -> bool {
    let Some((prefix, number)) = pattern.split_once('-') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|c| c.is_ascii_alphabetic())
        && !number.is_empty()
        && number.chars().all(|c| c.is_ascii_digit())
}

/// What to print when [`lookup`] finds nothing.
///
/// The old message was "No checks matching 'CB-030'. Run `pmat explain` to list
/// all." — which reads as "no such check". `CB-030` is a real check
/// (`CB-030: O(1) Hooks`); the registry below simply does not cover it. Only 11
/// of the ~153 `CB-*` IDs `pmat comply check` emits have an entry, so the
/// common case for a user pasting an ID out of a report was being told their
/// ID was wrong. Say which of the two it is, and point at the check's own
/// description as the fallback source of truth.
#[must_use]
pub fn miss_message(pattern: &str) -> String {
    let registered = EXPLANATIONS.len();
    if looks_like_check_id(pattern) {
        format!(
            "No explanation is registered for '{pattern}'.\n\
             The explanation registry covers {registered} IDs; it is not a catalogue of every \
             check pmat emits, so '{pattern}' may still be a real check.\n\
             \x20 • `pmat explain` lists every ID that has an explanation\n\
             \x20 • `pmat comply check --format json` reports each check's own name and message"
        )
    } else {
        format!(
            "No checks matching '{pattern}' among the {registered} registered explanations. \
             Run `pmat explain` to list all."
        )
    }
}

/// List all available check IDs grouped by domain.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn list_all() -> Vec<(&'static str, Vec<&'static CheckExplanation>)> {
    let domains = [
        ("Compliance (CB-xxx)", "CB-"),
        ("Provable Contracts (PV-xx)", "PV-"),
        ("TDG Grades", "TDG-"),
        ("Contract Score (D1-D5)", "D"),
        ("Codebase Score (CD1-CD5)", "CD"),
        ("Infra-Score", "CI-"),
        ("Rust Project Score", "RT-"),
    ];

    domains
        .iter()
        .filter_map(|(label, prefix)| {
            let matches: Vec<_> = EXPLANATIONS
                .iter()
                .filter(|e| e.id.starts_with(prefix))
                .collect();
            if matches.is_empty() {
                None
            } else {
                Some((*label, matches))
            }
        })
        .collect()
}

/// Format a single explanation for terminal output.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_explanation(e: &CheckExplanation) -> String {
    let mut out = String::new();
    out.push_str(&format!("{}: {}\n", e.id, e.name));
    out.push_str(&"═".repeat(e.id.len() + e.name.len() + 2));
    out.push('\n');
    out.push_str(&format!("\nWhat it checks:\n  {}\n", e.what));
    out.push_str(&format!("\nWhy it matters:\n  {}\n", e.why));

    if !e.fail_when.is_empty() {
        out.push_str("\nFAIL when:\n");
        for cond in e.fail_when {
            out.push_str(&format!("  • {cond}\n"));
        }
    }

    out.push_str(&format!("\nHow to fix:\n  {}\n", e.how_to_fix));

    if !e.see_also.is_empty() {
        out.push_str("\nSee also:\n");
        for sa in e.see_also {
            out.push_str(&format!("  • {sa}\n"));
        }
    }

    out
}

// ── Static Registry ──────────────────────────────────────────────────────

pub static EXPLANATIONS: &[CheckExplanation] = &[
    // ── CB-120..127: OIP Tarantula ────────────────────────────────────
    CheckExplanation {
        id: "CB-120",
        name: "NaN-Unsafe Comparison",
        what: "Detects .partial_cmp().unwrap() which panics on NaN values.",
        why: "Floating-point NaN comparisons silently produce None. Unwrapping panics at runtime on valid IEEE 754 inputs.",
        fail_when: &["Source contains .partial_cmp().unwrap() or .partial_cmp().expect()"],
        how_to_fix: "Use .total_cmp() (Rust 1.62+) or .unwrap_or(Ordering::Equal).",
        see_also: &["CB-121 (Lock Poisoning)", "pmat-book Ch42: ComputeBrick Compliance"],
    },
    CheckExplanation {
        id: "CB-121",
        name: "Lock Poisoning",
        what: "Detects .lock().unwrap() on Mutex/RwLock which panics if another thread panicked.",
        why: "Lock poisoning propagates panics across thread boundaries, causing cascading failures.",
        fail_when: &[".lock().unwrap() or .write().unwrap() on std Mutex/RwLock"],
        how_to_fix: "Use .lock().unwrap_or_else(|e| e.into_inner()) or switch to parking_lot.",
        see_also: &["CB-120 (NaN-Unsafe)", "pmat-book Ch42"],
    },
    CheckExplanation {
        id: "CB-200",
        name: "TDG Grade Gate",
        what: "Enforces minimum TDG grade for all non-test functions.",
        why: "Functions below grade A have high technical debt density, increasing defect probability.",
        fail_when: &["Any non-test function scores below the configured minimum grade (default: A)"],
        how_to_fix: "Reduce complexity, add tests, or configure min_tdg_grade in .pmat.yaml.",
        see_also: &["TDG-A (Grade A definition)", "pmat-book Ch4: TDG"],
    },
    CheckExplanation {
        id: "CB-506",
        name: "String Byte Indexing",
        what: "Detects &str[n..m] byte indexing which panics on non-ASCII input.",
        why: "Byte indexing on UTF-8 strings panics if indices fall within multi-byte characters.",
        fail_when: &["Source contains &str[n..m] or &string[n..m] byte range indexing"],
        how_to_fix: "Use .chars().skip(n).take(m-n) or .get(n..m).unwrap_or_default().",
        see_also: &["CB-500 (Rust Best Practices)", "pmat-book Ch46"],
    },

    // ── CB-1200..1214: Provable Contracts ─────────────────────────────
    CheckExplanation {
        id: "CB-1200",
        name: "Contract Existence",
        what: "Checks that contracts/ directory exists with schema-valid YAML files.",
        why: "Provable contracts provide Design by Contract enforcement for numeric kernels.",
        fail_when: &["No contracts/ directory or no valid YAML files found"],
        how_to_fix: "Create contracts/ directory with YAML files following provable-contracts schema.",
        see_also: &["CB-1201 (PV Lint)", "pmat-book Ch62: Provable Contracts"],
    },
    CheckExplanation {
        id: "CB-1201",
        name: "PV Lint",
        what: "Runs `pv lint` to validate contract YAML schema and quality.",
        why: "Invalid contract YAML silently produces no assertions. Lint catches schema violations early.",
        fail_when: &["pv lint reports errors (when pv_lint_is_error: true in .pmat.yaml)"],
        how_to_fix: "Run `pv lint contracts/` and fix reported issues.",
        see_also: &["CB-1200 (Contract Existence)", "pmat-book Ch62"],
    },
    CheckExplanation {
        id: "CB-1203",
        name: "Contract Annotations",
        what: "Checks that functions matching contract equations have contract macros.",
        why: "Functions without contract_pre_*/contract_post_* invocations have no runtime assertions.",
        fail_when: &["Bound function lacks #[contract], #[requires], contract_pre_*, or // Contract: annotation"],
        how_to_fix: "Add contract_pre_<equation>!(input) at function entry point.",
        see_also: &["CB-1208 (Binding Existence)", "pmat-book Ch62"],
    },
    CheckExplanation {
        id: "CB-1208",
        name: "Binding Existence",
        what: "Verifies binding.yaml entries reference functions that exist in source.",
        why: "Ghost bindings (paper-only) provide zero enforcement. L0 repos claim coverage they don't have.",
        fail_when: &[
            "L0 (paper-only): binding.yaml exists but no build.rs or trait enforcement",
            "Verified function percentage below min_binding_existence threshold",
        ],
        how_to_fix: "Add build.rs with AllImplemented policy and/or tests/contract_traits.rs.",
        see_also: &["CB-1209 (Trait Enforcement)", "pmat-book Ch62"],
    },
    CheckExplanation {
        id: "CB-1210",
        name: "Precondition Quality",
        what: "Scans YAML preconditions for diversity and flags placeholder patterns.",
        why: "Placeholder preconditions like !input.is_empty() provide zero domain-specific protection.",
        fail_when: &[
            "YAML precondition diversity < 30% (>70% identical)",
            ">5% of equations have only placeholder preconditions",
        ],
        how_to_fix: "Replace placeholders with domain expressions: 'x.iter().all(|v| v.is_finite())'.",
        see_also: &["CB-1211 (Codegen Fidelity)", "pmat-book Ch62"],
    },
    CheckExplanation {
        id: "CB-1211",
        name: "Codegen Fidelity",
        what: "Checks that generated debug_assert! assertions are not dominated by placeholders.",
        why: "If codegen regresses to hardcoding !_contract_input.is_empty(), all assertions become trivially true.",
        fail_when: &[
            "Placeholder assertions > 50% of total generated assertions",
            "0 assertions generated from N YAML preconditions (all skipped)",
        ],
        how_to_fix: "Run `pv codegen contracts/ -o src/generated_contracts.rs` with fixed codegen.",
        see_also: &["CB-1210 (Precondition Quality)", "pmat-book Ch62"],
    },
    CheckExplanation {
        id: "CB-1214",
        name: "Enforcement Quality",
        what: "Measures contract call-site penetration and quality via pv coverage --enforcement.",
        why: "Contracts that exist but are never invoked provide zero protection at runtime.",
        fail_when: &[
            "quality < 0.3 AND >30 call sites AND mixed E-levels (regression)",
            "0 call sites found (contracts never invoked)",
        ],
        how_to_fix: "Add contract_pre_*!(input) at call sites. Upgrade E0→E1 by passing real arguments.",
        see_also: &["PV-05 (Infra-Score Enforcement)", "pmat-book Ch62"],
    },

    // ── PV-01..05: Infra-Score Bonus ──────────────────────────────────
    CheckExplanation {
        id: "PV-01",
        name: "PV Lint Passes",
        what: "Awards 3 bonus points if `pv lint` passes on contracts/ directory.",
        why: "Valid contract YAML is the foundation for all provable-contracts enforcement.",
        fail_when: &["pv lint fails or pv CLI not available"],
        how_to_fix: "Install pv CLI: cargo install provable-contracts-cli. Fix lint errors.",
        see_also: &["CB-1201 (PV Lint)", "PV-04 (Contract Existence)"],
    },
    CheckExplanation {
        id: "PV-04",
        name: "Contract Directory Exists",
        what: "Awards 2 bonus points if contracts/ exists with schema-valid YAML.",
        why: "Basic entry point for provable-contracts adoption.",
        fail_when: &["No contracts/ directory or no YAML files with provable-contracts schema markers"],
        how_to_fix: "Create contracts/ with at least one YAML file containing equations: or proof_obligations:.",
        see_also: &["PV-01 (PV Lint)", "CB-1200 (Contract Existence)"],
    },
    CheckExplanation {
        id: "PV-05",
        name: "Enforcement Quality (Infra-Score)",
        what: "Awards 2 bonus points if pv coverage --enforcement finds call sites in source.",
        why: "Contracts without call-site enforcement are documentation, not protection.",
        fail_when: &["0 contract call sites found in source or pv CLI not available"],
        how_to_fix: "Add contract_pre_*/contract_post_* macro invocations at function entry/exit.",
        see_also: &["CB-1214 (Enforcement Quality)", "pmat-book Ch62"],
    },

    // ── TDG Grades ────────────────────────────────────────────────────
    //
    // ONE band per grade `pmat tdg` can print, and the bands are the ones in
    // `crate::tdg::Grade`'s GRADE_BANDS. What used to sit here was a
    // hand-written FIVE-grade table whose numbers contradicted the analyzer:
    // it said A was "Score 85-94" where the analyzer grades 85-89 as A-, and it
    // had no entry at all for A-, B+, B-, C+, C- or D — so `pmat explain TDG-A-`
    // answered "No checks matching 'TDG-A-'" for a grade the tool prints
    // routinely. `tdg_grade_bands_match_the_analyzer` below fails if the two
    // ever drift again.
    CheckExplanation {
        id: "TDG-A+",
        name: "Grade A+ (Excellent)",
        what: "Score [95, 100]. Minimal complexity, full test coverage, clean documentation.",
        why: "A+ functions have near-zero defect probability and serve as reference implementations.",
        fail_when: &[],
        how_to_fix: "Already excellent. Maintain quality.",
        see_also: &["TDG-A (Grade A)", "pmat-book Ch4: TDG"],
    },
    CheckExplanation {
        id: "TDG-A",
        name: "Grade A (Good)",
        what: "Score [90, 95). Low complexity, good test coverage, acceptable documentation.",
        why: "Grade A is the minimum acceptable for production code in projects with min_tdg_grade: A.",
        fail_when: &[],
        how_to_fix: "Reduce cyclomatic complexity, add edge-case tests, improve naming.",
        see_also: &["TDG-A+ (Excellent)", "TDG-A- (Next Band Down)", "CB-200 (TDG Grade Gate)"],
    },
    CheckExplanation {
        id: "TDG-A-",
        name: "Grade A- (Good, Bottom of the A Band)",
        what: "Score [85, 90). Low complexity with a thin margin on coverage or documentation.",
        why: "A- passes an `min_tdg_grade: A-` gate but fails an A gate — the most common near-miss.",
        fail_when: &["CB-200 fails if min_tdg_grade is set to A or A+"],
        how_to_fix: "Close the smallest component gap first (usually documentation or duplication).",
        see_also: &["TDG-A (Next Target)", "CB-200 (TDG Grade Gate)"],
    },
    CheckExplanation {
        id: "TDG-B+",
        name: "Grade B+ (Acceptable)",
        what: "Score [80, 85). Moderate complexity, or one weak component.",
        why: "B+ code carries measurably more defects than A code but is not a priority rewrite.",
        fail_when: &["CB-200 fails if min_tdg_grade is set to A- or higher"],
        how_to_fix: "Extract helper functions and add the missing test paths for the weakest component.",
        see_also: &["TDG-A- (Next Target)", "pmat-book Ch4: TDG"],
    },
    CheckExplanation {
        id: "TDG-B",
        name: "Grade B (Needs Improvement)",
        what: "Score [75, 80). Moderate complexity or coverage gaps.",
        why: "Grade B functions have elevated defect risk. Refactor to reduce complexity.",
        fail_when: &["CB-200 fails if min_tdg_grade is set to B+ or higher"],
        how_to_fix: "Extract helper functions, reduce nesting depth, add missing test paths.",
        see_also: &["TDG-B+ (Next Target)", "pmat-book Ch4: TDG"],
    },
    CheckExplanation {
        id: "TDG-B-",
        name: "Grade B- (Needs Improvement)",
        what: "Score [70, 75). Several components below target.",
        why: "This is also the grade a project is CAPPED at when any file grades F, regardless of average.",
        fail_when: &["CB-200 fails if min_tdg_grade is set to B or higher"],
        how_to_fix: "Fix the F-grade files first, then the weakest component of this one.",
        see_also: &["TDG-F (Project Cap)", "pmat-book Ch4: TDG"],
    },
    CheckExplanation {
        id: "TDG-C+",
        name: "Grade C+ (Poor)",
        what: "Score [65, 70). High complexity, significant coverage gaps.",
        why: "Grade C code is a primary defect source and a priority refactoring target.",
        fail_when: &["CB-200 fails if min_tdg_grade is set to B- or higher"],
        how_to_fix: "Break into smaller functions and add comprehensive tests.",
        see_also: &["TDG-B- (Next Target)", "pmat five-whys"],
    },
    CheckExplanation {
        id: "TDG-C",
        name: "Grade C (Poor)",
        what: "Score [60, 65). High complexity, significant coverage gaps.",
        why: "Grade C functions are primary defect sources. Priority refactoring target.",
        fail_when: &["CB-200 fails for any grade gate setting above C"],
        how_to_fix: "Break into smaller functions, add comprehensive tests, reduce cyclomatic complexity below 20.",
        see_also: &["TDG-C+ (Next Target)", "pmat five-whys"],
    },
    CheckExplanation {
        id: "TDG-C-",
        name: "Grade C- (Poor)",
        what: "Score [55, 60). High complexity with little or no test coverage.",
        why: "At this level the cheapest fix is usually tests, not restructuring — measure first.",
        fail_when: &["CB-200 fails for any grade gate setting above C-"],
        how_to_fix: "Add characterization tests, then split the largest function.",
        see_also: &["TDG-C (Next Target)", "pmat five-whys"],
    },
    CheckExplanation {
        id: "TDG-D",
        name: "Grade D (Very Poor)",
        what: "Score [50, 55). Extreme complexity with almost no coverage.",
        why: "D is one band above the F cap: a small regression here starts capping the project grade.",
        fail_when: &["CB-200 fails for any grade gate setting above D"],
        how_to_fix: "Treat as a rewrite candidate: extract testable units before changing behaviour.",
        see_also: &["TDG-F (Critical)", "pmat five-whys"],
    },
    CheckExplanation {
        id: "TDG-F",
        name: "Grade F (Critical)",
        what: "Score [0, 50). Extreme complexity, untested, high defect density.",
        why: "Grade F files cap the entire PROJECT grade at B — a 99.8/100 project still reports (B) if one file grades F. `pmat tdg` names the cap and the F-grade count when it applies.",
        fail_when: &["Any F-grade file causes the project grade to be capped at B"],
        how_to_fix: "Rewrite the function. Extract logic into testable units. Add property-based tests.",
        see_also: &["TDG-D (Intermediate Target)", "pmat five-whys"],
    },
];

#[cfg(test)]
mod tdg_grade_registry_tests {
    use super::*;
    use crate::tdg::Grade;

    /// The band text an entry must open with, derived from the analyzer's own
    /// table rather than restated here.
    fn band_text(grade: Grade) -> String {
        let (floor, ceiling) = grade.score_band();
        if grade == Grade::APlus {
            format!("Score [{floor}, {ceiling}]")
        } else {
            format!("Score [{floor}, {ceiling})")
        }
    }

    /// Every grade `pmat tdg` can print must be explainable, with the bands the
    /// analyzer actually uses. Before this, `explain` documented five grades
    /// with a stale table (A = "Score 85-94" against the analyzer's 90..95) and
    /// `explain TDG-A-` reported that no such check existed.
    #[test]
    fn tdg_grade_bands_match_the_analyzer() {
        for grade in Grade::all() {
            let id = format!("TDG-{grade}");
            let entries: Vec<_> = EXPLANATIONS.iter().filter(|e| e.id == id).collect();
            assert_eq!(
                entries.len(),
                1,
                "{id}: expected exactly one explain entry, found {}",
                entries.len()
            );
            let expected = band_text(grade);
            assert!(
                entries[0].what.starts_with(&expected),
                "{id} documents {:?} but the analyzer's band is {expected}",
                entries[0].what
            );
        }
    }

    /// `pmat explain TDG-A-` must answer, not report an unknown check.
    #[test]
    fn every_grade_id_is_looked_up_exactly() {
        for grade in Grade::all() {
            let id = format!("TDG-{grade}");
            let found = lookup(&id);
            assert_eq!(
                found.len(),
                1,
                "lookup({id}) returned {} entries",
                found.len()
            );
            assert_eq!(found[0].id, id);
        }
    }

    /// ... and the registry must not document a grade the analyzer cannot emit.
    #[test]
    fn no_tdg_entry_without_a_grade() {
        let known: Vec<String> = Grade::all().iter().map(|g| format!("TDG-{g}")).collect();
        for entry in EXPLANATIONS.iter().filter(|e| e.id.starts_with("TDG-")) {
            assert!(
                known.iter().any(|k| k == entry.id),
                "{} documents a grade no score maps to",
                entry.id
            );
        }
    }
}

#[cfg(test)]
mod miss_message_tests {
    use super::*;

    /// `CB-030` is a real check (`CB-030: O(1) Hooks`) with no registry entry.
    /// Telling the user "No checks matching 'CB-030'" reads as "you typed a
    /// check that does not exist" — the registry's gap, reported as the user's
    /// mistake.
    #[test]
    fn an_unregistered_but_real_check_id_is_not_called_nonexistent() {
        let msg = miss_message("CB-030");
        assert!(
            !msg.contains("No checks matching"),
            "must not claim the ID does not exist, got: {msg}"
        );
        assert!(
            msg.contains("may still be a real check"),
            "must say the registry, not the ID, is incomplete, got: {msg}"
        );
        assert!(
            msg.contains("comply check"),
            "must point at the check's own description, got: {msg}"
        );
    }

    #[test]
    fn a_non_id_pattern_still_reads_as_no_match() {
        let msg = miss_message("zzzz nonsense");
        assert!(msg.contains("No checks matching"), "got: {msg}");
    }

    #[test]
    fn id_shape_detection() {
        for id in ["CB-030", "CB-1210", "PV-04", "RT-12", "cb-030"] {
            assert!(looks_like_check_id(id), "{id} is ID-shaped");
        }
        for other in ["unwrap", "CB030", "CB-", "-030", "CB-12a", "complexity"] {
            assert!(!looks_like_check_id(other), "{other} is not ID-shaped");
        }
    }
}
