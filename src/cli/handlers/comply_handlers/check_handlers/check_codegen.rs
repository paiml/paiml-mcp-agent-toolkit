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
//   CB-1631 (L2) — every `#[pmat_work_contract(id = X)]` in `src/` has a
//                  corresponding `contracts/work/X.rs` file
//   CB-1632 (L2) — attribute's `require = "Y"` / `ensure = "Y"` IDs each
//                  match a clause id in the referenced ticket's contract.json
//   CB-1634 (L3) — every clause with an `expr` field also has `binds_to`
//   CB-1638 (L3) — generated modules under `contracts/work/*.rs` are tracked
//                  in git (not ungenerated transient state)
//
// The remaining checks (CB-1630 codegen CLI success, CB-1633 manifest SHA
// drift, CB-1635 `binds_to` function modification, CB-1636 macro compile
// in release/debug, CB-1637 L2+ public-function coverage, CB-1639 Kani
// harness macro reference) surface as Skip with a "Deferred — requires X"
// message so config plumbing is wired for the follow-up work.

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

// ─── CB-1630, 1633, 1635, 1636, 1637, 1639 — deferred stubs ─────────────────

pub(crate) fn check_codegen_cli_succeeds(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1630: pmat work codegen Succeeds",
        "requires `pmat work codegen --check` CLI (Component 30 §CLI Surface)",
    )
}

pub(crate) fn check_manifest_sha_drift(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1633: Manifest SHA Drift",
        "requires `contracts/work/<ID>.manifest.json` emitted by codegen",
    )
}

pub(crate) fn check_binds_to_function_modified(_project_path: &Path) -> ComplianceCheck {
    deferred(
        "CB-1635: binds_to Function Actually Modified",
        "requires `binds_to` schema field and git diff integration",
    )
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
            ("CB-1630", check_codegen_cli_succeeds(path)),
            ("CB-1633", check_manifest_sha_drift(path)),
            ("CB-1635", check_binds_to_function_modified(path)),
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
}
