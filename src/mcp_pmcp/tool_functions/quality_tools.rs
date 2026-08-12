#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_quality_gates(paths: &[PathBuf], strict: bool) -> Result<Value> {
    use crate::tdg::analyzer_simple::TdgAnalyzer;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    // Create TDG analyzer
    let analyzer = TdgAnalyzer::new()?;

    // EVERY path the caller handed in, not `paths[0]`. `quality_gate` used to
    // read `let project_path = &paths[0]` and drop the rest on the floor:
    // `{"paths": ["ok.rs", "a.sh"]}` answered
    // `{"passed":true,"score":90.0,"grade":"A","not_measured":[],"files_analyzed":1}`
    // — half the input measured, and `not_measured: []` claiming full coverage
    // of the run. A path this gate cannot grade is a HOLE, and every hole is
    // named below with the reason it exists.
    let mut graded = Vec::new();
    let mut ungraded: Vec<(PathBuf, String)> = Vec::new();
    let mut satd: Vec<Value> = Vec::new();

    for path in paths {
        if path.is_file() {
            // Same refusal as the single-file entry point below: `analyze_file`
            // errors on anything outside TDG's language set, so `paths: ["a.sh"]`
            // failed the whole call rather than reporting the verdict the CLI
            // gate reports. A file that *should* grade but does not (unparseable,
            // unreadable) is disclosed exactly like one inside a directory is,
            // rather than aborting the whole call for the other paths.
            if crate::tdg::grades_source(path) {
                match analyzer.analyze_file(path) {
                    Ok(score) => graded.push(score),
                    Err(e) => ungraded.push((path.clone(), e.to_string())),
                }
            } else {
                ungraded.push((path.clone(), not_graded_reason(path, true)));
            }
            // SATD comes from the same detector `pmat quality-gate` runs: this
            // tool and the CLI gate carry one name and must not be two different
            // checks. A TDG-only verdict let a `TODO:` the CLI fails on pass
            // over MCP.
            satd.extend(satd_violations_for_file(path));
        } else {
            // Analyze entire project. The ungraded list is load-bearing: `files_analyzed`
            // counts what was GRADED, so without it a shrinking denominator is invisible
            // — this tool answered `passed:true, grade:"A", files_analyzed:1` for a
            // 9-file tree whose other 8 files the same build refuses one at a time.
            let (project, refused) = analyzer.analyze_project_reporting_ungraded(path)?;
            graded.extend(project.files);
            ungraded.extend(refused);
            satd.extend(
                crate::cli::analysis_utilities::check_satd(path)
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
                    }),
            );
        }
    }

    // Two paths can name the same file, and the lists are serialised verbatim in
    // a JSON payload: identical input must serialise identically.
    ungraded.sort();
    ungraded.dedup();
    let project_score = crate::tdg::ProjectScore::aggregate(graded);

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

    // A language TDG does not grade leaves `tdg_passed` false for want of a
    // score — and that is where it must stay. This used to read
    // `(tdg_passed || ungraded_language)`, which credited the missing score to
    // the file: `quality_gate` over one `a.sh` answered
    // `{"passed":true,"score":null,"grade":null,"not_measured":["score","grade"],
    // "files_analyzed":0}` — a green verdict over input the same payload
    // declares unmeasured, from a gate whose own invariant is spelled out in
    // `unmeasured_cannot_pass_tests.rs`. The identical "nothing was graded"
    // state already returned `passed:false` for a DIRECTORY, so one tool was
    // giving two answers to one question. GH #704 is untouched: a path TDG
    // *does* grade but scored badly still fails, and the SATD findings below
    // are still reported either way.
    violations.extend(satd);

    // Every source file the analyzer REFUSED is a hole in this verdict, and the
    // hole belongs in the payload. `analyze_project` drops those files with a
    // warning on stderr, which an MCP client never sees. One unparseable file is
    // enough to refuse in `--file` mode, so one is enough here: this is the
    // zero-graded rule below, extended to the partial case that actually occurs.
    for (file, reason) in &ungraded {
        violations.push(json!({
            "check_type": "not_graded",
            "severity": "error",
            "file": file.display().to_string(),
            "message": format!("{reason} — this file is not part of the score"),
        }));
    }

    // A path where nothing could be graded must SAY so. `analyze_project` skips
    // files it cannot read or parse, so a directory whose only source file is
    // `fn main( { let x = ;;;` came back with an empty score set — and before
    // the analyzer refused to grade unparseable Rust, with 90.0/A. Zero graded
    // files is not a measurement of quality.
    // A language TDG does not grade is not a *failure* to grade, so it gets its
    // own reason string — but it is still a path this gate has no score for, and
    // it is disclosed and refused exactly like the others. Saying "not measured"
    // in one field and "passed" in another is the contradiction this whole file
    // exists to remove.
    if project_score.total_files == 0 && ungraded.is_empty() {
        for path in paths {
            violations.push(json!({
                "check_type": "not_graded",
                "severity": "error",
                "file": path.display().to_string(),
                "message": not_graded_reason(path, path.is_file() && !crate::tdg::grades_source(path)),
            }));
        }
    }

    // THE verdict rule, in one place and applied to every finding this gate
    // produces (see `is_verdict_bearing`). This used to be
    // `tdg_passed && satd.is_empty()`, i.e. any finding of any severity decided
    // the verdict: nine unchanged Rust files scored 81.67/B+ and came back
    // `passed:false` because the SATD detector matched the literal text
    // "// TODO" inside a sentence *describing* a requirement, and classified it
    // `severity:"info"`. Advisory findings are still reported in `violations`;
    // they no longer flip a verdict.
    let blocking = violations.iter().filter(|v| is_verdict_bearing(v)).count();
    let passed = tdg_passed && blocking == 0;

    // With nothing graded there is no score to report. Reporting 0.0/"F" reads as
    // "measured, and terrible"; `analyze_deep_context` already answers this case
    // with nulls plus a `not_measured` list, and one server must not use two
    // conventions for the same fact.
    let graded = project_score.total_files > 0;
    // `not_measured` is what a reader consults to learn what a verdict does NOT
    // cover, so an empty list is a positive claim of full coverage. It answered
    // `[]` for a run that graded 1 of 9 files — the field asserting the opposite
    // of the fact it exists to disclose. Files that could not be graded are named
    // here for the same reason `score`/`grade` are named when nothing graded at
    // all: never make a reader infer "not measured" from a number that looks fine.
    let ungraded_names: Vec<String> = ungraded
        .iter()
        .map(|(file, _)| file.display().to_string())
        .collect();
    let (score, grade, not_measured) = if graded {
        (
            json!(project_score.average_score),
            // GH #703: this was `format!("{:?}", ..)`, so MCP answered "AMinus"
            // where `pmat tdg --format json` answered "A-" for the same score.
            // One spelling on the wire: `Display`, which `Serialize` now matches.
            json!(project_score.average_grade.map(|g| g.to_string())),
            json!(ungraded_names),
        )
    } else {
        let mut unmeasured = vec!["score".to_string(), "grade".to_string()];
        unmeasured.extend(ungraded_names);
        (Value::Null, Value::Null, json!(unmeasured))
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
        // `violations` now legitimately contains rows that did NOT decide the
        // verdict, so the count that DID is stated rather than left to be
        // inferred from `passed:true` next to a non-empty list.
        "blocking_violations": blocking,
        "violations": violations
    }))
}

/// The severity a finding must NOT have to decide a verdict.
const ADVISORY_SEVERITY: &str = "info";

/// Does this finding decide the pass/fail verdict?
///
/// ONE rule, both `quality_gate` entry points, every check type. `error` and
/// `warning` are actionable and fail the gate; `info` is advisory — reported,
/// never verdict-bearing. Nine unchanged Rust files were failed by a single
/// `severity:"info"` SATD row whose "finding" was the literal text "// TODO"
/// quoted inside a sentence describing a CLI requirement.
///
/// A finding with no severity at all is verdict-bearing: fail closed, because an
/// unclassified finding must not be silently demoted to advice.
fn is_verdict_bearing(violation: &Value) -> bool {
    violation.get("severity").and_then(Value::as_str) != Some(ADVISORY_SEVERITY)
}

/// Why this gate has no grade for `path`, in the reader's terms.
///
/// One sentence, one place: both entry points of `quality_gate` report the same
/// `not_graded` violation, so a client cannot get "unmeasured" from one and
/// "passed" from the other for the same file.
fn not_graded_reason(path: &Path, ungraded_language: bool) -> String {
    if ungraded_language {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map_or_else(|| "this file".to_string(), |e| format!(".{e}"));
        format!(
            "TDG does not grade {ext} — score and grade are not measured here, so this gate has no verdict to give (the language-agnostic checks below still apply)"
        )
    } else {
        "no file under this path could be graded (unreadable, unparseable, or not source) — the score is not a measurement".to_string()
    }
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

    // TDG grades only its own language set and `analyze_file` errors on
    // anything else, which turned this tool into a hard error for the .sh /
    // .md / .toml files `pmat quality-gate --file` reports a verdict on — the
    // same CLI-vs-MCP contradiction the SATD wiring below exists to remove,
    // reintroduced from the other side. A language TDG does not grade is not
    // bad input: the SATD checks still measure the file, so score and grade are
    // reported as not measured instead of as a refusal.
    let file_score = if crate::tdg::grades_source(file_path) {
        Some(analyzer.analyze_file(file_path)?)
    } else {
        None
    };

    // Determine pass/fail threshold based on strict mode
    let threshold_score = if strict { 70.0 } else { 50.0 };
    let threshold_grade = if strict {
        crate::tdg::Grade::B
    } else {
        crate::tdg::Grade::D
    };

    // Grade's derived Ord is inverted (better grades compare as smaller),
    // so use the semantic helper instead of a raw `>=` comparison. This was
    // `is_none_or`, which turned "there is no grade" into "there is no threshold
    // to fail" — the same credit-for-a-missing-measurement that let
    // `check_quality_gates` above answer `passed:true, score:null` for one
    // `a.sh`. No grade is no verdict, and it is reported as a `not_graded`
    // violation below rather than silently passed. GH #704 is untouched: a file
    // TDG *does* grade but scored badly still fails.
    let tdg_passed = file_score.as_ref().is_some_and(|score| {
        score.total >= threshold_score && score.grade.meets_threshold(threshold_grade)
    });

    // TDG penalty attributions are NOT gate violations — they are the score's
    // breakdown. They used to be reported as `violations`, which is why this
    // tool and `pmat quality-gate --file` were two different checks under one
    // name: the CLI failed tiny/src/lib.rs on a `TODO:` at line 4 while MCP
    // returned {"passed":true,"grade":"A","violations":[]} for the same file in
    // the same session that `analyze_satd` flagged that very TODO.
    let tdg_penalties: Vec<Value> = file_score
        .iter()
        .flat_map(|score| score.penalties_applied.iter())
        .map(|p| {
            json!({
                "category": format!("{:?}", p.source_metric),
                "penalty": p.amount,
                "description": p.issue,
            })
        })
        .collect();

    // The gate's violations come from the same SATD detector the CLI gate runs.
    let mut violations = satd_violations_for_file(file_path);
    // Same rule, same wording, same shape as `check_quality_gates`: a hole in
    // the verdict is a row in `violations`, never an unexplained `passed:true`.
    if file_score.is_none() {
        violations.push(json!({
            "check_type": "not_graded",
            "severity": "error",
            "file": file_path.display().to_string(),
            "message": not_graded_reason(file_path, true),
        }));
    }
    // The SAME rule `check_quality_gates` applies, from the same function: this
    // was `violations.is_empty()`, so the two entry points of one tool disagreed
    // about whether an advisory finding is a failure the moment either was
    // changed. `not_graded` is `severity:"error"`, so a file with no grade still
    // cannot pass.
    let blocking = violations.iter().filter(|v| is_verdict_bearing(v)).count();
    let passed = tdg_passed && blocking == 0;

    // Same convention as `check_quality_gates` above and `analyze_deep_context`:
    // what was not measured goes on the wire as null plus a `not_measured` list,
    // never as a number a client would read as a measurement.
    let (score, grade, metrics, not_measured) = match file_score.as_ref() {
        Some(s) => (
            json!(s.total),
            json!(s.grade.to_string()),
            json!({
                "structural_complexity": s.structural_complexity,
                "semantic_complexity": s.semantic_complexity,
                "duplication_ratio": s.duplication_ratio,
                "coupling_score": s.coupling_score,
                "doc_coverage": s.doc_coverage,
                "consistency_score": s.consistency_score,
            }),
            json!([]),
        ),
        None => (
            Value::Null,
            Value::Null,
            Value::Null,
            json!(["score", "grade", "metrics"]),
        ),
    };

    Ok(json!({
        "status": "completed",
        "message": format!(
            "Quality gate check completed for file ({} mode)",
            if strict { "strict" } else { "standard" }
        ),
        "file": file_path.display().to_string(),
        "passed": passed,
        "score": score,
        "grade": grade,
        "not_measured": not_measured,
        "threshold": threshold_score,
        "blocking_violations": blocking,
        "violations": violations,
        "tdg_penalties": tdg_penalties,
        "metrics": metrics
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

    // Same rule as `check_quality_gates`: a file the analyzer refused is a hole
    // in the summary and is named. `analyze_project` throws the refusals away,
    // so `not_measured` here was `ProjectScore::not_measured` — which is only
    // ever non-empty when NOTHING graded. Over a tree of twelve source files
    // with seven graded it read `[]`, i.e. full coverage of a run that covered
    // seven twelfths.
    let (project_score, ungraded) = if project_path.is_file() {
        // Analyze single file and wrap in ProjectScore
        let file_score = analyzer.analyze_file(project_path)?;
        (
            crate::tdg::ProjectScore::aggregate(vec![file_score]),
            Vec::new(),
        )
    } else {
        // Analyze entire project
        analyzer.analyze_project_reporting_ungraded(project_path)?
    };
    let mut not_measured = project_score.not_measured.clone();
    not_measured.extend(ungraded.iter().map(|(file, _)| file.display().to_string()));

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
            "not_measured": not_measured,
            "ungraded_files": ungraded.iter().map(|(file, reason)| json!({
                "file": file.display().to_string(),
                "reason": reason,
            })).collect::<Vec<_>>(),
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
    ///
    /// The marker is `FIXME` — `severity:"error"` — and NOT the `TODO` this
    /// fixture used to carry. `TODO` is classified `severity:"info"`, and an
    /// informational finding no longer decides an MCP verdict (see
    /// `is_verdict_bearing`): nine unchanged Rust files were failed by one
    /// `info` row that had matched the literal text "// TODO" quoted inside a
    /// sentence describing a CLI requirement. The parity this test exists to
    /// pin is "a real SATD finding fails both surfaces", which an
    /// error-severity marker states without depending on the classification of
    /// the weakest one.
    ///
    /// KNOWN REMAINING SPLIT, not fixed here because it is not this crate's MCP
    /// surface: `pmat quality-gate` fails on ANY violation regardless of
    /// severity, so an `info` SATD row still fails the CLI while passing MCP.
    /// The severity rule belongs in one place for both; the CLI half lives in
    /// `src/cli/handlers/quality_gates_handler*.rs`.
    #[tokio::test]
    async fn test_mcp_file_gate_fails_on_the_satd_the_cli_gate_fails_on() {
        let tmp = TempDir::new().expect("tempdir");
        let file = write(
            tmp.path(),
            "lib.rs",
            "/// Adds.\npub fn add(a: i32, b: i32) -> i32 {\n    // FIXME: handle overflow\n    a + b\n}\n",
        );

        let result = check_quality_gate_file(&file, false).await.expect("gate");

        let violations = result["violations"].as_array().expect("violations array");
        assert!(
            violations.iter().any(|v| v["check_type"] == "satd"),
            "the FIXME must surface as a violation: {result}"
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
