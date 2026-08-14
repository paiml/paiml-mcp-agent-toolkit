// Annotation builders, scoring functions, and metadata loaders for git history.
//
// Provides: hotspot aggregation, co-change counting, code/dead-code/bug-hunter
// annotations, work ticket loading, commit quality metadata, and decay/risk scores.

// ── O(1) annotation builders ────────────────────────────────────────────────

/// Count pairwise co-changes for files in a single commit
fn count_pairwise_cochanges(
    file_paths: &[&str],
    cochange_counts: &mut HashMap<(String, String), usize>,
) {
    let n = file_paths.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let (a, b) = if file_paths[i] < file_paths[j] {
                (file_paths[i], file_paths[j])
            } else {
                (file_paths[j], file_paths[i])
            };
            *cochange_counts.entry((a.to_string(), b.to_string())).or_insert(0) += 1;
        }
    }
}

/// Build file-level annotations from the code index + cached data
fn aggregate_hotspots(
    commits: &[CommitInfo],
) -> (HashMap<String, FileHotspot>, HashMap<(String, String), usize>) {
    let mut hotspots: HashMap<String, FileHotspot> = HashMap::new();
    let mut cochange_counts: HashMap<(String, String), usize> = HashMap::new();
    for commit in commits {
        for fc in &commit.files {
            let entry = hotspots.entry(fc.path.clone()).or_default();
            entry.commit_count += 1;
            if commit.is_fix { entry.fix_count += 1; }
            if commit.is_feat { entry.feat_count += 1; }
            entry.lines_added += fc.lines_added as u64;
            entry.lines_deleted += fc.lines_deleted as u64;
            *entry.authors.entry(commit.author_name.clone()).or_insert(0) += 1;
        }
        // Skip co-change for commits touching >15 files (merges/refactors are noise)
        let n = commit.files.len();
        if n > 1 && n <= 15 {
            let file_paths: Vec<&str> = commit.files.iter().map(|f| f.path.as_str()).collect();
            count_pairwise_cochanges(&file_paths, &mut cochange_counts);
        }
    }
    (hotspots, cochange_counts)
}

fn build_code_annotations(
    index: &AgentContextIndex,
    hotspots: &HashMap<String, FileHotspot>,
) -> HashMap<String, FileAnnotation> {
    let mut file_annots: HashMap<String, FileAnnotation> = HashMap::new();
    for file_path in hotspots.keys() {
        let funcs = index.get_by_file(file_path);
        if funcs.is_empty() { continue; }
        let annot = annotate_file_functions(index, file_path, &funcs);
        file_annots.insert(file_path.clone(), annot);
    }
    file_annots
}

#[allow(clippy::field_reassign_with_default)]
#[allow(clippy::cast_possible_truncation)]
fn annotate_file_functions(
    index: &AgentContextIndex,
    file_path: &str,
    funcs: &[&crate::services::agent_context::FunctionEntry],
) -> FileAnnotation {
    let mut annot = FileAnnotation::default();
    annot.function_count = funcs.len();
    // tdg_score is 0-100 HIGHER-is-better, so the WORST function in a file is
    // the one with the LOWEST score. This used to take the maximum (correct on
    // the old 0-10 debt scale), which after the flip would have reported each
    // file's BEST function as its grade.
    let mut worst_tdg_score = f32::INFINITY;
    let mut worst_grade = String::new();
    let mut total_complexity: f32 = 0.0;
    let mut max_pr: f32 = 0.0;
    let mut total_faults = 0usize;
    for (i, func) in funcs.iter().enumerate() {
        if func.quality.tdg_score < worst_tdg_score {
            worst_tdg_score = func.quality.tdg_score;
            worst_grade = func.quality.tdg_grade.clone();
        }
        total_complexity += func.quality.complexity as f32;
        total_faults += func.fault_annotations.len();
        if let Some(func_idx) = index.file_index.get(file_path) {
            if i < func_idx.len() && func_idx[i] < index.graph_metrics.len() {
                let pr = index.graph_metrics[func_idx[i]].pagerank;
                if pr > max_pr { max_pr = pr; }
            }
        }
    }
    // No functions means no grade. Reporting "A" for a file nothing was
    // measured in is the "empty means clean" failure, not an annotation.
    annot.tdg_grade = if worst_grade.is_empty() { None } else { Some(worst_grade) };
    annot.avg_complexity = Some(total_complexity / funcs.len() as f32);
    annot.max_pagerank = Some(max_pr);
    annot.fault_count = total_faults;
    annot
}

fn load_dead_code_annotations(
    project_path: &std::path::Path,
    file_annots: &mut HashMap<String, FileAnnotation>,
    hotspots: &mut HashMap<String, FileHotspot>,
) {
    let dead_code_path = project_path.join(".pmat/dead-code-cache.json");
    let data = match std::fs::read_to_string(&dead_code_path) {
        Ok(d) => d,
        Err(_) => return,
    };
    let cache: DeadCodeCache = match serde_json::from_str(&data) {
        Ok(c) => c,
        Err(_) => return,
    };
    for dc_file in &cache.report.files_with_dead_code {
        if let Some(annot) = file_annots.get_mut(&dc_file.file_path) {
            annot.dead_code_count = dc_file.dead_items.len();
            annot.dead_code_pct = dc_file.file_dead_percentage;
        }
        if let Some(hotspot) = hotspots.get_mut(&dc_file.file_path) {
            hotspot.annotation.dead_code_count = dc_file.dead_items.len();
            hotspot.annotation.dead_code_pct = dc_file.file_dead_percentage;
        }
    }
}

fn aggregate_bug_hunter_faults(bug_hunter_dir: &std::path::Path) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let entries = match std::fs::read_dir(bug_hunter_dir) { Ok(e) => e, Err(_) => return counts };
    // Only read the most recent cache file (by mtime) to avoid parsing multiple large JSONs
    let newest = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .max_by_key(|e| e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH));
    let entry = match newest {
        Some(e) => e,
        None => return counts,
    };
    let data = match std::fs::read_to_string(entry.path()) { Ok(d) => d, Err(_) => return counts };
    let cache: BugHunterCache = match serde_json::from_str(&data) { Ok(c) => c, Err(_) => return counts };
    for finding in &cache.findings {
        *counts.entry(finding.file.clone()).or_insert(0) += 1;
    }
    counts
}

fn load_bug_hunter_annotations(
    project_path: &std::path::Path,
    file_annots: &mut HashMap<String, FileAnnotation>,
) {
    let bug_hunter_dir = project_path.join(".pmat/bug-hunter-cache");
    if !bug_hunter_dir.is_dir() { return; }
    let counts = aggregate_bug_hunter_faults(&bug_hunter_dir);
    for (file, count) in &counts {
        if let Some(annot) = file_annots.get_mut(file) {
            if *count > annot.fault_count { annot.fault_count = *count; }
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn compute_cochange_pairs(
    cochange_counts: HashMap<(String, String), usize>,
    hotspots: &HashMap<String, FileHotspot>,
) -> Vec<CoChangePair> {
    let mut pairs: Vec<CoChangePair> = cochange_counts
        .into_iter()
        .filter(|(_, count)| *count >= 3)
        .map(|((a, b), count)| {
            let ca = hotspots.get(&a).map_or(1, |h| h.commit_count);
            let cb = hotspots.get(&b).map_or(1, |h| h.commit_count);
            let union = ca + cb - count;
            let jaccard = if union > 0 { count as f32 / union as f32 } else { 0.0 };
            CoChangePair { file_a: a, file_b: b, count, jaccard }
        })
        .collect();
    pairs.sort_by_key(|b| std::cmp::Reverse(b.count));
    pairs.truncate(5);
    pairs
}

fn build_file_annotations(
    index: &AgentContextIndex,
    project_path: &std::path::Path,
    commits: &[CommitInfo],
) -> (
    HashMap<String, FileHotspot>,
    Vec<CoChangePair>,
    HashMap<String, FileAnnotation>,
) {
    let (mut hotspots, cochange_counts) = aggregate_hotspots(commits);
    let mut file_annots = build_code_annotations(index, &hotspots);
    load_dead_code_annotations(project_path, &mut file_annots, &mut hotspots);
    load_bug_hunter_annotations(project_path, &mut file_annots);
    for (path, hotspot) in hotspots.iter_mut() {
        if let Some(annot) = file_annots.get(path) {
            hotspot.annotation = annot.clone();
        }
    }
    let cochange_pairs = compute_cochange_pairs(cochange_counts, &hotspots);
    (hotspots, cochange_pairs, file_annots)
}

/// Load work ticket info for issue refs
fn load_work_ticket(project_path: &std::path::Path, issue_ref: &str) -> Option<WorkTicketInfo> {
    // Try matching PMAT-### style refs
    let ticket_id = if issue_ref.starts_with("PMAT-") || issue_ref.starts_with("pmat-") {
        issue_ref.to_uppercase()
    } else {
        let stripped = issue_ref.strip_prefix('#')?;
        // Try GH-### format
        format!("PMAT-{}", stripped)
    };

    let contract_path = project_path
        .join(".pmat-work")
        .join(&ticket_id)
        .join("contract.json");

    if !contract_path.exists() {
        return None;
    }

    let data = std::fs::read_to_string(&contract_path).ok()?;
    let contract: serde_json::Value = serde_json::from_str(&data).ok()?;

    let claims = contract.get("claims")?.as_array()?;
    let claims_total = claims.len();
    let claims_passed = claims
        .iter()
        .filter(|c| {
            c.get("result")
                .and_then(|r| r.get("falsified"))
                .and_then(|f| f.as_bool())
                .is_some_and(|f| !f) // passed = not falsified
        })
        .count();

    let baseline_tdg = contract
        .get("baseline_tdg")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    Some(WorkTicketInfo {
        ticket_id,
        claims_passed,
        claims_total,
        baseline_tdg,
    })
}

/// Load commit quality metadata from .pmat-metrics/
fn load_commit_quality(
    project_path: &std::path::Path,
    commit_hash: &str,
) -> Option<CommitQualityMeta> {
    let short_hash = commit_hash.get(..7.min(commit_hash.len())).unwrap_or(commit_hash);
    let meta_path = project_path
        .join(".pmat-metrics")
        .join(format!("commit-{}-meta.json", short_hash));

    if !meta_path.exists() {
        return None;
    }

    let data = std::fs::read_to_string(&meta_path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Risk weight used when a file carries no TDG grade at all.
///
/// Kept at the neutral midpoint deliberately: an unknown grade must not be
/// ranked as if it were measured and clean (0.0) nor as if it were measured
/// and terrible (1.0).
const UNKNOWN_GRADE_RISK: f32 = 0.5;

/// Map a TDG grade letter to a 0-1 risk weight, derived from the SAME band
/// table `crate::tdg::Grade` grades with.
///
/// The old body was a five-arm `match` on `"A".."F"`, which after the
/// eleven-grade unification would have collapsed `A+`, `A-`, `B+`, `B-`, `C+`,
/// `C-` into the "unknown" fallback -- six of eleven grades scoring identically
/// to a file that was never measured.
fn grade_risk(grade: &str) -> f32 {
    match crate::services::agent_context::query::grades::parse_grade(grade) {
        // `score_band().0` is the score floor of the grade's band, so risk
        // falls monotonically as the grade improves: A+ -> 0.05, F -> 1.0.
        Some(g) => 1.0 - (g.score_band().0 / 100.0),
        None => UNKNOWN_GRADE_RISK,
    }
}

/// Compute code decay score for a file
/// decay = (1 - TDG_normalized) x churn_ratio x fix_ratio x (1 + dead_code_fraction)
#[allow(clippy::cast_possible_truncation)]
pub(super) fn compute_decay_score(hotspot: &FileHotspot, total_commits: usize) -> f32 {
    let tdg_score = hotspot
        .annotation
        .tdg_grade
        .as_deref()
        .map_or(UNKNOWN_GRADE_RISK, grade_risk);

    let churn_ratio = if total_commits > 0 {
        (hotspot.commit_count as f32 / total_commits as f32).min(1.0)
    } else {
        0.0
    };

    let fix_ratio = if hotspot.commit_count > 0 {
        hotspot.fix_count as f32 / hotspot.commit_count as f32
    } else {
        0.0
    };

    let dead_factor = 1.0 + (hotspot.annotation.dead_code_pct / 100.0);

    (tdg_score * churn_ratio * (1.0 + fix_ratio) * dead_factor).min(1.0)
}

/// Compute impact x risk score
/// impact_risk = pagerank x churn_ratio x (1 + fault_density)
#[allow(clippy::cast_possible_truncation)]
pub(super) fn compute_impact_risk(hotspot: &FileHotspot, total_commits: usize) -> f32 {
    let pagerank = hotspot.annotation.max_pagerank.unwrap_or(0.0);
    let churn_ratio = if total_commits > 0 {
        hotspot.commit_count as f32 / total_commits as f32
    } else {
        0.0
    };
    let fault_density = hotspot.annotation.fault_count as f32;

    (pagerank * 10000.0 * churn_ratio * (1.0 + fault_density)).min(100.0)
}
