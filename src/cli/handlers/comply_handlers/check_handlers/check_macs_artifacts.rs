// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1656: the committed root `mcp.json` must advertise exactly the live
/// tool set (MACS F6). Compares the tool NAME set against the canonical
/// `LIVE_MCP_TOOLS` source — version churn is intentionally ignored so a
/// release bump does not turn this red; only tool add/remove/rename drifts.
pub(crate) fn check_mcp_manifest_faithful(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1656: MCP Manifest Faithful";
    let manifest_path = project_path.join("mcp.json");
    if !manifest_path.exists() {
        return skip_check(name, "No root mcp.json");
    }
    let Ok(text) = std::fs::read_to_string(&manifest_path) else {
        return skip_check(name, "mcp.json unreadable");
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: "mcp.json is not valid JSON".to_string(),
            severity: Severity::Error,
        };
    };

    let declared = crate::mcp_pmcp::tool_manifest::manifest_tool_names(&value);
    let canonical = crate::mcp_pmcp::tool_manifest::canonical_tool_names();
    if declared == canonical {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: format!("mcp.json advertises all {} live tools", canonical.len()),
            severity: Severity::Info,
        };
    }

    let missing: Vec<String> = canonical
        .iter()
        .filter(|t| !declared.contains(t))
        .cloned()
        .collect();
    let extra: Vec<String> = declared
        .iter()
        .filter(|t| !canonical.contains(t))
        .cloned()
        .collect();
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Fail,
        message: format!(
            "mcp.json drifted from the live tool set ({} declared vs {} live). \
             Regenerate: cargo test --lib regenerate_mcp_json -- --ignored. \
             missing={missing:?} extra={extra:?}",
            declared.len(),
            canonical.len()
        ),
        severity: Severity::Error,
    }
}

/// Marks a document as a historical record, exempting it from CB-1657.
///
/// Put it anywhere in the file, ideally in the front matter:
/// `<!-- pmat: historical-model-ids -->`
///
/// It exempts the document from the superseded-id scan and nothing else. It
/// exists so a record of what was true on a date can stay accurate, instead of
/// being rewritten to satisfy a check about what is true now (#987).
pub(crate) const HISTORICAL_MARKER: &str = "<!-- pmat: historical-model-ids -->";

/// CB-1657: no superseded model id (`claude-3-*`, `claude-2*`, `gpt-4-turbo`)
/// appears in `docs/` outside the allow-listed registry (MACS F6). Stale ids
/// in agent-facing docs are executable misinformation.
///
/// Exempt: `docs/archive/**` and any file carrying [`HISTORICAL_MARKER`].
pub(crate) fn check_doc_model_drift(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1657: Doc Model Drift";
    let docs_dir = project_path.join("docs");
    if !docs_dir.exists() {
        return skip_check(name, "No docs/ directory");
    }

    // Allow-listed: the registry (history table) and the MACS spec itself
    // (a spec about purging ids must name the ids it purges).
    let allow: [&str; 2] = [
        "docs/agent-models.md",
        "docs/specifications/components/modern-agentic-coding-support.md",
    ];

    let mut hits: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    // Counted separately so "nothing was scanned" can never be reported as
    // "there was nothing to scan" (#987). A tree whose only docs are archived
    // used to come back `Skip: No markdown docs to scan`, which is a false
    // statement about the tree and hides the fact that an exemption is load
    // bearing. Exempted docs are now named in the message.
    let mut exempt = 0usize;
    macs_walk_markdown(&docs_dir, &mut |path, text| {
        let rel = path
            .strip_prefix(project_path)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        if allow.iter().any(|a| rel == *a) {
            return;
        }
        // A historical document is not drift. #987: an archived analysis dated
        // 2026-02-23 quoted a cost-tier match expression from the code as it
        // stood that day. CB-1657 flagged its five superseded ids and the only
        // ways to pass were to EDIT the historical record or DELETE it — both
        // of which make the archive describe something that did not happen, in
        // order to satisfy a present-day gate. A check that can only be
        // satisfied by falsifying the record is worse than no check.
        //
        // Two exemptions, both narrow:
        //  - an in-file marker, so the exemption lives beside the content it
        //    covers and is visible in review rather than drifting away in a
        //    config file;
        //  - `docs/archive/` by default, because archived documentation is a
        //    common enough pattern to be worth a default.
        // Neither weakens the check for live docs, which is what F6 is about:
        // stale ids in agent-facing documentation are executable misinformation.
        if rel.starts_with("docs/archive/") || text.contains(HISTORICAL_MARKER) {
            exempt += 1;
            return;
        }
        scanned += 1;
        for (lineno, line) in text.lines().enumerate() {
            if macs_line_has_denied_model(line) {
                hits.push(format!("{rel}:{}: {}", lineno + 1, line.trim()));
            }
        }
    });

    let exempt_note = if exempt > 0 {
        format!(" ({exempt} exempt: docs/archive/ or `{HISTORICAL_MARKER}`)")
    } else {
        String::new()
    };
    if scanned == 0 {
        return skip_check(
            name,
            &if exempt > 0 {
                format!(
                    "no doc left to scan — all {exempt} markdown doc(s) are exempt \
                     (docs/archive/ or `{HISTORICAL_MARKER}`)"
                )
            } else {
                "No markdown docs to scan".to_string()
            },
        );
    }
    if hits.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: format!("{scanned} doc(s) clean of superseded model ids{exempt_note}"),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Fail,
        message: format!(
            "{} superseded model id(s) outside docs/agent-models.md (MACS F6):\n{}",
            hits.len(),
            format_violation_list(&hits)
        ),
        severity: Severity::Error,
    }
}

/// Deny-list match: `claude-3-*`, `claude-2<non-digit>`, `gpt-4-turbo`.
fn macs_line_has_denied_model(line: &str) -> bool {
    if line.contains("claude-3-") || line.contains("gpt-4-turbo") {
        return true;
    }
    // claude-2 but not claude-20xx-style future ids: require a non-digit after.
    // Scan EVERY occurrence — a first allowed `claude-2099` must not mask a
    // later denied `claude-2` on the same line (adversarial-review fix).
    let mut rest = line;
    while let Some(idx) = rest.find("claude-2") {
        let after = rest[idx + "claude-2".len()..].chars().next();
        if after.is_none_or(|c| !c.is_ascii_digit()) {
            return true;
        }
        rest = &rest[idx + "claude-2".len()..];
    }
    false
}

/// Recursively invoke `f(path, contents)` for every `*.md` under `dir`,
/// skipping gitignored worktree copies (`.claude/worktrees`).
fn macs_walk_markdown(dir: &Path, f: &mut dyn FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            if name == "worktrees" || name == ".git" {
                continue;
            }
            macs_walk_markdown(&path, f);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(text) = std::fs::read_to_string(&path) {
                f(&path, &text);
            }
        }
    }
}

/// CB-1655: ROADMAP.yaml must be at least as fresh as the ledger (MACS F6).
/// Red when `.pmat-work/ledger.jsonl` is newer than `ROADMAP.yaml` (or the
/// roadmap is absent while a ledger exists) — run `pmat roadmap sync`.
pub(crate) fn check_roadmap_fresh(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1655: Canonical Roadmap Fresh";
    match crate::roadmap::sync::roadmap_is_stale(project_path) {
        None => skip_check(name, "No ledger to measure freshness against"),
        Some(false) => ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: "ROADMAP.yaml is at least as fresh as the ledger".to_string(),
            severity: Severity::Info,
        },
        Some(true) => ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Warn,
            message: "ROADMAP.yaml is stale (ledger is newer) — run `pmat roadmap sync`"
                .to_string(),
            severity: Severity::Warning,
        },
    }
}
