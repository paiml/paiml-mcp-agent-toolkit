//! Git history: annotation builders, formatters, and log parsing.

use super::options::*;
use crate::services::agent_context::AgentContextIndex;
use crate::services::git_history::{
    ChangeType, CommitInfo, FileChange, GitSearchResult,
};
use std::collections::HashMap;

/// Timing breakdown for git history search phases
pub(super) struct GitHistoryProfile {
    pub(super) git_log_ms: u128,
    pub(super) parse_ms: u128,
    pub(super) index_ms: u128,
    pub(super) search_ms: u128,
    pub(super) annotate_ms: u128,
    pub(super) total_ms: u128,
    pub(super) commit_count: usize,
}

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
    let mut worst_tdg_score: f32 = 0.0;
    let mut worst_grade = String::from("A");
    let mut total_complexity: f32 = 0.0;
    let mut max_pr: f32 = 0.0;
    let mut total_faults = 0usize;
    for (i, func) in funcs.iter().enumerate() {
        if func.quality.tdg_score > worst_tdg_score {
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
    annot.tdg_grade = Some(worst_grade);
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
    pairs.sort_by(|a, b| b.count.cmp(&a.count));
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
    } else if let Some(stripped) = issue_ref.strip_prefix('#') {
        // Try GH-### format
        format!("PMAT-{}", stripped)
    } else {
        return None;
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

/// Compute code decay score for a file
/// decay = (1 - TDG_normalized) x churn_ratio x fix_ratio x (1 + dead_code_fraction)
#[allow(clippy::cast_possible_truncation)]
pub(super) fn compute_decay_score(hotspot: &FileHotspot, total_commits: usize) -> f32 {
    let tdg_score = hotspot
        .annotation
        .tdg_grade
        .as_ref()
        .map(|g| match g.as_str() {
            "A" => 0.0,
            "B" => 0.25,
            "C" => 0.5,
            "D" => 0.75,
            "F" => 1.0,
            _ => 0.5,
        })
        .unwrap_or(0.5);

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

// ── Colorized output formatter ──────────────────────────────────────────────

/// Format git history results with ANSI colors and O(1) quality annotations
pub(super) fn format_git_history_colorized(
    hits: &[GitSearchResult],
    project_path: &std::path::Path,
    index: &AgentContextIndex,
    all_commits: &[CommitInfo],
) -> String {
    let mut out = String::new();
    let total_commits = all_commits.len();
    let (hotspots, cochange_pairs, _file_annots) =
        build_file_annotations(index, project_path, all_commits);

    out.push_str(&format!(
        "\n{BOLD}{UNDERLINE}Git History (RRF-fused){RESET}\n\n"
    ));

    for (i, hit) in hits.iter().enumerate() {
        format_commit_entry(&mut out, i, hit, &hotspots, project_path, total_commits);
    }

    if !hotspots.is_empty() {
        let mut sorted_hotspots: Vec<(&String, &FileHotspot)> = hotspots.iter().collect();
        sorted_hotspots.sort_by(|a, b| b.1.commit_count.cmp(&a.1.commit_count));

        format_hotspot_section(&mut out, &hotspots, total_commits);
        format_defect_introductions(&mut out, all_commits);
        format_churn_velocity(&mut out, &sorted_hotspots, all_commits);
        format_cochange_section(&mut out, &cochange_pairs);
    }

    out
}

// ── format_git_history_colorized extracted helpers ───────────────────────────

/// Format a single commit entry in the git history output
fn format_commit_entry(
    out: &mut String,
    i: usize,
    hit: &GitSearchResult,
    hotspots: &HashMap<String, FileHotspot>,
    project_path: &std::path::Path,
    total_commits: usize,
) {
    let commit = &hit.commit;
    let short_hash = commit.hash.get(..7.min(commit.hash.len())).unwrap_or(&commit.hash);
    let (type_color, type_tag) = classify_commit_type(&commit.message_subject);
    let score_color = if hit.relevance_score > 0.7 {
        BRIGHT_GREEN
    } else if hit.relevance_score > 0.3 {
        GREEN
    } else {
        DIM
    };

    out.push_str(&format!(
        "  {DIM}{}.{RESET} {YELLOW}{}{RESET} {type_color}{type_tag}{RESET} {WHITE}{}{RESET} {score_color}({:.3}){RESET}\n",
        i + 1, short_hash, commit.message_subject, hit.relevance_score,
    ));

    format_commit_metadata(out, commit, project_path);

    if !hit.files.is_empty() {
        format_commit_files(out, &hit.files, hotspots, total_commits);
    }

    if let Some(ref body) = commit.message_body {
        if !body.is_empty() {
            let truncated = if body.len() > 120 {
                #[allow(clippy::incompatible_msrv)]
                { format!("{}...", body.get(..body.floor_char_boundary(120)).unwrap_or(body)) }
            } else {
                body.clone()
            };
            out.push_str(&format!("     {DIM}{}{RESET}\n", truncated));
        }
    }
}

/// Format commit metadata line: author, date, issue refs, tickets, quality
fn format_commit_metadata(
    out: &mut String,
    commit: &CommitInfo,
    project_path: &std::path::Path,
) {
    let date = format_timestamp(commit.timestamp);
    out.push_str(&format!(
        "     {CYAN}{}{RESET} {DIM}{}{RESET}",
        commit.author_name, date,
    ));

    if !commit.issue_refs.is_empty() {
        out.push_str(&format!(" {YELLOW}{}{RESET}", commit.issue_refs.join(" ")));
    }

    for issue_ref in &commit.issue_refs {
        if let Some(ticket) = load_work_ticket(project_path, issue_ref) {
            let ticket_color = if ticket.claims_passed == ticket.claims_total { GREEN } else { YELLOW };
            out.push_str(&format!(
                " {ticket_color}[{}: {}/{} claims]{RESET}",
                ticket.ticket_id, ticket.claims_passed, ticket.claims_total,
            ));
        }
    }

    if let Some(meta) = load_commit_quality(project_path, &commit.hash) {
        let tdg_color = if meta.tdg_score >= 80.0 { GREEN } else if meta.tdg_score >= 60.0 { YELLOW } else { RED };
        out.push_str(&format!(" {tdg_color}TDG:{:.0}{RESET}", meta.tdg_score));
        if let Some(rs) = meta.rust_project_score {
            out.push_str(&format!(" {DIM}RS:{:.0}{RESET}", rs));
        }
    }

    out.push('\n');
}

/// Format file list for a commit with quality annotations
fn format_commit_files(
    out: &mut String,
    files: &[String],
    hotspots: &HashMap<String, FileHotspot>,
    total_commits: usize,
) {
    out.push_str("     ");
    for (fi, file_path) in files.iter().enumerate() {
        if fi > 0 { out.push_str(", "); }
        if let Some(hotspot) = hotspots.get(file_path.as_str()) {
            format_annotated_file(out, file_path, hotspot, total_commits);
        } else {
            out.push_str(&format!("{DIM_CYAN}{}{RESET}", file_path));
        }
    }
    out.push('\n');
}

/// Format a single file path with quality annotations from hotspot data
#[allow(clippy::cast_possible_truncation)]
fn format_annotated_file(out: &mut String, file_path: &str, hotspot: &FileHotspot, total_commits: usize) {
    let grade = hotspot.annotation.tdg_grade.as_deref().unwrap_or("?");
    let grade_color = grade_to_color(grade);
    out.push_str(&format!("{DIM_CYAN}{}{RESET} {grade_color}[{grade}]{RESET}", file_path));

    if hotspot.fix_count > 2 {
        let fix_pct = if total_commits > 0 { (hotspot.fix_count as f32 / total_commits as f32 * 100.0) as u32 } else { 0 };
        out.push_str(&format!("{RED}({} fixes, {}%){RESET}", hotspot.fix_count, fix_pct));
    }
    if hotspot.annotation.dead_code_count > 0 {
        out.push_str(&format!(" {DIM}dead:{}{RESET}", hotspot.annotation.dead_code_count));
    }
    if hotspot.annotation.fault_count > 0 {
        out.push_str(&format!(" {MAGENTA}faults:{}{RESET}", hotspot.annotation.fault_count));
    }
}

/// Map TDG grade letter to ANSI color code
fn grade_to_color(grade: &str) -> &'static str {
    match grade {
        "A" | "B" => GREEN,
        "C" => YELLOW,
        "D" => RED,
        "F" => BRIGHT_RED,
        _ => DIM,
    }
}

/// Format the hotspot section showing top changed files
fn format_hotspot_section(out: &mut String, hotspots: &HashMap<String, FileHotspot>, total_commits: usize) {
    let mut sorted: Vec<(&String, &FileHotspot)> = hotspots.iter().collect();
    sorted.sort_by(|a, b| b.1.commit_count.cmp(&a.1.commit_count));

    out.push_str(&format!(
        "\n  {BOLD}{UNDERLINE}Hotspots{RESET} {DIM}(top changed files across {} commits){RESET}\n",
        total_commits
    ));
    for (path, hotspot) in sorted.iter().take(8) {
        format_hotspot_entry(out, path, hotspot, total_commits);
    }
}

/// Format a single hotspot entry
#[allow(clippy::cast_possible_truncation)]
fn format_hotspot_entry(out: &mut String, path: &str, hotspot: &FileHotspot, total_commits: usize) {
    let pct = if total_commits > 0 { hotspot.commit_count as f32 / total_commits as f32 * 100.0 } else { 0.0 };
    let churn_color = if pct > 30.0 { BRIGHT_RED } else if pct > 15.0 { RED } else if pct > 5.0 { YELLOW } else { DIM };
    let grade = hotspot.annotation.tdg_grade.as_deref().unwrap_or("-");
    let grade_color = grade_to_color(grade);

    let fix_indicator = format_fix_indicator(hotspot);
    let decay = compute_decay_score(hotspot, total_commits);
    let decay_indicator = format_decay_indicator(decay);
    let impact_risk = compute_impact_risk(hotspot, total_commits);
    let risk_indicator = format_risk_indicator(impact_risk);
    let top_author = format_top_author(hotspot);

    out.push_str(&format!(
        "    {DIM_CYAN}{:<50}{RESET} {churn_color}{:>3} commits ({:>4.1}%){RESET} {grade_color}[{grade}]{RESET}{fix_indicator}{decay_indicator}{risk_indicator}{top_author}\n",
        path, hotspot.commit_count, pct,
    ));
}

#[allow(clippy::cast_possible_truncation)]
fn format_fix_indicator(hotspot: &FileHotspot) -> String {
    let fix_ratio = if hotspot.commit_count > 0 { hotspot.fix_count as f32 / hotspot.commit_count as f32 } else { 0.0 };
    if fix_ratio > 0.5 { format!(" {BRIGHT_RED}!!{} fixes{RESET}", hotspot.fix_count) }
    else if hotspot.fix_count > 0 { format!(" {RED}{} fixes{RESET}", hotspot.fix_count) }
    else { String::new() }
}

fn format_decay_indicator(decay: f32) -> String {
    if decay > 0.5 { format!(" {BRIGHT_RED}decay:{:.2}{RESET}", decay) }
    else if decay > 0.2 { format!(" {YELLOW}decay:{:.2}{RESET}", decay) }
    else { String::new() }
}

fn format_risk_indicator(impact_risk: f32) -> String {
    if impact_risk > 10.0 { format!(" {BRIGHT_RED}risk:{:.1}{RESET}", impact_risk) }
    else if impact_risk > 1.0 { format!(" {YELLOW}risk:{:.1}{RESET}", impact_risk) }
    else { String::new() }
}

#[allow(clippy::cast_possible_truncation)]
fn format_top_author(hotspot: &FileHotspot) -> String {
    hotspot.authors.iter().max_by_key(|(_, count)| *count)
        .map(|(name, count)| {
            let pct = *count as f32 / hotspot.commit_count as f32 * 100.0;
            format!(" {CYAN}{}:{:.0}%{RESET}", name, pct)
        })
        .unwrap_or_default()
}

/// Format defect introduction tracking section
fn format_defect_introductions(out: &mut String, all_commits: &[CommitInfo]) {
    let feat_commits: Vec<&CommitInfo> = all_commits.iter().filter(|c| c.is_feat).collect();
    let mut defect_introductions: Vec<(String, String, usize)> = Vec::new();

    for feat in &feat_commits {
        let feat_ts = feat.timestamp;
        let thirty_days = 30 * 24 * 3600;
        let feat_files: std::collections::HashSet<&str> =
            feat.files.iter().map(|f| f.path.as_str()).collect();
        let fix_count: usize = all_commits.iter()
            .filter(|c| c.is_fix && c.timestamp > feat_ts && c.timestamp < feat_ts + thirty_days
                && c.files.iter().any(|f| feat_files.contains(f.path.as_str())))
            .count();
        if fix_count > 0 {
            let files_str = feat.files.iter().take(3).map(|f| f.path.clone()).collect::<Vec<_>>().join(", ");
            defect_introductions.push((feat.hash.get(..7).unwrap_or(&feat.hash).to_string(), files_str, fix_count));
        }
    }

    if !defect_introductions.is_empty() {
        defect_introductions.sort_by(|a, b| b.2.cmp(&a.2));
        out.push_str(&format!(
            "\n  {BOLD}{UNDERLINE}Defect Introduction{RESET} {DIM}(feat commits patched within 30 days){RESET}\n"
        ));
        for (hash, files, fix_count) in defect_introductions.iter().take(5) {
            out.push_str(&format!(
                "    {YELLOW}{}{RESET} {DIM_CYAN}{}{RESET} {RED}{} fixes within 30d{RESET}\n",
                hash, files, fix_count,
            ));
        }
    }
}

/// Format churn velocity section
#[allow(clippy::cast_possible_truncation)]
fn format_churn_velocity(out: &mut String, sorted_hotspots: &[(&String, &FileHotspot)], all_commits: &[CommitInfo]) {
    let (newest, oldest) = match (
        all_commits.iter().map(|c| c.timestamp).max(),
        all_commits.iter().map(|c| c.timestamp).min(),
    ) {
        (Some(n), Some(o)) => (n, o),
        _ => return,
    };
    let span_weeks = ((newest - oldest) as f32 / (7.0 * 24.0 * 3600.0)).max(1.0);
    let mut velocity_files: Vec<(&str, f32)> = sorted_hotspots.iter().take(5)
        .map(|(path, h)| (path.as_str(), h.commit_count as f32 / span_weeks))
        .filter(|(_, v)| *v > 0.5)
        .collect();
    velocity_files.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    if velocity_files.is_empty() { return; }
    out.push_str(&format!(
        "\n  {BOLD}{UNDERLINE}Churn Velocity{RESET} {DIM}(commits/week over {:.0} weeks){RESET}\n",
        span_weeks
    ));
    for (path, vel) in velocity_files.iter().take(5) {
        let vel_color = if *vel > 3.0 { BRIGHT_RED } else if *vel > 1.0 { YELLOW } else { DIM };
        out.push_str(&format!("    {DIM_CYAN}{:<50}{RESET} {vel_color}{:.1}/wk{RESET}\n", path, vel));
    }
}

/// Format co-change coupling section
fn format_cochange_section(out: &mut String, cochange_pairs: &[CoChangePair]) {
    if cochange_pairs.is_empty() { return; }
    out.push_str(&format!(
        "\n  {BOLD}{UNDERLINE}Co-Change Coupling{RESET} {DIM}(files that always change together){RESET}\n"
    ));
    for pair in cochange_pairs {
        let coupling_color = if pair.jaccard > 0.7 { BRIGHT_RED } else if pair.jaccard > 0.3 { YELLOW } else { DIM };
        out.push_str(&format!(
            "    {DIM_CYAN}{}{RESET} <-> {DIM_CYAN}{}{RESET} {coupling_color}({} co-changes, J={:.2}){RESET}\n",
            pair.file_a, pair.file_b, pair.count, pair.jaccard,
        ));
    }
}

/// Commit type classification rules: (prefix, contains, color, tag)
const COMMIT_TYPE_RULES: &[(&[&str], &[&str], &str, &str)] = &[
    (&["fix"], &["fix:", "bugfix"], RED, "[fix]"),
    (&["feat", "add "], &["feat:"], GREEN, "[feat]"),
    (&["refactor"], &["refactor:"], MAGENTA, "[refactor]"),
    (&["docs"], &["docs:"], CYAN, "[docs]"),
    (&["test"], &["test:"], YELLOW, "[test]"),
    (&["perf"], &["perf:"], BRIGHT_GREEN, "[perf]"),
    (&["chore"], &["chore:"], DIM, "[chore]"),
    (&["ci"], &["ci:"], DIM, "[ci]"),
    (&["merge"], &[], DIM, "[merge]"),
];

/// Classify commit type from subject line and return (color, tag)
pub(super) fn classify_commit_type(subject: &str) -> (&'static str, &'static str) {
    let lower = subject.to_lowercase();
    for &(prefixes, contains, color, tag) in COMMIT_TYPE_RULES {
        if prefixes.iter().any(|p| lower.starts_with(p))
            || contains.iter().any(|c| lower.contains(c))
        {
            return (color, tag);
        }
    }
    (DIM, "")
}

/// Format a unix timestamp as a short date string
pub(super) fn format_timestamp(ts: i64) -> String {
    // Civil date from Unix timestamp using the algorithm from
    // Howard Hinnant's date library (public domain)
    let z = ts / 86400 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

// ── Git log parsing ─────────────────────────────────────────────────────────

/// Parse git log output with format:
/// PMAT_START
/// H:<hash>
/// S:<subject>
/// N:<author_name>
/// E:<author_email>
/// T:<timestamp>
/// PMAT_FILES
/// M\tfile1.rs
/// A\tfile2.rs
fn parse_file_change_line(line: &str) -> Option<FileChange> {
    let parts: Vec<&str> = line.splitn(2, '\t').collect();
    if parts.len() != 2 { return None; }
    let change_type = match parts[0].chars().next() {
        Some('A') => ChangeType::Added,
        Some('D') => ChangeType::Deleted,
        _ => ChangeType::Modified,
    };
    Some(FileChange {
        path: parts[1].trim().to_string(),
        change_type,
        lines_added: 0,
        lines_deleted: 0,
    })
}

fn parse_header_line(
    line: &str,
    hash: &mut String, subject: &mut String,
    author_name: &mut String, author_email: &mut String, timestamp: &mut i64,
) {
    if let Some(val) = line.strip_prefix("H:") { *hash = val.to_string(); }
    else if let Some(val) = line.strip_prefix("S:") { *subject = val.to_string(); }
    else if let Some(val) = line.strip_prefix("N:") { *author_name = val.to_string(); }
    else if let Some(val) = line.strip_prefix("E:") { *author_email = val.to_string(); }
    else if let Some(val) = line.strip_prefix("T:") { *timestamp = val.parse().unwrap_or(0); }
}

fn parse_commit_block(block: &str) -> Option<CommitInfo> {
    let mut hash = String::new();
    let mut subject = String::new();
    let mut author_name = String::new();
    let mut author_email = String::new();
    let mut timestamp: i64 = 0;
    let mut files = Vec::new();
    let mut in_files = false;

    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if line == "PMAT_FILES" { in_files = true; continue; }
        if in_files {
            if let Some(fc) = parse_file_change_line(line) { files.push(fc); }
        } else {
            parse_header_line(line, &mut hash, &mut subject, &mut author_name, &mut author_email, &mut timestamp);
        }
    }
    if hash.is_empty() { return None; }

    let (is_fix, is_feat, is_merge) = classify_commit_type_from_subject(&subject);
    let issue_refs = extract_issue_refs_from_subject(&subject);
    Some(CommitInfo {
        hash, message_subject: subject, message_body: None,
        author_name, author_email, timestamp,
        is_merge, is_fix, is_feat, issue_refs, files,
    })
}

fn classify_commit_type_from_subject(subject: &str) -> (bool, bool, bool) {
    let s = subject.to_lowercase();
    let is_fix = s.starts_with("fix") || s.contains("fix:") || s.contains("bugfix");
    let is_feat = s.starts_with("feat") || s.contains("feat:") || s.starts_with("add ");
    let is_merge = s.starts_with("merge ");
    (is_fix, is_feat, is_merge)
}

fn extract_issue_refs_from_subject(subject: &str) -> Vec<String> {
    subject
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| c == '(' || c == ')' || c == ',' || c == '.'))
        .filter(|w| {
            (w.starts_with('#') && w.len() > 1)
                || w.starts_with("PMAT-") || w.starts_with("pmat-")
                || w.starts_with("GH-") || w.starts_with("gh-")
        })
        .map(|w| w.to_string())
        .collect()
}

pub(super) fn parse_git_log(log_text: &str) -> Vec<CommitInfo> {
    log_text
        .split("PMAT_START")
        .skip(1)
        .filter_map(parse_commit_block)
        .collect()
}
