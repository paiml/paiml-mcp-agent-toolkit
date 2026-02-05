//! Query Handler - Semantic code search for agents (PMAT-470)
//!
//! Provides RAG-powered code search with quality annotations.
//! Designed as a grep replacement for AI agents.

use crate::cli::QueryOutputFormat;
use crate::services::agent_context::{
    enrich_results_with_churn, enrich_results_with_duplicates, enrich_results_with_entropy,
    enrich_results_with_faults, format_json, format_markdown, format_text, format_text_with_code,
    AgentContextIndex, QueryOptions, RankBy,
};
use crate::services::git_history::{
    ChangeType, CommitInfo, FileChange, GitHistoryIndex, GitHistorySearchEngine, GitSearchOptions,
    GitSearchResult,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

// ── ANSI color constants ────────────────────────────────────────────────────

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const UNDERLINE: &str = "\x1b[4m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[1;37m";
const BRIGHT_GREEN: &str = "\x1b[1;32m";
const BRIGHT_RED: &str = "\x1b[1;31m";
const DIM_CYAN: &str = "\x1b[2;36m";

// ── Data structures ─────────────────────────────────────────────────────────

/// Timing breakdown for git history search phases
struct GitHistoryProfile {
    git_log_ms: u128,
    parse_ms: u128,
    index_ms: u128,
    search_ms: u128,
    annotate_ms: u128,
    total_ms: u128,
    commit_count: usize,
}

/// Quality annotations for a file referenced in git history
#[derive(Default, Clone)]
struct FileAnnotation {
    tdg_grade: Option<String>,
    avg_complexity: Option<f32>,
    max_pagerank: Option<f32>,
    function_count: usize,
    dead_code_count: usize,
    dead_code_pct: f32,
    fault_count: usize,
}

/// Aggregated hotspot info for a file across all commits
#[derive(Default, Clone)]
struct FileHotspot {
    commit_count: usize,
    fix_count: usize,
    feat_count: usize,
    lines_added: u64,
    lines_deleted: u64,
    authors: HashMap<String, usize>,
    annotation: FileAnnotation,
}

/// Co-change pair
struct CoChangePair {
    file_a: String,
    file_b: String,
    count: usize,
    jaccard: f32,
}

/// Per-commit enrichment (reserved for JSON output format)
#[allow(dead_code)]
struct CommitAnnotation {
    work_ticket: Option<WorkTicketInfo>,
    commit_quality: Option<CommitQualityMeta>,
    decay_score: f32,
    impact_risk: f32,
}

/// Work ticket cross-reference
struct WorkTicketInfo {
    ticket_id: String,
    claims_passed: usize,
    claims_total: usize,
    #[allow(dead_code)]
    baseline_tdg: f64,
}

/// Quality metadata from .pmat-metrics/commit-*-meta.json
#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct CommitQualityMeta {
    #[serde(default)]
    work_item_id: String,
    #[serde(default)]
    tdg_score: f64,
    #[serde(default)]
    repo_score: f64,
    #[serde(default)]
    rust_project_score: Option<f64>,
}

/// Dead code cache entry
#[derive(serde::Deserialize)]
struct DeadCodeCache {
    #[serde(default)]
    report: DeadCodeReport,
}

#[derive(serde::Deserialize, Default)]
struct DeadCodeReport {
    #[serde(default)]
    files_with_dead_code: Vec<DeadCodeFile>,
}

#[derive(serde::Deserialize)]
struct DeadCodeFile {
    file_path: String,
    #[serde(default)]
    dead_items: Vec<serde_json::Value>,
    #[serde(default)]
    file_dead_percentage: f32,
}

/// Bug hunter cache entry
#[derive(serde::Deserialize)]
struct BugHunterCache {
    #[serde(default)]
    findings: Vec<BugHunterFinding>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct BugHunterFinding {
    #[serde(default)]
    file: String,
    #[serde(default)]
    severity: String,
    #[serde(default)]
    suspiciousness: f32,
}

/// Handle the `pmat query` command
///
/// # Arguments
/// * `query` - Natural language query
/// * `limit` - Maximum number of results
/// * `min_grade` - Minimum TDG grade filter
/// * `max_complexity` - Maximum complexity filter
/// * `language` - Language filter
/// * `path_pattern` - File path pattern filter
/// * `project_path` - Project root to search
/// * `format` - Output format
/// * `include_source` - Include full source code
/// * `rebuild_index` - Force rebuild index
/// * `rank_by` - Ranking strategy (relevance, pagerank, centrality, indegree)
/// * `min_pagerank` - Minimum PageRank score filter
/// * `include_project` - Additional project paths to include in search
/// * `churn` - Enrich results with git churn data (commit count, volatility)
/// * `duplicates` - Enrich results with duplicate code detection
/// * `entropy` - Enrich results with entropy/pattern diversity metrics
/// * `faults` - Enrich results with batuta fault pattern annotations
/// * `definition_type` - Filter by definition type (fn, struct, enum, trait, type)
/// * `code` - Show source code inline (default: true, use --summary to disable)
/// * `git_history` - Include git commit history in search via RRF fusion
#[allow(clippy::too_many_arguments)]
pub async fn handle_query(
    query: String,
    limit: usize,
    min_grade: Option<String>,
    max_complexity: Option<u32>,
    language: Option<String>,
    path_pattern: Option<String>,
    project_path: PathBuf,
    format: QueryOutputFormat,
    include_source: bool,
    rebuild_index: bool,
    exclude_tests: bool,
    rank_by: Option<String>,
    min_pagerank: Option<f32>,
    include_project: Vec<PathBuf>,
    churn: bool,
    duplicates: bool,
    entropy: bool,
    faults: bool,
    definition_type: Option<String>,
    code: bool,
    git_history: bool,
) -> anyhow::Result<()> {
    // Check for existing index
    let index_path = project_path.join(".pmat/context.idx");
    let workspace_idx = project_path.join(".pmat/workspace.idx");

    // Suppress status messages for JSON format (issue #145)
    let quiet = matches!(format, QueryOutputFormat::Json);

    // Auto-discover sibling projects with indexes (check early for workspace fast path)
    let mut siblings = AgentContextIndex::discover_sibling_indexes(&project_path);

    // Add explicitly included projects (--include-project option)
    for project in &include_project {
        let idx_path = project.join(".pmat/context.idx");
        if idx_path.exists() {
            let name = project
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| project.display().to_string());
            // Avoid duplicates
            if !siblings.iter().any(|(_, n)| n == &name) {
                siblings.push((idx_path, name));
            }
        } else if !quiet {
            eprintln!(
                "Warning: No index at {:?}, run 'pmat query --rebuild-index' in that project first",
                idx_path
            );
        }
    }

    // Fast path: if workspace cache is fresh, load directly without checking local index
    let index = if !siblings.is_empty()
        && !rebuild_index
        && is_workspace_cache_fresh(&workspace_idx, &siblings, &index_path)
    {
        if !quiet {
            eprintln!("Loading cached workspace index...");
        }
        match AgentContextIndex::load(&workspace_idx) {
            Ok(cached) => cached,
            Err(_) => {
                // Cache corrupted, fall back to normal path
                load_and_merge_index(
                    &project_path,
                    &index_path,
                    &workspace_idx,
                    &siblings,
                    rebuild_index,
                    quiet,
                )?
            }
        }
    } else {
        // Normal path: load local, incremental update, merge if needed
        load_and_merge_index(
            &project_path,
            &index_path,
            &workspace_idx,
            &siblings,
            rebuild_index,
            quiet,
        )?
    };

    if !quiet {
        let manifest = index.manifest();
        eprintln!(
            "Index: {} functions in {} files (avg TDG: {:.1})",
            manifest.function_count, manifest.file_count, manifest.avg_tdg_score
        );
    }

    // Parse rank_by option
    let rank_by_enum = match rank_by {
        Some(ref s) => s.parse::<RankBy>().unwrap_or_default(),
        None => RankBy::default(),
    };

    // Execute query (--code implies --include-source)
    let options = QueryOptions {
        limit,
        min_grade,
        max_complexity,
        max_loc: None,
        language,
        path_pattern,
        include_source: include_source || code,
        rank_by: rank_by_enum,
        min_pagerank,
    };

    let mut results = index
        .query(&query, options)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    // Filter out test functions if requested
    if exclude_tests {
        results.retain(|r| {
            !r.function_name.starts_with("test_")
                && !r.file_path.starts_with("tests/")
                && !r.file_path.contains("/tests/")
                && !r.file_path.contains("_tests.")
                && !r.file_path.contains("_test.")
        });
    }

    // Filter by definition type if requested
    if let Some(ref def_type) = definition_type {
        let def_type_lower = def_type.to_lowercase();
        let filter_type = match def_type_lower.as_str() {
            "fn" | "func" | "function" => "function".to_string(),
            "struct" | "structs" => "struct".to_string(),
            "enum" | "enums" => "enum".to_string(),
            "trait" | "traits" => "trait".to_string(),
            "type" | "types" | "typealias" => "typealias".to_string(),
            other => other.to_string(),
        };
        results.retain(|r| r.definition_type == filter_type);
    }

    // Enrich with git churn data if requested
    if churn && !results.is_empty() {
        if !quiet {
            eprintln!("Computing git churn metrics...");
        }
        if let Err(e) = enrich_results_with_churn(&mut results, &project_path, 90).await {
            if !quiet {
                eprintln!("Warning: Could not compute churn: {}", e);
            }
        }
    }

    // Enrich with duplicate detection if requested
    if duplicates && !results.is_empty() {
        if !quiet {
            eprintln!("Detecting code duplicates...");
        }
        if let Err(e) = enrich_results_with_duplicates(&mut results, &project_path).await {
            if !quiet {
                eprintln!("Warning: Could not detect duplicates: {}", e);
            }
        }
    }

    // Enrich with entropy/pattern diversity if requested
    if entropy && !results.is_empty() {
        if !quiet {
            eprintln!("Computing pattern diversity...");
        }
        if let Err(e) = enrich_results_with_entropy(&mut results, &project_path).await {
            if !quiet {
                eprintln!("Warning: Could not compute entropy: {}", e);
            }
        }
    }

    // Enrich with batuta fault pattern annotations if requested
    if faults && !results.is_empty() {
        if !quiet {
            eprintln!("Detecting fault patterns (batuta)...");
        }
        if let Err(e) = enrich_results_with_faults(&mut results, &project_path).await {
            if !quiet {
                eprintln!("Warning: Could not detect faults: {}", e);
            }
        }
    }

    // Git history RAG fusion
    let (git_results_for_display, all_parsed_commits) = if git_history {
        if !quiet {
            eprintln!("Searching git history...");
        }
        match search_git_history_profiled(&project_path, &query, limit, &index, quiet) {
            Ok((git_hits, profile, all_commits)) => {
                if !quiet {
                    eprintln!(
                        "Git history: {} commits in {}ms (log: {}ms, parse: {}ms, index: {}ms, search: {}ms, annotate: {}ms)",
                        profile.commit_count,
                        profile.total_ms,
                        profile.git_log_ms,
                        profile.parse_ms,
                        profile.index_ms,
                        profile.search_ms,
                        profile.annotate_ms,
                    );
                }
                if !quiet && !git_hits.is_empty() {
                    eprintln!("Found {} relevant commits", git_hits.len());
                }
                (Some(git_hits), Some(all_commits))
            }
            Err(e) => {
                if !quiet {
                    eprintln!("Warning: Git history search failed: {}", e);
                }
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    if results.is_empty() && git_results_for_display.as_ref().map_or(true, |g| g.is_empty()) {
        eprintln!("No matching functions found for: {}", query);
        return Ok(());
    }

    // Format and output results
    let output = match format {
        QueryOutputFormat::Text => {
            if code {
                format_text_with_code(&results)
            } else {
                format_text(&results)
            }
        }
        QueryOutputFormat::Json => format_json(&results).map_err(|e| anyhow::anyhow!("{}", e))?,
        QueryOutputFormat::Markdown => format_markdown(&results),
    };

    println!("{}", output);

    // Append git history results if available
    if let Some(ref git_hits) = git_results_for_display {
        if !git_hits.is_empty() {
            let all_commits = all_parsed_commits.as_deref().unwrap_or(&[]);
            let git_output =
                format_git_history_colorized(git_hits, &project_path, &index, all_commits);
            println!("{}", git_output);
        }
    }

    Ok(())
}

// ── Git history search with profiling ───────────────────────────────────────

/// Search git history with timing profile and O(1) annotations
/// Returns (search_results, profile, all_parsed_commits)
fn search_git_history_profiled(
    project_path: &std::path::Path,
    query: &str,
    limit: usize,
    index: &AgentContextIndex,
    _quiet: bool,
) -> anyhow::Result<(Vec<GitSearchResult>, GitHistoryProfile, Vec<CommitInfo>)> {
    let total_start = Instant::now();

    // Phase 1: git log
    let git_start = Instant::now();
    let output = std::process::Command::new("git")
        .args([
            "log",
            "--format=PMAT_START%nH:%H%nS:%s%nN:%an%nE:%ae%nT:%at%nPMAT_FILES",
            "--name-status",
            "-500",
        ])
        .current_dir(project_path)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run git log: {}", e))?;
    let git_log_ms = git_start.elapsed().as_millis();

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Phase 2: parse
    let parse_start = Instant::now();
    let log_text = String::from_utf8_lossy(&output.stdout);
    let commits = parse_git_log(&log_text);
    let commit_count = commits.len();
    let parse_ms = parse_start.elapsed().as_millis();

    if commits.is_empty() {
        return Ok((
            vec![],
            GitHistoryProfile {
                git_log_ms,
                parse_ms,
                index_ms: 0,
                search_ms: 0,
                annotate_ms: 0,
                total_ms: total_start.elapsed().as_millis(),
                commit_count: 0,
            },
            vec![],
        ));
    }

    // Phase 3: index build
    let index_start = Instant::now();
    let mut git_index = GitHistoryIndex::in_memory()
        .map_err(|e| anyhow::anyhow!("Failed to create git history index: {}", e))?;
    git_index
        .insert_commits(&commits)
        .map_err(|e| anyhow::anyhow!("Failed to index commits: {}", e))?;
    let index_ms = index_start.elapsed().as_millis();

    // Phase 4: search
    let search_start = Instant::now();
    let mut engine = GitHistorySearchEngine::new(&git_index);
    let options = GitSearchOptions {
        limit,
        ..Default::default()
    };
    let results = engine
        .search(query, options)
        .map_err(|e| anyhow::anyhow!("Git history search failed: {}", e))?;
    let search_ms = search_start.elapsed().as_millis();

    // Phase 5: annotate (we just time the annotation prep here; actual formatting is separate)
    let annotate_start = Instant::now();
    // Pre-warm: verify index lookups work for changed files
    let _ = build_file_annotations(index, project_path, &commits);
    let annotate_ms = annotate_start.elapsed().as_millis();

    let profile = GitHistoryProfile {
        git_log_ms,
        parse_ms,
        index_ms,
        search_ms,
        annotate_ms,
        total_ms: total_start.elapsed().as_millis(),
        commit_count,
    };

    Ok((results, profile, commits))
}

// ── O(1) annotation builders ────────────────────────────────────────────────

/// Build file-level annotations from the code index + cached data
fn build_file_annotations(
    index: &AgentContextIndex,
    project_path: &std::path::Path,
    commits: &[CommitInfo],
) -> (
    HashMap<String, FileHotspot>,
    Vec<CoChangePair>,
    HashMap<String, FileAnnotation>,
) {
    // 1. Aggregate file hotspots from all commits
    let mut hotspots: HashMap<String, FileHotspot> = HashMap::new();
    // Track co-changes for coupling analysis
    let mut cochange_counts: HashMap<(String, String), usize> = HashMap::new();

    for commit in commits {
        let file_paths: Vec<&str> = commit.files.iter().map(|f| f.path.as_str()).collect();

        for fc in &commit.files {
            let entry = hotspots.entry(fc.path.clone()).or_default();
            entry.commit_count += 1;
            if commit.is_fix {
                entry.fix_count += 1;
            }
            if commit.is_feat {
                entry.feat_count += 1;
            }
            entry.lines_added += fc.lines_added as u64;
            entry.lines_deleted += fc.lines_deleted as u64;
            *entry.authors.entry(commit.author_name.clone()).or_insert(0) += 1;
        }

        // Co-change pairs (sorted to avoid duplicates)
        for i in 0..file_paths.len() {
            for j in (i + 1)..file_paths.len() {
                let (a, b) = if file_paths[i] < file_paths[j] {
                    (file_paths[i].to_string(), file_paths[j].to_string())
                } else {
                    (file_paths[j].to_string(), file_paths[i].to_string())
                };
                *cochange_counts.entry((a, b)).or_insert(0) += 1;
            }
        }
    }

    // 2. Build file annotations from code index (O(1) lookups)
    let mut file_annots: HashMap<String, FileAnnotation> = HashMap::new();
    for file_path in hotspots.keys() {
        let funcs = index.get_by_file(file_path);
        if !funcs.is_empty() {
            let mut annot = FileAnnotation::default();
            annot.function_count = funcs.len();

            // Aggregate TDG: use worst grade among functions
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

                // PageRank from graph_metrics (index-aligned)
                if let Some(func_idx) = index.file_index.get(file_path) {
                    if i < func_idx.len() {
                        let global_idx = func_idx[i];
                        if global_idx < index.graph_metrics.len() {
                            let pr = index.graph_metrics[global_idx].pagerank;
                            if pr > max_pr {
                                max_pr = pr;
                            }
                        }
                    }
                }
            }

            annot.tdg_grade = Some(worst_grade);
            annot.avg_complexity = Some(total_complexity / funcs.len() as f32);
            annot.max_pagerank = Some(max_pr);
            annot.fault_count = total_faults;

            file_annots.insert(file_path.clone(), annot);
        }
    }

    // 3. Load dead code cache (O(1) file read + HashMap)
    let dead_code_path = project_path.join(".pmat/dead-code-cache.json");
    if let Ok(data) = std::fs::read_to_string(&dead_code_path) {
        if let Ok(cache) = serde_json::from_str::<DeadCodeCache>(&data) {
            for dc_file in &cache.report.files_with_dead_code {
                if let Some(annot) = file_annots.get_mut(&dc_file.file_path) {
                    annot.dead_code_count = dc_file.dead_items.len();
                    annot.dead_code_pct = dc_file.file_dead_percentage;
                }
                // Also annotate hotspots that aren't in the code index
                if let Some(hotspot) = hotspots.get_mut(&dc_file.file_path) {
                    hotspot.annotation.dead_code_count = dc_file.dead_items.len();
                    hotspot.annotation.dead_code_pct = dc_file.file_dead_percentage;
                }
            }
        }
    }

    // 4. Load bug hunter cache (aggregate by file)
    let bug_hunter_dir = project_path.join(".pmat/bug-hunter-cache");
    if bug_hunter_dir.is_dir() {
        let mut file_fault_counts: HashMap<String, usize> = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(&bug_hunter_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().map_or(false, |e| e == "json") {
                    if let Ok(data) = std::fs::read_to_string(entry.path()) {
                        if let Ok(cache) = serde_json::from_str::<BugHunterCache>(&data) {
                            for finding in &cache.findings {
                                *file_fault_counts.entry(finding.file.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }
        for (file, count) in &file_fault_counts {
            if let Some(annot) = file_annots.get_mut(file) {
                // Use max of code-index faults and bug-hunter faults
                if *count > annot.fault_count {
                    annot.fault_count = *count;
                }
            }
        }
    }

    // 5. Merge annotations into hotspots
    for (path, hotspot) in hotspots.iter_mut() {
        if let Some(annot) = file_annots.get(path) {
            hotspot.annotation = annot.clone();
        }
    }

    // 6. Compute top co-change pairs
    let mut cochange_pairs: Vec<CoChangePair> = cochange_counts
        .into_iter()
        .filter(|(_, count)| *count >= 3) // Only show meaningful coupling
        .map(|((a, b), count)| {
            // Jaccard: cochanges / (commits_a + commits_b - cochanges)
            let ca = hotspots.get(&a).map_or(1, |h| h.commit_count);
            let cb = hotspots.get(&b).map_or(1, |h| h.commit_count);
            let union = ca + cb - count;
            let jaccard = if union > 0 {
                count as f32 / union as f32
            } else {
                0.0
            };
            CoChangePair {
                file_a: a,
                file_b: b,
                count,
                jaccard,
            }
        })
        .collect();
    cochange_pairs.sort_by(|a, b| b.count.cmp(&a.count));
    cochange_pairs.truncate(5);

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
                .map_or(false, |f| !f) // passed = not falsified
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
    let short_hash = &commit_hash[..7.min(commit_hash.len())];
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
/// decay = (1 - TDG_normalized) × churn_ratio × fix_ratio × (1 + dead_code_fraction)
fn compute_decay_score(hotspot: &FileHotspot, total_commits: usize) -> f32 {
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

/// Compute impact × risk score
/// impact_risk = pagerank × churn_ratio × (1 + fault_density)
fn compute_impact_risk(hotspot: &FileHotspot, total_commits: usize) -> f32 {
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
fn format_git_history_colorized(
    hits: &[GitSearchResult],
    project_path: &std::path::Path,
    index: &AgentContextIndex,
    all_commits: &[CommitInfo],
) -> String {
    let mut out = String::new();

    // Use ALL parsed commits (not just search hits) for hotspot analysis
    let total_commits = all_commits.len();
    let (hotspots, cochange_pairs, _file_annots) =
        build_file_annotations(index, project_path, all_commits);

    // Section header
    out.push_str(&format!(
        "\n{BOLD}{UNDERLINE}Git History (RRF-fused){RESET}\n\n"
    ));

    for (i, hit) in hits.iter().enumerate() {
        let commit = &hit.commit;
        let short_hash = &commit.hash[..7.min(commit.hash.len())];

        // Commit type prefix color
        let (type_color, type_tag) = classify_commit_type(&commit.message_subject);

        // Relevance score color
        let score_color = if hit.relevance_score > 0.7 {
            BRIGHT_GREEN
        } else if hit.relevance_score > 0.3 {
            GREEN
        } else {
            DIM
        };

        // Line 1: index, hash, type tag, subject, score
        out.push_str(&format!(
            "  {DIM}{}.{RESET} {YELLOW}{}{RESET} {type_color}{type_tag}{RESET} {WHITE}{}{RESET} {score_color}({:.3}){RESET}\n",
            i + 1,
            short_hash,
            commit.message_subject,
            hit.relevance_score,
        ));

        // Line 2: author, timestamp, issue refs
        let date = format_timestamp(commit.timestamp);
        out.push_str(&format!(
            "     {CYAN}{}{RESET} {DIM}{}{RESET}",
            commit.author_name, date,
        ));

        // Issue refs
        if !commit.issue_refs.is_empty() {
            out.push_str(&format!(
                " {YELLOW}{}{RESET}",
                commit.issue_refs.join(" ")
            ));
        }

        // Work ticket link
        for issue_ref in &commit.issue_refs {
            if let Some(ticket) = load_work_ticket(project_path, issue_ref) {
                let ticket_color = if ticket.claims_passed == ticket.claims_total {
                    GREEN
                } else {
                    YELLOW
                };
                out.push_str(&format!(
                    " {ticket_color}[{}: {}/{} claims]{RESET}",
                    ticket.ticket_id, ticket.claims_passed, ticket.claims_total,
                ));
            }
        }

        // Commit quality metadata
        if let Some(meta) = load_commit_quality(project_path, &commit.hash) {
            let tdg_color = if meta.tdg_score >= 80.0 {
                GREEN
            } else if meta.tdg_score >= 60.0 {
                YELLOW
            } else {
                RED
            };
            out.push_str(&format!(
                " {tdg_color}TDG:{:.0}{RESET}",
                meta.tdg_score,
            ));
            if let Some(rs) = meta.rust_project_score {
                out.push_str(&format!(" {DIM}RS:{:.0}{RESET}", rs));
            }
        }

        out.push('\n');

        // Line 3: files with TDG grade overlay
        if !hit.files.is_empty() {
            out.push_str("     ");
            for (fi, file_path) in hit.files.iter().enumerate() {
                if fi > 0 {
                    out.push_str(", ");
                }
                // Get file annotation for TDG grade
                if let Some(hotspot) = hotspots.get(file_path.as_str()) {
                    let grade = hotspot
                        .annotation
                        .tdg_grade
                        .as_deref()
                        .unwrap_or("?");
                    let grade_color = match grade {
                        "A" => GREEN,
                        "B" => GREEN,
                        "C" => YELLOW,
                        "D" => RED,
                        "F" => BRIGHT_RED,
                        _ => DIM,
                    };

                    out.push_str(&format!(
                        "{DIM_CYAN}{}{RESET} {grade_color}[{grade}]{RESET}",
                        file_path,
                    ));

                    // Fix magnet indicator
                    if hotspot.fix_count > 2 {
                        let fix_pct = if total_commits > 0 {
                            (hotspot.fix_count as f32 / total_commits as f32 * 100.0) as u32
                        } else {
                            0
                        };
                        out.push_str(&format!("{RED}({} fixes, {}%){RESET}", hotspot.fix_count, fix_pct));
                    }

                    // Dead code flag
                    if hotspot.annotation.dead_code_count > 0 {
                        out.push_str(&format!(
                            " {DIM}dead:{}{RESET}",
                            hotspot.annotation.dead_code_count
                        ));
                    }

                    // Fault density
                    if hotspot.annotation.fault_count > 0 {
                        out.push_str(&format!(
                            " {MAGENTA}faults:{}{RESET}",
                            hotspot.annotation.fault_count
                        ));
                    }
                } else {
                    out.push_str(&format!("{DIM_CYAN}{}{RESET}", file_path));
                }
            }
            out.push('\n');
        }

        // Commit body (if present, truncated)
        if let Some(ref body) = commit.message_body {
            if !body.is_empty() {
                let truncated = if body.len() > 120 {
                    format!("{}...", &body[..120])
                } else {
                    body.clone()
                };
                out.push_str(&format!("     {DIM}{}{RESET}\n", truncated));
            }
        }
    }

    // ── Hotspot section ─────────────────────────────────────────────────
    if !hotspots.is_empty() {
        let mut sorted_hotspots: Vec<(&String, &FileHotspot)> = hotspots.iter().collect();
        sorted_hotspots.sort_by(|a, b| b.1.commit_count.cmp(&a.1.commit_count));

        out.push_str(&format!(
            "\n  {BOLD}{UNDERLINE}Hotspots{RESET} {DIM}(top changed files across {} commits){RESET}\n",
            total_commits
        ));

        for (path, hotspot) in sorted_hotspots.iter().take(8) {
            let pct = if total_commits > 0 {
                hotspot.commit_count as f32 / total_commits as f32 * 100.0
            } else {
                0.0
            };

            let churn_color = if pct > 30.0 {
                BRIGHT_RED
            } else if pct > 15.0 {
                RED
            } else if pct > 5.0 {
                YELLOW
            } else {
                DIM
            };

            let grade = hotspot
                .annotation
                .tdg_grade
                .as_deref()
                .unwrap_or("-");
            let grade_color = match grade {
                "A" => GREEN,
                "B" => GREEN,
                "C" => YELLOW,
                "D" => RED,
                "F" => BRIGHT_RED,
                _ => DIM,
            };

            // Fix ratio (Tarantula-style defect magnetism)
            let fix_ratio = if hotspot.commit_count > 0 {
                hotspot.fix_count as f32 / hotspot.commit_count as f32
            } else {
                0.0
            };
            let fix_indicator = if fix_ratio > 0.5 {
                format!(" {BRIGHT_RED}!!{} fixes{RESET}", hotspot.fix_count)
            } else if hotspot.fix_count > 0 {
                format!(" {RED}{} fixes{RESET}", hotspot.fix_count)
            } else {
                String::new()
            };

            // Decay score
            let decay = compute_decay_score(hotspot, total_commits);
            let decay_indicator = if decay > 0.5 {
                format!(" {BRIGHT_RED}decay:{:.2}{RESET}", decay)
            } else if decay > 0.2 {
                format!(" {YELLOW}decay:{:.2}{RESET}", decay)
            } else {
                String::new()
            };

            // Impact × Risk
            let impact_risk = compute_impact_risk(hotspot, total_commits);
            let risk_indicator = if impact_risk > 10.0 {
                format!(" {BRIGHT_RED}risk:{:.1}{RESET}", impact_risk)
            } else if impact_risk > 1.0 {
                format!(" {YELLOW}risk:{:.1}{RESET}", impact_risk)
            } else {
                String::new()
            };

            // Author ownership
            let top_author = hotspot
                .authors
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(name, count)| {
                    let pct = *count as f32 / hotspot.commit_count as f32 * 100.0;
                    format!(" {CYAN}{}:{:.0}%{RESET}", name, pct)
                })
                .unwrap_or_default();

            out.push_str(&format!(
                "    {DIM_CYAN}{:<50}{RESET} {churn_color}{:>3} commits ({:>4.1}%){RESET} {grade_color}[{grade}]{RESET}{fix_indicator}{decay_indicator}{risk_indicator}{top_author}\n",
                path, hotspot.commit_count, pct,
            ));
        }

        // ── Defect introduction tracking ────────────────────────────────
        // Find feat commits whose files got fix commits within 30 days
        let mut defect_introductions: Vec<(String, String, usize)> = Vec::new(); // (feat_hash, file, fix_count)
        let feat_commits: Vec<&CommitInfo> = all_commits
            .iter()
            .filter(|c| c.is_feat)
            .collect();

        for feat in &feat_commits {
            let feat_ts = feat.timestamp;
            let thirty_days = 30 * 24 * 3600;
            let feat_files: std::collections::HashSet<&str> =
                feat.files.iter().map(|f| f.path.as_str()).collect();

            // Count fix commits touching same files within 30 days
            let fix_count: usize = all_commits
                .iter()
                .filter(|c| {
                    c.is_fix
                        && c.timestamp > feat_ts
                        && c.timestamp < feat_ts + thirty_days
                        && c.files.iter().any(|f| feat_files.contains(f.path.as_str()))
                })
                .count();

            if fix_count > 0 {
                let files_str = feat
                    .files
                    .iter()
                    .take(3)
                    .map(|f| f.path.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                defect_introductions.push((
                    feat.hash[..7].to_string(),
                    files_str,
                    fix_count,
                ));
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

        // ── Churn velocity ──────────────────────────────────────────────
        // Compute commits/week for top hotspot files
        if let (Some(newest), Some(oldest)) = (
            all_commits.iter().map(|c| c.timestamp).max(),
            all_commits.iter().map(|c| c.timestamp).min(),
        ) {
            let span_weeks = ((newest - oldest) as f32 / (7.0 * 24.0 * 3600.0)).max(1.0);
            let mut velocity_files: Vec<(&str, f32)> = sorted_hotspots
                .iter()
                .take(5)
                .map(|(path, h)| (path.as_str(), h.commit_count as f32 / span_weeks))
                .filter(|(_, v)| *v > 0.5) // Only show files with >0.5 commits/week
                .collect();
            velocity_files.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            if !velocity_files.is_empty() {
                out.push_str(&format!(
                    "\n  {BOLD}{UNDERLINE}Churn Velocity{RESET} {DIM}(commits/week over {:.0} weeks){RESET}\n",
                    span_weeks
                ));
                for (path, vel) in velocity_files.iter().take(5) {
                    let vel_color = if *vel > 3.0 {
                        BRIGHT_RED
                    } else if *vel > 1.0 {
                        YELLOW
                    } else {
                        DIM
                    };
                    out.push_str(&format!(
                        "    {DIM_CYAN}{:<50}{RESET} {vel_color}{:.1}/wk{RESET}\n",
                        path, vel,
                    ));
                }
            }
        }

        // ── Co-change coupling ──────────────────────────────────────────
        if !cochange_pairs.is_empty() {
            out.push_str(&format!(
                "\n  {BOLD}{UNDERLINE}Co-Change Coupling{RESET} {DIM}(files that always change together){RESET}\n"
            ));
            for pair in &cochange_pairs {
                let coupling_color = if pair.jaccard > 0.7 {
                    BRIGHT_RED
                } else if pair.jaccard > 0.3 {
                    YELLOW
                } else {
                    DIM
                };
                out.push_str(&format!(
                    "    {DIM_CYAN}{}{RESET} <-> {DIM_CYAN}{}{RESET} {coupling_color}({} co-changes, J={:.2}){RESET}\n",
                    pair.file_a, pair.file_b, pair.count, pair.jaccard,
                ));
            }
        }
    }

    out
}

/// Classify commit type from subject line and return (color, tag)
fn classify_commit_type(subject: &str) -> (&'static str, &'static str) {
    let lower = subject.to_lowercase();
    if lower.starts_with("fix") || lower.contains("fix:") || lower.contains("bugfix") {
        (RED, "[fix]")
    } else if lower.starts_with("feat") || lower.contains("feat:") || lower.starts_with("add ") {
        (GREEN, "[feat]")
    } else if lower.starts_with("refactor") || lower.contains("refactor:") {
        (MAGENTA, "[refactor]")
    } else if lower.starts_with("docs") || lower.contains("docs:") {
        (CYAN, "[docs]")
    } else if lower.starts_with("test") || lower.contains("test:") {
        (YELLOW, "[test]")
    } else if lower.starts_with("perf") || lower.contains("perf:") {
        (BRIGHT_GREEN, "[perf]")
    } else if lower.starts_with("chore") || lower.contains("chore:") {
        (DIM, "[chore]")
    } else if lower.starts_with("ci") || lower.contains("ci:") {
        (DIM, "[ci]")
    } else if lower.starts_with("merge") {
        (DIM, "[merge]")
    } else {
        (DIM, "")
    }
}

/// Format a unix timestamp as a short date string
fn format_timestamp(ts: i64) -> String {
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

// ── Original parse_git_log (unchanged) ──────────────────────────────────────

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
fn parse_git_log(log_text: &str) -> Vec<CommitInfo> {
    let mut commits = Vec::new();

    // Split into commit blocks by PMAT_START
    let blocks: Vec<&str> = log_text.split("PMAT_START").collect();

    for block in blocks.iter().skip(1) {
        // skip first empty element
        let mut hash = String::new();
        let mut subject = String::new();
        let mut author_name = String::new();
        let mut author_email = String::new();
        let mut timestamp: i64 = 0;
        let mut files = Vec::new();
        let mut in_files = false;

        for line in block.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line == "PMAT_FILES" {
                in_files = true;
                continue;
            }

            if in_files {
                // Next PMAT_START would be caught by the split above
                let parts: Vec<&str> = line.splitn(2, '\t').collect();
                if parts.len() == 2 {
                    let change_type = match parts[0].chars().next() {
                        Some('A') => ChangeType::Added,
                        Some('D') => ChangeType::Deleted,
                        _ => ChangeType::Modified,
                    };
                    files.push(FileChange {
                        path: parts[1].trim().to_string(),
                        change_type,
                        lines_added: 0,
                        lines_deleted: 0,
                    });
                }
            } else if let Some(val) = line.strip_prefix("H:") {
                hash = val.to_string();
            } else if let Some(val) = line.strip_prefix("S:") {
                subject = val.to_string();
            } else if let Some(val) = line.strip_prefix("N:") {
                author_name = val.to_string();
            } else if let Some(val) = line.strip_prefix("E:") {
                author_email = val.to_string();
            } else if let Some(val) = line.strip_prefix("T:") {
                timestamp = val.parse().unwrap_or(0);
            }
        }

        if hash.is_empty() {
            continue;
        }

        // Detect conventional commit types
        let subject_lower = subject.to_lowercase();
        let is_fix = subject_lower.starts_with("fix")
            || subject_lower.contains("fix:")
            || subject_lower.contains("bugfix");
        let is_feat = subject_lower.starts_with("feat")
            || subject_lower.contains("feat:")
            || subject_lower.starts_with("add ");
        let is_merge = subject_lower.starts_with("merge ");

        // Extract issue refs (#123 and PMAT-123 style)
        let issue_refs: Vec<String> = subject
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| c == '(' || c == ')' || c == ',' || c == '.'))
            .filter(|w| {
                (w.starts_with('#') && w.len() > 1)
                    || w.starts_with("PMAT-") || w.starts_with("pmat-")
                    || w.starts_with("GH-") || w.starts_with("gh-")
            })
            .map(|w| w.to_string())
            .collect();

        commits.push(CommitInfo {
            hash,
            message_subject: subject,
            message_body: None,
            author_name,
            author_email,
            timestamp,
            is_merge,
            is_fix,
            is_feat,
            issue_refs,
            files,
        });
    }

    commits
}

// ── Index management (unchanged) ────────────────────────────────────────────

/// Load local index, do incremental update if needed, and merge siblings.
fn load_and_merge_index(
    project_path: &PathBuf,
    index_path: &PathBuf,
    workspace_idx: &std::path::Path,
    siblings: &[(PathBuf, String)],
    rebuild_index: bool,
    quiet: bool,
) -> anyhow::Result<AgentContextIndex> {
    let mut index = if index_path.exists() && !rebuild_index {
        if !quiet {
            eprintln!("Loading index from {:?}...", index_path);
        }
        match AgentContextIndex::load(index_path) {
            Ok(existing) => {
                // Try incremental update if checksums are available
                if !existing.manifest().file_checksums.is_empty() {
                    if !quiet {
                        eprintln!("Checking for incremental updates...");
                    }
                    match AgentContextIndex::build_incremental(project_path, &existing) {
                        Ok(updated) => {
                            // Only save if there were actual changes
                            if updated.manifest().last_incremental_changes > 0 {
                                let _ = updated.save(index_path);
                            }
                            updated
                        }
                        Err(_) => existing,
                    }
                } else {
                    existing
                }
            }
            Err(e) => {
                eprintln!("Failed to load index ({}), rebuilding...", e);
                build_and_save_index(project_path, index_path)?
            }
        }
    } else {
        if !quiet {
            eprintln!("Building index for {:?}...", project_path);
        }
        build_and_save_index(project_path, index_path)?
    };

    // Merge siblings if any
    if !siblings.is_empty() {
        merge_and_cache_workspace(&mut index, siblings, workspace_idx, quiet);
    }

    Ok(index)
}

/// Check if the cached workspace index is newer than all sibling indexes and local index.
fn is_workspace_cache_fresh(
    workspace_idx: &std::path::Path,
    siblings: &[(PathBuf, String)],
    local_idx: &std::path::Path,
) -> bool {
    // Use manifest.json mtime (not directory mtime) for consistent comparison
    let cache_manifest = workspace_idx.join("manifest.json");
    let cache_mtime = match std::fs::metadata(&cache_manifest).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return false, // No cache
    };

    // Check local index is not newer than cache
    let local_manifest = local_idx.join("manifest.json");
    if let Ok(local_mtime) = std::fs::metadata(&local_manifest).and_then(|m| m.modified()) {
        if local_mtime > cache_mtime {
            return false; // Local index updated since cache
        }
    }

    // Cache is fresh if it's newer than every sibling's index
    siblings.iter().all(|(idx_path, _)| {
        // Check manifest.json mtime (always written on save)
        let manifest = idx_path.join("manifest.json");
        match std::fs::metadata(&manifest).and_then(|m| m.modified()) {
            Ok(sibling_mtime) => cache_mtime > sibling_mtime,
            Err(_) => true, // Sibling gone, cache still valid for others
        }
    })
}

/// Merge siblings into index and save the combined result as workspace cache.
fn merge_and_cache_workspace(
    index: &mut AgentContextIndex,
    siblings: &[(PathBuf, String)],
    workspace_idx: &std::path::Path,
    quiet: bool,
) {
    if !quiet {
        eprintln!("Merging {} sibling project(s):", siblings.len());
    }
    index.merge_siblings(siblings);

    // Cache the merged index for next time
    match index.save(workspace_idx) {
        Ok(()) => {
            if !quiet {
                eprintln!("Workspace index cached.");
            }
        }
        Err(e) => {
            if !quiet {
                eprintln!("Failed to cache workspace index: {}", e);
            }
        }
    }
}

/// Build index and save to disk
fn build_and_save_index(
    project_path: &PathBuf,
    index_path: &PathBuf,
) -> anyhow::Result<AgentContextIndex> {
    let index = AgentContextIndex::build(project_path)
        .map_err(|e| anyhow::anyhow!("Failed to build index: {}", e))?;

    // Create .pmat directory if needed
    if let Some(parent) = index_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Save index
    index
        .save(index_path)
        .map_err(|e| anyhow::anyhow!("Failed to save index: {}", e))?;

    eprintln!("Index saved to {:?}", index_path);

    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_handle_query_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create empty project
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(project_path.join("src/main.rs"), "").unwrap();

        let result = handle_query(
            "test".to_string(),
            10,
            None,
            None,
            None,
            None,
            project_path,
            QueryOutputFormat::Text,
            false,
            false,
            false,
            None,  // rank_by
            None,  // min_pagerank
            vec![], // include_project
            false, // churn
            false, // duplicates
            false, // entropy
            false, // faults
            None,  // definition_type
            false, // code
            false, // git_history
        )
        .await;

        // Should not error, just find nothing
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_query_with_functions() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_path_buf();

        // Create project with a function
        std::fs::create_dir_all(project_path.join("src")).unwrap();
        std::fs::write(
            project_path.join("src/main.rs"),
            r#"
/// Handle errors in the API layer
fn handle_api_error(err: String) -> String {
    format!("Error: {}", err)
}

fn main() {
    println!("Hello");
}
"#,
        )
        .unwrap();

        let result = handle_query(
            "error handling".to_string(),
            10,
            None,
            None,
            None,
            None,
            project_path,
            QueryOutputFormat::Json,
            false,
            true, // Force rebuild
            false,
            None,  // rank_by
            None,  // min_pagerank
            vec![], // include_project
            false, // churn
            false, // duplicates
            false, // entropy
            false, // faults
            None,  // definition_type
            false, // code
            false, // git_history
        )
        .await;

        assert!(result.is_ok());
    }

    #[test]
    fn test_classify_commit_type() {
        assert_eq!(classify_commit_type("fix: null pointer").1, "[fix]");
        assert_eq!(classify_commit_type("feat: add auth").1, "[feat]");
        assert_eq!(classify_commit_type("refactor: simplify parser").1, "[refactor]");
        assert_eq!(classify_commit_type("docs: update README").1, "[docs]");
        assert_eq!(classify_commit_type("chore: bump deps").1, "[chore]");
        assert_eq!(classify_commit_type("random commit").1, "");
        assert_eq!(classify_commit_type("Merge branch main").1, "[merge]");
    }

    #[test]
    fn test_format_timestamp() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        let ts = 1704067200_i64;
        let formatted = format_timestamp(ts);
        // Should produce something reasonable (approximate date)
        assert!(formatted.starts_with("2024"));
    }

    #[test]
    fn test_compute_decay_score() {
        let mut hotspot = FileHotspot::default();
        hotspot.commit_count = 10;
        hotspot.fix_count = 5;
        hotspot.annotation.tdg_grade = Some("D".to_string());
        hotspot.annotation.dead_code_pct = 10.0;

        let decay = compute_decay_score(&hotspot, 100);
        assert!(decay > 0.0);
        assert!(decay <= 1.0);

        // Grade A with no fixes should have low decay
        let mut healthy = FileHotspot::default();
        healthy.commit_count = 5;
        healthy.fix_count = 0;
        healthy.annotation.tdg_grade = Some("A".to_string());
        let healthy_decay = compute_decay_score(&healthy, 100);
        assert!(healthy_decay < decay, "Healthy file should have lower decay");
    }

    #[test]
    fn test_compute_impact_risk() {
        let mut hotspot = FileHotspot::default();
        hotspot.commit_count = 50;
        hotspot.annotation.max_pagerank = Some(0.01);
        hotspot.annotation.fault_count = 3;

        let risk = compute_impact_risk(&hotspot, 100);
        assert!(risk > 0.0);

        // Zero pagerank = zero risk
        let mut low_risk = FileHotspot::default();
        low_risk.commit_count = 50;
        low_risk.annotation.max_pagerank = Some(0.0);
        assert_eq!(compute_impact_risk(&low_risk, 100), 0.0);
    }

    #[test]
    fn test_parse_git_log_with_issue_refs() {
        let log = "PMAT_START\nH:abc1234567890123456789012345678901234567\nS:feat: add auth (PMAT-472)\nN:noah\nE:noah@test.com\nT:1704067200\nPMAT_FILES\nM\tsrc/main.rs";
        let commits = parse_git_log(log);
        assert_eq!(commits.len(), 1);
        assert!(commits[0].issue_refs.contains(&"PMAT-472".to_string()) || commits[0].issue_refs.contains(&"(PMAT-472)".to_string()));
        assert!(commits[0].is_feat);
    }

    #[test]
    fn test_file_annotation_default() {
        let annot = FileAnnotation::default();
        assert_eq!(annot.tdg_grade, None);
        assert_eq!(annot.function_count, 0);
        assert_eq!(annot.dead_code_count, 0);
        assert_eq!(annot.fault_count, 0);
    }
}
