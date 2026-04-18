// CB-1633: Manifest SHA drift check.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

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
