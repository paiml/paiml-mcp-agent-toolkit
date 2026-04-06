#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_quality_gates(paths: &[PathBuf], strict: bool) -> Result<Value> {
    debug_assert!(!paths.is_empty(), "paths must not be empty");
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

    let passed = project_score.average_score >= threshold_score
        && project_score.average_grade >= threshold_grade;

    // Collect violations (files below threshold)
    let violations: Vec<Value> = project_score
        .files
        .iter()
        .filter(|score| score.total < threshold_score)
        .map(|score| {
            json!({
                "file": score.file_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "unknown".to_string()),
                "score": score.total,
                "grade": format!("{:?}", score.grade),
                "issues": score.penalties_applied.iter().map(|p| p.issue.clone()).collect::<Vec<_>>()
            })
        })
        .collect();

    Ok(json!({
        "status": "completed",
        "message": format!(
            "Quality gate check completed ({} mode)",
            if strict { "strict" } else { "standard" }
        ),
        "passed": passed,
        "score": project_score.average_score,
        "grade": format!("{:?}", project_score.average_grade),
        "threshold": threshold_score,
        "files_analyzed": project_score.total_files,
        "violations": violations
    }))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn check_quality_gate_file(file_path: &Path, strict: bool) -> Result<Value> {
    debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
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

    let passed = file_score.total >= threshold_score && file_score.grade >= threshold_grade;

    // Collect violations (penalty details)
    let violations: Vec<Value> = file_score
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

    Ok(json!({
        "status": "completed",
        "message": format!(
            "Quality gate check completed for file ({} mode)",
            if strict { "strict" } else { "standard" }
        ),
        "file": file_path.display().to_string(),
        "passed": passed,
        "score": file_score.total,
        "grade": format!("{:?}", file_score.grade),
        "threshold": threshold_score,
        "violations": violations,
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
    debug_assert!(!paths.is_empty(), "paths must not be empty");
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
        .filter(|s| s.total >= threshold_score && s.grade >= threshold_grade)
        .count();
    let failed_files = project_score.total_files - passed_files;

    // Calculate grade distribution
    let mut grade_distribution = std::collections::HashMap::new();
    for score in &project_score.files {
        *grade_distribution
            .entry(format!("{:?}", score.grade))
            .or_insert(0) += 1;
    }

    Ok(json!({
        "status": "completed",
        "message": "Quality gate summary generated",
        "summary": {
            "total_files": project_score.total_files,
            "passed_files": passed_files,
            "failed_files": failed_files,
            "average_score": project_score.average_score,
            "average_grade": format!("{:?}", project_score.average_grade),
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
    debug_assert!(!paths.is_empty(), "paths must not be empty");
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
                    .map(|(grade, count)| (format!("{:?}", grade), count))
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
    debug_assert!(baseline.exists(), "baseline must exist: {}", baseline.display());
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
