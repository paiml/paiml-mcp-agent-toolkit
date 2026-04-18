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
//   CB-1636 (L3) — `.pmat-work/codegen/compile-status.json` must report
//                  the generated macros compiling in both debug and
//                  release profiles. Skip-if-absent.
//   CB-1637 (L2+) — every modified `.rs` file declaring `pub fn` carries a
//                   matching `#[pmat_work_contract(id = X)]` attribute.
//                   Skip-if-absent (needs L2+ tickets + modified-files.json).
//   CB-1638 (L3) — generated modules under `contracts/work/*.rs` are tracked
//                  in git (not ungenerated transient state)
//   CB-1639 (L4+) — every L4+ ticket's `kani_harnesses[]` names resolve to a
//                   harness body in `kani/`, `tests/`, `harnesses/`, or
//                   `src/` that references `contracts::work::<ID>` (or the
//                   `#[pmat_work_contract(id = ...)]` attribute). Skip-if-
//                   absent until Kani integration is authored.
//
// All CB-163x checks are now active (skip-if-absent). The module retains
// schema-pragmatic parsers so Component 30's writers can converge on any
// reasonable receipt shape without breaking these gates.
//
// ── File layout (split via `include!()` to keep each partition under 600 lines) ──
//
// This file holds shared imports and helper functions used across all checks.
// Each check function lives in its own sibling partition file, included at
// the bottom of this file. Partition files are bare code chunks — they do
// NOT carry their own `use` imports or `#![...]` inner attributes; they
// inherit the scope established here.

use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use super::types::*;

// ─── Shared helpers: attribute scanning ──────────────────────────────────────

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

// ─── Shared helpers: contract.json introspection ─────────────────────────────

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

/// Iterate every clause in require/ensure/invariant, returning references.
fn iter_clauses(contract: &serde_json::Value) -> impl Iterator<Item = &serde_json::Value> {
    ["require", "ensure", "invariant"]
        .into_iter()
        .filter_map(|s| contract.get(s).and_then(|v| v.as_array()))
        .flatten()
}

/// Parse the `verification_level` string from a ticket JSON and compare to a
/// target. Accepts `"L3"`, `"L3 (kani_proof)"`, `"l2"`, etc. Unknown shapes
/// return `false` so the check gates conservatively (skip not fail).
fn contract_level_at_least(contract: &serde_json::Value, target: u8) -> bool {
    let Some(s) = contract.get("verification_level").and_then(|v| v.as_str()) else {
        return false;
    };
    let token = s.split_whitespace().next().unwrap_or("");
    let trimmed = token.trim();
    let after_l = trimmed
        .strip_prefix(|c: char| c == 'L' || c == 'l')
        .unwrap_or(trimmed);
    let digits: String = after_l.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse::<u8>().map(|n| n >= target).unwrap_or(false)
}

// ─── Shared helpers: receipt JSON interpretation ─────────────────────────────

/// Outcome of parsing a codegen run or compile-status receipt. Used by
/// CB-1630 and CB-1636 — both kept small by moving their per-check
/// interpretation logic to their own partition files.
enum ReceiptOutcome {
    Pass,
    Fail(String),
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

// ─── Shared helpers: binds_to path resolution ────────────────────────────────

/// Resolve `crate::a::b::c` → candidate source paths. The terminal identifier
/// is treated as the function name (dropped); parents cascade from most
/// specific to least (`src/a/b.rs`, `src/a/b/mod.rs`, …, `src/lib.rs`).
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

/// Load `.pmat-work/<ID>/modified-files.json` as `Vec<String>`. Accepts three
/// shapes: top-level array, `{"files": [...]}`, or `{"modified": [...]}`.
/// Returns `None` if the file is absent/malformed so callers can skip.
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

// ─── Check implementations (split across partition files) ────────────────────

include!("check_codegen_attribute.rs");
include!("check_codegen_clauses.rs");
include!("check_codegen_receipts.rs");
include!("check_codegen_manifest_sha.rs");
include!("check_codegen_binds_modified.rs");
include!("check_codegen_compile_profiles.rs");
include!("check_codegen_pub_fn_coverage.rs");
include!("check_codegen_harness_refs.rs");

// ─── Tests (split across partition files) ────────────────────────────────────

include!("check_codegen_tests_attribute.rs");
include!("check_codegen_tests_manifest.rs");
include!("check_codegen_tests_receipts.rs");
include!("check_codegen_tests_harness.rs");
