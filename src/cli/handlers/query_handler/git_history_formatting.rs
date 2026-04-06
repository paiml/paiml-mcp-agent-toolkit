// Colorized output formatting for git history search results.
//
// Provides: format_git_history_colorized (main entry point), plus helpers
// for commit entries, metadata, files, hotspots, defect introductions,
// churn velocity, co-change coupling, and commit type classification.

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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
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
    debug_assert!(!files.is_empty(), "files must not be empty");
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
    debug_assert!(!file_path.is_empty(), "file_path must not be empty");
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
    debug_assert!(!grade.is_empty(), "grade must not be empty");
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
    debug_assert!(!path.is_empty(), "path must not be empty");
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
