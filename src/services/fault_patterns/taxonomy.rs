//! Static pattern taxonomy for native bug-hunter (PMAT-613).
//!
//! Ported from batuta/src/bug_hunter/defect_patterns.rs. Pure data — no
//! scanning logic here. All mutable decisions live in scanner.rs.

use super::types::{DefectCategory, FindingSeverity, PatternRule};

/// Runtime fault patterns that always apply (even when PMAT SATD is active).
pub const RUNTIME_PATTERNS: &[PatternRule] = &[
    PatternRule {
        literal: "unwrap()",
        category: DefectCategory::LogicErrors,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.4,
    },
    PatternRule {
        literal: "expect(",
        category: DefectCategory::LogicErrors,
        severity: FindingSeverity::Low,
        suspiciousness: 0.3,
    },
    // SAFETY: the next two are string literals used for pattern matching, not actual unsafe code.
    PatternRule {
        literal: "unsafe {",
        category: DefectCategory::MemorySafety,
        severity: FindingSeverity::High,
        suspiciousness: 0.7,
    },
    PatternRule {
        literal: "transmute",
        category: DefectCategory::MemorySafety,
        severity: FindingSeverity::High,
        suspiciousness: 0.8,
    },
    PatternRule {
        literal: "panic!",
        category: DefectCategory::LogicErrors,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.5,
    },
    PatternRule {
        literal: "unreachable!",
        category: DefectCategory::LogicErrors,
        severity: FindingSeverity::Low,
        suspiciousness: 0.3,
    },
];

/// Tech debt markers. Suppressed when pmat's SATD analyzer is active (avoids duplication).
pub const SATD_PATTERNS: &[PatternRule] = &[
    PatternRule {
        literal: "TODO",
        category: DefectCategory::LogicErrors,
        severity: FindingSeverity::Low,
        suspiciousness: 0.3,
    },
    PatternRule {
        literal: "FIXME",
        category: DefectCategory::LogicErrors,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.5,
    },
    PatternRule {
        literal: "HACK",
        category: DefectCategory::LogicErrors,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.5,
    },
    PatternRule {
        literal: "XXX",
        category: DefectCategory::LogicErrors,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.5,
    },
];

/// Silent-degradation, test-debt, GPU-kernel, and hidden-debt euphemism patterns.
pub const CROSSCUTTING_PATTERNS: &[PatternRule] = &[
    // Silent degradation
    PatternRule {
        literal: ".unwrap_or_else(|_|",
        category: DefectCategory::SilentDegradation,
        severity: FindingSeverity::High,
        suspiciousness: 0.7,
    },
    PatternRule {
        literal: "if let Err(_) =",
        category: DefectCategory::SilentDegradation,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.5,
    },
    PatternRule {
        literal: "Err(_) => {}",
        category: DefectCategory::SilentDegradation,
        severity: FindingSeverity::High,
        suspiciousness: 0.75,
    },
    PatternRule {
        literal: "Ok(_) => {}",
        category: DefectCategory::SilentDegradation,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.4,
    },
    PatternRule {
        literal: "// fallback",
        category: DefectCategory::SilentDegradation,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.5,
    },
    PatternRule {
        literal: "// degraded",
        category: DefectCategory::SilentDegradation,
        severity: FindingSeverity::High,
        suspiciousness: 0.7,
    },
    // Test debt
    PatternRule {
        literal: "#[ignore]",
        category: DefectCategory::TestDebt,
        severity: FindingSeverity::High,
        suspiciousness: 0.7,
    },
    PatternRule {
        literal: "// skip",
        category: DefectCategory::TestDebt,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.5,
    },
    PatternRule {
        literal: "// skipped",
        category: DefectCategory::TestDebt,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.5,
    },
    PatternRule {
        literal: "// broken",
        category: DefectCategory::TestDebt,
        severity: FindingSeverity::High,
        suspiciousness: 0.8,
    },
    PatternRule {
        literal: "// fails",
        category: DefectCategory::TestDebt,
        severity: FindingSeverity::High,
        suspiciousness: 0.75,
    },
    PatternRule {
        literal: "// disabled",
        category: DefectCategory::TestDebt,
        severity: FindingSeverity::Medium,
        suspiciousness: 0.6,
    },
    PatternRule {
        literal: "test removed",
        category: DefectCategory::TestDebt,
        severity: FindingSeverity::Critical,
        suspiciousness: 0.9,
    },
    PatternRule {
        literal: "were removed",
        category: DefectCategory::TestDebt,
        severity: FindingSeverity::Critical,
        suspiciousness: 0.9,
    },
    PatternRule {
        literal: "tests hang",
        category: DefectCategory::TestDebt,
        severity: FindingSeverity::Critical,
        suspiciousness: 0.9,
    },
    // GPU / kernel
    PatternRule {
        literal: "CUDA_ERROR",
        category: DefectCategory::GpuKernelBugs,
        severity: FindingSeverity::Critical,
        suspiciousness: 0.9,
    },
    PatternRule {
        literal: "INVALID_PTX",
        category: DefectCategory::GpuKernelBugs,
        severity: FindingSeverity::Critical,
        suspiciousness: 0.95,
    },
    PatternRule {
        literal: "PTX error",
        category: DefectCategory::GpuKernelBugs,
        severity: FindingSeverity::Critical,
        suspiciousness: 0.9,
    },
    PatternRule {
        literal: "kernel fail",
        category: DefectCategory::GpuKernelBugs,
        severity: FindingSeverity::High,
        suspiciousness: 0.8,
    },
    // Hidden debt euphemisms
    PatternRule {
        literal: "placeholder",
        category: DefectCategory::HiddenDebt,
        severity: FindingSeverity::High,
        suspiciousness: 0.75,
    },
    PatternRule {
        literal: "stub",
        category: DefectCategory::HiddenDebt,
        severity: FindingSeverity::High,
        suspiciousness: 0.7,
    },
    PatternRule {
        literal: "dummy",
        category: DefectCategory::HiddenDebt,
        severity: FindingSeverity::High,
        suspiciousness: 0.7,
    },
    PatternRule {
        literal: "not implemented",
        category: DefectCategory::HiddenDebt,
        severity: FindingSeverity::Critical,
        suspiciousness: 0.9,
    },
    PatternRule {
        literal: "unimplemented",
        category: DefectCategory::HiddenDebt,
        severity: FindingSeverity::Critical,
        suspiciousness: 0.9,
    },
    PatternRule {
        literal: "tech debt",
        category: DefectCategory::HiddenDebt,
        severity: FindingSeverity::High,
        suspiciousness: 0.8,
    },
    PatternRule {
        literal: "technical debt",
        category: DefectCategory::HiddenDebt,
        severity: FindingSeverity::High,
        suspiciousness: 0.8,
    },
];

/// Materialize the full active rule set given whether SATD is handled elsewhere.
pub fn active_rules(pmat_satd_active: bool) -> Vec<PatternRule> {
    let mut rules = Vec::with_capacity(
        RUNTIME_PATTERNS.len() + CROSSCUTTING_PATTERNS.len() + SATD_PATTERNS.len(),
    );
    rules.extend_from_slice(RUNTIME_PATTERNS);
    rules.extend_from_slice(CROSSCUTTING_PATTERNS);
    if !pmat_satd_active {
        rules.extend_from_slice(SATD_PATTERNS);
    }
    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_rules_includes_runtime_and_crosscutting() {
        let rules = active_rules(true);
        assert!(rules.iter().any(|r| r.literal == "unwrap()"));
        assert!(rules.iter().any(|r| r.literal == "CUDA_ERROR"));
    }

    #[test]
    fn active_rules_excludes_satd_when_pmat_active() {
        let rules = active_rules(true);
        assert!(!rules.iter().any(|r| r.literal == "TODO"));
    }

    #[test]
    fn active_rules_includes_satd_when_pmat_inactive() {
        let rules = active_rules(false);
        assert!(rules.iter().any(|r| r.literal == "TODO"));
        assert!(rules.iter().any(|r| r.literal == "FIXME"));
    }

    #[test]
    fn no_duplicate_literals_in_active_set() {
        let rules = active_rules(false);
        let mut literals: Vec<&str> = rules.iter().map(|r| r.literal).collect();
        literals.sort();
        let original_len = literals.len();
        literals.dedup();
        assert_eq!(
            literals.len(),
            original_len,
            "duplicate literals in taxonomy"
        );
    }
}
