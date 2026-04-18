// CB-1636: Generated macros compile in both debug and release profiles.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

/// Extract the outcome for a single profile from the compile-status JSON.
/// Accepts: nested object `{"success": bool}`, numeric exit code, or flat
/// `<profile>_success: bool` key.
fn compile_profile_outcome(v: &serde_json::Value, profile: &str) -> Option<ReceiptOutcome> {
    if let Some(obj) = v.get(profile) {
        if let Some(b) = obj.get("success").and_then(|x| x.as_bool()) {
            return Some(if b {
                ReceiptOutcome::Pass
            } else {
                ReceiptOutcome::Fail("success=false".into())
            });
        }
        if let Some(s) = obj.get("status").and_then(|x| x.as_str()) {
            return Some(match s {
                "pass" | "ok" | "success" => ReceiptOutcome::Pass,
                other => ReceiptOutcome::Fail(format!("status=\"{}\"", other)),
            });
        }
        if let Some(code) = obj.as_i64() {
            return Some(if code == 0 {
                ReceiptOutcome::Pass
            } else {
                ReceiptOutcome::Fail(format!("exit_code={}", code))
            });
        }
    }
    let flat_key = format!("{}_success", profile);
    if let Some(b) = v.get(&flat_key).and_then(|x| x.as_bool()) {
        return Some(if b {
            ReceiptOutcome::Pass
        } else {
            ReceiptOutcome::Fail(format!("{}=false", flat_key))
        });
    }
    None
}

/// CB-1636 (L3): the generated `contracts/work/*.rs` modules must
/// compile under both `debug` and `release` profiles. Invoking
/// `cargo check` from inside a compliance check would be slow and
/// prone to cache thrash, so this check reads a receipt the work CLI
/// is expected to drop at `.pmat-work/codegen/compile-status.json`.
///
/// Accepted shapes (whichever Component 30's writer settles on):
///
/// ```json
/// { "debug":   {"success": true}, "release": {"success": true} }
/// { "debug":   0,                  "release": 0                 }  // exit codes
/// { "debug_success": true, "release_success": true }
/// ```
///
/// # Skip semantics (tiered)
///
/// * no `.pmat-work/codegen/` directory               → Skip
/// * no `compile-status.json` receipt                 → Skip
/// * receipt has neither profile recognisable         → Skip (schema
///                                                     not settled)
///
/// # Fail
///
/// * either profile explicitly reports failure
/// * receipt is malformed JSON
///
/// # Pass
///
/// * both profiles report success
pub(crate) fn check_macros_compile_debug_and_release(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1636: Generated Macros Compile (debug + release)";
    let dir = project_path.join(".pmat-work").join("codegen");
    if !dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/codegen/` directory — codegen has not been run".into(),
            severity: Severity::Info,
        };
    }
    let receipt = dir.join("compile-status.json");
    let Ok(contents) = std::fs::read_to_string(&receipt) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message:
                "No `.pmat-work/codegen/compile-status.json` — codegen has not emitted a compile receipt yet"
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

    let debug = compile_profile_outcome(&v, "debug");
    let release = compile_profile_outcome(&v, "release");
    match (debug, release) {
        (None, None) => ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "Compile receipt has neither profile recognisable — schema not settled".into(),
            severity: Severity::Info,
        },
        (Some(ReceiptOutcome::Pass), Some(ReceiptOutcome::Pass)) => ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "Generated macros compile in both debug and release".into(),
            severity: Severity::Info,
        },
        (d, r) => {
            let mut failures: Vec<String> = Vec::new();
            for (label, o) in [("debug", d), ("release", r)] {
                match o {
                    Some(ReceiptOutcome::Fail(detail)) => {
                        failures.push(format!("{}: {}", label, detail));
                    }
                    None => failures.push(format!("{}: no evidence", label)),
                    Some(ReceiptOutcome::Pass) => {}
                }
            }
            ComplianceCheck {
                name: name.into(),
                status: CheckStatus::Fail,
                message: format!(
                    "Generated macros failed to compile: {}",
                    failures.join("; ")
                ),
                severity: Severity::Error,
            }
        }
    }
}
