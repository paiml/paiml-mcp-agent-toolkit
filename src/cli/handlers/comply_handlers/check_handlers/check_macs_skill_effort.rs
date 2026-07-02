// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1650: every repo skill pins a model-level `effort:` in frontmatter
/// (MACS F4). Session-only values (`max`, `ultracode`) are rejected — they
/// cannot be pinned per-skill by design (spec E1/E4), so allowing them here
/// would record an unreproducible configuration.
pub(crate) fn check_skill_effort_pinned(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1650: Skill Effort Pinned";
    let skills_dir = project_path.join(".claude").join("skills");
    if !skills_dir.exists() {
        return skip_check(name, "No .claude/skills directory");
    }

    let mut checked = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for skill_file in macs_skill_files(&skills_dir) {
        checked += 1;
        let rel = skill_file
            .strip_prefix(project_path)
            .unwrap_or(&skill_file)
            .display()
            .to_string();
        let Ok(text) = std::fs::read_to_string(&skill_file) else {
            violations.push(format!("{rel}: unreadable"));
            continue;
        };
        match macs_frontmatter_effort(&text) {
            None => violations.push(format!("{rel}: no `effort:` in frontmatter")),
            Some(value) => match value.as_str() {
                "low" | "medium" | "high" | "xhigh" => {}
                "max" | "ultracode" => violations.push(format!(
                    "{rel}: effort '{value}' is session-only and cannot be pinned (spec E1)"
                )),
                other => violations.push(format!(
                    "{rel}: effort '{other}' not in {{low, medium, high, xhigh}}"
                )),
            },
        }
    }

    if checked == 0 {
        return skip_check(name, "No skill files under .claude/skills");
    }
    if violations.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Pass,
            message: format!("{checked} skill(s) pin a model-level effort"),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Fail,
        message: format!(
            "{} skill(s) without a valid effort pin (MACS F4):\n{}",
            violations.len(),
            format_violation_list(&violations)
        ),
        severity: Severity::Error,
    }
}

/// All `SKILL.md`/`skill.md` files one level under `.claude/skills/`.
fn macs_skill_files(skills_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(skills_dir) else {
        return files;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        for candidate in ["SKILL.md", "skill.md"] {
            let path = entry.path().join(candidate);
            if path.exists() {
                files.push(path);
                break; // one skill file per dir
            }
        }
    }
    files.sort();
    files
}

/// Extract `effort: <value>` from YAML frontmatter (between the leading
/// `---` fence pair). Returns None when absent.
fn macs_frontmatter_effort(text: &str) -> Option<String> {
    let rest = text.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    for line in rest[..end].lines() {
        if let Some(value) = line.strip_prefix("effort:") {
            return Some(value.trim().to_string());
        }
    }
    None
}
