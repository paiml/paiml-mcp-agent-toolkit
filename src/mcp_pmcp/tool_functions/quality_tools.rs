#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_quality_gates(paths: &[PathBuf], strict: bool) -> Result<Value> {
    use crate::tdg::analyzer_simple::TdgAnalyzer;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    // Create TDG analyzer
    let analyzer = TdgAnalyzer::new()?;

    // Analyze the first path (typically project root)
    let project_path = &paths[0];

    let project_score = if project_path.is_file() {
        // Analyze single file and wrap in ProjectScore
        let file_score = analyzer.analyze_file(project_path)?;
        crate::tdg::ProjectScore::aggregate(vec![file_score])
    } else {
        // Analyze entire project
        analyzer.analyze_project(project_path)?
    };

    // Determine pass/fail threshold based on strict mode
    let threshold_score = if strict { 70.0 } else { 50.0 };
    let threshold_grade = if strict {
        crate::tdg::Grade::B
    } else {
        crate::tdg::Grade::D
    };

    // Grade's derived Ord is inverted (better grades compare as smaller),
    // so use the semantic helper instead of a raw `>=` comparison.
    // GH #704: both are `Option` — nothing analysed means nothing measured, and
    // a gate that could not measure must not report a pass.
    let tdg_passed = project_score
        .average_score
        .is_some_and(|score| score >= threshold_score)
        && project_score
            .average_grade
            .is_some_and(|grade| grade.meets_threshold(threshold_grade));

    // Collect violations (files below threshold)
    let mut violations: Vec<Value> = project_score
        .files
        .iter()
        .filter(|score| score.total < threshold_score)
        .map(|score| {
            json!({
                "check_type": "tdg",
                "file": score.file_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "unknown".to_string()),
                "score": score.total,
                "grade": score.grade.to_string(),
                "issues": score.penalties_applied.iter().map(|p| p.issue.clone()).collect::<Vec<_>>()
            })
        })
        .collect();

    // SATD comes from the same detector `pmat quality-gate` runs: this tool and
    // the CLI gate carry one name and must not be two different checks. A
    // TDG-only verdict let a `TODO:` the CLI fails on pass over MCP.
    let satd: Vec<Value> = if project_path.is_file() {
        satd_violations_for_file(project_path)
    } else {
        crate::cli::analysis_utilities::check_satd(project_path)
            .await
            .unwrap_or_default()
            .iter()
            .map(|v| {
                json!({
                    "check_type": v.check_type,
                    "severity": v.severity,
                    "file": v.file,
                    "line": v.line,
                    "message": v.message,
                })
            })
            .collect()
    };
    let mut passed = tdg_passed && satd.is_empty();
    violations.extend(satd);

    // A path where nothing could be graded must SAY so. `analyze_project` skips
    // files it cannot read or parse, so a directory whose only source file is
    // `fn main( { let x = ;;;` came back with an empty score set — and before
    // the analyzer refused to grade unparseable Rust, with 90.0/A. Zero graded
    // files is not a measurement of quality.
    if project_score.total_files == 0 {
        passed = false;
        violations.push(json!({
            "check_type": "not_graded",
            "severity": "error",
            "file": project_path.display().to_string(),
            "message": "no file under this path could be graded (unreadable, unparseable, or not source) — the score is not a measurement",
        }));
    }

    // With nothing graded there is no score to report. Reporting 0.0/"F" reads as
    // "measured, and terrible"; `analyze_deep_context` already answers this case
    // with nulls plus a `not_measured` list, and one server must not use two
    // conventions for the same fact.
    let graded = project_score.total_files > 0;
    let (score, grade, not_measured) = if graded {
        (
            json!(project_score.average_score),
            // GH #703: this was `format!("{:?}", ..)`, so MCP answered "AMinus"
            // where `pmat tdg --format json` answered "A-" for the same score.
            // One spelling on the wire: `Display`, which `Serialize` now matches.
            json!(project_score.average_grade.map(|g| g.to_string())),
            json!([]),
        )
    } else {
        (Value::Null, Value::Null, json!(["score", "grade"]))
    };

    Ok(json!({
        "status": "completed",
        "message": format!(
            "Quality gate check completed ({} mode)",
            if strict { "strict" } else { "standard" }
        ),
        "passed": passed,
        "score": score,
        "grade": grade,
        "not_measured": not_measured,
        "threshold": threshold_score,
        "files_analyzed": project_score.total_files,
        "violations": violations
    }))
}

/// SATD violations for one file, from the detector `pmat quality-gate` uses.
///
/// Shape matches the CLI gate's `violations` entries (check_type/severity/
/// file/line/message) so an agent reading either gate sees the same rows.
fn satd_violations_for_file(file_path: &Path) -> Vec<Value> {
    use crate::services::satd_detector::{SATDDetector, Severity};

    let Ok(source) = std::fs::read_to_string(file_path) else {
        return Vec::new();
    };
    let Ok(debts) = SATDDetector::new().extract_from_content(&source, file_path) else {
        return Vec::new();
    };

    debts
        .iter()
        .map(|debt| {
            json!({
                "check_type": "satd",
                "severity": match debt.severity {
                    Severity::Critical | Severity::High => "error",
                    Severity::Medium => "warning",
                    Severity::Low => "info",
                },
                "file": debt.file.display().to_string(),
                "line": debt.line,
                "message": format!("{}: {}", debt.category, debt.text),
            })
        })
        .collect()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_quality_gate_file(file_path: &Path, strict: bool) -> Result<Value> {
    use crate::tdg::analyzer_simple::TdgAnalyzer;

    if !file_path.exists() {
        return Err(anyhow::anyhow!(
            "File does not exist: {}",
            file_path.display()
        ));
    }

    // Create TDG analyzer
    let analyzer = TdgAnalyzer::new()?;

    // Analyze the file
    let file_score = analyzer.analyze_file(file_path)?;

    // Determine pass/fail threshold based on strict mode
    let threshold_score = if strict { 70.0 } else { 50.0 };
    let threshold_grade = if strict {
        crate::tdg::Grade::B
    } else {
        crate::tdg::Grade::D
    };

    // Grade's derived Ord is inverted (better grades compare as smaller),
    // so use the semantic helper instead of a raw `>=` comparison.
    let tdg_passed =
        file_score.total >= threshold_score && file_score.grade.meets_threshold(threshold_grade);

    // TDG penalty attributions are NOT gate violations — they are the score's
    // breakdown. They used to be reported as `violations`, which is why this
    // tool and `pmat quality-gate --file` were two different checks under one
    // name: the CLI failed tiny/src/lib.rs on a `TODO:` at line 4 while MCP
    // returned {"passed":true,"grade":"A","violations":[]} for the same file in
    // the same session that `analyze_satd` flagged that very TODO.
    let tdg_penalties: Vec<Value> = file_score
        .penalties_applied
        .iter()
        .map(|p| {
            json!({
                "category": format!("{:?}", p.source_metric),
                "penalty": p.amount,
                "description": p.issue,
            })
        })
        .collect();

    // The gate's violations come from the same SATD detector the CLI gate runs.
    let violations = satd_violations_for_file(file_path);
    let passed = tdg_passed && violations.is_empty();

    Ok(json!({
        "status": "completed",
        "message": format!(
            "Quality gate check completed for file ({} mode)",
            if strict { "strict" } else { "standard" }
        ),
        "file": file_path.display().to_string(),
        "passed": passed,
        "score": file_score.total,
        "grade": file_score.grade.to_string(),
        "threshold": threshold_score,
        "violations": violations,
        "tdg_penalties": tdg_penalties,
        "metrics": {
            "structural_complexity": file_score.structural_complexity,
            "semantic_complexity": file_score.semantic_complexity,
            "duplication_ratio": file_score.duplication_ratio,
            "coupling_score": file_score.coupling_score,
            "doc_coverage": file_score.doc_coverage,
            "consistency_score": file_score.consistency_score,
        }
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn quality_gate_summary(paths: &[PathBuf]) -> Result<Value> {
    use crate::tdg::analyzer_simple::TdgAnalyzer;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    // Create TDG analyzer
    let analyzer = TdgAnalyzer::new()?;

    // Analyze the first path (typically project root)
    let project_path = &paths[0];

    let project_score = if project_path.is_file() {
        // Analyze single file and wrap in ProjectScore
        let file_score = analyzer.analyze_file(project_path)?;
        crate::tdg::ProjectScore::aggregate(vec![file_score])
    } else {
        // Analyze entire project
        analyzer.analyze_project(project_path)?
    };

    // Standard threshold for summary (not strict)
    let threshold_score = 50.0;
    let threshold_grade = crate::tdg::Grade::D;

    // Count passed/failed files
    let passed_files = project_score
        .files
        .iter()
        .filter(|s| s.total >= threshold_score && s.grade.meets_threshold(threshold_grade))
        .count();
    let failed_files = project_score.total_files - passed_files;

    // Calculate grade distribution
    let mut grade_distribution = std::collections::HashMap::new();
    for score in &project_score.files {
        *grade_distribution
            .entry(score.grade.to_string())
            .or_insert(0) += 1;
    }

    Ok(json!({
        "status": "completed",
        "message": "Quality gate summary generated",
        "summary": {
            "total_files": project_score.total_files,
            "passed_files": passed_files,
            "failed_files": failed_files,
            // GH #704: an unmeasured aggregate is null + `not_measured`,
            // the same convention `quality_gate` above already uses.
            "average_score": project_score.average_score,
            "average_grade": project_score.average_grade.map(|g| g.to_string()),
            "not_measured": project_score.not_measured,
            "threshold_score": threshold_score,
            "grade_distribution": grade_distribution,
            "language_distribution": project_score.language_distribution.iter()
                .map(|(lang, count)| (format!("{:?}", lang), count))
                .collect::<std::collections::HashMap<_, _>>()
        }
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn quality_gate_baseline(paths: &[PathBuf], output: Option<&Path>) -> Result<Value> {
    use crate::models::git_context::GitContext;
    use crate::tdg::analyzer_simple::TdgAnalyzer;
    use crate::tdg::baseline::{BaselineEntry, TdgBaseline};
    use crate::tdg::storage::ComponentScores;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    let project_path = &paths[0];

    // Try to get git context (optional)
    let git_context = GitContext::from_current_dir(project_path).ok();

    // Create new baseline
    let mut baseline = TdgBaseline::new(git_context);

    // Analyze all files in the project
    let analyzer = TdgAnalyzer::new()?;

    // If it's a directory, analyze the project
    if project_path.is_dir() {
        let project_score = analyzer.analyze_project(project_path)?;

        // Add each file to baseline
        for file_score in &project_score.files {
            if let Some(file_path) = &file_score.file_path {
                // Create baseline entry
                let mut complexity_breakdown = HashMap::new();
                complexity_breakdown
                    .insert("structural".to_string(), file_score.structural_complexity);
                complexity_breakdown.insert("semantic".to_string(), file_score.semantic_complexity);
                complexity_breakdown.insert("entropy".to_string(), file_score.entropy_score);

                let entry = BaselineEntry {
                    content_hash: blake3::hash(
                        std::fs::read(file_path).unwrap_or_default().as_slice(),
                    ),
                    score: file_score.clone(),
                    components: ComponentScores {
                        complexity_breakdown,
                        duplication_sources: Vec::new(),
                        coupling_dependencies: Vec::new(),
                        doc_missing_items: Vec::new(),
                        consistency_violations: Vec::new(),
                    },
                    git_context: GitContext::from_current_dir(file_path).ok(),
                };

                baseline.add_entry(file_path.clone(), entry);
            }
        }
    } else if project_path.is_file() {
        // Analyze single file
        let file_score = analyzer.analyze_file(project_path)?;

        let mut complexity_breakdown = HashMap::new();
        complexity_breakdown.insert("structural".to_string(), file_score.structural_complexity);
        complexity_breakdown.insert("semantic".to_string(), file_score.semantic_complexity);
        complexity_breakdown.insert("entropy".to_string(), file_score.entropy_score);

        let entry = BaselineEntry {
            content_hash: blake3::hash(std::fs::read(project_path).unwrap_or_default().as_slice()),
            score: file_score.clone(),
            components: ComponentScores {
                complexity_breakdown,
                duplication_sources: Vec::new(),
                coupling_dependencies: Vec::new(),
                doc_missing_items: Vec::new(),
                consistency_violations: Vec::new(),
            },
            git_context: GitContext::from_current_dir(project_path).ok(),
        };

        baseline.add_entry(project_path.clone(), entry);
    }

    // Save baseline to file if output path provided
    let file_path = if let Some(output_path) = output {
        baseline.save(output_path)?;
        output_path.display().to_string()
    } else {
        // Default to system scratch location
        let temp_path = std::env::temp_dir().join("pmat_baseline.json");
        baseline.save(&temp_path)?;
        temp_path.display().to_string()
    };

    Ok(json!({
        "status": "completed",
        "message": "Quality gate baseline created successfully",
        "baseline": {
            "file_path": file_path,
            "timestamp": baseline.created_at.to_rfc3339(),
            "summary": {
                "total_files": baseline.summary.total_files,
                "avg_score": baseline.summary.avg_score,
                "grade_distribution": baseline.summary.grade_distribution.iter()
                    .map(|(grade, count)| (grade.to_string(), count))
                    .collect::<HashMap<_, _>>(),
                "languages": baseline.summary.languages.clone(),
            },
            "git_context": baseline.git_context.as_ref().map(|ctx| json!({
                "commit_sha": ctx.commit_sha_short.clone(),
                "branch": ctx.branch.clone(),
                "is_clean": ctx.is_clean,
            })),
        }
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn quality_gate_compare(baseline: &Path, paths: &[PathBuf]) -> Result<Value> {
    use crate::tdg::baseline::TdgBaseline;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    if !baseline.exists() {
        return Err(anyhow::anyhow!(
            "Baseline file not found: {}",
            baseline.display()
        ));
    }

    // Load existing baseline
    let old_baseline = TdgBaseline::load(baseline)?;

    // Create new baseline from current state
    let temp_new_baseline = std::env::temp_dir().join("pmat_baseline_new.json");
    quality_gate_baseline(paths, Some(&temp_new_baseline)).await?;
    let new_baseline = TdgBaseline::load(&temp_new_baseline)?;

    // Compare baselines
    let comparison = old_baseline.compare(&new_baseline);

    Ok(json!({
        "status": "completed",
        "message": "Quality gate comparison completed successfully",
        "comparison": {
            "improved": comparison.improved.len(),
            "regressed": comparison.regressed.len(),
            "unchanged": comparison.unchanged.len(),
            "added": comparison.added.len(),
            "removed": comparison.removed.len(),
            "improved_files": comparison.improved.iter().take(5).map(|fc| json!({
                "path": fc.path.display().to_string(),
                "old_score": fc.old_score.total,
                "new_score": fc.new_score.total,
                "delta": fc.delta,
            })).collect::<Vec<_>>(),
            "regressed_files": comparison.regressed.iter().take(5).map(|fc| json!({
                "path": fc.path.display().to_string(),
                "old_score": fc.old_score.total,
                "new_score": fc.new_score.total,
                "delta": fc.delta,
            })).collect::<Vec<_>>(),
            "has_regressions": !comparison.regressed.is_empty(),
            "total_changes": comparison.improved.len() + comparison.regressed.len() + comparison.added.len() + comparison.removed.len(),
        }
    }))
}

/// The MCP `quality_gate` tool and `pmat quality-gate` carry one name, so they
/// must reach one verdict — and neither may grade a file that did not parse.
#[cfg(test)]
mod mcp_quality_gate_parity_tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, body).expect("write fixture");
        p
    }

    /// The CLI gate fails this file on SATD; MCP used to return
    /// `{"passed":true,"grade":"A","violations":[]}` for it.
    #[tokio::test]
    async fn test_mcp_file_gate_fails_on_the_satd_the_cli_gate_fails_on() {
        let tmp = TempDir::new().expect("tempdir");
        let file = write(
            tmp.path(),
            "lib.rs",
            "/// Adds.\npub fn add(a: i32, b: i32) -> i32 {\n    // TODO: handle overflow\n    a + b\n}\n",
        );

        let result = check_quality_gate_file(&file, false).await.expect("gate");

        let violations = result["violations"].as_array().expect("violations array");
        assert!(
            violations.iter().any(|v| v["check_type"] == "satd"),
            "the TODO must surface as a violation: {result}"
        );
        assert_eq!(
            result["passed"], false,
            "a file the CLI gate fails must not pass the MCP gate: {result}"
        );
        // The TDG breakdown is still reported, just not as gate violations.
        assert!(result["tdg_penalties"].is_array(), "{result}");
    }

    /// A clean file must still pass — the gate is not fail-by-default.
    #[tokio::test]
    async fn test_mcp_file_gate_still_passes_a_clean_file() {
        let tmp = TempDir::new().expect("tempdir");
        let file = write(
            tmp.path(),
            "clean.rs",
            "/// Adds two numbers.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
        );

        let result = check_quality_gate_file(&file, false).await.expect("gate");
        assert_eq!(result["passed"], true, "{result}");
        assert!(result["violations"].as_array().expect("array").is_empty());
    }

    /// `fn main( { let x = ;;;` scored 90.0/A with `violations: []` in strict
    /// mode: the heuristic scorer never parsed, so a file rustc rejects earned
    /// every untouched component cap.
    #[tokio::test]
    async fn test_unparseable_source_is_not_an_a_grade_pass() {
        let tmp = TempDir::new().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join("src")).expect("mkdir");
        write(&tmp.path().join("src"), "main.rs", "fn main( { let x = ;;;\n");

        let result = check_quality_gates(&[tmp.path().to_path_buf()], true)
            .await
            .expect("gate");

        assert_eq!(
            result["passed"], false,
            "a project whose only source file does not parse cannot pass: {result}"
        );
        let violations = result["violations"].as_array().expect("violations array");
        assert!(
            !violations.is_empty(),
            "the payload must say why nothing was graded: {result}"
        );
    }

    /// Nothing graded ⇒ nothing to report. `analyze_deep_context` answers this
    /// case with nulls + `not_measured`; the gate used to answer 0.0/"F", which
    /// reads as a measured verdict.
    #[tokio::test]
    async fn test_ungraded_path_reports_not_measured_not_zero_f() {
        let tmp = TempDir::new().expect("tempdir");
        let result = check_quality_gates(&[tmp.path().to_path_buf()], false)
            .await
            .expect("empty dir is not an error");
        assert_eq!(result["files_analyzed"], 0);
        assert!(
            result["score"].is_null(),
            "score must be null when nothing was graded: {result}"
        );
        assert!(
            result["grade"].is_null(),
            "grade must be null when nothing was graded: {result}"
        );
        let not_measured = result["not_measured"]
            .as_array()
            .expect("not_measured array");
        assert!(not_measured.iter().any(|v| v == "score"));
        assert!(not_measured.iter().any(|v| v == "grade"));
    }

    /// The single-file entry point must refuse outright rather than score.
    #[tokio::test]
    async fn test_unparseable_file_is_an_error_not_a_score() {
        let tmp = TempDir::new().expect("tempdir");
        let file = write(tmp.path(), "broken.rs", "fn main( { let x = ;;;\n");
        let err = check_quality_gate_file(&file, false)
            .await
            .expect_err("a file that does not parse must not be graded");
        assert!(err.to_string().contains("not parseable"), "{err}");
    }
}
