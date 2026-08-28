//! Churn-related complexity analysis.

use anyhow::Result;
use std::path::PathBuf;

/// Handle churn analysis command
#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_churn(
    project_path: PathBuf,
    days: u32,
    format: crate::models::churn::ChurnOutputFormat,
    output: Option<PathBuf>,
    top_files: usize,
    include: Vec<String>,
    exclude: Vec<String>,
) -> Result<()> {
    use crate::services::git_analysis::GitAnalysisService;

    crate::status_eprintln!("📊 Analyzing code churn for the last {days} days...");

    // Create and apply file filters
    let filter = create_and_report_file_filter(include, exclude)?;

    // Analyze code churn
    let mut analysis = GitAnalysisService::analyze_code_churn(&project_path, days)
        .map_err(|e| anyhow::anyhow!("Churn analysis failed: {e}"))?;

    // Apply filtering and limits
    let listing = apply_churn_filters(&mut analysis, &filter, top_files);

    // Issue #1050 P4. This banner was `analysis.files.len()` AFTER the
    // `--top-files` truncation, i.e. `min(--top-files, total)` — a display knob
    // rendered as a measurement. On a 15-file fixture it read
    // `✅ Analyzed 10 files with changes` two lines above its own
    // `Files changed: 15`, and a flag sweep confirmed it tracked the knob:
    // `--top-files 3` → 3, `--top-files 25` → 15. Real instances: forjar
    // reported 10 against 543, aprender 10 against 454, bashrs 10 against 433.
    //
    // The measurement is `total_files_changed`. The cap is a separate sentence,
    // printed only when it actually cut something — the shape `analyze
    // complexity` already uses.
    crate::status_eprintln!(
        "✅ Analyzed {} files with changes",
        analysis.summary.total_files_changed
    );
    if listing.truncated {
        crate::status_eprintln!(
            "ℹ️  Listing the {} most-changed of {} files (--top-files {}); \
             the summary covers all {}.",
            listing.files_listed,
            listing.files_analyzed,
            listing.top_files,
            listing.files_analyzed
        );
    }

    // Format and write output
    format_and_write_churn_output(analysis, format, output, listing).await
}

/// What the emitted `files` array is a slice of (issue #1050 P4).
///
/// `analyze complexity`, `analyze satd` and `analyze big-o` all disclose this;
/// churn silently cut its JSON array to `--top-files` with no marker anywhere
/// in the document, so a consumer reading `files` believed it had the whole
/// list. Same struct shape, same key names, so the two commands can be parsed
/// by one reader.
pub(super) struct ChurnListing {
    pub top_files: usize,
    pub files_listed: usize,
    pub files_analyzed: usize,
    pub truncated: bool,
}

/// Create file filter and report filter settings
fn create_and_report_file_filter(
    include: Vec<String>,
    exclude: Vec<String>,
) -> Result<crate::utils::file_filter::FileFilter> {
    if !include.is_empty() || !exclude.is_empty() {
        crate::status_eprintln!("🔍 Applying file filters...");
        if !include.is_empty() {
            crate::status_eprintln!("  Include patterns: {include:?}");
        }
        if !exclude.is_empty() {
            crate::status_eprintln!("  Exclude patterns: {exclude:?}");
        }
    }

    crate::utils::file_filter::FileFilter::new(include, exclude)
}

/// Apply file filters and top files limit to churn analysis
fn apply_churn_filters(
    analysis: &mut crate::models::churn::CodeChurnAnalysis,
    filter: &crate::utils::file_filter::FileFilter,
    top_files: usize,
) -> ChurnListing {
    // Apply file filter if filters are active
    if filter.has_filters() {
        analysis
            .files
            .retain(|file| filter.should_include(&file.path));

        // Update summary. Only the FILE count is derivable from the retained
        // rows: `total_commits` is the count of DISTINCT commits in the window
        // (see git_analysis::generate_summary), and this used to overwrite it
        // with `sum(file.commit_count)` — a sum of file-touch events, which
        // counts one commit once per file it touched. On this repo that turned
        // a true 13 commits into 779 under `--include '**/*.rs'`, i.e. a filter
        // made the total grow. Per-file commit SHAs are not carried on
        // FileChurnMetrics, so the union cannot be recomputed here; leave the
        // distinct-commit total for the window alone rather than replace it
        // with a number that measures something else.
        analysis.summary.total_files_changed = analysis.files.len();
    }

    // The denominator is fixed BEFORE the cap: after `truncate` there is no
    // way to recover how many files the analysis actually covered, which is
    // how the banner came to quote the cap.
    let files_analyzed = analysis.files.len();

    // Apply top_files limit if specified (0 means show all)
    if top_files > 0 && analysis.files.len() > top_files {
        analysis
            .files
            .sort_by_key(|b| std::cmp::Reverse(b.commit_count));
        analysis.files.truncate(top_files);
    }

    ChurnListing {
        top_files,
        files_listed: analysis.files.len(),
        files_analyzed,
        truncated: analysis.files.len() < files_analyzed,
    }
}

/// Format churn analysis output and write to file or stdout
async fn format_and_write_churn_output(
    analysis: crate::models::churn::CodeChurnAnalysis,
    format: crate::models::churn::ChurnOutputFormat,
    output: Option<PathBuf>,
    listing: ChurnListing,
) -> Result<()> {
    use crate::models::churn::ChurnOutputFormat;

    let content = match format {
        ChurnOutputFormat::Json => serde_json::to_string_pretty(&churn_json(&analysis, &listing)?)?,
        ChurnOutputFormat::Summary => {
            crate::cli::analysis_utilities::format_churn_as_summary(&analysis)?
        }
        ChurnOutputFormat::Markdown => {
            crate::cli::analysis_utilities::format_churn_as_markdown(&analysis)?
        }
        ChurnOutputFormat::Csv => crate::cli::analysis_utilities::format_churn_as_csv(&analysis)?,
    };

    crate::cli::analysis_utilities::write_churn_output(content, output).await
}

/// The churn document, with the listing disclosure the array needs.
///
/// Issue #1050 P4: the whole document was `to_string_pretty(&analysis)`, whose
/// `files` array is post-truncation and whose only other count,
/// `summary.total_files_changed`, is pre-truncation — two numbers that
/// contradict each other with nothing in the document saying why.
fn churn_json(
    analysis: &crate::models::churn::CodeChurnAnalysis,
    listing: &ChurnListing,
) -> Result<serde_json::Value> {
    let mut value = serde_json::to_value(analysis)?;
    if let Some(obj) = value.as_object_mut() {
        if listing.top_files > 0 {
            obj.insert(
                "top_files_limit".to_string(),
                serde_json::json!(listing.top_files),
            );
        }
        obj.insert(
            "files_listed".to_string(),
            serde_json::json!(listing.files_listed),
        );
        obj.insert(
            "files_analyzed".to_string(),
            serde_json::json!(listing.files_analyzed),
        );
        obj.insert(
            "files_truncated".to_string(),
            serde_json::json!(listing.truncated),
        );
    }
    Ok(value)
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod churn_handler_tests {
    //! Covers the pure-compute helpers in complexity_handlers/churn.rs
    //! (50 uncov on broad, 0% cov).
    use super::*;
    use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
    use crate::utils::file_filter::FileFilter;
    use chrono::Utc;
    use std::path::PathBuf;

    fn make_file(path: &str, commits: usize) -> FileChurnMetrics {
        FileChurnMetrics {
            path: PathBuf::from(path),
            relative_path: path.to_string(),
            commit_count: commits,
            unique_authors: vec!["a".into()],
            additions: 10,
            deletions: 5,
            churn_score: commits as f32 * 0.1,
            last_modified: Utc::now(),
            first_seen: Utc::now(),
        }
    }

    fn make_analysis(files: Vec<FileChurnMetrics>) -> CodeChurnAnalysis {
        let total_commits: usize = files.iter().map(|f| f.commit_count).sum();
        CodeChurnAnalysis {
            generated_at: Utc::now(),
            period_days: 30,
            repository_root: PathBuf::from("."),
            files,
            summary: ChurnSummary {
                total_commits,
                total_files_changed: 0,
                hotspot_files: vec![],
                stable_files: vec![],
                author_contributions: Default::default(),
                mean_churn_score: 0.0,
                variance_churn_score: 0.0,
                stddev_churn_score: 0.0,
            },
        }
    }

    // ── create_and_report_file_filter ──

    #[test]
    fn test_create_and_report_file_filter_no_patterns() {
        let filter = create_and_report_file_filter(vec![], vec![]).unwrap();
        assert!(!filter.has_filters());
    }

    #[test]
    fn test_create_and_report_file_filter_include_only() {
        let filter = create_and_report_file_filter(vec!["src/**".into()], vec![]).unwrap();
        assert!(filter.has_filters());
    }

    #[test]
    fn test_create_and_report_file_filter_exclude_only() {
        let filter = create_and_report_file_filter(vec![], vec!["tests/**".into()]).unwrap();
        assert!(filter.has_filters());
    }

    #[test]
    fn test_create_and_report_file_filter_both_sides() {
        let filter =
            create_and_report_file_filter(vec!["src/**".into()], vec!["tests/**".into()]).unwrap();
        assert!(filter.has_filters());
    }

    /// Issue #1050 P4. `--top-files` is a DISPLAY knob; the banner quoted it
    /// as the measurement, and the JSON array was cut to it with no marker.
    ///
    /// RED CONTROL: reverting `files_analyzed` to `analysis.files.len()` read
    /// after the truncate makes `files_analyzed` 2 here instead of 5, and
    /// dropping the `churn_json` wrapper removes all three keys.
    #[test]
    fn the_listing_disclosure_separates_the_cap_from_the_measurement() {
        let mut a = make_analysis(vec![
            make_file("a.rs", 9),
            make_file("b.rs", 8),
            make_file("c.rs", 7),
            make_file("d.rs", 6),
            make_file("e.rs", 5),
        ]);
        a.summary.total_files_changed = 5;
        let filter = FileFilter::new(vec![], vec![]).expect("filter");

        let listing = apply_churn_filters(&mut a, &filter, 2);

        assert_eq!(listing.files_listed, 2, "the array is the capped slice");
        assert_eq!(
            listing.files_analyzed, 5,
            "the denominator must survive the cap"
        );
        assert!(listing.truncated, "a cut list must say it was cut");

        let doc = churn_json(&a, &listing).expect("json");
        assert_eq!(doc["files_listed"], 2);
        assert_eq!(doc["files_analyzed"], 5);
        assert_eq!(doc["files_truncated"], true);
        assert_eq!(doc["top_files_limit"], 2);
        assert_eq!(
            doc["files"].as_array().map(Vec::len),
            Some(2),
            "the array itself is unchanged; only the disclosure is new"
        );
    }

    /// COUNTER-TEST: a run that lists everything must not claim truncation.
    /// A `files_truncated: true` on every document is as useless as none.
    #[test]
    fn an_uncapped_run_does_not_claim_to_be_truncated() {
        let mut a = make_analysis(vec![make_file("a.rs", 2), make_file("b.rs", 1)]);
        a.summary.total_files_changed = 2;
        let filter = FileFilter::new(vec![], vec![]).expect("filter");

        let listing = apply_churn_filters(&mut a, &filter, 10);

        assert!(!listing.truncated, "10 > 2 cuts nothing");
        assert_eq!(listing.files_listed, listing.files_analyzed);
        let doc = churn_json(&a, &listing).expect("json");
        assert_eq!(doc["files_truncated"], false);
    }

    // ── apply_churn_filters ──

    #[test]
    fn test_apply_churn_filters_no_filters_and_no_limit_keeps_all() {
        let mut a = make_analysis(vec![
            make_file("a.rs", 3),
            make_file("b.rs", 1),
            make_file("c.rs", 5),
        ]);
        let filter = FileFilter::new(vec![], vec![]).unwrap();
        apply_churn_filters(&mut a, &filter, 0);
        assert_eq!(a.files.len(), 3);
    }

    #[test]
    fn test_apply_churn_filters_top_limit_truncates_by_commit_count_desc() {
        let mut a = make_analysis(vec![
            make_file("a.rs", 3),
            make_file("b.rs", 10),
            make_file("c.rs", 1),
        ]);
        let filter = FileFilter::new(vec![], vec![]).unwrap();
        apply_churn_filters(&mut a, &filter, 2);
        assert_eq!(a.files.len(), 2);
        assert_eq!(a.files[0].commit_count, 10); // highest first
        assert_eq!(a.files[1].commit_count, 3);
    }

    #[test]
    fn test_apply_churn_filters_limit_larger_than_count_keeps_all() {
        let mut a = make_analysis(vec![make_file("a.rs", 1), make_file("b.rs", 2)]);
        let filter = FileFilter::new(vec![], vec![]).unwrap();
        apply_churn_filters(&mut a, &filter, 99);
        assert_eq!(a.files.len(), 2);
    }

    #[test]
    fn test_apply_churn_filters_with_include_pattern_retains_only_match() {
        let mut a = make_analysis(vec![
            make_file("src/a.rs", 3),
            make_file("tests/b.rs", 5),
            make_file("src/c.rs", 1),
        ]);
        let filter = FileFilter::new(vec!["src/**".into()], vec![]).unwrap();
        apply_churn_filters(&mut a, &filter, 0);
        assert_eq!(a.files.len(), 2);
        assert!(a.files.iter().all(|f| f.relative_path.starts_with("src/")));
        // The file count describes the retained rows.
        assert_eq!(a.summary.total_files_changed, 2);
    }

    /// A display filter can only remove rows; it must never raise
    /// `total_commits`. The old code recomputed it as `sum(commit_count)`, so
    /// `--include '**/*.rs'` reported 779 commits for a 30-day window holding
    /// 13 — the sum of file-touch events, not commits.
    #[test]
    fn test_apply_churn_filters_does_not_inflate_total_commits() {
        // 3 files touched by 4 distinct commits between them.
        let mut a = make_analysis(vec![
            make_file("src/a.rs", 3),
            make_file("tests/b.rs", 2),
            make_file("src/c.rs", 4),
        ]);
        a.summary.total_commits = 4; // the DISTINCT commit count for the window
        let filter = FileFilter::new(vec!["src/**".into()], vec![]).unwrap();
        apply_churn_filters(&mut a, &filter, 0);

        assert!(
            a.summary.total_commits <= 4,
            "filtering raised total_commits to {} — a filter cannot add commits",
            a.summary.total_commits
        );
        assert_eq!(a.summary.total_commits, 4);
    }
}
