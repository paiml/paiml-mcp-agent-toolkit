//! Duplicate detection analysis - finds duplicate code blocks

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
// `HashMap` is still used by `find_duplicate_blocks` for hash grouping (order
// there is erased by an explicit sort); every map that reaches OUTPUT is a
// `BTreeMap` so JSON key order cannot vary between runs.
use std::path::{Path, PathBuf};

include!("duplicates_detection.rs");
include!("duplicates_extraction.rs");
include!("duplicates_output.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod near_miss_tests {
    //! `structural_similarities` counted duplicate blocks with
    //! `0.8 <= similarity < 1.0`, but the only production constructor of a
    //! block set `similarity: 1.0` as a literal — detection was pure hash
    //! bucketing — so the predicate was unsatisfiable for EVERY input. The JSON
    //! printed a hard 0 beside `exact_duplicates: 176` and
    //! `duplication_percentage: 72.46` on a corpus of 121 files, and 0 on an
    //! empty directory: the same number for "no near-misses here" and "we never
    //! looked".
    use super::*;

    /// Two functions that do the same work in nearly the same way: `beta` adds
    /// one more guarded branch. Structurally similar, not identical, and not
    /// reachable by any hash-equality test.
    const ALPHA: &str = "\
pub fn tally_alpha(values: &[i64], limit: i64) -> i64 {
    let mut total = 0i64;
    let mut seen = 0usize;
    for value in values {
        if *value > limit {
            total = total.wrapping_add(value * 3);
            seen += 1;
        } else if *value < 0 {
            total = total.wrapping_sub(value / 2);
        } else {
            total = total.wrapping_add(1);
        }
        if seen > 100 {
            break;
        }
    }
    total
}
";

    const BETA: &str = "\
pub fn tally_beta(items: &[i64], bound: i64) -> i64 {
    let mut sum = 0i64;
    let mut counted = 0usize;
    for item in items {
        if *item > bound {
            sum = sum.wrapping_add(item * 3);
            counted += 1;
        } else if *item < 0 {
            sum = sum.wrapping_sub(item / 2);
        } else {
            sum = sum.wrapping_add(1);
        }
        if counted % 7 == 0 {
            sum = sum.wrapping_mul(2);
        }
        if counted > 100 {
            break;
        }
    }
    sum
}
";

    async fn report_for(detection: crate::cli::DuplicateType, threshold: f32) -> DuplicateReport {
        let temp = tempfile::TempDir::new().unwrap();
        std::fs::write(temp.path().join("alpha.rs"), ALPHA).unwrap();
        std::fs::write(temp.path().join("beta.rs"), BETA).unwrap();

        detect_duplicates(temp.path(), detection, threshold, 5, 1000, &None, &None)
            .await
            .expect("detection must succeed")
    }

    fn structural_similarities(report: &DuplicateReport) -> usize {
        let json: serde_json::Value =
            serde_json::from_str(&format_json_output(report).unwrap()).unwrap();
        json["structural_similarities"].as_u64().unwrap() as usize
    }

    /// The leaf itself: a near-miss clone pair must move `structural_similarities`
    /// off zero, under the DEFAULT detection type.
    #[tokio::test]
    async fn a_near_miss_clone_pair_is_counted_as_a_structural_similarity() {
        let report = report_for(crate::cli::DuplicateType::All, 0.7).await;
        assert!(
            structural_similarities(&report) > 0,
            "two near-miss clones must be reported as a structural similarity, got blocks {:?}",
            report
                .duplicate_blocks
                .iter()
                .map(|b| (b.hash.clone(), b.similarity))
                .collect::<Vec<_>>()
        );
    }

    /// And the similarity carried is a MEASUREMENT, not the cut-off echoed back
    /// and not the literal 1.0: it sits strictly between the threshold and 1.0,
    /// and does not move when the threshold does.
    #[tokio::test]
    async fn the_reported_similarity_is_measured_not_the_threshold() {
        let loose = report_for(crate::cli::DuplicateType::All, 0.5).await;
        let tighter = report_for(crate::cli::DuplicateType::All, 0.6).await;

        let near_miss = |r: &DuplicateReport| -> Vec<f32> {
            let mut sims: Vec<f32> = r
                .duplicate_blocks
                .iter()
                .filter(|b| b.similarity < 1.0)
                .map(|b| b.similarity)
                .collect();
            sims.sort_by(f32::total_cmp);
            sims
        };

        let loose_sims = near_miss(&loose);
        assert!(!loose_sims.is_empty(), "no near-miss block was reported");
        for sim in &loose_sims {
            assert!(
                *sim > 0.6 && *sim < 1.0,
                "similarity {sim} is neither the 0.5 cut-off nor an exact match"
            );
        }
        assert_eq!(
            loose_sims,
            near_miss(&tighter),
            "the measured similarity must not move with --threshold"
        );
    }

    /// `--threshold` now decides what is close enough, so raising it past the
    /// measured similarity must drop the pair. A flag that changes the answer is
    /// the only proof it is not inert.
    #[tokio::test]
    async fn the_threshold_selects_which_near_misses_survive() {
        let loose = report_for(crate::cli::DuplicateType::All, 0.5).await;
        let impossible = report_for(crate::cli::DuplicateType::All, 0.999).await;

        assert!(structural_similarities(&loose) > 0);
        assert_eq!(
            structural_similarities(&impossible),
            0,
            "nothing is 99.9% similar here, so the count must fall to zero"
        );
    }

    /// `exact` is Type-1 by definition: a zero there is a measurement of what
    /// was looked for, not a missing detector.
    #[tokio::test]
    async fn exact_mode_reports_no_structural_similarities() {
        let report = report_for(crate::cli::DuplicateType::Exact, 0.5).await;
        assert_eq!(structural_similarities(&report), 0);
    }

    /// An empty project still reports zero — and now that zero means something,
    /// because the same command reports non-zero on the fixture above.
    #[tokio::test]
    async fn an_empty_project_still_reports_zero() {
        let temp = tempfile::TempDir::new().unwrap();
        let report = detect_duplicates(
            temp.path(),
            crate::cli::DuplicateType::All,
            0.7,
            5,
            1000,
            &None,
            &None,
        )
        .await
        .unwrap();
        assert_eq!(structural_similarities(&report), 0);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn strip_ansi(s: &str) -> String {
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        re.replace_all(s, "").to_string()
    }

    #[test]
    fn test_normalize_block() {
        let lines = vec!["  fn test() {", "    // comment", "    let x = 1;", "  }"];
        let normalized = normalize_block(&lines);
        assert!(!normalized.contains("// comment"));
        assert!(normalized.contains("fn test()"));
        assert_eq!(normalized, "fn test() {\nlet x = 1;\n}");
    }

    #[test]
    fn test_count_tokens() {
        assert_eq!(count_tokens("fn test() { }"), 4);
        assert_eq!(count_tokens("let x = 1;"), 4);
        assert_eq!(count_tokens(""), 0);
        assert_eq!(count_tokens("  \n  \t  "), 0);
    }

    #[test]
    fn test_is_function_declaration() {
        assert!(is_function_declaration("fn main() {"));
        assert!(is_function_declaration("function test() {"));
        assert!(is_function_declaration("def calculate():"));
        assert!(!is_function_declaration("let x = 1;"));
    }

    #[test]
    fn test_is_type_declaration() {
        assert!(is_type_declaration("class Foo {"));
        assert!(is_type_declaration("struct Bar {"));
        assert!(is_type_declaration("impl Display for Foo {"));
        assert!(!is_type_declaration("let x = 1;"));
    }

    #[test]
    fn test_is_block_opening() {
        assert!(is_block_opening("fn main() {"));
        assert!(is_block_opening("if true {"));
        assert!(!is_block_opening("{ x: 1 }"));
        assert!(!is_block_opening("let x = 1;"));
    }

    #[test]
    fn test_is_block_start() {
        // Function declarations
        assert!(is_block_start("fn main() {"));
        assert!(is_block_start("function test() {"));
        assert!(is_block_start("def calculate():"));

        // Type declarations
        assert!(is_block_start("class Foo {"));
        assert!(is_block_start("struct Bar {"));
        assert!(is_block_start("impl Display for Foo {"));

        // Block openings
        assert!(is_block_start("if condition {"));

        // Not block starts
        assert!(!is_block_start("let x = 1;"));
        assert!(!is_block_start("{ x: 1 }"));
    }

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("test.rs")));
        assert!(is_source_file(Path::new("test.js")));
        assert!(is_source_file(Path::new("test.ts")));
        assert!(is_source_file(Path::new("test.py")));
        assert!(is_source_file(Path::new("test.java")));
        assert!(is_source_file(Path::new("test.cpp")));
        assert!(is_source_file(Path::new("test.c")));
        assert!(is_source_file(Path::new("test.kt")));
        assert!(is_source_file(Path::new("test.kts")));
        assert!(!is_source_file(Path::new("test.txt")));
        assert!(!is_source_file(Path::new("README.md")));
    }

    #[test]
    fn test_should_process_file() {
        let path = Path::new("src/main.rs");

        // No filters
        assert!(should_process_file(path, &None, &None));

        // Include filter
        assert!(should_process_file(path, &Some("src".to_string()), &None));
        assert!(!should_process_file(
            path,
            &Some("tests".to_string()),
            &None
        ));

        // Exclude filter
        assert!(!should_process_file(path, &None, &Some("src".to_string())));
        assert!(should_process_file(path, &None, &Some("tests".to_string())));

        // Both filters (exclude takes precedence)
        assert!(!should_process_file(
            path,
            &Some("src".to_string()),
            &Some("src".to_string())
        ));
    }

    #[test]
    fn test_find_block_end() {
        let lines = vec![
            "fn test() {",
            "    let x = 1;",
            "    if true {",
            "        println!(\"hello\");",
            "    }",
            "}",
        ];

        assert_eq!(find_block_end(&lines), Some(6));

        let lines2 = vec!["fn test() {", "    let x = 1;"];
        assert_eq!(find_block_end(&lines2), None);
    }

    #[test]
    fn test_extract_exact_blocks() {
        let lines = vec![
            "fn test1() {",
            "    let x = 1;",
            "    println!(\"x = {}\", x);",
            "}",
            "",
            "fn test2() {",
            "    let y = 2;",
            "    println!(\"y = {}\", y);",
            "}",
        ];

        let mut blocks = Vec::new();
        extract_exact_blocks(&mut blocks, &lines, "test.rs", 3, 100);

        // Should find multiple sliding windows
        assert!(!blocks.is_empty());
        assert!(blocks.iter().all(|(_, file, _, _, _)| file == "test.rs"));
    }

    /// A run of comments is not a code block, so it cannot be half of a
    /// duplicate pair.
    ///
    /// `normalize_block` DELETES comments and blank lines, and the exact pass
    /// hashed whatever it left behind with only an upper bound on tokens
    /// (`--max-tokens`) and no lower bound. A window covering nothing but
    /// comments therefore normalised to `""` and hashed to `30406ea523c53def`
    /// — the hash of the empty string — in EVERY file of the project, so all of
    /// them landed in one bucket and were reported as one exact duplicate at
    /// similarity 1.0 with `tokens: 0`.
    #[test]
    fn comment_only_windows_are_not_blocks() {
        let lines = vec![
            "// alpha one",
            "// alpha two",
            "// alpha three",
            "// alpha four",
            "// alpha five",
            "",
            "   ",
        ];

        let mut blocks = Vec::new();
        extract_exact_blocks(&mut blocks, &lines, "a.rs", 5, 100);

        assert!(
            blocks.is_empty(),
            "comments and blank lines carry no code, so no block can be cut \
             from them; got {blocks:?}"
        );
    }

    /// Every block the exact pass emits must carry `min_lines` lines of real
    /// code, and must report the span of that code rather than the span of the
    /// comments around it.
    #[test]
    fn exact_blocks_span_only_substantive_lines() {
        let lines = vec![
            "// leading comment",
            "",
            "let a = 1;",
            "let b = 2;",
            "// interleaved note",
            "let c = 3;",
            "let d = 4;",
            "let e = 5;",
            "",
            "// trailing comment",
        ];

        let mut blocks = Vec::new();
        extract_exact_blocks(&mut blocks, &lines, "a.rs", 5, 100);

        assert_eq!(
            blocks.len(),
            1,
            "five substantive lines admit exactly one 5-line window"
        );
        let (_, _, start, end, content) = &blocks[0];
        // Lines 3..=8 in 1-based terms, i.e. `let a` through `let e`, skipping
        // the note on line 5.
        assert_eq!((*start, *end), (3, 8), "block must span the code it holds");
        assert!(
            !content.contains("//"),
            "normalised content must hold no comments: {content:?}"
        );
        assert_eq!(
            content.lines().count(),
            5,
            "a 5-line block must have 5 lines of evidence: {content:?}"
        );
    }

    #[test]
    fn test_find_duplicate_blocks_no_duplicates() {
        let blocks = vec![
            (
                "hash1".to_string(),
                "file1.rs".to_string(),
                1,
                10,
                "content1".to_string(),
            ),
            (
                "hash2".to_string(),
                "file2.rs".to_string(),
                1,
                10,
                "content2".to_string(),
            ),
        ];

        let duplicates = find_duplicate_blocks(blocks);
        assert!(duplicates.is_empty());
    }

    /// DETERMINISM (round-3 sweep): `find_duplicate_blocks` iterated a
    /// `HashMap` of hash groups and then sorted only by `lines` — a stable sort,
    /// so every block of equal length kept the map's per-process order.
    /// `analyze duplicates --format json` on a fixed two-file fixture produced
    /// 5 DIFFERENT md5 sums over 5 runs: the same 14 block hashes, reordered
    /// (run 1 began 9d399b, 272e45, f64c5c, ff7a0d…; run 2 began 9d399b,
    /// a6cbc6, ff7a0d, a2e02d…).
    ///
    /// Every group here is the SAME length, so `lines` decides nothing and the
    /// tie-break is the whole answer. Each iteration builds a fresh `HashMap`
    /// inside the function: the in-process stand-in for re-running the binary.
    #[test]
    fn duplicate_block_order_is_stable_across_fresh_hash_maps() {
        fn order() -> Vec<String> {
            let mut blocks = Vec::new();
            // Ten 6-line duplicate pairs, all identical in length.
            for group in 0..10 {
                for site in 0..2 {
                    blocks.push((
                        format!("hash{group:02}"),
                        format!("file{site}.rs"),
                        1 + group * 10,
                        6 + group * 10,
                        format!("body{group}"),
                    ));
                }
            }
            find_duplicate_blocks(blocks)
                .into_iter()
                .map(|b| b.hash)
                .collect()
        }

        let first = order();
        assert_eq!(first.len(), 10, "fixture must produce ten tied blocks");
        for i in 0..10 {
            assert_eq!(
                order(),
                first,
                "iteration {i}: identical input must produce an identical block order"
            );
        }
        // Ties resolve on (first location, hash), so the order is a property of
        // the code rather than of the hash seed.
        let mut expected = first.clone();
        expected.sort();
        assert_eq!(first, expected);
    }

    #[test]
    fn test_find_duplicate_blocks_with_duplicates() {
        let blocks = vec![
            (
                "hash1".to_string(),
                "file1.rs".to_string(),
                1,
                10,
                "content1".to_string(),
            ),
            (
                "hash1".to_string(),
                "file2.rs".to_string(),
                20,
                29,
                "content1".to_string(),
            ),
            (
                "hash2".to_string(),
                "file3.rs".to_string(),
                1,
                5,
                "content2".to_string(),
            ),
        ];

        let duplicates = find_duplicate_blocks(blocks);
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].hash, "hash1");
        assert_eq!(duplicates[0].locations.len(), 2);
        assert_eq!(duplicates[0].lines, 10);
    }

    #[test]
    fn test_file_stats_calculation() {
        let mut stats = FileStats {
            duplicate_lines: 20,
            total_lines: 100,
            duplication_percentage: 0.0,
        };

        // Calculate percentage
        stats.duplication_percentage =
            (stats.duplicate_lines as f32 / stats.total_lines as f32) * 100.0;
        assert_eq!(stats.duplication_percentage, 20.0);
    }

    #[tokio::test]
    async fn test_detect_duplicates_empty_project() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let result = detect_duplicates(
            temp_dir.path(),
            crate::cli::DuplicateType::Exact,
            0.8,
            5,
            100,
            &None,
            &None,
        )
        .await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.total_duplicates, 0);
        assert_eq!(report.duplicate_lines, 0);
        assert_eq!(report.total_lines, 0);
        assert_eq!(report.duplication_percentage, 0.0);
    }

    #[test]
    fn test_format_json_output() {
        let report = DuplicateReport {
            total_duplicates: 1,
            duplicate_lines: 10,
            total_lines: 100,
            duplication_percentage: 10.0,
            duplicate_blocks: vec![],
            file_statistics: BTreeMap::new(),
        };

        let result = format_json_output(&report);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("\"total_duplicates\": 1"));
        assert!(json.contains("\"duplication_percentage\": 10.0"));
    }

    #[test]
    fn test_format_human_output() {
        let report = DuplicateReport {
            total_duplicates: 2,
            duplicate_lines: 20,
            total_lines: 100,
            duplication_percentage: 20.0,
            duplicate_blocks: vec![DuplicateBlock {
                hash: "hash1".to_string(),
                locations: vec![
                    DuplicateLocation {
                        file: "file1.rs".to_string(),
                        start_line: 10,
                        end_line: 20,
                        content_preview: "fn test() {".to_string(),
                    },
                    DuplicateLocation {
                        file: "file2.rs".to_string(),
                        start_line: 30,
                        end_line: 40,
                        content_preview: "fn test() {".to_string(),
                    },
                ],
                lines: 10,
                tokens: 20,
                similarity: 1.0,
            }],
            file_statistics: BTreeMap::new(),
        };

        let result = format_human_output(&report);
        assert!(result.is_ok());
        let output = strip_ansi(&result.unwrap());
        assert!(output.contains("Duplicate Code Analysis"));
        assert!(output.contains("Total duplicate blocks:"));
        assert!(output.contains("2"));
        assert!(output.contains("Block 1"));
        assert!(output.contains("10 lines, 2 locations"));
    }

    // ─────────────────────────────────────────────────────────────────────
    // duplicates_output.rs: format_output dispatcher + sarif/csv +
    // write_top_files_section + write_remaining_blocks_count + extract_filename
    // ─────────────────────────────────────────────────────────────────────

    fn populated_report_with_blocks(n_blocks: usize) -> DuplicateReport {
        let blocks = (0..n_blocks)
            .map(|i| DuplicateBlock {
                hash: format!("h{i}"),
                locations: vec![
                    DuplicateLocation {
                        file: format!("a-{i}.rs"),
                        start_line: 1,
                        end_line: 5,
                        content_preview: "preview".to_string(),
                    },
                    DuplicateLocation {
                        file: format!("b-{i}.rs"),
                        start_line: 1,
                        end_line: 5,
                        content_preview: "preview".to_string(),
                    },
                ],
                lines: 5,
                tokens: 10,
                similarity: 1.0,
            })
            .collect();
        let mut file_stats = BTreeMap::new();
        file_stats.insert(
            "src/x.rs".to_string(),
            FileStats {
                duplicate_lines: 10,
                total_lines: 50,
                duplication_percentage: 20.0,
            },
        );
        file_stats.insert(
            "src/y.rs".to_string(),
            FileStats {
                duplicate_lines: 5,
                total_lines: 100,
                duplication_percentage: 5.0,
            },
        );
        DuplicateReport {
            total_duplicates: n_blocks,
            duplicate_lines: n_blocks * 5,
            total_lines: 1000,
            duplication_percentage: 5.0,
            duplicate_blocks: blocks,
            file_statistics: file_stats,
        }
    }

    #[test]
    fn test_format_output_dispatcher_human_arm() {
        let r = populated_report_with_blocks(1);
        let out = format_output(&r, crate::cli::DuplicateOutputFormat::Human).unwrap();
        let stripped = strip_ansi(&out);
        assert!(stripped.contains("Duplicate Code Analysis"));
    }

    #[test]
    fn test_format_output_dispatcher_summary_and_detailed_arms() {
        let r = populated_report_with_blocks(1);
        // Summary + Detailed used to share the human-output arm, which is what
        // made all three formats byte-identical; each renders its own detail
        // level now, so assert the difference rather than only that both
        // succeed (see `summary_omits_the_per_block_listing`).
        let summary = format_output(&r, crate::cli::DuplicateOutputFormat::Summary).unwrap();
        let detailed = format_output(&r, crate::cli::DuplicateOutputFormat::Detailed).unwrap();
        assert!(!strip_ansi(&summary).contains("Duplicate Blocks"));
        assert!(strip_ansi(&detailed).contains("Duplicate Blocks"));
    }

    #[test]
    fn test_format_output_dispatcher_json_arm() {
        let r = populated_report_with_blocks(2);
        let out = format_output(&r, crate::cli::DuplicateOutputFormat::Json).unwrap();
        assert!(out.contains("\"total_duplicates\""));
        assert!(out.contains("\"metrics\""));

        // This assertion used to require `entropy_analysis` to be PRESENT, which
        // pinned a fabrication: the block was emitted with the constants
        // average_entropy 0.5 / high_entropy_blocks 0 in every run, and this
        // command performs no entropy analysis at all. The test was enforcing
        // the bug. It now enforces the fix.
        assert!(
            !out.contains("\"entropy_analysis\""),
            "duplicates JSON must not emit an entropy block it never measured"
        );
        assert!(
            !out.contains("\"analysis_time_ms\""),
            "duplicates JSON must not emit a hardcoded analysis time"
        );
    }

    #[test]
    fn test_format_output_dispatcher_sarif_arm() {
        let r = populated_report_with_blocks(2);
        let out = format_output(&r, crate::cli::DuplicateOutputFormat::Sarif).unwrap();
        assert!(out.contains("\"version\": \"2.1.0\""));
        assert!(out.contains("duplicate-code"));
    }

    #[test]
    fn test_format_output_dispatcher_csv_arm() {
        let r = populated_report_with_blocks(2);
        let out = format_output(&r, crate::cli::DuplicateOutputFormat::Csv).unwrap();
        assert!(out.starts_with("Type,File1,Start1,End1,File2,Start2,End2\n"));
        // 2 blocks → 2 data rows.
        assert_eq!(out.lines().filter(|l| l.starts_with("exact,")).count(), 2);
    }

    #[test]
    fn test_format_csv_skips_blocks_with_under_two_locations() {
        let mut r = populated_report_with_blocks(0);
        r.duplicate_blocks.push(DuplicateBlock {
            hash: "single".to_string(),
            locations: vec![DuplicateLocation {
                file: "only.rs".to_string(),
                start_line: 1,
                end_line: 2,
                content_preview: String::new(),
            }],
            lines: 1,
            tokens: 1,
            similarity: 1.0,
        });
        let out = format_csv_output(&r).unwrap();
        // Header only — single-location block dropped.
        assert_eq!(out, "Type,File1,Start1,End1,File2,Start2,End2\n");
    }

    #[test]
    fn test_human_output_with_file_stats_emits_top_files() {
        let r = populated_report_with_blocks(1);
        let out = strip_ansi(&format_human_output(&r).unwrap());
        // file_statistics non-empty → "Top Files by Duplication" section emitted.
        assert!(out.contains("Top Files by Duplication"));
        // Filenames extracted from full paths.
        assert!(out.contains("x.rs"));
        assert!(out.contains("y.rs"));
    }

    #[test]
    fn test_human_output_skips_top_files_when_stats_empty() {
        let mut r = populated_report_with_blocks(0);
        r.file_statistics.clear();
        let out = strip_ansi(&format_human_output(&r).unwrap());
        assert!(!out.contains("Top Files by Duplication"));
    }

    #[test]
    fn test_human_output_with_more_than_20_blocks_shows_remaining_count() {
        let r = populated_report_with_blocks(25);
        let out = strip_ansi(&format_human_output(&r).unwrap());
        // First 20 emitted; remainder shown as "... and 5 more blocks".
        assert!(out.contains("... and 5 more blocks"));
    }

    #[test]
    fn test_human_output_no_remaining_count_when_blocks_le_20() {
        let r = populated_report_with_blocks(15);
        let out = strip_ansi(&format_human_output(&r).unwrap());
        assert!(!out.contains("more blocks"));
    }

    /// Regression: "Top Files by Duplication" printed `Path::file_name()`, so
    /// this repo's two `core_tests_properties.rs` files rendered as the SAME
    /// row twice — identical name, identical counts, identical percentage —
    /// occupying two of the ten slots with no way to tell them apart.
    #[test]
    fn test_top_files_distinguishes_files_sharing_a_basename() {
        let mut r = populated_report_with_blocks(0);
        r.file_statistics.clear();
        r.file_statistics.insert(
            "./src/ast/core_tests_properties.rs".to_string(),
            FileStats {
                duplicate_lines: 292,
                total_lines: 292,
                duplication_percentage: 100.0,
            },
        );
        r.file_statistics.insert(
            "./src/ast/core/core_tests_properties.rs".to_string(),
            FileStats {
                duplicate_lines: 292,
                total_lines: 292,
                duplication_percentage: 100.0,
            },
        );

        let out = strip_ansi(&format_human_output(&r).unwrap());
        let rows: Vec<&str> = out
            .lines()
            .filter(|l| l.contains("core_tests_properties.rs"))
            .collect();
        assert_eq!(rows.len(), 2, "both files listed:\n{out}");
        assert_ne!(
            rows[0].trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ' '),
            rows[1].trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ' '),
            "two different files must not render as the same row:\n{out}"
        );
        assert!(
            out.contains("src/ast/core/core_tests_properties.rs"),
            "{out}"
        );
        // The leading "./" of the analyzer's key is trimmed, nothing else.
        assert!(!out.contains("./src/ast"), "{out}");
    }

    #[test]
    fn test_get_sorted_file_stats_sorts_by_dup_pct_desc() {
        let mut stats = BTreeMap::new();
        stats.insert(
            "low.rs".to_string(),
            FileStats {
                duplicate_lines: 1,
                total_lines: 100,
                duplication_percentage: 1.0,
            },
        );
        stats.insert(
            "high.rs".to_string(),
            FileStats {
                duplicate_lines: 50,
                total_lines: 100,
                duplication_percentage: 50.0,
            },
        );
        stats.insert(
            "mid.rs".to_string(),
            FileStats {
                duplicate_lines: 10,
                total_lines: 100,
                duplication_percentage: 10.0,
            },
        );
        let sorted = get_sorted_file_stats(&stats);
        // Descending order: high → mid → low.
        assert_eq!(sorted[0].0, "high.rs");
        assert_eq!(sorted[1].0, "mid.rs");
        assert_eq!(sorted[2].0, "low.rs");
    }

    // ── per-file duplication can never exceed the file ──────────────────────

    /// Round 3: round 2 fixed only the aggregate. Two byte-identical 17-line
    /// files still reported `file_statistics[a.rs] = {duplicate_lines: 76,
    /// total_lines: 17, duplication_percentage: 447.06}` — a part 4.5x its own
    /// whole — because overlapping sliding windows that hash differently were
    /// still summed per file.
    #[tokio::test]
    async fn test_per_file_duplication_never_exceeds_the_file() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        let body = (0..17)
            .map(|i| format!("    let v{} = {} + 1;\n", i % 4, i))
            .collect::<String>();
        std::fs::write(temp.path().join("a.rs"), &body).unwrap();
        std::fs::write(temp.path().join("b.rs"), &body).unwrap();

        for detection in [
            crate::cli::DuplicateType::Exact,
            crate::cli::DuplicateType::All,
        ] {
            let report = detect_duplicates(temp.path(), detection, 0.8, 5, 100, &None, &None)
                .await
                .expect("detection must not fail");

            for (path, stats) in &report.file_statistics {
                assert!(
                    stats.duplicate_lines <= stats.total_lines,
                    "{path}: {} duplicate lines in a {}-line file",
                    stats.duplicate_lines,
                    stats.total_lines
                );
                assert!(
                    stats.duplication_percentage <= 100.0,
                    "{path}: {}% duplication",
                    stats.duplication_percentage
                );
            }
            assert!(
                report.duplicate_lines <= report.total_lines,
                "project total {} > {} lines counted",
                report.duplicate_lines,
                report.total_lines
            );
            assert!(report.duplication_percentage <= 100.0);
            // The headline must be the sum of the rows it heads.
            let row_sum: usize = report
                .file_statistics
                .values()
                .map(|s| s.duplicate_lines)
                .sum();
            assert_eq!(
                report.duplicate_lines, row_sum,
                "the project total must agree with the per-file rows"
            );
        }
    }

    /// Two files that share ZERO lines share zero duplication.
    ///
    /// BLOCKER (round-5 sweep). The fixture below is the minimal one: `comm -12`
    /// over the two files sorted is EMPTY, and `analyze duplicates` reported
    ///
    /// ```text
    ///   Total duplicate blocks: 1
    ///   Duplication percentage: 62.5%
    ///   exact_duplicates: 1
    ///   hash 30406ea523c53def, tokens: 0, similarity: 1.0, content_preview: ""
    /// ```
    ///
    /// The block was lines 1-5 of both files: their comment headers, which
    /// `normalize_block` deletes, leaving the empty string, whose hash is
    /// `30406ea523c53def`. A detector that calls two unrelated files two-thirds
    /// duplicated is worse than one that finds nothing — it points the wrong
    /// way, and `--duplicates` is how this project hunts its own copy-paste.
    ///
    /// Every detection type is checked: `exact` hashes normalised source and
    /// `fuzzy`/`renamed` hash a structural signature, but both were fed by
    /// extractors with no lower bound on content.
    #[tokio::test]
    async fn files_sharing_no_lines_report_no_duplication() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("a.rs"),
            "// alpha one\n// alpha two\n// alpha three\n// alpha four\n// alpha five\n\
             \n// alpha six\n// alpha seven\n\
             fn alpha_unique_function_name() { let alpha_value = 111; \
             println!(\"alpha {}\", alpha_value); }\n",
        )
        .unwrap();
        std::fs::write(
            temp.path().join("b.rs"),
            "// beta one\n// beta two\n// beta three\n// beta four\n// beta five\n\
             \n// beta six\n// beta seven\n\
             struct BetaThing { beta_field: u64 }\n",
        )
        .unwrap();

        for detection in [
            crate::cli::DuplicateType::Exact,
            crate::cli::DuplicateType::Fuzzy,
            crate::cli::DuplicateType::Renamed,
            crate::cli::DuplicateType::All,
        ] {
            let report =
                detect_duplicates(temp.path(), detection.clone(), 0.85, 5, 100, &None, &None)
                    .await
                    .expect("detection must not fail");

            assert_eq!(
                report.total_duplicates, 0,
                "--detection-type {detection}: two files with no line in common \
                 have no duplicate block, got {:#?}",
                report.duplicate_blocks
            );
            assert_eq!(
                report.duplicate_lines, 0,
                "--detection-type {detection}: no line is duplicated"
            );
            assert_eq!(
                report.duplication_percentage, 0.0,
                "--detection-type {detection}: 62.5% was reported here"
            );
        }
    }

    /// No block may ever be built out of nothing, whatever the input.
    ///
    /// The empty normalised block is the failure mode, and it is invisible in
    /// the summary output: it prints as a block with an empty
    /// `content_preview`. This asserts the invariant directly over a file that
    /// is mostly comments.
    #[tokio::test]
    async fn no_reported_block_is_empty() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        for (name, word) in [("a.rs", "alpha"), ("b.rs", "beta")] {
            let mut body = String::new();
            for i in 0..9 {
                body.push_str(&format!("// {word} comment {i}\n\n"));
            }
            body.push_str(&format!("fn {word}_fn() {{ let {word} = 1; }}\n"));
            std::fs::write(temp.path().join(name), body).unwrap();
        }

        let report = detect_duplicates(
            temp.path(),
            crate::cli::DuplicateType::All,
            0.85,
            5,
            100,
            &None,
            &None,
        )
        .await
        .unwrap();

        for block in &report.duplicate_blocks {
            assert!(
                block.tokens > 0,
                "a block with zero tokens is a block built out of deleted \
                 comments: {block:#?}"
            );
            for loc in &block.locations {
                assert!(
                    !loc.content_preview.trim().is_empty(),
                    "a duplicate with nothing to show is not a duplicate: {block:#?}"
                );
            }
        }
    }

    /// Two identical files are 100% duplicated — not 447%, and not 50%.
    #[tokio::test]
    async fn test_two_identical_files_are_fully_duplicated() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        let body = (0..12)
            .map(|i| format!("    let v{i} = {i} + 1;\n"))
            .collect::<String>();
        std::fs::write(temp.path().join("a.rs"), &body).unwrap();
        std::fs::write(temp.path().join("b.rs"), &body).unwrap();

        let report = detect_duplicates(
            temp.path(),
            crate::cli::DuplicateType::Exact,
            0.8,
            5,
            100,
            &None,
            &None,
        )
        .await
        .unwrap();

        assert_eq!(report.total_lines, 24);
        // Exactly all of both files. This used to read "nearly all", because
        // `for i in 0..lines.len().saturating_sub(min_lines)` stops one window
        // early and never covered the last line — 11 of 12 lines, 91.7%.
        // Windowing over substantive lines with `.windows(min_lines)` has no
        // such off-by-one, so the honest answer for two identical files is
        // available and this asserts it rather than a range that would also
        // accept the old undercount.
        assert_eq!(
            report.duplication_percentage, 100.0,
            "two identical files are entirely duplicated, got {}%",
            report.duplication_percentage
        );
        for (path, stats) in &report.file_statistics {
            assert_eq!(stats.total_lines, 12, "{path}");
            assert_eq!(
                stats.duplicate_lines, 12,
                "{path}: every line of an identical pair is duplicated"
            );
        }
    }

    /// The per-file statistics are accumulated from a HashMap of blocks; five
    /// runs over the same input must agree exactly.
    #[tokio::test]
    async fn test_file_statistics_are_identical_across_5_runs() {
        use tempfile::TempDir;
        let temp = TempDir::new().unwrap();
        for name in ["a.rs", "b.rs", "c.rs"] {
            let body = (0..15)
                .map(|i| format!("    let v{} = {} * 3;\n", i % 5, i))
                .collect::<String>();
            std::fs::write(temp.path().join(name), body).unwrap();
        }

        let snapshot = |report: &DuplicateReport| {
            let mut rows: Vec<String> = report
                .file_statistics
                .iter()
                .map(|(p, s)| {
                    format!(
                        "{p}|{}|{}|{:.4}",
                        s.duplicate_lines, s.total_lines, s.duplication_percentage
                    )
                })
                .collect();
            rows.sort();
            format!("{}|{}|{rows:?}", report.duplicate_lines, report.total_lines)
        };

        let mut first: Option<String> = None;
        for run in 0..5 {
            let report = detect_duplicates(
                temp.path(),
                crate::cli::DuplicateType::Exact,
                0.8,
                5,
                100,
                &None,
                &None,
            )
            .await
            .unwrap();
            let current = snapshot(&report);
            match &first {
                None => first = Some(current),
                Some(expected) => assert_eq!(*expected, current, "run {run} differed"),
            }
        }
    }
}
