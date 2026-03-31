#![cfg_attr(coverage_nightly, coverage(off))]
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

/// List all available check IDs grouped by domain.
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
    CheckExplanation {
        id: "TDG-A+",
        name: "Grade A+ (Excellent)",
        what: "Score 95-100. Minimal complexity, full test coverage, clean documentation.",
        why: "A+ functions have near-zero defect probability and serve as reference implementations.",
        fail_when: &[],
        how_to_fix: "Already excellent. Maintain quality.",
        see_also: &["TDG-A (Grade A)", "pmat-book Ch4: TDG"],
    },
    CheckExplanation {
        id: "TDG-A",
        name: "Grade A (Good)",
        what: "Score 85-94. Low complexity, good test coverage, acceptable documentation.",
        why: "Grade A is the minimum acceptable for production code in projects with min_tdg_grade: A.",
        fail_when: &[],
        how_to_fix: "Reduce cyclomatic complexity, add edge-case tests, improve naming.",
        see_also: &["TDG-A+ (Excellent)", "TDG-B (Needs Improvement)", "CB-200 (TDG Grade Gate)"],
    },
    CheckExplanation {
        id: "TDG-B",
        name: "Grade B (Needs Improvement)",
        what: "Score 70-84. Moderate complexity or coverage gaps.",
        why: "Grade B functions have elevated defect risk. Refactor to reduce complexity.",
        fail_when: &["CB-200 fails if min_tdg_grade is set to A or higher"],
        how_to_fix: "Extract helper functions, reduce nesting depth, add missing test paths.",
        see_also: &["TDG-A (Target)", "pmat-book Ch4: TDG"],
    },
    CheckExplanation {
        id: "TDG-C",
        name: "Grade C (Poor)",
        what: "Score 50-69. High complexity, significant coverage gaps.",
        why: "Grade C functions are primary defect sources. Priority refactoring target.",
        fail_when: &["CB-200 fails for any grade gate setting above F"],
        how_to_fix: "Break into smaller functions, add comprehensive tests, reduce cyclomatic complexity below 20.",
        see_also: &["TDG-B (Next Target)", "pmat five-whys"],
    },
    CheckExplanation {
        id: "TDG-F",
        name: "Grade F (Critical)",
        what: "Score below 50. Extreme complexity, untested, high defect density.",
        why: "Grade F functions cap the entire project score at B. Fix these first.",
        fail_when: &["Any F-grade function causes project-level score cap"],
        how_to_fix: "Rewrite the function. Extract logic into testable units. Add property-based tests.",
        see_also: &["TDG-C (Intermediate Target)", "pmat five-whys"],
    },
];
