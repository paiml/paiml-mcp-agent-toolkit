//! What the command line accepts for the two verbs.

use crate::cli::commands::Cli;
use clap::Parser;

/// Whether `pmat work <args>` parses. The full `Cli` derive needs more stack than a
/// test thread has.
fn work_parses(args: &[&str]) -> Result<(), String> {
    let argv: Vec<String> = ["pmat", "work"]
        .iter()
        .chain(args)
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
        .expect("parsing `pmat work` must not panic")
}

#[test]
fn ticket_quality_cli_issue_mode_refuses_the_flags_it_would_ignore() {
    let issue = ["score", "--issue", "3", "--repo", "o/r", "--json"];
    assert_eq!(work_parses(&issue), Ok(()));
    let contract = ["score", "PMAT-1", "--min-score", "0.5", "--format", "json"];
    assert_eq!(work_parses(&contract), Ok(()));
    for flag in [["--min-score", "0.5"], ["--format", "json"]] {
        let err = work_parses(&["score", "--issue", "3", flag[0], flag[1]]).unwrap_err();
        assert!(
            err.contains("cannot be used with") && err.contains(flag[0]),
            "{err}"
        );
    }
}

#[test]
fn ticket_quality_cli_findings_refuses_numbers_that_would_panic_or_hide_findings() {
    for ok in [
        "--duplicate-threshold=0",
        "--duplicate-threshold=1",
        "--stale-ticket-days=0",
        "--stale-ticket-days=36500",
    ] {
        assert_eq!(work_parses(&["findings", ok]), Ok(()), "{ok}");
    }
    for bad in [
        "--duplicate-threshold=NaN",
        "--duplicate-threshold=1.5",
        "--duplicate-threshold=-0.1",
        "--duplicate-threshold=x",
        "--stale-ticket-days=-1",
        "--stale-ticket-days=999999999999999",
    ] {
        let err = work_parses(&["findings", bad]).unwrap_err();
        assert!(err.contains("invalid value"), "{bad}: {err}");
    }
}
