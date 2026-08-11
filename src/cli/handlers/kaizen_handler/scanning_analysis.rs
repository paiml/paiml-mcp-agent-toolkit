// Kaizen scanning: comply, defects, GitHub issues, and custom score scanners
// Included from scanning.rs — no `use` imports, shares parent scope.

/// Scan for PMAT compliance violations
fn scan_comply(path: &Path) -> Result<Vec<KaizenFinding>> {
    let output = Command::new("pmat")
        .args([
            "comply",
            "check",
            "-p",
            &path.to_string_lossy(),
            "-f",
            "json",
        ])
        .current_dir(path)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()), // pmat not available
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    Ok(comply_findings_from_json(&json))
}

/// Turn a `pmat comply check -f json` report into kaizen findings.
///
/// The report's checks carry `name`, `status` and `severity` — there is no `id`
/// field, and the statuses are capitalised (`Pass`/`Warn`/`Fail`/`Skip`).
/// Reading `id` and comparing the status against lowercase `"pass"`/`"skip"`
/// meant every passing check came back as a HIGH finding called
/// `comply::CB-???`: 154 of them on an empty directory, which kaizen (without
/// `--dry-run`) would then commit and file as GitHub issues.
fn comply_findings_from_json(json: &serde_json::Value) -> Vec<KaizenFinding> {
    let mut findings = Vec::new();

    let Some(checks) = json.get("checks").and_then(|c| c.as_array()) else {
        return findings;
    };

    for check in checks {
        let status = check.get("status").and_then(|s| s.as_str()).unwrap_or("");
        if status.eq_ignore_ascii_case("pass") || status.eq_ignore_ascii_case("skip") {
            continue;
        }

        // A check we cannot name is a check nobody can act on — skip it rather
        // than emit a placeholder rule id.
        let Some(id) = check.get("name").and_then(|s| s.as_str()) else {
            continue;
        };
        let msg = check
            .get("message")
            .and_then(|s| s.as_str())
            .unwrap_or("Compliance violation");

        // Report the severity comply itself assigned instead of calling
        // everything High.
        let severity = match check
            .get("severity")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str()
        {
            "critical" => FindingSeverity::Critical,
            "error" => FindingSeverity::High,
            "info" => FindingSeverity::Low,
            _ => FindingSeverity::Medium,
        };

        findings.push(KaizenFinding {
            source: FindingSource::Comply,
            severity,
            category: format!("comply::{id}"),
            message: msg.to_string(),
            file: None,
            auto_fixable: false,
            agent_fixable: true,
            fix_applied: false,
            agent_prompt: Some(format!(
                "Fix PMAT compliance violation {id}: {msg}. \
                 Run `pmat comply check` after fixing to verify."
            )),
            suspiciousness_score: None,
            crate_name: None,
        });
    }

    findings
}

/// Scan for known defect patterns (batuta bug-hunt: unwrap, panic, unsafe, etc.)
fn scan_defects(path: &Path) -> Result<Vec<KaizenFinding>> {
    let output = Command::new("pmat")
        .args([
            "analyze",
            "defects",
            "-p",
            &path.to_string_lossy(),
            "--format",
            "json",
        ])
        .current_dir(path)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let mut findings = Vec::new();

    let defects = json.get("defects").and_then(|d| d.as_array());
    let Some(defects) = defects else {
        return Ok(findings);
    };

    for defect in defects {
        let id = defect
            .get("id")
            .and_then(|s| s.as_str())
            .unwrap_or("DEFECT-???");
        let name = defect
            .get("name")
            .and_then(|s| s.as_str())
            .unwrap_or("Unknown defect");
        let sev_str = defect
            .get("severity")
            .and_then(|s| s.as_str())
            .unwrap_or("Medium");
        let fix = defect
            .get("fix_recommendation")
            .and_then(|s| s.as_str())
            .unwrap_or("");

        let severity = match sev_str.to_lowercase().as_str() {
            "critical" => FindingSeverity::Critical,
            "high" => FindingSeverity::High,
            "low" => FindingSeverity::Low,
            _ => FindingSeverity::Medium,
        };

        let instances = defect.get("instances").and_then(|i| i.as_array());
        let instance_count = instances.map(|i| i.len()).unwrap_or(0);

        // Create one finding per defect pattern (not per instance) for actionability
        let first_file = instances
            .and_then(|insts| insts.first())
            .and_then(|inst| inst.get("file"))
            .and_then(|f| f.as_str())
            .map(String::from);

        findings.push(KaizenFinding {
            source: FindingSource::Defects,
            severity,
            category: format!("defect::{id}"),
            message: format!("{name} ({instance_count} instances)"),
            file: first_file.clone(),
            auto_fixable: false,
            agent_fixable: true,
            fix_applied: false,
            agent_prompt: Some(format!(
                "Fix defect pattern {id}: {name}. {fix} \
                 There are {instance_count} instances. \
                 Start with file {} and fix all instances. \
                 Run `pmat analyze defects` after fixing to verify.",
                first_file.as_deref().unwrap_or("the project")
            )),
            suspiciousness_score: None,
            crate_name: None,
        });
    }

    Ok(findings)
}

/// Scan for open GitHub issues
fn scan_github_issues(path: &Path) -> Result<Vec<KaizenFinding>> {
    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--json",
            "number,title,labels",
            "--state",
            "open",
            "--limit",
            "20",
        ])
        .current_dir(path)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()), // gh not available
    };

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let issues: Vec<serde_json::Value> = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let mut findings = Vec::new();

    for issue in &issues {
        let number = issue.get("number").and_then(|n| n.as_u64()).unwrap_or(0);
        let title = issue
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Untitled");

        let is_bug = issue
            .get("labels")
            .and_then(|l| l.as_array())
            .map(|labels| {
                labels.iter().any(|l| {
                    l.get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n.to_lowercase().contains("bug"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        let severity = if is_bug {
            FindingSeverity::High
        } else {
            FindingSeverity::Medium
        };

        findings.push(KaizenFinding {
            source: FindingSource::GitHubIssue,
            severity,
            category: format!("github::issue#{number}"),
            message: format!("#{number}: {title}"),
            file: None,
            auto_fixable: false,
            agent_fixable: true,
            fix_applied: false,
            agent_prompt: Some(format!(
                "Fix GitHub issue #{number}: {title}. \
                 Read the issue with `gh issue view {number}` first. \
                 Run tests after fixing."
            )),
            suspiciousness_score: None,
            crate_name: None,
        });
    }

    Ok(findings)
}

/// Scan custom project scores from .pmat.yaml scoring plugins
fn scan_custom_scores(path: &Path) -> Vec<KaizenFinding> {
    let config = match crate::models::comply_config::PmatYamlConfig::load(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    if config.scoring.custom_scores.is_empty() {
        return Vec::new();
    }

    let mut findings = Vec::new();

    for score_def in &config.scoring.custom_scores {
        let output = Command::new("sh")
            .args(["-c", &score_def.command])
            .current_dir(path)
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => continue,
        };

        if !output.status.success() {
            findings.push(KaizenFinding {
                source: FindingSource::Comply,
                severity: FindingSeverity::High,
                category: format!("score::{}", score_def.id),
                message: format!("{}: command failed", score_def.name),
                file: None,
                auto_fixable: false,
                agent_fixable: true,
                fix_applied: false,
                agent_prompt: Some(format!(
                    "Fix failing score check '{}': command `{}` failed. \
                     Investigate and fix the underlying issue.",
                    score_def.name, score_def.command
                )),
                suspiciousness_score: None,
                crate_name: None,
            });
            continue;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Reuse the same score extraction from the comply handler
        let score = extract_score_from_command_output(&stdout);

        if let (Some(actual), Some(min)) = (score, score_def.min_score) {
            if actual < min {
                let severity = match score_def.severity {
                    crate::models::comply_config::CheckSeverity::Critical => {
                        FindingSeverity::Critical
                    }
                    crate::models::comply_config::CheckSeverity::Error => FindingSeverity::High,
                    crate::models::comply_config::CheckSeverity::Warning => FindingSeverity::Medium,
                    crate::models::comply_config::CheckSeverity::Info => FindingSeverity::Low,
                };
                findings.push(KaizenFinding {
                    source: FindingSource::Comply,
                    severity,
                    category: format!("score::{}", score_def.id),
                    message: format!(
                        "{}: score {:.1} below minimum {:.1}",
                        score_def.name, actual, min
                    ),
                    file: None,
                    auto_fixable: false,
                    agent_fixable: true,
                    fix_applied: false,
                    agent_prompt: Some(format!(
                        "Improve '{}' score from {:.1} to at least {:.1}. \
                         The score command is: `{}`",
                        score_def.name, actual, min, score_def.command
                    )),
                    suspiciousness_score: None,
                    crate_name: None,
                });
            }
        }
    }

    findings
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod comply_json_tests {
    use super::*;

    /// Shaped exactly like `pmat comply check -f json` output: capitalised
    /// statuses, a `name` (never an `id`), and a per-check severity.
    fn comply_report() -> serde_json::Value {
        serde_json::json!({
            "checks": [
                {"name": "version-compliance", "status": "Pass",
                 "message": "up to date", "severity": "Info"},
                {"name": "hooks-installed", "status": "Skip",
                 "message": "no .git dir", "severity": "Info"},
                {"name": "CB-125-B binding scope", "status": "Warn",
                 "message": "2 contracts unbound", "severity": "Warning"},
                {"name": "quality-gate", "status": "Fail",
                 "message": "complexity over budget", "severity": "Error"},
            ]
        })
    }

    #[test]
    fn test_comply_findings_skip_passing_and_skipped_checks() {
        // Regression: statuses are capitalised, so the lowercase comparison
        // turned Pass/Skip checks into findings.
        let findings = comply_findings_from_json(&comply_report());
        assert_eq!(findings.len(), 2);
        assert!(findings
            .iter()
            .all(|f| !f.category.contains("version-compliance")));
        assert!(findings
            .iter()
            .all(|f| !f.category.contains("hooks-installed")));
    }

    #[test]
    fn test_comply_findings_use_the_check_name_not_a_placeholder() {
        let findings = comply_findings_from_json(&comply_report());
        assert!(findings.iter().all(|f| !f.category.contains("CB-???")));
        assert!(findings
            .iter()
            .any(|f| f.category == "comply::CB-125-B binding scope"));
    }

    #[test]
    fn test_comply_findings_carry_the_reports_own_severity() {
        let findings = comply_findings_from_json(&comply_report());
        let warn = findings
            .iter()
            .find(|f| f.category.contains("binding scope"))
            .expect("Warn check must produce a finding");
        assert_eq!(warn.severity, FindingSeverity::Medium);
        let fail = findings
            .iter()
            .find(|f| f.category.contains("quality-gate"))
            .expect("Fail check must produce a finding");
        assert_eq!(fail.severity, FindingSeverity::High);
    }

    #[test]
    fn test_comply_findings_drop_unnamed_checks() {
        let json = serde_json::json!({
            "checks": [{"status": "Fail", "message": "boom", "severity": "Error"}]
        });
        assert!(comply_findings_from_json(&json).is_empty());
    }
}
