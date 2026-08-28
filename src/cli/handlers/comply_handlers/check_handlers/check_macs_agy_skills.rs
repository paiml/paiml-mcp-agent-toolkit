// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1665: `.agents/skills/*/SKILL.md` frontmatter schema (PMAT-INIT-002 claim 5).
///
/// Requires a `---` fenced frontmatter block carrying every key in
/// [`AGY_REQUIRED_SKILL_KEYS`], and — for `effort` — one of the model-level
/// tiers CB-1650 accepts. Session-only tiers (`max`, `ultracode`) are rejected
/// for the same reason there: they cannot be pinned per-skill, so recording one
/// records a configuration that will not reproduce.
///
/// Every violation names the missing key. "invalid frontmatter" is not
/// actionable; "missing `description`" is.
///
/// Deliberate divergence from CB-1650: a `.agents/skills/` that exists with no
/// SKILL.md in it FAILS rather than skipping. CB-1650 skips on
/// `No skill files under .claude/skills`, which is the exact shape of a Skip
/// that reads like a Pass — the directory is there, the client loads nothing
/// from it, and the report says "Skip".
pub(crate) fn check_agy_skill_frontmatter(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1665: AGY Skill Frontmatter";
    let agents_dir = project_path.join(AGY_DIR);
    if !agents_dir.is_dir() {
        return skip_check(
            name,
            "no .agents/ directory in this project — 0 skills judged",
        );
    }
    let skills_dir = agents_dir.join("skills");
    if !skills_dir.is_dir() {
        return skip_check(
            name,
            ".agents/ is present but has no skills/ directory — 0 skills judged \
             (this project ships no AGY skill)",
        );
    }

    let skill_dirs = agy_skill_dirs(&agents_dir);
    if skill_dirs.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: ".agents/skills/ exists but contains no skill directory — \
                      0 skills judged. The client loads nothing from it"
                .to_string(),
            severity: Severity::Error,
        };
    }

    let mut hard: Vec<String> = Vec::new();
    let soft: Vec<String> = Vec::new();
    let mut judged = 0usize;
    for dir in &skill_dirs {
        let rel_dir = agy_rel(project_path, dir);
        let Some(file) = agy_skill_file(dir) else {
            // CB-1663 owns "skill dir with no SKILL.md"; recorded here too so
            // this check's own denominator stays honest about what it opened.
            hard.push(format!("{rel_dir}/: no SKILL.md to read frontmatter from"));
            continue;
        };
        judged += 1;
        agy_judge_skill_file(project_path, &file, &mut hard);
    }

    if judged == 0 {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "{} skill director(ies) under .agents/skills/ but 0 SKILL.md files to judge:\n{}",
                skill_dirs.len(),
                format_violation_list(&hard)
            ),
            severity: Severity::Error,
        };
    }

    let summary = format!(
        "judged {judged} SKILL.md file(s) under .agents/skills/ against required keys {:?}",
        AGY_REQUIRED_SKILL_KEYS
    );
    agy_verdict(name, &summary, &hard, &soft)
}

fn agy_judge_skill_file(project_path: &Path, file: &Path, hard: &mut Vec<String>) {
    let rel = agy_rel(project_path, file);
    let Ok(text) = std::fs::read_to_string(file) else {
        hard.push(format!("{rel}: unreadable"));
        return;
    };
    let Some(block) = agy_frontmatter_block(&text) else {
        hard.push(format!(
            "{rel}: no `---` fenced YAML frontmatter (required keys {:?} cannot be read)",
            AGY_REQUIRED_SKILL_KEYS
        ));
        return;
    };
    let keys = agy_frontmatter_keys(block);
    for required in AGY_REQUIRED_SKILL_KEYS {
        if !keys.iter().any(|k| k == required) {
            hard.push(format!(
                "{rel}: missing required frontmatter key `{required}` (found {keys:?})"
            ));
        }
    }
    // `effort` is present: judge its value with the same tier list CB-1650 uses.
    if keys.iter().any(|k| k == "effort") {
        match macs_frontmatter_effort(&text).as_deref() {
            Some("low" | "medium" | "high" | "xhigh") => {}
            Some(v @ ("max" | "ultracode")) => hard.push(format!(
                "{rel}: `effort: {v}` is session-only and cannot be pinned per-skill (MACS E1)"
            )),
            Some(other) => hard.push(format!(
                "{rel}: `effort: {other}` not in {{low, medium, high, xhigh}}"
            )),
            None => hard.push(format!("{rel}: `effort` key present but has no value")),
        }
    }
}

/// The text between the leading `---` fence and the next `\n---`, or None when
/// the file carries no frontmatter block.
fn agy_frontmatter_block(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Top-level keys declared in a frontmatter block.
///
/// Indent-0 only, so the continuation lines of a YAML block scalar
/// (`description: |`) are not mistaken for keys of their own — every skill in
/// this repo writes `description` that way.
fn agy_frontmatter_keys(block: &str) -> Vec<String> {
    block
        .lines()
        .filter(|l| !l.starts_with([' ', '\t', '#', '-']))
        .filter_map(|l| l.split_once(':'))
        .map(|(k, _)| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .collect()
}
