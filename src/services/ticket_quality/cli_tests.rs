//! What the command line accepts for the two verbs.

use crate::cli::commands::Cli;
use clap::Parser;

/// Whether `pmat work score <extra>` parses. The full `Cli` derive needs more stack
/// than a test thread has.
fn score_parses(extra: &[&str]) -> Result<(), String> {
    let argv: Vec<String> = ["pmat", "work", "score"]
        .iter()
        .chain(extra)
        .map(|s| (*s).to_string())
        .collect();
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            Cli::try_parse_from(&argv)
                .map(|_| ())
                .map_err(|e| e.to_string())
        })
        .expect("spawn the parse thread")
        .join()
        .expect("parsing `pmat work score` must not panic")
}

#[test]
fn ticket_quality_cli_issue_mode_refuses_the_flags_it_would_ignore() {
    assert_eq!(
        score_parses(&["--issue", "3", "--repo", "o/r", "--json"]),
        Ok(())
    );
    assert_eq!(
        score_parses(&["PMAT-1", "--min-score", "0.5", "--format", "json"]),
        Ok(())
    );
    for flag in [["--min-score", "0.5"], ["--format", "json"]] {
        let err = score_parses(&["--issue", "3", flag[0], flag[1]]).unwrap_err();
        assert!(
            err.contains("cannot be used with") && err.contains(flag[0]),
            "{err}"
        );
    }
}
