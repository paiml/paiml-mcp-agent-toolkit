// CB-1630: `pmat work codegen` run-status receipt check.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

/// Interpret a codegen run receipt JSON into an outcome. Returns `None` if
/// the schema isn't recognised so the caller can skip cleanly.
fn codegen_receipt_outcome(v: &serde_json::Value) -> Option<ReceiptOutcome> {
    if let Some(b) = v.get("success").and_then(|s| s.as_bool()) {
        return Some(if b {
            ReceiptOutcome::Pass
        } else {
            ReceiptOutcome::Fail("success=false".into())
        });
    }
    if let Some(code) = v.get("exit_code").and_then(|s| s.as_i64()) {
        return Some(if code == 0 {
            ReceiptOutcome::Pass
        } else {
            ReceiptOutcome::Fail(format!("exit_code={}", code))
        });
    }
    if let Some(s) = v.get("status").and_then(|s| s.as_str()) {
        return Some(match s {
            "pass" | "ok" | "success" => ReceiptOutcome::Pass,
            other => ReceiptOutcome::Fail(format!("status=\"{}\"", other)),
        });
    }
    None
}

// ─── CB-163x check implementations (all active, skip-if-absent) ────────────

/// CB-1630 (L2): the most recent `pmat work codegen` run must have
/// succeeded. Component 30 is expected to drop a run-status receipt at
/// `.pmat-work/codegen/last-run.json` containing any of the following
/// shapes (the writer hasn't shipped yet — we accept whichever settles):
///
/// ```json
/// { "success": true }
/// { "exit_code": 0 }
/// { "status": "pass" }
/// ```
///
/// # Skip semantics (tiered)
///
/// * no `.pmat-work/codegen/` directory              → Skip
/// * no `last-run.json` receipt inside it            → Skip
/// * receipt exists but carries none of the three
///   recognised keys                                 → Skip (schema not settled; don't fail on unknown shapes)
/// * receipt indicates success                       → Pass
/// * receipt indicates failure                       → Fail
pub(crate) fn check_codegen_cli_succeeds(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1630: pmat work codegen Succeeds";
    let dir = project_path.join(".pmat-work").join("codegen");
    if !dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/codegen/` directory — codegen has not been run".into(),
            severity: Severity::Info,
        };
    }
    let receipt = dir.join("last-run.json");
    let Ok(contents) = std::fs::read_to_string(&receipt) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message:
                "No `.pmat-work/codegen/last-run.json` — codegen has not emitted a run receipt yet"
                    .into(),
            severity: Severity::Info,
        };
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&contents) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "Malformed JSON in `{}`",
                receipt
                    .strip_prefix(project_path)
                    .unwrap_or(&receipt)
                    .display()
            ),
            severity: Severity::Error,
        };
    };

    match codegen_receipt_outcome(&v) {
        Some(ReceiptOutcome::Pass) => ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "Codegen receipt reports success".into(),
            severity: Severity::Info,
        },
        Some(ReceiptOutcome::Fail(detail)) => ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Codegen receipt reports failure: {}", detail),
            severity: Severity::Error,
        },
        None => ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message:
                "Codegen receipt carries none of `success` / `exit_code` / `status` — schema not settled"
                    .into(),
            severity: Severity::Info,
        },
    }
}
