/// Everything a `quality_gate` call measured, and every hole in that
/// measurement.
///
/// ONE implementation of "turn a `paths` list into a score", shared by every
/// entry point of this tool family. There used to be two. `check_quality_gates`
/// looped over every path while `quality_gate_summary` read
/// `let project_path = &paths[0];` and never looked at the rest — the literal
/// R13 defect, still live in the second copy after the first was fixed — and the
/// summary's file branch called `analyze_file` directly, so a `.sh` path HARD
/// ERRORED out of one tool while the other reported it as `not_measured`. One
/// tool family, two answers. Two implementations of one rule agree only by
/// coincidence, and these two had already stopped agreeing, so one of them stops
/// existing here.
struct GateScope {
    /// Files that produced a real TDG score. Deduplicated by path: two entries
    /// in `paths` may name the same file (`[dir, dir/a.rs]`, or the same path
    /// twice), and counting it twice moved the average and inflated
    /// `files_analyzed` for a population that never changed on disk.
    graded: Vec<crate::tdg::TdgScore>,
    /// Every path this gate has NO score for, with the reason. An empty list is
    /// a positive claim of full coverage, so nothing may be absorbed into it
    /// silently — see `measure`.
    ungraded: Vec<(PathBuf, String)>,
}

impl GateScope {
    /// Grade every path the caller named, and name every path that produced no
    /// measurement.
    ///
    /// The second half is the rule that kept getting lost: a path that yields
    /// neither a score nor a named refusal — an empty directory, a directory of
    /// non-source files — used to vanish, and `paths: ["ok.rs", "emptydir"]`
    /// answered `{"passed":true,"score":95.0,"not_measured":[],"files_analyzed":1}`
    /// while `paths: ["emptydir"]` alone answered `passed:false`. Unmeasured is
    /// a distinct state from clean, and the difference cannot depend on whether
    /// some *other* path in the same call happened to grade.
    fn measure(
        analyzer: &crate::tdg::analyzer_simple::TdgAnalyzer,
        paths: &[PathBuf],
    ) -> Result<Self> {
        let mut graded: Vec<crate::tdg::TdgScore> = Vec::new();
        let mut ungraded: Vec<(PathBuf, String)> = Vec::new();

        for path in paths {
            let graded_before = graded.len();
            let ungraded_before = ungraded.len();

            if path.is_file() {
                // `analyze_file` errors on anything outside TDG's language set,
                // so `paths: ["a.sh"]` failed the whole call rather than
                // reporting the verdict the CLI gate reports. A file that
                // *should* grade but does not (unparseable, unreadable) is
                // disclosed exactly like one inside a directory is, rather than
                // aborting the whole call for the other paths.
                match crate::tdg::analyzer_simple::not_gradable_reason(path) {
                    None => match analyzer.analyze_file(path) {
                        Ok(score) => graded.push(score),
                        Err(e) => ungraded.push((path.clone(), e.to_string())),
                    },
                    Some(reason) => ungraded.push((path.clone(), reason)),
                }
            } else {
                // `files_analyzed` counts what was GRADED, so without the
                // refusal list a shrinking denominator is invisible — this tool
                // answered `passed:true, grade:"A", files_analyzed:1` for a
                // 9-file tree whose other 8 files the same build refuses one at
                // a time.
                let (project, refused) = analyzer.analyze_project_reporting_ungraded(path)?;
                graded.extend(project.files);
                ungraded.extend(refused);
            }

            // A path that contributed neither a score nor a named hole IS the
            // hole. Nothing gets absorbed just because a sibling path measured.
            if graded.len() == graded_before && ungraded.len() == ungraded_before {
                ungraded.push((path.clone(), NOTHING_GRADABLE_HERE.to_string()));
            }
        }

        // Two paths can name the same file, and the lists are serialised
        // verbatim in a JSON payload: identical input must serialise
        // identically, and the same file on disk must weigh once.
        let mut seen = std::collections::HashSet::new();
        graded.retain(|score| match score.file_path.as_ref() {
            Some(path) => seen.insert(path.clone()),
            None => true,
        });
        ungraded.sort();
        ungraded.dedup();

        Ok(Self { graded, ungraded })
    }

    /// The paths with no score, in the spelling both tools put on the wire.
    fn ungraded_names(&self) -> Vec<String> {
        self.ungraded
            .iter()
            .map(|(file, _)| file.display().to_string())
            .collect()
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_quality_gates(paths: &[PathBuf], strict: bool) -> Result<Value> {
    use crate::tdg::analyzer_simple::TdgAnalyzer;

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one path must be provided"));
    }

    // Create TDG analyzer
    let analyzer = TdgAnalyzer::new()?;

    // EVERY path the caller handed in, not `paths[0]`, and every path that
    // produced no measurement named rather than absorbed — from the ONE
    // implementation `quality_gate_summary` also uses.
    let scope = GateScope::measure(&analyzer, paths)?;
    let ungraded = &scope.ungraded;

    // The advertised checks, RUN — every one of them, from the same functions
    // `pmat quality-gate --checks all` runs (see `run_gate_suite`). This tool
    // and the CLI gate carry one name and must not be two different checks.
    // This was a call to SATD alone, so over a fixture whose only debt was two
    // markers the CLI answered `{satd: 2, coverage: 1}` and this tool answered
    // `{satd: 2}` — and, because the missing row is coverage's own "was NOT
    // measured" disclosure, the surface that ran two of nine checks was the one
    // reporting `not_measured: []`.
    let suite = crate::cli::analysis_utilities::run_gate_suite_over(paths).await?;
    let suite_findings = gate_suite_findings(&suite);

    let project_score = crate::tdg::ProjectScore::aggregate(scope.graded.clone());

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
    // *does* grade but scored badly still fails, and the suite's findings below
    // are still reported either way.
    violations.extend(suite_findings);

    // Every source file the analyzer REFUSED is a hole in this verdict, and the
    // hole belongs in the payload. `analyze_project` drops those files with a
    // warning on stderr, which an MCP client never sees. One unparseable file is
    // enough to refuse in `--file` mode, so one is enough here: this is the
    // zero-graded rule below, extended to the partial case that actually occurs.
    // A path where nothing could be graded must SAY so, and `GateScope::measure`
    // guarantees it is in this list — including the partial case, where a
    // sibling path DID grade. That guard used to be
    // `total_files == 0 && ungraded.is_empty()`, i.e. it only fired when the
    // WHOLE call measured nothing, so `["ok.rs", "emptydir"]` came back
    // `passed:true, not_measured: []` while `["emptydir"]` alone came back
    // `passed:false`: one question, two answers, decided by what else was in the
    // list. `analyze_project` skips files it cannot read or parse, so a
    // directory whose only source file is `fn main( { let x = ;;;` came back
    // with an empty score set — and before the analyzer refused to grade
    // unparseable Rust, with 90.0/A. Zero graded files is not a measurement of
    // quality, and saying "not measured" in one field and "passed" in another is
    // the contradiction this whole file exists to remove.
    for (file, reason) in ungraded {
        violations.push(json!({
            "check_type": "not_graded",
            "severity": "error",
            "file": file.display().to_string(),
            // The reason is reported VERBATIM. It used to be suffixed with
            // " — this file is not part of the score", which the shared refusals
            // already end with, so the payload said it twice.
            "message": reason,
        }));
    }

    // THE verdict rule, in one place and applied to every finding this gate
    // produces (see `is_verdict_bearing`). This used to be
    // `tdg_passed && satd.is_empty()`, i.e. any finding of any severity decided
    // the verdict: nine unchanged Rust files scored 81.67/B+ and came back
    // `passed:false` because the SATD detector matched the literal text
    // "// TODO" inside a sentence *describing* a requirement, and classified it
    // `severity:"info"`. Advisory findings are still reported in `violations`;
    // they no longer flip a verdict.
    let blocking = violations
        .iter()
        .filter(|v| crate::cli::analysis_utilities::json_is_verdict_bearing(v))
        .count();
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
    let ungraded_names: Vec<String> = scope.ungraded_names();
    // …and the advertised CHECKS that did not run, for the same reason. A path
    // that is a single file cannot answer the five project-wide checks, and this
    // field asserting `[]` beside seven checks that never ran is the defect this
    // whole change removes.
    let mut not_measured: Vec<String> = Vec::new();
    let (score, grade) = if graded {
        not_measured.extend(ungraded_names);
        (
            json!(project_score.average_score),
            // GH #703: this was `format!("{:?}", ..)`, so MCP answered "AMinus"
            // where `pmat tdg --format json` answered "A-" for the same score.
            // One spelling on the wire: `Display`, which `Serialize` now matches.
            json!(project_score.average_grade.map(|g| g.to_string())),
        )
    } else {
        not_measured.push("score".to_string());
        not_measured.push("grade".to_string());
        not_measured.extend(ungraded_names);
        (Value::Null, Value::Null)
    };
    not_measured.extend(suite.not_run_names());

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
        // Which checks produced the verdict, named rather than left to be
        // inferred from an empty `violations` list — "no complexity finding" and
        // "complexity never ran" are not the same fact, and the reason each
        // unrun check did not run travels with it.
        "checks": checks_payload(&suite),
        "threshold": threshold_score,
        "files_analyzed": project_score.total_files,
        // `violations` now legitimately contains rows that did NOT decide the
        // verdict, so the count that DID is stated rather than left to be
        // inferred from `passed:true` next to a non-empty list.
        "blocking_violations": blocking,
        "violations": violations
    }))
}

/// What the gate ran and what it did not, on the wire.
///
/// ONE encoding, shared by both entry points of this tool family — the reason a
/// check did not run is a sentence from
/// `crate::cli::analysis_utilities`, never one re-typed here.
fn checks_payload(suite: &crate::cli::analysis_utilities::GateSuite) -> Value {
    json!({
        "ran": suite.ran,
        "not_run": suite.not_run.iter().map(|unrun| json!({
            "check": unrun.check,
            "path": unrun.path.display().to_string(),
            "reason": unrun.reason,
        })).collect::<Vec<_>>(),
    })
}

/// Said of a path that produced no measurement AND no per-file refusal — an
/// empty directory, or a directory holding nothing this build calls source.
///
/// This is the only sentence this file still writes for itself, because it is
/// the only one about a PATH rather than about a file: every per-file refusal
/// comes from `crate::tdg::analyzer_simple::not_gradable_reason`.
const NOTHING_GRADABLE_HERE: &str =
    "no file under this path could be graded (unreadable, unparseable, or not source) — the score is not a measurement";

/// The gate suite's findings for one path, file or directory, as JSON rows.
///
/// ONE implementation, and it owns nothing but the encoding: which checks run
/// for a file and which for a directory is
/// `crate::cli::analysis_utilities::run_gate_suite`'s answer, i.e. the CLI
/// gate's answer, and the rows are serialised from `QualityViolation` itself
/// rather than re-typed field by field. This used to be `satd_findings`, which
/// ran exactly one of the nine advertised checks; before that it was
/// `satd_violations_for_file`, a THIRD copy of the detector-severity mapping
/// (`Critical|High => "error"`, …) alongside the CLI's two, plus a second
/// message format (`"{category}: {text}"` against the CLI's
/// `"{category}: {text} (at column {column})"`), so one tool family described
/// one finding two ways depending on whether the caller named a file or the
/// directory containing it.
fn gate_suite_findings(suite: &crate::cli::analysis_utilities::GateSuite) -> Vec<Value> {
    suite
        .violations
        .iter()
        .filter_map(|v| serde_json::to_value(v).ok())
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
    // THE skip-or-grade rule, asked of the one function that owns it, and its
    // refusal sentence reported verbatim below. This branch used to invent its
    // own wording ("TDG does not grade .sh"), which is the wrong rule for a
    // `tests/bad.rs` — perfectly gradable Rust, still not graded — and left the
    // MCP gate describing a refusal `pmat tdg` describes differently.
    let not_gradable = crate::tdg::analyzer_simple::not_gradable_reason(file_path);
    let file_score = match &not_gradable {
        None => Some(analyzer.analyze_file(file_path)?),
        Some(_) => None,
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

    // The gate's violations come from the same check suite the CLI gate runs,
    // through the same function `check_quality_gates` uses — the four checks
    // `pmat quality-gate --file` can answer for a single file, with the five
    // project-wide ones disclosed in `not_measured` below rather than skipped in
    // silence.
    let suite = crate::cli::analysis_utilities::run_gate_suite(file_path).await?;
    let mut violations = gate_suite_findings(&suite);
    // Same rule, same wording, same shape as `check_quality_gates`: a hole in
    // the verdict is a row in `violations`, never an unexplained `passed:true`.
    if let Some(reason) = not_gradable {
        violations.push(json!({
            "check_type": "not_graded",
            "severity": "error",
            "file": file_path.display().to_string(),
            "message": reason,
        }));
    }
    // The SAME rule `check_quality_gates` applies, from the same function: this
    // was `violations.is_empty()`, so the two entry points of one tool disagreed
    // about whether an advisory finding is a failure the moment either was
    // changed. `not_graded` is `severity:"error"`, so a file with no grade still
    // cannot pass.
    let blocking = violations
        .iter()
        .filter(|v| crate::cli::analysis_utilities::json_is_verdict_bearing(v))
        .count();
    let passed = tdg_passed && blocking == 0;

    // Same convention as `check_quality_gates` above and `analyze_deep_context`:
    // what was not measured goes on the wire as null plus a `not_measured` list,
    // never as a number a client would read as a measurement.
    let (score, grade, metrics, mut not_measured) = match file_score.as_ref() {
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
            Vec::new(),
        ),
        None => (
            Value::Null,
            Value::Null,
            Value::Null,
            vec![
                "score".to_string(),
                "grade".to_string(),
                "metrics".to_string(),
            ],
        ),
    };
    // The advertised checks this path did not run, named for exactly the reason
    // `score`/`grade`/`metrics` are named above.
    not_measured.extend(suite.not_run_names());

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
        // The same disclosure `check_quality_gates` publishes, from the same
        // encoder: one tool family, one answer to "what did you actually run?".
        "checks": checks_payload(&suite),
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

    // THE SAME implementation `check_quality_gates` uses — not a second copy of
    // the same rule. This function read `let project_path = &paths[0];` and
    // never looked at `paths[1..]`, so `{"paths":["ok.rs","dirty.rs"]}` summarised
    // exactly one of them; and its file branch called `analyze_file` directly,
    // so a `.sh` path was a hard error here and a disclosed `not_measured` row
    // over in `quality_gate`. One tool family cannot hold two answers, and the
    // way to stop it holding two is for there to be only one implementation.
    let scope = GateScope::measure(&analyzer, paths)?;
    let project_score = crate::tdg::ProjectScore::aggregate(scope.graded.clone());
    let ungraded = &scope.ungraded;
    // A file the analyzer refused is a hole in the summary and is named.
    // `analyze_project` throws the refusals away, so `not_measured` here was
    // `ProjectScore::not_measured` — which is only ever non-empty when NOTHING
    // graded. Over a tree of twelve source files with seven graded it read `[]`,
    // i.e. full coverage of a run that covered seven twelfths.
    let mut not_measured = project_score.not_measured.clone();
    not_measured.extend(scope.ungraded_names());

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

    // `paths[0]` is used ONLY as the repository hint for the baseline's own git
    // context — a baseline is stamped with one commit, and the first path is as
    // good a place as any to look for it. It is NOT the analysed population: see
    // below.
    let git_context = GitContext::from_current_dir(&paths[0]).ok();

    // Create new baseline
    let mut baseline = TdgBaseline::new(git_context);

    // Analyze all files in the project
    let analyzer = TdgAnalyzer::new()?;

    // THE SAME implementation the other two entry points use. This was a THIRD
    // copy of the rule — `let project_path = &paths[0];` plus its own is_dir /
    // is_file split — so a baseline taken over `["src", "benches"]` recorded
    // `src` and silently omitted `benches`, and `quality_gate.compare` then
    // reported every file under `benches` as `unchanged` because neither side
    // had ever measured one. A file missing from a baseline is indistinguishable
    // from a file that did not move.
    let scope = GateScope::measure(&analyzer, paths)?;
    for file_score in &scope.graded {
        let Some(file_path) = file_score.file_path.as_ref() else {
            continue;
        };
        let mut complexity_breakdown = HashMap::new();
        complexity_breakdown.insert("structural".to_string(), file_score.structural_complexity);
        complexity_breakdown.insert("semantic".to_string(), file_score.semantic_complexity);
        complexity_breakdown.insert("entropy".to_string(), file_score.entropy_score);

        let entry = BaselineEntry {
            content_hash: blake3::hash(std::fs::read(file_path).unwrap_or_default().as_slice()),
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
            // A baseline is the population a later comparison calls "unchanged",
            // so the paths it has NO entry for have to be named here for the
            // same reason `quality_gate` names them: a file absent from both
            // sides of a diff looks exactly like a file that did not move.
            "not_measured": scope.ungraded_names(),
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
        write(
            &tmp.path().join("src"),
            "main.rs",
            "fn main( { let x = ;;;\n",
        );

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
