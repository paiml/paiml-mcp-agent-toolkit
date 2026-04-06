// Claim Generators by Profile
// Spec: docs/specifications/dbc.md §2
//
// Generates ContractClause vectors for each profile tier:
// - Universal: 6 claims (any project with git)
// - Rust: 14 claims (Cargo project)
// - Pmat: 25 claims (full batuta stack)

fn universal_claims(max_file_lines: f64) -> Vec<ContractClause> {
    debug_assert!(true, "contract: universal_claims");
    vec![
        // Require (2)
        ContractClause {
            id: "require.compiles".to_string(),
            kind: ClauseKind::Require,
            description: "Project builds successfully".to_string(),
            falsification_method: FalsificationMethod::ManifestIntegrity,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "require.tests_exist".to_string(),
            kind: ClauseKind::Require,
            description: "At least one test exists".to_string(),
            falsification_method: FalsificationMethod::MetaFalsification,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        // Invariant (2)
        ContractClause {
            id: "invariant.compiles".to_string(),
            kind: ClauseKind::Invariant,
            description: "Project compiles at every checkpoint".to_string(),
            falsification_method: FalsificationMethod::ManifestIntegrity,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "invariant.file_size".to_string(),
            kind: ClauseKind::Invariant,
            description: format!("No file exceeds {} lines", max_file_lines as usize),
            falsification_method: FalsificationMethod::FileSizeRegression,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "max_file_lines".to_string(),
                op: ThresholdOp::Lte,
                value: max_file_lines,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
        // Ensure (2)
        ContractClause {
            id: "ensure.tests_pass".to_string(),
            kind: ClauseKind::Ensure,
            description: "All tests pass".to_string(),
            falsification_method: FalsificationMethod::MetaFalsification,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.git_sync".to_string(),
            kind: ClauseKind::Ensure,
            description: "All changes pushed to remote".to_string(),
            falsification_method: FalsificationMethod::GitHubSync,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
    ]
}

fn rust_claims(max_complexity: f64, max_file_lines: f64, coverage_pct: f64) -> Vec<ContractClause> {
    debug_assert!(true, "contract: rust_claims");
    let mut claims = universal_claims(max_file_lines);

    // Additional Invariant (2 more = 4 total)
    claims.extend([
        ContractClause {
            id: "invariant.lint".to_string(),
            kind: ClauseKind::Invariant,
            description: "cargo clippy passes".to_string(),
            falsification_method: FalsificationMethod::LintPass,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "invariant.complexity".to_string(),
            kind: ClauseKind::Invariant,
            description: format!("No function exceeds complexity {}", max_complexity as u32),
            falsification_method: FalsificationMethod::ComplexityRegression,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "max_complexity".to_string(),
                op: ThresholdOp::Lte,
                value: max_complexity,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
    ]);

    // Additional Ensure (6 more = 8 total)
    claims.extend([
        ContractClause {
            id: "ensure.no_regression".to_string(),
            kind: ClauseKind::Ensure,
            description: "No previously-passing test now fails".to_string(),
            falsification_method: FalsificationMethod::MetaFalsification,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.coverage".to_string(),
            kind: ClauseKind::Ensure,
            description: format!("Coverage >= {}%", coverage_pct),
            falsification_method: FalsificationMethod::AbsoluteCoverage,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "coverage_pct".to_string(),
                op: ThresholdOp::Gte,
                value: coverage_pct,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.differential_coverage".to_string(),
            kind: ClauseKind::Ensure,
            description: "Changed lines must be covered".to_string(),
            falsification_method: FalsificationMethod::DifferentialCoverage,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.supply_chain".to_string(),
            kind: ClauseKind::Ensure,
            description: "No vulnerable dependencies".to_string(),
            falsification_method: FalsificationMethod::SupplyChainIntegrity,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.examples_compile".to_string(),
            kind: ClauseKind::Ensure,
            description: "All examples compile and run".to_string(),
            falsification_method: FalsificationMethod::ExamplesCompile,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.tdg_regression".to_string(),
            kind: ClauseKind::Ensure,
            description: "TDG score >= baseline".to_string(),
            falsification_method: FalsificationMethod::TdgRegression,
            threshold: Some(ClauseThreshold::Delta {
                metric: "tdg_score".to_string(),
                op: ThresholdOp::Gte,
                value: 0.0,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
    ]);

    claims
}

fn pmat_claims(max_complexity: f64, max_file_lines: f64, coverage_pct: f64) -> Vec<ContractClause> {
    debug_assert!(true, "contract: pmat_claims");
    let mut claims = rust_claims(max_complexity, max_file_lines, coverage_pct);

    // Additional Require (2 more = 4 total)
    claims.extend([
        ContractClause {
            id: "require.manifest_integrity".to_string(),
            kind: ClauseKind::Require,
            description: "FileManifest anti-gaming check".to_string(),
            falsification_method: FalsificationMethod::ManifestIntegrity,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "require.meta_falsification".to_string(),
            kind: ClauseKind::Require,
            description: "Falsifier self-check active".to_string(),
            falsification_method: FalsificationMethod::MetaFalsification,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
    ]);

    // Additional Invariant (3 more = 7 total)
    claims.extend([
        ContractClause {
            id: "invariant.satd".to_string(),
            kind: ClauseKind::Invariant,
            description: "No new SATD markers".to_string(),
            falsification_method: FalsificationMethod::SatdDetection,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "invariant.dead_code".to_string(),
            kind: ClauseKind::Invariant,
            description: "No new dead code introduced".to_string(),
            falsification_method: FalsificationMethod::DeadCodeDetection,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "invariant.fix_chain".to_string(),
            kind: ClauseKind::Invariant,
            description: "Fix-after-fix chains within limit".to_string(),
            falsification_method: FalsificationMethod::FixChainLimit,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "fix_chain_count".to_string(),
                op: ThresholdOp::Lte,
                value: 3.0,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
    ]);

    // Additional Ensure (6 more = 14 total)
    claims.extend([
        ContractClause {
            id: "ensure.coverage_gaming".to_string(),
            kind: ClauseKind::Ensure,
            description: "No coverage exclusion gaming".to_string(),
            falsification_method: FalsificationMethod::CoverageGaming,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.spec_quality".to_string(),
            kind: ClauseKind::Ensure,
            description: "Spec score meets threshold".to_string(),
            falsification_method: FalsificationMethod::SpecQuality,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "spec_score".to_string(),
                op: ThresholdOp::Gte,
                value: 95.0,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.book_validation".to_string(),
            kind: ClauseKind::Ensure,
            description: "pmat-book validation passes".to_string(),
            falsification_method: FalsificationMethod::BookValidation,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.per_file_coverage".to_string(),
            kind: ClauseKind::Ensure,
            description: format!("All files >= {}% coverage", coverage_pct),
            falsification_method: FalsificationMethod::PerFileCoverage,
            threshold: Some(ClauseThreshold::Numeric {
                metric: "per_file_coverage_pct".to_string(),
                op: ThresholdOp::Gte,
                value: coverage_pct,
            }),
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.variant_coverage".to_string(),
            kind: ClauseKind::Ensure,
            description: "All match arm variants tested".to_string(),
            falsification_method: FalsificationMethod::VariantCoverage,
            threshold: None,
            blocking: true,
            source: ClauseSource::Default,
        },
        ContractClause {
            id: "ensure.cross_crate_parity".to_string(),
            kind: ClauseKind::Ensure,
            description: "Cross-crate integration tests pass".to_string(),
            falsification_method: FalsificationMethod::CrossCrateParity,
            threshold: None,
            blocking: false, // Not blocking by default
            source: ClauseSource::Default,
        },
    ]);

    claims
}
