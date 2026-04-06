// DBC Lint Rules for pmat work contracts
// Spec: docs/specifications/dbc.md §13.3, §14.5
//
// 13 rules modeled after provable-contracts PV-* rules, adapted for
// pmat work's Popperian falsification + Meyer DBC triad.
// Includes SARIF v2.1.0 output (§13.4).

/// Severity levels for lint findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for LintSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LintSeverity::Error => write!(f, "error"),
            LintSeverity::Warning => write!(f, "warning"),
            LintSeverity::Info => write!(f, "info"),
        }
    }
}

/// A single lint finding against a work contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintFinding {
    /// Rule ID (e.g., "DBC-VAL-001")
    pub rule_id: String,
    /// Severity
    pub severity: LintSeverity,
    /// Human-readable message
    pub message: String,
    /// Optional clause ID this finding relates to
    pub clause_id: Option<String>,
}

/// Result of running all lint rules against a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintReport {
    /// All findings
    pub findings: Vec<LintFinding>,
    /// Whether the contract passes lint (no errors)
    pub passed: bool,
    /// Count of errors
    pub error_count: usize,
    /// Count of warnings
    pub warning_count: usize,
    /// Count of infos
    pub info_count: usize,
}

/// Run all DBC lint rules against a work contract.
///
/// Returns a LintReport with findings from all 13 rules.
/// The contract passes if there are zero error-severity findings.
pub fn lint_contract(
    contract: &WorkContract,
    project_path: &Path,
    min_score: f64,
) -> LintReport {
    debug_assert!((0.0..=1.0).contains(&min_score), "min_score must be 0.0-1.0: {}", min_score);
    let mut findings = Vec::new();

    // Gate 1: Validation rules (DBC-VAL-*)
    lint_val_001(contract, &mut findings);
    lint_val_002(contract, &mut findings);
    lint_val_003(contract, &mut findings);
    lint_val_004(contract, &mut findings);

    // Gate 2: Audit rules (DBC-AUD-*)
    lint_aud_001(contract, &mut findings);
    lint_aud_002(contract, project_path, &mut findings);
    lint_aud_003(contract, &mut findings);

    // Gate 3: Score rules (DBC-SCR-*)
    lint_scr_001(contract, project_path, min_score, &mut findings);
    lint_scr_002(contract, &mut findings);

    // Gate 4: Provability rules (DBC-PRV-*)
    lint_prv_001(contract, &mut findings);

    // Gate 5: Drift/Trend rules (DBC-DRF-*, DBC-TRD-*)
    lint_drf_001(contract, project_path, &mut findings);
    lint_trd_001(contract, project_path, &mut findings);
    lint_trd_002(contract, project_path, &mut findings);

    let error_count = findings
        .iter()
        .filter(|f| f.severity == LintSeverity::Error)
        .count();
    let warning_count = findings
        .iter()
        .filter(|f| f.severity == LintSeverity::Warning)
        .count();
    let info_count = findings
        .iter()
        .filter(|f| f.severity == LintSeverity::Info)
        .count();

    let report = LintReport {
        passed: error_count == 0,
        findings,
        error_count,
        warning_count,
        info_count,
    };
    debug_assert_eq!(report.passed, report.error_count == 0,
        "passed flag inconsistent with error_count");
    report
}

// === Gate 1: Validation Rules ===

/// DBC-VAL-001: Missing preconditions (require empty in v5.0 contract)
fn lint_val_001(contract: &WorkContract, findings: &mut Vec<LintFinding>) {
    if contract.is_dbc() && contract.require.is_empty() {
        findings.push(LintFinding {
            rule_id: "DBC-VAL-001".to_string(),
            severity: LintSeverity::Warning,
            message: "v5.0 contract has no preconditions (require section empty)".to_string(),
            clause_id: None,
        });
    }
}

/// DBC-VAL-002: Missing postconditions (ensure empty in v5.0 contract)
fn lint_val_002(contract: &WorkContract, findings: &mut Vec<LintFinding>) {
    if contract.is_dbc() && contract.ensure.is_empty() {
        findings.push(LintFinding {
            rule_id: "DBC-VAL-002".to_string(),
            severity: LintSeverity::Error,
            message: "v5.0 contract has no postconditions (ensure section empty)".to_string(),
            clause_id: None,
        });
    }
}

/// DBC-VAL-003: Missing invariants (invariant empty in v5.0 contract)
fn lint_val_003(contract: &WorkContract, findings: &mut Vec<LintFinding>) {
    if contract.is_dbc() && contract.invariant.is_empty() {
        findings.push(LintFinding {
            rule_id: "DBC-VAL-003".to_string(),
            severity: LintSeverity::Warning,
            message: "v5.0 contract has no invariants (invariant section empty)".to_string(),
            clause_id: None,
        });
    }
}

/// DBC-VAL-004: Empty claim hypothesis
fn lint_val_004(contract: &WorkContract, findings: &mut Vec<LintFinding>) {
    for (i, claim) in contract.claims.iter().enumerate() {
        if claim.hypothesis.trim().is_empty() {
            findings.push(LintFinding {
                rule_id: "DBC-VAL-004".to_string(),
                severity: LintSeverity::Error,
                message: format!("Claim #{} has empty hypothesis", i),
                clause_id: None,
            });
        }
    }
}

// === Gate 2: Audit Rules ===

/// DBC-AUD-001: Postcondition without falsification test
fn lint_aud_001(contract: &WorkContract, findings: &mut Vec<LintFinding>) {
    if !contract.is_dbc() {
        return;
    }
    for clause in &contract.ensure {
        // Check if there's a matching claim with a result
        let has_verification = contract
            .claims
            .iter()
            .any(|c| c.falsification_method == clause.falsification_method && c.result.is_some());
        if !has_verification {
            findings.push(LintFinding {
                rule_id: "DBC-AUD-001".to_string(),
                severity: LintSeverity::Warning,
                message: format!(
                    "Postcondition '{}' has no falsification test result",
                    clause.id
                ),
                clause_id: Some(clause.id.clone()),
            });
        }
    }
}

/// DBC-AUD-002: Invariant without checkpoint evaluation
fn lint_aud_002(
    contract: &WorkContract,
    project_path: &Path,
    findings: &mut Vec<LintFinding>,
) {
    if !contract.is_dbc() || contract.invariant.is_empty() {
        return;
    }
    let checkpoints = CheckpointRecord::load_all(project_path, &contract.work_item_id);
    if checkpoints.is_empty() {
        findings.push(LintFinding {
            rule_id: "DBC-AUD-002".to_string(),
            severity: LintSeverity::Info,
            message: "Contract has invariants but no checkpoint evaluations yet".to_string(),
            clause_id: None,
        });
    }
}

/// DBC-AUD-003: Claim defined but never verified
fn lint_aud_003(contract: &WorkContract, findings: &mut Vec<LintFinding>) {
    let unverified: Vec<_> = contract
        .claims
        .iter()
        .filter(|c| c.result.is_none() && c.override_info.is_none())
        .collect();

    if !unverified.is_empty() {
        findings.push(LintFinding {
            rule_id: "DBC-AUD-003".to_string(),
            severity: LintSeverity::Info,
            message: format!(
                "{} claim(s) defined but not yet verified",
                unverified.len()
            ),
            clause_id: None,
        });
    }
}

// === Gate 3: Score Rules ===

/// DBC-SCR-001: Contract score below threshold
fn lint_scr_001(
    contract: &WorkContract,
    project_path: &Path,
    min_score: f64,
    findings: &mut Vec<LintFinding>,
) {
    let score = score_contract(contract, project_path);
    if score.total < min_score {
        findings.push(LintFinding {
            rule_id: "DBC-SCR-001".to_string(),
            severity: LintSeverity::Error,
            message: format!(
                "Contract score {:.2} ({}) is below threshold {:.2}",
                score.total, score.grade, min_score
            ),
            clause_id: None,
        });
    }
}

// === Gate 4: Provability Rules ===

/// DBC-PRV-001: Subcontracting violation detected
fn lint_prv_001(contract: &WorkContract, findings: &mut Vec<LintFinding>) {
    if contract.iteration <= 1 || contract.inherited_postconditions.is_empty() {
        return;
    }

    if let Err(violation) =
        validate_subcontracting(&contract.inherited_postconditions, &contract.ensure)
    {
        findings.push(LintFinding {
            rule_id: "DBC-PRV-001".to_string(),
            severity: LintSeverity::Error,
            message: format!("Subcontracting violation: {}", violation),
            clause_id: None,
        });
    }
}

// === Gate 5: Drift Rules ===

/// DBC-DRF-001: Contract drift exceeds bound
fn lint_drf_001(
    contract: &WorkContract,
    project_path: &Path,
    findings: &mut Vec<LintFinding>,
) {
    let drift = compute_drift_metrics(contract, project_path);
    if drift.is_stale {
        findings.push(LintFinding {
            rule_id: "DBC-DRF-001".to_string(),
            severity: LintSeverity::Warning,
            message: format!(
                "Contract stale: {:.1}h since last checkpoint (drift bound: {:.2})",
                drift.hours_since_checkpoint, drift.bounded_drift
            ),
            clause_id: None,
        });
    }
}

/// DBC-SCR-002: More than 30% of claims excluded via --without (§14.5)
fn lint_scr_002(contract: &WorkContract, findings: &mut Vec<LintFinding>) {
    if !contract.is_dbc() || contract.excluded_claims.is_empty() {
        return;
    }
    let total = contract.triad_claim_count() + contract.excluded_claims.len();
    if total == 0 {
        return;
    }
    let exclusion_pct = contract.excluded_claims.len() as f64 / total as f64;
    if exclusion_pct > 0.30 {
        findings.push(LintFinding {
            rule_id: "DBC-SCR-002".to_string(),
            severity: LintSeverity::Warning,
            message: format!(
                "{:.0}% of claims excluded ({}/{}). Consider closing toolchain gaps.",
                exclusion_pct * 100.0,
                contract.excluded_claims.len(),
                total
            ),
            clause_id: None,
        });
    }
}

/// DBC-TRD-001: Contract quality dropped >10% across iterations (§14.5)
fn lint_trd_001(
    contract: &WorkContract,
    project_path: &Path,
    findings: &mut Vec<LintFinding>,
) {
    let trend = load_quality_trend(project_path, &contract.work_item_id);
    if trend.drift_detected {
        findings.push(LintFinding {
            rule_id: "DBC-TRD-001".to_string(),
            severity: LintSeverity::Warning,
            message: format!(
                "Quality trend declining: {:.2} delta from rolling avg {:.2} ({})",
                trend.delta_from_average, trend.rolling_average, trend.direction
            ),
            clause_id: None,
        });
    }
}

/// DBC-TRD-002: Rescue success rate below 50% (§14.5, ABC drift bounds)
fn lint_trd_002(
    contract: &WorkContract,
    project_path: &Path,
    findings: &mut Vec<LintFinding>,
) {
    let rescue_records = RescueRecord::load_all(project_path, &contract.work_item_id);
    if rescue_records.len() < 2 {
        return; // Not enough data
    }
    // FixSuggested is the only "successful" outcome
    let successes = rescue_records
        .iter()
        .filter(|r| matches!(r.outcome, RescueOutcome::FixSuggested))
        .count();
    let rate = successes as f64 / rescue_records.len() as f64;
    if rate < 0.50 {
        findings.push(LintFinding {
            rule_id: "DBC-TRD-002".to_string(),
            severity: LintSeverity::Info,
            message: format!(
                "Rescue success rate {:.0}% ({}/{}) — below 50% threshold",
                rate * 100.0,
                successes,
                rescue_records.len()
            ),
            clause_id: None,
        });
    }
}

// === SARIF Output (DBC spec §13.4) ===

/// Convert a LintReport to SARIF v2.1.0 JSON.
///
/// Produces OASIS SARIF output for CI/CD integration with GitHub Code Scanning,
/// VS Code SARIF Viewer, etc.
pub fn lint_report_to_sarif(report: &LintReport, contract_path: &str) -> serde_json::Value {
    let results: Vec<serde_json::Value> = report
        .findings
        .iter()
        .map(|f| {
            serde_json::json!({
                "ruleId": f.rule_id,
                "level": match f.severity {
                    LintSeverity::Error => "error",
                    LintSeverity::Warning => "warning",
                    LintSeverity::Info => "note",
                },
                "message": { "text": f.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": contract_path },
                    }
                }],
            })
        })
        .collect();

    let rules: Vec<serde_json::Value> = RULE_CATALOG
        .iter()
        .map(|(id, severity, desc)| {
            serde_json::json!({
                "id": id,
                "shortDescription": { "text": desc },
                "defaultConfiguration": {
                    "level": match *severity {
                        "error" => "error",
                        "warning" => "warning",
                        _ => "note",
                    }
                }
            })
        })
        .collect();

    serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "pmat-dbc-lint",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/paiml/paiml-mcp-agent-toolkit",
                    "rules": rules,
                }
            },
            "results": results,
        }]
    })
}

/// Rule catalog for SARIF embedding (id, default_severity, description)
const RULE_CATALOG: &[(&str, &str, &str)] = &[
    ("DBC-VAL-001", "warning", "Missing preconditions (require empty)"),
    ("DBC-VAL-002", "error", "Missing postconditions (ensure empty)"),
    ("DBC-VAL-003", "warning", "Missing invariants (invariant empty)"),
    ("DBC-VAL-004", "error", "Empty claim hypothesis"),
    ("DBC-AUD-001", "warning", "Postcondition without falsification test"),
    ("DBC-AUD-002", "info", "Invariant without checkpoint evaluation"),
    ("DBC-AUD-003", "info", "Claim defined but never verified"),
    ("DBC-SCR-001", "error", "Contract score below threshold"),
    ("DBC-SCR-002", "warning", "More than 30% of claims excluded"),
    ("DBC-PRV-001", "error", "Subcontracting violation detected"),
    ("DBC-DRF-001", "warning", "Contract drift exceeds bound"),
    ("DBC-TRD-001", "warning", "Quality trend declining"),
    ("DBC-TRD-002", "info", "Rescue success rate below 50%"),
];
