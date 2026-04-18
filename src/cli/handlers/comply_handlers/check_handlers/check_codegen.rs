// Work Compile-Time Codegen checks (CB-1630..1639) — Component 30
//
// Sub-spec: docs/specifications/components/pmat-work-compile-time-codegen.md
//
// `pv codegen` emits contract macros from provable-contracts YAML. Component 30
// inverts this: tickets under `.pmat-work/<ID>/contract.json` get a generated
// `contracts/work/<ID>.rs` module of `debug_assert!` macros, and a
// `#[pmat_work_contract(id = "...", require = "R1", ensure = "E1")]` attribute
// wraps target functions with the generated preconditions/postconditions.
//
// Neither the codegen CLI nor the `pmat-work-macros` proc-macro crate ship
// yet. Today's cut implements the checks that can run against today's
// infrastructure (static source scanning, JSON introspection, git ls-files):
//
//   CB-1630 (L2) — most recent `pmat work codegen` run succeeded, read
//                  from `.pmat-work/codegen/last-run.json`. Accepts
//                  success/exit_code/status shapes. Skip-if-absent.
//   CB-1631 (L2) — every `#[pmat_work_contract(id = X)]` in `src/` has a
//                  corresponding `contracts/work/X.rs` file
//   CB-1632 (L2) — attribute's `require = "Y"` / `ensure = "Y"` IDs each
//                  match a clause id in the referenced ticket's contract.json
//   CB-1633 (L3) — `contracts/work/<ID>.manifest.json` entries carry SHA-256
//                  hashes that match current bytes on disk. Skip-if-absent
//                  until codegen emits manifests.
//   CB-1634 (L3) — every clause with an `expr` field also has `binds_to`
//   CB-1635 (L3) — every `binds_to: "crate::a::b::c"` target must map to
//                  a file present in the ticket's
//                  `.pmat-work/<ID>/modified-files.json`. Skip-if-absent
//                  until the work CLI starts emitting diff receipts.
//   CB-1638 (L3) — generated modules under `contracts/work/*.rs` are tracked
//                  in git (not ungenerated transient state)
//
// The remaining checks (CB-1636 macro compile in release/debug,
// CB-1637 L2+ public-function coverage, CB-1639 Kani harness macro
// reference) surface as Skip with a "Deferred — requires X" message so
// config plumbing is wired for the follow-up work.

use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use super::types::*;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn deferred(name: &str, reason: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Skip,
        message: format!("Deferred — {}", reason),
        severity: Severity::Info,
    }
}

/// One parsed occurrence of the `#[pmat_work_contract(...)]` attribute.
/// `file` is the path the hit was read from; line numbers are not tracked
/// since the check emits repo-wide aggregates.
#[derive(Debug, Clone)]
struct AttributeUsage {
    file: PathBuf,
    id: String,
    requires: Vec<String>,
    ensures: Vec<String>,
}

/// Compile the attribute and inner-key regexes once per call. These would
/// be `OnceLock` globals in a hotter path, but per-check compile is cheap
/// enough here — the regex engine caches internally.
///
/// The attribute regex is line-anchored (`(?m)^[\s]*`) so occurrences of the
/// literal attribute text inside raw string literals (e.g. a test fixture
/// that writes `r#"#[pmat_work_contract(...)]"#` to a temp file) are ignored.
/// Real proc-macro usage always appears at the start of a source line.
fn attribute_parser() -> (Regex, Regex, Regex, Regex) {
    let attr = Regex::new(r"(?m)^[ \t]*#\[pmat_work_contract\(([^)]*)\)\]").unwrap();
    let id = Regex::new(r#"id\s*=\s*"([^"]+)""#).unwrap();
    let requires = Regex::new(r#"require\s*=\s*"([^"]+)""#).unwrap();
    let ensures = Regex::new(r#"ensure\s*=\s*"([^"]+)""#).unwrap();
    (attr, id, requires, ensures)
}

/// Skip files that only reference the attribute as documentation or in their
/// own test fixtures. Currently exempts this very file and its sibling test
/// harness paths; keep additions minimal and justify each entry with a
/// comment.
fn is_self_reference(path: &Path) -> bool {
    let s = path.to_string_lossy();
    // check_codegen.rs itself contains attribute literals in its test fixtures
    // and module-header comments; those are not real proc-macro uses.
    s.contains("check_codegen.rs")
}

/// Recursively scan `src/` for `#[pmat_work_contract(...)]` attribute usages.
/// Returns an empty vec when the directory is absent or no attribute is
/// found — callers use the vec length as a Skip gate.
fn collect_attribute_usages(project_path: &Path) -> Vec<AttributeUsage> {
    let src = project_path.join("src");
    if !src.exists() {
        return Vec::new();
    }
    let (attr, id_rx, req_rx, ens_rx) = attribute_parser();
    let mut out = Vec::new();
    for entry in WalkDir::new(&src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .filter(|e| !is_self_reference(e.path()))
    {
        let Ok(text) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        for cap in attr.captures_iter(&text) {
            let body = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let Some(id) = id_rx.captures(body).and_then(|c| c.get(1)) else {
                continue;
            };
            let requires: Vec<String> = req_rx
                .captures_iter(body)
                .filter_map(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .collect();
            let ensures: Vec<String> = ens_rx
                .captures_iter(body)
                .filter_map(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .collect();
            out.push(AttributeUsage {
                file: entry.path().to_path_buf(),
                id: id.as_str().to_string(),
                requires,
                ensures,
            });
        }
    }
    out
}

/// Load `.pmat-work/<ID>/contract.json` as `serde_json::Value` so we can read
/// `expr`/`binds_to` fields even though they aren't part of the typed
/// `ContractClause` yet (§Schema migration deferred). Missing file returns
/// `None`; malformed JSON is treated as missing.
fn load_contract_json(project_path: &Path, ticket_id: &str) -> Option<serde_json::Value> {
    let path = project_path
        .join(".pmat-work")
        .join(ticket_id)
        .join("contract.json");
    let text = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str::<serde_json::Value>(&text).ok()
}

/// Return every clause id present in the ticket JSON's require/ensure/invariant
/// arrays. Used by CB-1632 to validate that attribute IDs line up with the
/// ticket's declared clauses.
fn clause_ids_from_json(contract: &serde_json::Value) -> Vec<String> {
    let mut ids = Vec::new();
    for section in ["require", "ensure", "invariant"] {
        if let Some(arr) = contract.get(section).and_then(|v| v.as_array()) {
            for c in arr {
                if let Some(id) = c.get("id").and_then(|v| v.as_str()) {
                    ids.push(id.to_string());
                }
            }
        }
    }
    ids
}

/// Iterate every clause in require/ensure/invariant, returning references.
fn iter_clauses(contract: &serde_json::Value) -> impl Iterator<Item = &serde_json::Value> {
    ["require", "ensure", "invariant"]
        .into_iter()
        .filter_map(|s| contract.get(s).and_then(|v| v.as_array()))
        .flatten()
}

// ─── CB-1631: Attribute references generated module ─────────────────────────

/// CB-1631 (L2): Every `#[pmat_work_contract(id = "PMAT-530")]` in the
/// codebase requires a `contracts/work/PMAT-530.rs` file to exist. A missing
/// file means the attribute references a closed/purged ticket or the user
/// forgot to run `pmat work codegen`.
pub(crate) fn check_attribute_has_generated_module(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1631: Attribute Has Generated Module";
    let usages = collect_attribute_usages(project_path);
    if usages.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `#[pmat_work_contract]` attribute usage found in `src/`".into(),
            severity: Severity::Info,
        };
    }
    let mut missing: Vec<String> = Vec::new();
    for usage in &usages {
        let generated = project_path
            .join("contracts")
            .join("work")
            .join(format!("{}.rs", usage.id));
        if !generated.exists() {
            missing.push(format!(
                "{}: attribute id=\"{}\" but {} is missing",
                usage.file.display(),
                usage.id,
                generated.display()
            ));
        }
    }
    if missing.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "All {} `#[pmat_work_contract]` usage(s) resolve to generated modules",
                usages.len()
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Missing generated module(s): {}", missing.join("; ")),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1632: Attribute's require/ensure IDs match clauses ───────────────────

/// CB-1632 (L2): Every `require = "X"` and `ensure = "Y"` argument in a
/// `#[pmat_work_contract]` attribute must match a clause id in the referenced
/// ticket's `contract.json`. Typos here compile — the proc macro would fail
/// only at generation time — so this static check catches them early.
pub(crate) fn check_attribute_clause_ids_exist(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1632: Attribute Clause IDs Exist";
    let usages = collect_attribute_usages(project_path);
    if usages.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `#[pmat_work_contract]` attribute usage found in `src/`".into(),
            severity: Severity::Info,
        };
    }
    let mut mismatches: Vec<String> = Vec::new();
    for usage in &usages {
        let Some(contract) = load_contract_json(project_path, &usage.id) else {
            mismatches.push(format!(
                "{}: ticket `{}` has no `.pmat-work/{}/contract.json`",
                usage.file.display(),
                usage.id,
                usage.id
            ));
            continue;
        };
        let ids = clause_ids_from_json(&contract);
        for claim in usage.requires.iter().chain(usage.ensures.iter()) {
            if !ids.iter().any(|i| i == claim) {
                mismatches.push(format!(
                    "{}: attribute references `{}` on {} but no matching clause id",
                    usage.file.display(),
                    claim,
                    usage.id
                ));
            }
        }
    }
    if mismatches.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "All {} attribute clause id(s) resolve to ticket clauses",
                usages.len()
            ),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Clause id mismatches: {}", mismatches.join("; ")),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1634: Clauses with `expr` have `binds_to` ───────────────────────────

/// CB-1634 (L3): A clause with an `expr` field (codegen-ready Rust
/// expression) must also have a `binds_to` field (fully-qualified function
/// path). Without `binds_to`, the generator has no target to wrap — the
/// clause exists but doesn't apply to any code.
pub(crate) fn check_expr_clauses_have_binds_to(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1634: expr Clauses Have binds_to";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory present".into(),
            severity: Severity::Info,
        };
    }
    let mut orphaned: Vec<String> = Vec::new();
    let mut saw_expr = false;
    let Ok(entries) = std::fs::read_dir(&work_dir) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "Unable to read `.pmat-work/`".into(),
            severity: Severity::Info,
        };
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(ticket_id) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if ticket_id.starts_with('.') || ticket_id == "ledger" {
            continue;
        }
        let Some(contract) = load_contract_json(project_path, &ticket_id) else {
            continue;
        };
        for clause in iter_clauses(&contract) {
            let has_expr = clause.get("expr").is_some_and(|v| !v.is_null());
            if !has_expr {
                continue;
            }
            saw_expr = true;
            let has_binds = clause.get("binds_to").is_some_and(|v| !v.is_null());
            if !has_binds {
                let id = clause
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unknown>");
                orphaned.push(format!("{}#{}", ticket_id, id));
            }
        }
    }
    if !saw_expr {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No clause has an `expr` field yet".into(),
            severity: Severity::Info,
        };
    }
    if orphaned.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: "All clauses with `expr` also declare `binds_to`".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!(
                "{} clause(s) with `expr` missing `binds_to`: {}",
                orphaned.len(),
                orphaned.join(", ")
            ),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1638: Generated modules tracked in git ──────────────────────────────

/// CB-1638 (L3): Every `.rs` file under `contracts/work/` must be tracked
/// in git. An untracked file here means a developer ran `pmat work codegen`
/// without committing the output — next contributor's build will silently
/// regenerate.
pub(crate) fn check_generated_modules_tracked(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1638: Generated Modules Git-Tracked";
    let dir = project_path.join("contracts").join("work");
    if !dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `contracts/work/` directory present".into(),
            severity: Severity::Info,
        };
    }
    let mut untracked: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for entry in WalkDir::new(&dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        checked += 1;
        let out = std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch"])
            .arg(p)
            .current_dir(project_path)
            .output();
        match out {
            Ok(o) if o.status.success() => {}
            _ => untracked.push(p.display().to_string()),
        }
    }
    if checked == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.rs` files under `contracts/work/`".into(),
            severity: Severity::Info,
        };
    }
    if untracked.is_empty() {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!("All {} generated module(s) git-tracked", checked),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: format!("Untracked generated file(s): {}", untracked.join(", ")),
            severity: Severity::Error,
        }
    }
}

// ─── CB-1633, 1635, 1636, 1637, 1639 — deferred stubs ──────────────────────

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
///   recognised keys                                 → Skip (schema
///                                                   isn't settled;
///                                                   don't fail on
///                                                   unknown shapes)
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

enum ReceiptOutcome {
    Pass,
    Fail(String),
}

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

/// Parse a manifest JSON Value and return `(path, sha)` tuples. Accepts
/// three plausible shapes the Component 30 codegen writer might emit:
///
/// ```json
/// { "entries": [{ "path": "src/lib.rs", "sha": "..." }] }
/// { "files":   [{ "path": "src/lib.rs", "sha": "..." }] }
/// { "sources": [{ "path": "src/lib.rs", "sha": "..." }] }
/// ```
///
/// Returns `None` if no such array is present — caller treats that as a
/// malformed manifest, not as "no entries to check".
fn manifest_entries(v: &serde_json::Value) -> Option<Vec<(String, String)>> {
    let arr = v
        .get("entries")
        .or_else(|| v.get("files"))
        .or_else(|| v.get("sources"))
        .and_then(|v| v.as_array())?;
    let mut out = Vec::new();
    for item in arr {
        let Some(path) = item.get("path").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(sha) = item.get("sha").and_then(|v| v.as_str()) else {
            continue;
        };
        out.push((path.to_string(), sha.to_string()));
    }
    Some(out)
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    let mut s = String::with_capacity(d.len() * 2);
    for b in d {
        use std::fmt::Write;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

/// CB-1633 (L3): `contracts/work/<ID>.manifest.json` is emitted by codegen
/// to pin which source bytes the generated macros were derived from. If
/// the recorded SHA for an entry drifts from the current bytes on disk,
/// someone edited the source without re-running codegen and the generated
/// modules are stale.
///
/// Tiered skip semantics:
///   - no `contracts/work/` directory            → Skip
///   - no `*.manifest.json` files present        → Skip
///   - present but all entries match             → Pass
///   - else                                      → Fail
pub(crate) fn check_manifest_sha_drift(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1633: Manifest SHA Drift";
    let dir = project_path.join("contracts/work");
    if !dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `contracts/work/` directory — codegen hasn't emitted yet".into(),
            severity: Severity::Info,
        };
    }

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "`contracts/work/` unreadable".into(),
            severity: Severity::Info,
        };
    };

    let mut manifest_files: Vec<PathBuf> = Vec::new();
    for e in entries.flatten() {
        let path = e.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|f| f.to_str())
                .map(|s| s.ends_with(".manifest.json"))
                .unwrap_or(false)
        {
            manifest_files.push(path);
        }
    }

    if manifest_files.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message:
                "No `contracts/work/*.manifest.json` files — codegen hasn't emitted manifests yet"
                    .into(),
            severity: Severity::Info,
        };
    }

    let mut drifted: Vec<String> = Vec::new();
    let mut missing_files: Vec<String> = Vec::new();
    let mut malformed: Vec<String> = Vec::new();
    let mut entries_checked = 0usize;

    for manifest_path in &manifest_files {
        let manifest_name = manifest_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("<unknown>")
            .to_string();
        let Ok(contents) = std::fs::read_to_string(manifest_path) else {
            malformed.push(format!("  {} (unreadable)", manifest_name));
            continue;
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) else {
            malformed.push(format!("  {} (not valid JSON)", manifest_name));
            continue;
        };
        let Some(list) = manifest_entries(&json) else {
            malformed.push(format!(
                "  {} (missing `entries`/`files`/`sources` array)",
                manifest_name
            ));
            continue;
        };
        for (rel_path, recorded_sha) in list {
            entries_checked += 1;
            let abs = project_path.join(&rel_path);
            let Ok(bytes) = std::fs::read(&abs) else {
                missing_files.push(format!("  {} -> {} (not found)", manifest_name, rel_path));
                continue;
            };
            let current = sha256_hex(&bytes);
            if !current.eq_ignore_ascii_case(&recorded_sha) {
                drifted.push(format!(
                    "  {} -> {} recorded={}… current={}…",
                    manifest_name,
                    rel_path,
                    &recorded_sha[..recorded_sha.len().min(8)],
                    &current[..current.len().min(8)]
                ));
            }
        }
    }

    if drifted.is_empty() && missing_files.is_empty() && malformed.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} manifest(s), {} entry/entries — all SHA(s) match",
                manifest_files.len(),
                entries_checked
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = String::new();
    if !drifted.is_empty() {
        msg.push_str(&format!(
            "{} manifest entry/entries drifted:\n",
            drifted.len()
        ));
        for line in &drifted {
            msg.push_str(line);
            msg.push('\n');
        }
    }
    if !missing_files.is_empty() {
        msg.push_str(&format!(
            "{} referenced file(s) missing:\n",
            missing_files.len()
        ));
        for line in &missing_files {
            msg.push_str(line);
            msg.push('\n');
        }
    }
    if !malformed.is_empty() {
        msg.push_str(&format!("{} malformed manifest(s):\n", malformed.len()));
        for line in &malformed {
            msg.push_str(line);
            msg.push('\n');
        }
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
}

/// CB-1635 (L3): every clause that carries `binds_to: "crate::a::b::c"`
/// must point at a file that was actually modified by the ticket.
/// Otherwise the contract declares an obligation over untouched code — a
/// silent no-op that erodes provenance.
///
/// The ticket-level modified-file list is expected at
/// `.pmat-work/<ID>/modified-files.json` as `{"files": ["src/a/b.rs",
/// ...]}` (also accepts a top-level array or a `modified` key). Component
/// 30's work CLI is expected to populate it from `git diff --name-only`
/// against the ticket's base ref.
///
/// # Skip semantics (tiered)
///
/// * no `.pmat-work/*/contract.json` tickets                 → Skip
/// * no ticket has any clause with `binds_to`                → Skip
/// * no ticket has `modified-files.json` yet                 → Skip
///
/// # Fail
///
/// * any `binds_to` path resolves to a candidate file (or
///   module directory) that isn't in the ticket's modified
///   list                                                    → Fail
fn resolve_binds_to_candidates(path: &str) -> Vec<String> {
    // `crate::a::b::c` → [`src/a/b/c.rs`, `src/a/b/c/mod.rs`, `src/a/b.rs`,
    //                    `src/a/b/mod.rs`, `src/a.rs`, `src/a/mod.rs`, `src/lib.rs`]
    // Strip the trailing `::function` segment — we want the *defining* module.
    let without_crate = path.strip_prefix("crate::").unwrap_or(path);
    let mut parts: Vec<&str> = without_crate.split("::").collect();
    // Drop the terminal identifier — it's the function name, not part of the path
    if parts.len() > 1 {
        parts.pop();
    }
    let mut out = Vec::new();
    // Start most specific and walk up
    while !parts.is_empty() {
        let joined = parts.join("/");
        out.push(format!("src/{}.rs", joined));
        out.push(format!("src/{}/mod.rs", joined));
        parts.pop();
    }
    // Fallback — crate root
    out.push("src/lib.rs".into());
    out.push("src/main.rs".into());
    out
}

fn load_modified_files(project_path: &Path, ticket_id: &str) -> Option<Vec<String>> {
    let p = project_path
        .join(".pmat-work")
        .join(ticket_id)
        .join("modified-files.json");
    let text = std::fs::read_to_string(&p).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    if let Some(arr) = v.as_array() {
        return Some(
            arr.iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect(),
        );
    }
    for key in ["files", "modified"] {
        if let Some(arr) = v.get(key).and_then(|x| x.as_array()) {
            return Some(
                arr.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect(),
            );
        }
    }
    None
}

pub(crate) fn check_binds_to_function_modified(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1635: binds_to Function Actually Modified";
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/` directory present".into(),
            severity: Severity::Info,
        };
    }
    let Ok(entries) = std::fs::read_dir(&work_dir) else {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "Unable to read `.pmat-work/`".into(),
            severity: Severity::Info,
        };
    };

    let mut saw_any_binds = false;
    let mut evaluated_tickets = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(ticket_id) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if ticket_id.starts_with('.') || ticket_id == "ledger" {
            continue;
        }
        let Some(contract) = load_contract_json(project_path, &ticket_id) else {
            continue;
        };

        let binds_to_paths: Vec<(String, String)> = iter_clauses(&contract)
            .filter_map(|c| {
                let bt = c.get("binds_to").and_then(|v| v.as_str())?;
                let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("<unknown>");
                Some((id.to_string(), bt.to_string()))
            })
            .collect();
        if binds_to_paths.is_empty() {
            continue;
        }
        saw_any_binds = true;

        let Some(modified) = load_modified_files(project_path, &ticket_id) else {
            continue;
        };
        evaluated_tickets += 1;

        for (clause_id, bt) in &binds_to_paths {
            let candidates = resolve_binds_to_candidates(bt);
            let hit = candidates.iter().any(|c| modified.iter().any(|m| m == c));
            if !hit {
                violations.push(format!(
                    "  {}#{} → {} (tried: {})",
                    ticket_id,
                    clause_id,
                    bt,
                    candidates.join(", ")
                ));
            }
        }
    }

    if !saw_any_binds {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No ticket has a clause with `binds_to`".into(),
            severity: Severity::Info,
        };
    }
    if evaluated_tickets == 0 {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message:
                "No `.pmat-work/<ID>/modified-files.json` — work CLI has not emitted diff receipts yet"
                    .into(),
            severity: Severity::Info,
        };
    }

    if !violations.is_empty() {
        let mut msg = format!(
            "{} `binds_to` clause(s) target files the ticket did not modify:\n",
            violations.len()
        );
        let preview: Vec<&String> = violations.iter().take(5).collect();
        for line in preview {
            msg.push_str(line);
            msg.push('\n');
        }
        if violations.len() > 5 {
            msg.push_str(&format!("  …and {} more\n", violations.len() - 5));
        }
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Fail,
            message: msg,
            severity: Severity::Error,
        };
    }

    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Pass,
        message: format!(
            "{} ticket(s): every `binds_to` target appears in modified files",
            evaluated_tickets
        ),
        severity: Severity::Info,
    }
}

pub(crate) fn check_macros_compile_debug_and_release(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1636: Generated Macros Compile (debug + release)",
        "requires generated modules under `contracts/work/` to exercise",
    )
}

pub(crate) fn check_l2_public_fn_coverage(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1637: L2+ Public Function Coverage",
        "requires codegen + git diff + target_level ≥ L2 cross-reference",
    )
}

pub(crate) fn check_kani_harness_macro_reference(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1639: Kani Harnesses Reference Generated Macros",
        "requires derived YAML with `kani_harnesses[]` pointing at macros",
    )
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn attribute_parser_extracts_id_and_clauses() {
        let (attr, id_rx, req_rx, ens_rx) = attribute_parser();
        let src = r#"#[pmat_work_contract(id = "PMAT-530", require = "R1", ensure = "E1", ensure = "E2")]"#;
        let body = attr.captures(src).unwrap().get(1).unwrap().as_str();
        assert_eq!(
            id_rx.captures(body).unwrap().get(1).unwrap().as_str(),
            "PMAT-530"
        );
        let requires: Vec<&str> = req_rx
            .captures_iter(body)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str())
            .collect();
        assert_eq!(requires, vec!["R1"]);
        let ensures: Vec<&str> = ens_rx
            .captures_iter(body)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str())
            .collect();
        assert_eq!(ensures, vec!["E1", "E2"]);
    }

    #[test]
    fn attribute_has_generated_module_skips_when_no_usage() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        let check = check_attribute_has_generated_module(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn attribute_has_generated_module_fails_when_file_missing() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("a.rs"),
            r#"#[pmat_work_contract(id = "PMAT-999")] fn f(){}"#,
        )
        .unwrap();
        let check = check_attribute_has_generated_module(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("PMAT-999"));
    }

    #[test]
    fn attribute_has_generated_module_passes_when_file_present() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        let gen_dir = tmp.path().join("contracts/work");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&gen_dir).unwrap();
        std::fs::write(
            src.join("a.rs"),
            r#"#[pmat_work_contract(id = "PMAT-100")] fn f(){}"#,
        )
        .unwrap();
        std::fs::write(gen_dir.join("PMAT-100.rs"), "// generated").unwrap();
        let check = check_attribute_has_generated_module(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn attribute_clause_ids_exist_skips_without_usage() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        let check = check_attribute_clause_ids_exist(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn attribute_clause_ids_exist_fails_on_typo() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        let work = tmp.path().join(".pmat-work/PMAT-100");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            src.join("a.rs"),
            r#"#[pmat_work_contract(id = "PMAT-100", require = "R1", ensure = "EX")] fn f(){}"#,
        )
        .unwrap();
        std::fs::write(
            work.join("contract.json"),
            r#"{"require":[{"id":"R1"}],"ensure":[{"id":"E1"}]}"#,
        )
        .unwrap();
        let check = check_attribute_clause_ids_exist(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("EX"));
    }

    #[test]
    fn attribute_clause_ids_exist_passes_when_all_match() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        let work = tmp.path().join(".pmat-work/PMAT-100");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            src.join("a.rs"),
            r#"#[pmat_work_contract(id = "PMAT-100", require = "R1", ensure = "E1")] fn f(){}"#,
        )
        .unwrap();
        std::fs::write(
            work.join("contract.json"),
            r#"{"require":[{"id":"R1"}],"ensure":[{"id":"E1"}]}"#,
        )
        .unwrap();
        let check = check_attribute_clause_ids_exist(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn expr_binds_to_skips_when_no_expr_in_any_clause() {
        let tmp = tempdir().unwrap();
        let work = tmp.path().join(".pmat-work/PMAT-100");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            work.join("contract.json"),
            r#"{"require":[{"id":"R1"}],"ensure":[{"id":"E1"}]}"#,
        )
        .unwrap();
        let check = check_expr_clauses_have_binds_to(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn expr_binds_to_fails_when_missing_binds_to() {
        let tmp = tempdir().unwrap();
        let work = tmp.path().join(".pmat-work/PMAT-100");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            work.join("contract.json"),
            r#"{"require":[{"id":"R1","expr":"x > 0"}]}"#,
        )
        .unwrap();
        let check = check_expr_clauses_have_binds_to(tmp.path());
        assert_eq!(check.status, CheckStatus::Fail);
        assert!(check.message.contains("PMAT-100#R1"));
    }

    #[test]
    fn expr_binds_to_passes_when_present() {
        let tmp = tempdir().unwrap();
        let work = tmp.path().join(".pmat-work/PMAT-100");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(
            work.join("contract.json"),
            r#"{"require":[{"id":"R1","expr":"x > 0","binds_to":"crate::f"}]}"#,
        )
        .unwrap();
        let check = check_expr_clauses_have_binds_to(tmp.path());
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn generated_modules_tracked_skips_without_dir() {
        let tmp = tempdir().unwrap();
        let check = check_generated_modules_tracked(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn generated_modules_tracked_skips_when_dir_empty() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("contracts/work")).unwrap();
        let check = check_generated_modules_tracked(tmp.path());
        assert_eq!(check.status, CheckStatus::Skip);
    }

    #[test]
    fn deferred_checks_return_skip_with_reason() {
        let path = Path::new(".");
        for (name, check) in [
            ("CB-1636", check_macros_compile_debug_and_release(path)),
            ("CB-1637", check_l2_public_fn_coverage(path)),
            ("CB-1639", check_kani_harness_macro_reference(path)),
        ] {
            assert_eq!(check.status, CheckStatus::Skip, "{}", name);
            assert!(
                check.message.starts_with("Deferred — "),
                "{}: {}",
                name,
                check.message
            );
        }
    }

    // ── CB-1633 manifest SHA drift tests ─────────────────────────────────

    fn write_file(project: &Path, rel: &str, body: &[u8]) -> String {
        let p = project.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, body).unwrap();
        sha256_hex(body)
    }

    fn write_manifest(project: &Path, name: &str, body: &str) {
        let dir = project.join("contracts/work");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{}.manifest.json", name)), body).unwrap();
    }

    #[test]
    fn manifest_sha_skip_when_no_dir() {
        let tmp = tempdir().unwrap();
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("contracts/work/"));
    }

    #[test]
    fn manifest_sha_skip_when_no_manifests() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("contracts/work")).unwrap();
        // A plain .rs module, not a manifest
        std::fs::write(tmp.path().join("contracts/work/PMAT-1.rs"), "// generated").unwrap();
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("manifest.json"));
    }

    #[test]
    fn manifest_sha_pass_when_all_match() {
        let tmp = tempdir().unwrap();
        let sha_a = write_file(tmp.path(), "src/a.rs", b"fn a(){}");
        let sha_b = write_file(tmp.path(), "src/b.rs", b"fn b(){}");
        write_manifest(
            tmp.path(),
            "PMAT-1",
            &format!(
                r#"{{"ticket":"PMAT-1","entries":[{{"path":"src/a.rs","sha":"{sha_a}"}},{{"path":"src/b.rs","sha":"{sha_b}"}}]}}"#
            ),
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 manifest"));
        assert!(r.message.contains("2 entry"));
    }

    #[test]
    fn manifest_sha_accepts_files_alias() {
        let tmp = tempdir().unwrap();
        let sha_a = write_file(tmp.path(), "src/a.rs", b"fn a(){}");
        // Alternate naming — `files` instead of `entries`
        write_manifest(
            tmp.path(),
            "PMAT-1",
            &format!(r#"{{"files":[{{"path":"src/a.rs","sha":"{sha_a}"}}]}}"#),
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn manifest_sha_accepts_sources_alias() {
        let tmp = tempdir().unwrap();
        let sha_a = write_file(tmp.path(), "src/a.rs", b"fn a(){}");
        write_manifest(
            tmp.path(),
            "PMAT-1",
            &format!(r#"{{"sources":[{{"path":"src/a.rs","sha":"{sha_a}"}}]}}"#),
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn manifest_sha_fails_on_drift() {
        let tmp = tempdir().unwrap();
        write_file(tmp.path(), "src/a.rs", b"fn a(){}");
        // Recorded sha is stale (file content differs from recorded hash)
        write_manifest(
            tmp.path(),
            "PMAT-1",
            r#"{"entries":[{"path":"src/a.rs","sha":"deadbeef"}]}"#,
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("drifted"));
        assert!(r.message.contains("src/a.rs"));
        assert!(r.message.contains("deadbeef"));
    }

    #[test]
    fn manifest_sha_fails_on_missing_file() {
        let tmp = tempdir().unwrap();
        write_manifest(
            tmp.path(),
            "PMAT-1",
            r#"{"entries":[{"path":"src/ghost.rs","sha":"deadbeef"}]}"#,
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("missing"));
        assert!(r.message.contains("src/ghost.rs"));
    }

    #[test]
    fn manifest_sha_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        write_manifest(tmp.path(), "PMAT-1", "not-json{");
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("not valid JSON"));
    }

    #[test]
    fn manifest_sha_fails_on_missing_entries_key() {
        let tmp = tempdir().unwrap();
        write_manifest(tmp.path(), "PMAT-1", r#"{"ticket":"PMAT-1"}"#);
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("missing `entries`"));
    }

    #[test]
    fn manifest_sha_case_insensitive_hex_match() {
        let tmp = tempdir().unwrap();
        let sha_upper = write_file(tmp.path(), "src/a.rs", b"fn a(){}").to_uppercase();
        write_manifest(
            tmp.path(),
            "PMAT-1",
            &format!(r#"{{"entries":[{{"path":"src/a.rs","sha":"{sha_upper}"}}]}}"#),
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn manifest_sha_aggregates_across_multiple_manifests() {
        let tmp = tempdir().unwrap();
        let sha_a = write_file(tmp.path(), "src/a.rs", b"fn a(){}");
        write_file(tmp.path(), "src/b.rs", b"fn b(){}");
        write_manifest(
            tmp.path(),
            "PMAT-1",
            &format!(r#"{{"entries":[{{"path":"src/a.rs","sha":"{sha_a}"}}]}}"#),
        );
        // Second manifest has stale hash → one drift
        write_manifest(
            tmp.path(),
            "PMAT-2",
            r#"{"entries":[{"path":"src/b.rs","sha":"cafebabe"}]}"#,
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("src/b.rs"));
        assert!(!r.message.contains("src/a.rs (drift)"));
    }

    // ── CB-1630 codegen-CLI-succeeds tests ───────────────────────────────

    fn write_codegen_receipt(project: &Path, body: &str) {
        let dir = project.join(".pmat-work").join("codegen");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("last-run.json"), body).unwrap();
    }

    #[test]
    fn cb1630_skips_when_codegen_dir_missing() {
        let tmp = tempdir().unwrap();
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/codegen/` directory"));
    }

    #[test]
    fn cb1630_skips_when_receipt_missing() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pmat-work").join("codegen")).unwrap();
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("last-run.json"));
    }

    #[test]
    fn cb1630_passes_on_success_true() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"success": true}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1630_fails_on_success_false() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"success": false}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("success=false"));
    }

    #[test]
    fn cb1630_passes_on_exit_code_zero() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"exit_code": 0}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1630_fails_on_exit_code_nonzero() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"exit_code": 1}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("exit_code=1"));
    }

    #[test]
    fn cb1630_passes_on_status_pass() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"status": "pass"}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1630_passes_on_status_ok_or_success() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"status": "ok"}"#);
        let r1 = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r1.status, CheckStatus::Pass, "{}", r1.message);

        let tmp2 = tempdir().unwrap();
        write_codegen_receipt(tmp2.path(), r#"{"status": "success"}"#);
        let r2 = check_codegen_cli_succeeds(tmp2.path());
        assert_eq!(r2.status, CheckStatus::Pass, "{}", r2.message);
    }

    #[test]
    fn cb1630_fails_on_status_fail() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"status": "fail"}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("status=\"fail\""));
    }

    #[test]
    fn cb1630_skips_on_unknown_schema() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"unexpected_field": 42}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("schema not settled"));
    }

    #[test]
    fn cb1630_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), "not-json");
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("Malformed JSON"));
    }

    #[test]
    fn cb1630_success_takes_precedence_over_exit_code() {
        // If both keys exist, `success` wins because it's the most explicit.
        let tmp = tempdir().unwrap();
        write_codegen_receipt(tmp.path(), r#"{"success": true, "exit_code": 99}"#);
        let r = check_codegen_cli_succeeds(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    // ── CB-1635 binds_to-function-modified tests ─────────────────────────

    fn write_ticket_contract(project: &Path, ticket: &str, body: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("contract.json"), body).unwrap();
    }

    fn write_modified_files(project: &Path, ticket: &str, body: &str) {
        let dir = project.join(".pmat-work").join(ticket);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("modified-files.json"), body).unwrap();
    }

    #[test]
    fn cb1635_resolves_binds_to_candidates() {
        let c = resolve_binds_to_candidates("crate::a::b::c::func");
        // Most specific first
        assert!(c[0] == "src/a/b/c.rs");
        assert!(c.iter().any(|s| s == "src/a/b/c/mod.rs"));
        assert!(c.iter().any(|s| s == "src/a/b.rs"));
        assert!(c.iter().any(|s| s == "src/a.rs"));
        assert!(c.iter().any(|s| s == "src/lib.rs"));
    }

    #[test]
    fn cb1635_resolves_bare_ident() {
        // `crate::func` has no module prefix — pop leaves parts empty,
        // so only crate-root fallbacks remain.
        let c = resolve_binds_to_candidates("crate::top_level_fn");
        assert!(c.iter().any(|s| s == "src/lib.rs"));
        assert!(c.iter().any(|s| s == "src/main.rs"));
    }

    #[test]
    fn cb1635_skips_when_no_work_dir() {
        let tmp = tempdir().unwrap();
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No `.pmat-work/`"));
    }

    #[test]
    fn cb1635_skips_when_no_ticket_has_binds_to() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","expr":"x > 0"}]}"#,
        );
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("No ticket has a clause with `binds_to`"));
    }

    #[test]
    fn cb1635_skips_when_no_modified_files_artifact() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::f"}]}"#,
        );
        // no modified-files.json
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip, "{}", r.message);
        assert!(r.message.contains("modified-files.json"));
    }

    #[test]
    fn cb1635_passes_when_binds_to_matches_modified_file() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::b::f"}]}"#,
        );
        write_modified_files(
            tmp.path(),
            "T-1",
            r#"{"files":["src/a/b.rs","src/other.rs"]}"#,
        );
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1635_fails_when_binds_to_target_untouched() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::b::f"}]}"#,
        );
        write_modified_files(tmp.path(), "T-1", r#"{"files":["src/elsewhere.rs"]}"#);
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-1"));
        assert!(r.message.contains("R1"));
    }

    #[test]
    fn cb1635_accepts_mod_rs_candidate() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::b::f"}]}"#,
        );
        // File is at src/a/b/mod.rs not src/a/b.rs — resolver should find it
        write_modified_files(tmp.path(), "T-1", r#"{"files":["src/a/b/mod.rs"]}"#);
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1635_accepts_top_level_array_shape() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::f"}]}"#,
        );
        // Plain array shape
        write_modified_files(tmp.path(), "T-1", r#"["src/a.rs"]"#);
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1635_accepts_modified_key_shape() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::f"}]}"#,
        );
        write_modified_files(tmp.path(), "T-1", r#"{"modified":["src/a.rs"]}"#);
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn cb1635_aggregates_across_clauses_and_tickets() {
        let tmp = tempdir().unwrap();
        // T-GOOD: binds_to target matches modified file
        write_ticket_contract(
            tmp.path(),
            "T-GOOD",
            r#"{"require":[{"id":"R1","binds_to":"crate::a::f"}]}"#,
        );
        write_modified_files(tmp.path(), "T-GOOD", r#"{"files":["src/a.rs"]}"#);

        // T-BAD: binds_to target untouched
        write_ticket_contract(
            tmp.path(),
            "T-BAD",
            r#"{"ensure":[{"id":"E1","binds_to":"crate::x::f"}]}"#,
        );
        write_modified_files(tmp.path(), "T-BAD", r#"{"files":["src/a.rs"]}"#);

        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("T-BAD"));
        assert!(!r.message.contains("T-GOOD"), "{}", r.message);
    }

    #[test]
    fn cb1635_binds_to_in_ensure_and_invariant_sections() {
        let tmp = tempdir().unwrap();
        write_ticket_contract(
            tmp.path(),
            "T-1",
            r#"{"ensure":[{"id":"E1","binds_to":"crate::a::f"}],"invariant":[{"id":"I1","binds_to":"crate::b::g"}]}"#,
        );
        write_modified_files(tmp.path(), "T-1", r#"{"files":["src/a.rs","src/b.rs"]}"#);
        let r = check_binds_to_function_modified(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }
}
