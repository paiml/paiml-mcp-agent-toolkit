//! `--color never` must reach the printers that interpolate raw ANSI constants.
//!
//! `analyze churn`, `analyze complexity`, `analyze graph-metrics`,
//! `analyze defects`, `analyze comprehensive` and `analyze deep-context` all
//! wrote escape sequences into a redirected file under `--color never` and under
//! `NO_COLOR=1`: 33, 2, 109, 87, 7 and 11 escape-bearing lines respectively. For
//! `graph-metrics` the `--color never` output was BYTE-IDENTICAL to `--color
//! always` — the flag moved nothing in either direction.
//!
//! The cause was not a missing check in each printer. It was that the colour
//! rule lived in helper functions (`c::header`, `c::number`, …) which these
//! printers do not use: they interpolate the `pub const` sequences directly, and
//! a `const &str` cannot consult [`crate::cli::colors::colors_enabled`]. The
//! constants are now `Sgr`, whose `Display` renders nothing when colour is off,
//! so gating happens at interpolation regardless of which printer does it.
//!
//! These tests pin the printers rather than the helper: the helper was already
//! covered while every one of these commands still leaked. `cargo test` captures
//! stdout, so `colors_enabled()` resolves to false here — the same state as a
//! redirected pipe or `--color never`.

use crate::cli::colors as c;
use crate::models::churn::{ChurnSummary, CodeChurnAnalysis, FileChurnMetrics};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;

/// The byte that must not appear in any output rendered with colour off.
const ESC: char = '\u{1b}';

fn assert_plain(what: &str, rendered: &str) {
    let leaking: Vec<&str> = rendered.lines().filter(|l| l.contains(ESC)).collect();
    assert!(
        leaking.is_empty(),
        "{what}: {} line(s) still carry ANSI with colour off; first: {:?}",
        leaking.len(),
        leaking.first()
    );
}

fn churn_analysis() -> CodeChurnAnalysis {
    let mut author_contributions = HashMap::new();
    author_contributions.insert("alice".to_string(), 4usize);
    CodeChurnAnalysis {
        generated_at: Utc::now(),
        period_days: 30,
        repository_root: PathBuf::from("/tmp/repo"),
        files: vec![
            FileChurnMetrics {
                path: PathBuf::from("src/hot.rs"),
                relative_path: "src/hot.rs".to_string(),
                commit_count: 42,
                unique_authors: vec!["alice".to_string()],
                additions: 100,
                deletions: 40,
                churn_score: 0.9,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
            FileChurnMetrics {
                path: PathBuf::from("src/warm.rs"),
                relative_path: "src/warm.rs".to_string(),
                commit_count: 7,
                unique_authors: vec!["bob".to_string()],
                additions: 20,
                deletions: 5,
                churn_score: 0.5,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
            FileChurnMetrics {
                path: PathBuf::from("src/cold.rs"),
                relative_path: "src/cold.rs".to_string(),
                commit_count: 1,
                unique_authors: vec!["carol".to_string()],
                additions: 2,
                deletions: 0,
                churn_score: 0.1,
                last_modified: Utc::now(),
                first_seen: Utc::now(),
            },
        ],
        summary: ChurnSummary {
            total_commits: 50,
            total_files_changed: 3,
            hotspot_files: vec![PathBuf::from("src/hot.rs")],
            stable_files: vec![PathBuf::from("src/cold.rs")],
            author_contributions,
            mean_churn_score: 0.5,
            variance_churn_score: 0.1,
            stddev_churn_score: 0.3,
        },
    }
}

#[test]
fn colour_is_off_in_this_test_binary() {
    // Every assertion below depends on this: `cargo test` captures stdout, so
    // the process is in exactly the state `--color never` produces.
    assert!(
        !c::colors_enabled(),
        "captured stdout must resolve colour to off"
    );
}

/// `analyze churn --color never` wrote 33 escape-bearing lines into a file, and
/// the same 33 with `--color always` — the flag changed nothing either way.
#[test]
fn churn_summary_is_plain_with_colour_off() {
    let rendered = super::format_churn_as_summary(&churn_analysis()).expect("churn summary");
    // Guard against a vacuous pass: the sections that carried the escapes must
    // actually be present in what we just asserted over.
    assert!(rendered.contains("Code Churn Analysis Summary"));
    assert!(rendered.contains("Top Files by Churn"));
    assert!(rendered.contains("Hotspot Files"));
    assert!(rendered.contains("Top Contributors"));
    assert_plain("format_churn_as_summary", &rendered);
}

/// `analyze complexity --color never` leaked on the two violation-count lines,
/// which are the only ones in that report built from raw constants.
#[test]
fn complexity_summary_is_plain_with_colour_off() {
    use crate::services::complexity::{
        ComplexityMetrics, ComplexityReport, ComplexitySummary, FileComplexityMetrics,
        FunctionComplexity, Violation,
    };

    let metrics = ComplexityMetrics {
        cyclomatic: 30,
        cognitive: 40,
        nesting_max: 5,
        lines: 200,
        halstead: None,
    };
    let report = ComplexityReport {
        summary: ComplexitySummary {
            total_files: 1,
            total_functions: 1,
            median_cyclomatic: 30.0,
            median_cognitive: 40.0,
            max_cyclomatic: 30,
            max_cognitive: 40,
            p90_cyclomatic: 30,
            p90_cognitive: 40,
            technical_debt_hours: 12.5,
        },
        violations: vec![
            Violation::Error {
                rule: "cyclomatic".to_string(),
                message: "too complex".to_string(),
                value: 30,
                threshold: 10,
                file: "src/a.rs".to_string(),
                line: 4,
                function: Some("big".to_string()),
            },
            Violation::Warning {
                rule: "cognitive".to_string(),
                message: "getting complex".to_string(),
                value: 40,
                threshold: 15,
                file: "src/a.rs".to_string(),
                line: 4,
                function: Some("big".to_string()),
            },
        ],
        hotspots: Vec::new(),
        files: vec![FileComplexityMetrics {
            path: "src/a.rs".to_string(),
            total_complexity: metrics,
            functions: vec![FunctionComplexity {
                name: "big".to_string(),
                line_start: 4,
                line_end: 40,
                metrics,
            }],
            classes: Vec::new(),
        }],
    };

    let rendered = crate::services::complexity::format_complexity_summary(&report);
    // The escapes lived on exactly these two lines; if the fixture stops
    // producing them the test would pass without exercising anything.
    assert!(rendered.contains("Errors:"), "{rendered}");
    assert!(rendered.contains("Warnings:"), "{rendered}");
    assert_plain("format_complexity_summary", &rendered);
}

/// The gate itself, stated once: with colour off, no `Sgr` renders bytes. This
/// is what makes the per-printer assertions above hold for printers nobody has
/// written a test for yet.
#[test]
fn no_sgr_constant_renders_bytes_with_colour_off() {
    for sgr in [
        c::RESET,
        c::BOLD,
        c::DIM,
        c::ITALIC,
        c::UNDERLINE,
        c::RED,
        c::GREEN,
        c::YELLOW,
        c::BLUE,
        c::MAGENTA,
        c::CYAN,
        c::WHITE,
        c::BOLD_RED,
        c::BOLD_GREEN,
        c::BOLD_YELLOW,
        c::BOLD_BLUE,
        c::BOLD_CYAN,
        c::BOLD_WHITE,
        c::DIM_WHITE,
        c::DIM_CYAN,
    ] {
        // `{sgr}` is the interpolation every leaking printer performs.
        assert_eq!(
            format!("{sgr}"),
            "",
            "constant {:?} still renders its escape with colour off",
            sgr.raw()
        );
    }
}
