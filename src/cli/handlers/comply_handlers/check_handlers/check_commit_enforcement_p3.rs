
/// CB-1331: Work Contract YAML Validity
///
/// Validates that active work contracts in .pmat-work/ have valid structure.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_work_contract_validity(project_path: &Path) -> ComplianceCheck {
    let work_dir = project_path.join(".pmat-work");
    if !work_dir.exists() {
        return ComplianceCheck {
            name: "CB-1331: Work Contract Validity".into(),
            status: CheckStatus::Skip,
            message: "No .pmat-work/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut valid = 0usize;
    let mut invalid: Vec<String> = Vec::new();

    let entries = match fs::read_dir(&work_dir) {
        Ok(e) => e,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1331: Work Contract Validity".into(),
                status: CheckStatus::Warn,
                message: "Could not read .pmat-work/".into(),
                severity: Severity::Warning,
            };
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let contract = path.join("contract.json");
        if !contract.exists() {
            let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            invalid.push(format!("{} (missing contract.json)", name));
            continue;
        }
        // Validate JSON structure
        match fs::read_to_string(&contract) {
            Ok(content) => {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
                match parsed {
                    Ok(v) => {
                        // Accept v4 contracts (work_item_id only) and v5 (version + work_item_id)
                        let has_id = v.get("work_item_id").is_some();
                        let has_claims = v.get("claims").is_some()
                            || v.get("require").is_some()
                            || v.get("ensure").is_some()
                            || v.get("falsifiable_claims").is_some();
                        if has_id || has_claims {
                            valid += 1;
                        } else {
                            let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                                .unwrap_or_default();
                            invalid.push(format!("{} (missing work_item_id and claims)", name));
                        }
                    }
                    Err(_) => {
                        let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        invalid.push(format!("{} (invalid JSON)", name));
                    }
                }
            }
            Err(_) => {
                let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                invalid.push(format!("{} (unreadable)", name));
            }
        }
    }

    if invalid.is_empty() {
        ComplianceCheck {
            name: "CB-1331: Work Contract Validity".into(),
            status: CheckStatus::Pass,
            message: format!("{} valid work contract(s)", valid),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1331: Work Contract Validity".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} valid, {} invalid: {}",
                valid,
                invalid.len(),
                invalid.join("; ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1322: SVG Asset Contract
///
/// Validates SVG files for viewBox, accessibility, and reasonable element count.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_svg_contract(project_path: &Path) -> ComplianceCheck {
    let mut svg_count = 0usize;
    let mut issues: Vec<String> = Vec::new();

    // Scan for SVG files in common locations
    let search_dirs = ["assets", "docs", "static", "."];
    for dir_name in &search_dirs {
        let dir = project_path.join(dir_name);
        if !dir.exists() {
            continue;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "svg") || !path.is_file() {
                continue;
            }
            svg_count += 1;
            if let Ok(content) = fs::read_to_string(&path) {
                let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();
                if !content.contains("viewBox") {
                    issues.push(format!("{}: missing viewBox", name));
                }
                if !content.contains("<title") && !content.contains("aria-label") {
                    issues.push(format!("{}: no accessibility (title or aria-label)", name));
                }
            }
        }
    }

    if svg_count == 0 {
        return ComplianceCheck {
            name: "CB-1322: SVG Asset Contract".into(),
            status: CheckStatus::Skip,
            message: "No SVG files found".into(),
            severity: Severity::Info,
        };
    }

    if issues.is_empty() {
        ComplianceCheck {
            name: "CB-1322: SVG Asset Contract".into(),
            status: CheckStatus::Pass,
            message: format!("{} SVG file(s) validated", svg_count),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1322: SVG Asset Contract".into(),
            status: CheckStatus::Warn,
            message: format!("{} issue(s): {}", issues.len(), issues.join(", ")),
            severity: Severity::Warning,
        }
    }
}

/// CB-1324: mdBook Contract
///
/// Validates mdBook SUMMARY.md links if book/ directory exists.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_mdbook_contract(project_path: &Path) -> ComplianceCheck {
    let book_dir = project_path.join("book");
    let summary = book_dir.join("src/SUMMARY.md");

    if !book_dir.exists() || !summary.exists() {
        return ComplianceCheck {
            name: "CB-1324: mdBook Contract".into(),
            status: CheckStatus::Skip,
            message: "No book/src/SUMMARY.md found".into(),
            severity: Severity::Info,
        };
    }

    let content = match fs::read_to_string(&summary) {
        Ok(c) => c,
        Err(_) => {
            return ComplianceCheck {
                name: "CB-1324: mdBook Contract".into(),
                status: CheckStatus::Warn,
                message: "Could not read SUMMARY.md".into(),
                severity: Severity::Warning,
            };
        }
    };

    let mut broken_links: Vec<String> = Vec::new();
    let book_src = book_dir.join("src");

    for line in content.lines() {
        // Extract markdown links: [text](path.md)
        if let Some(start) = line.find("](") {
            if let Some(end) = line[start + 2..].find(')') {
                let link = &line[start + 2..start + 2 + end];
                // Skip external links and anchors
                if link.starts_with("http") || link.starts_with('#') {
                    continue;
                }
                let link_path = book_src.join(link.split('#').next().unwrap_or(link));
                if !link_path.exists() {
                    broken_links.push(link.to_string());
                }
            }
        }
    }

    if broken_links.is_empty() {
        ComplianceCheck {
            name: "CB-1324: mdBook Contract".into(),
            status: CheckStatus::Pass,
            message: "SUMMARY.md links valid".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1324: mdBook Contract".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} broken link(s): {}",
                broken_links.len(),
                broken_links.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}

/// CB-1330: L-Level Ratchet
///
/// Checks that provable-contracts verification levels don't regress.
/// Reads contracts/ YAML for verification_summary.current_level fields
/// and warns if any are below target_level.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_verification_ratchet(project_path: &Path) -> ComplianceCheck {
    let contracts_dir = project_path.join("contracts");
    if !contracts_dir.exists() {
        return ComplianceCheck {
            name: "CB-1330: L-Level Ratchet".into(),
            status: CheckStatus::Skip,
            message: "No contracts/ directory".into(),
            severity: Severity::Info,
        };
    }

    let mut total = 0usize;
    let mut regressions: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&contracts_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().map_or(true, |e| e != "yaml" && e != "yml") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(path) {
            if content.contains("verification_summary") {
                total += 1;
                // Extract target and current levels
                let target = extract_level(&content, "target_level");
                let current = extract_level(&content, "current_level");
                if let (Some(t), Some(c)) = (target, current) {
                    if c < t {
                        let name = path.file_name().map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();
                        regressions.push(format!("{} (L{}→L{}, target L{})", name, t, c, t));
                    }
                }
            }
        }
    }

    if total == 0 {
        ComplianceCheck {
            name: "CB-1330: L-Level Ratchet".into(),
            status: CheckStatus::Skip,
            message: "No contracts with verification_summary".into(),
            severity: Severity::Info,
        }
    } else if regressions.is_empty() {
        ComplianceCheck {
            name: "CB-1330: L-Level Ratchet".into(),
            status: CheckStatus::Pass,
            message: format!("{} contract(s) at or above target level", total),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "CB-1330: L-Level Ratchet".into(),
            status: CheckStatus::Warn,
            message: format!(
                "{} regression(s): {}",
                regressions.len(),
                regressions.join(", ")
            ),
            severity: Severity::Warning,
        }
    }
}
