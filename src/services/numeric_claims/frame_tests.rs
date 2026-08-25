//! Stage-1 framing: what counts as a claim about this repository, now.

use super::frame::{
    frame_file, frame_line, mask_spans, numeral_tokens, parse_numeral, scan_lines, structural_drop,
    Dropped, Frame,
};

fn tokens(line: &str) -> Vec<String> {
    frame_line("doc.md", 1, line)
        .into_iter()
        .map(|n| n.token)
        .collect()
}

// ------------------------------------------------------------- the scanner --

#[test]
fn numeral_scanner_reads_the_written_forms() {
    assert_eq!(numeral_tokens("1,291 files"), vec!["1,291"]);
    assert_eq!(numeral_tokens("50_000_000 bytes"), vec!["50_000_000"]);
    assert_eq!(numeral_tokens("70 crates"), vec!["70"]);
    assert_eq!(numeral_tokens("modal share 0.60 here"), vec!["0.60"]);
    assert_eq!(
        numeral_tokens("657 pre, 20 post, 0 inv, 293 contracts"),
        vec!["657", "20", "0", "293"]
    );
}

/// A numeral glued to an identifier, a version or a date is not a numeral. This
/// is what keeps versions, SHAs and paths out of every cohort without a single
/// special case downstream.
#[test]
fn numeral_scanner_rejects_glued_digits() {
    for s in [
        "released as v0.34.0 today",
        "version 3.32.0 shipped",
        "in crate c0 and c1",
        "dated 2026-08-24",
        "sha256 of the tree",
        "see file2.rs",
        "path a/2/b",
        "id_42 is taken",
    ] {
        assert_eq!(numeral_tokens(s), Vec::<String>::new(), "scanned {s:?}");
    }
}

#[test]
fn numeral_values_are_parsed_from_the_written_form() {
    assert_eq!(parse_numeral("1,291"), Some(1291.0));
    assert_eq!(parse_numeral("50_000_000"), Some(50_000_000.0));
    assert_eq!(parse_numeral("0.60"), Some(0.60));
    assert_eq!(parse_numeral(""), None);
}

/// Masking is POSITIONAL, never line-wide. Dropping the whole line when it
/// carries a version silently ate the flagship finding, because
/// `78 workspace crates` and `v0.34.0` share a line in aprender's READMEs.
#[test]
fn masking_is_positional_not_line_wide() {
    let line = "Now 78 workspace crates, as of 2026-08-24, shipped in v0.34.0 [12].";
    let masked = mask_spans(line);
    assert_eq!(
        masked.len(),
        line.len(),
        "masking must preserve byte offsets so pre/post context still aligns"
    );
    assert_eq!(numeral_tokens(&masked), vec!["78"]);
    assert_eq!(tokens(line), vec!["78"]);
}

// ------------------------------------------------------------- the frames --

#[test]
fn count_frame_admits_a_repo_artifact_noun() {
    for line in [
        "Part of the monorepo — 70 workspace crates.",
        "The suite has 20,781 tests.",
        "It ships 41 commands.",
        "Adds 12 contracts.",
    ] {
        let framed = frame_line("doc.md", 1, line);
        assert_eq!(framed.len(), 1, "not framed: {line:?}");
        assert_eq!(framed[0].frame, Frame::Count, "{line:?}");
    }
}

#[test]
fn assert_frame_admits_an_assertive_marker() {
    let framed = frame_line("doc.md", 1, "// Total: 657 preconditions, 20 post");
    assert!(!framed.is_empty(), "an assertive marker frames the numeral");
    assert_eq!(framed[0].frame, Frame::Assert);
    assert_eq!(framed[0].token, "657");
}

/// Policy is not measurement. A target that differs between two files is two
/// teams' intentions, not two answers to one question.
#[test]
fn anti_frame_drops_policy() {
    for line in [
        "Coverage should be at least 80 percent.",
        "Complexity must stay under 20 functions.",
        "The goal is 95 tests.",
        "Requires >= 3 files.",
        "Budget: 150 checks.",
        "Estimated 40 contracts.",
        "The threshold is 10 violations.",
    ] {
        assert_eq!(
            structural_drop(line),
            Some(Dropped::AntiFrame),
            "policy survived framing: {line:?}"
        );
        assert!(frame_line("doc.md", 1, line).is_empty(), "{line:?}");
    }
}

#[test]
fn structural_drops_have_the_reason_attached() {
    let cases = [
        ("| tests | 1904 |", Dropped::TableRow),
        ("The full suite takes 30 minutes.", Dropped::Duration),
        (
            "Before: 570 violations in shipped code.",
            Dropped::PastState,
        ),
        ("Reduced from 1904 tests.", Dropped::PastState),
        ("// let n = compute(3);", Dropped::Code),
        ("### 3.1 Coverage Requirements", Dropped::SectionHeading),
        (
            "Scores are in the range (0.0 - 1.0).",
            Dropped::RangeEndpoint,
        ),
    ];
    for (line, reason) in cases {
        assert_eq!(structural_drop(line), Some(reason), "{line:?}");
        assert!(frame_line("doc.md", 1, line).is_empty(), "{line:?}");
    }
    assert_eq!(structural_drop("There are 70 crates."), None);
}

/// A year is a coordinate, not a count.
#[test]
fn years_are_not_counts() {
    assert!(tokens("There are 2026 files").is_empty());
    assert_eq!(tokens("There are 2101 files"), vec!["2101"]);
}

/// The price of the year rule, recorded rather than discovered later: a count
/// that happens to land in 1900..=2100 is invisible to R1. This repository's
/// own test count sits there, so CB-200's 1905-against-1904 could not be seen
/// even with seven sites — `.pmat-ratchet.toml` catches that one by
/// re-derivation, and R1 does not pretend to.
#[test]
fn counts_inside_the_year_band_are_invisible() {
    assert!(tokens("The suite has 1,904 tests.").is_empty());
    assert!(tokens("There are 2,026 contracts.").is_empty());
    assert_eq!(tokens("The suite has 1,899 tests."), vec!["1,899"]);
}

// ------------------------------------------------------------ line sources --

/// `*.rs` contributes COMMENTS only. A numeric literal in an expression is an
/// operand, not a claim, and reading them was 95% of the corpus for none of the
/// signal.
#[test]
fn rust_contributes_comments_only() {
    let text = "const CHUNK: usize = 8 * 1024 * 1024;\n\
                // There are 70 workspace crates.\n\
                let n = 12 tests;\n";
    let lines = scan_lines("src/a.rs", text);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].0, 2);
    assert_eq!(frame_file("src/a.rs", text).len(), 1);
}

#[test]
fn markdown_fenced_code_is_not_prose() {
    let text = "There are 70 crates.\n```\nlet x = 99 crates;\n```\nAnd 71 crates now.\n";
    let lines: Vec<usize> = scan_lines("doc.md", text)
        .into_iter()
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        lines,
        vec![1, 5],
        "fence markers and their body are skipped"
    );
}

#[test]
fn toml_and_yaml_contribute_key_lines() {
    let toml = "[thresholds]\nmax_unwrap_calls = 100  # Current: 570 violations\n";
    assert!(!scan_lines("a.toml", toml).is_empty());
    let yaml = "coverage:\n  target: 95  # there are 95 checks\n";
    assert!(!scan_lines("a.yml", yaml).is_empty());
}

#[test]
fn context_is_the_trimmed_source_line() {
    let framed = frame_line("doc.md", 9, "    There are 70 crates.   ");
    assert_eq!(framed.len(), 1);
    assert_eq!(framed[0].context, "There are 70 crates.");
    assert_eq!(framed[0].line, 9);
    assert_eq!(framed[0].file, "doc.md");
    assert_eq!(framed[0].value, 70.0);
}
