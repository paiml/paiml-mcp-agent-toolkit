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

        let duplicates = find_duplicate_blocks(blocks, 0.8);
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
            find_duplicate_blocks(blocks, 0.8)
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

        let duplicates = find_duplicate_blocks(blocks, 0.8);
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

    #[test]
    fn test_extract_filename_handles_paths_and_bare_names() {
        // Bare name → returned unchanged.
        assert_eq!(extract_filename("foo.rs"), "foo.rs");
        // Full path → only basename returned.
        assert_eq!(extract_filename("src/cli/foo.rs"), "foo.rs");
        // No extension → still returns basename.
        assert_eq!(extract_filename("src/cli/Makefile"), "Makefile");
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
        // The sliding window does not always reach the final line of a file,
        // so this is "nearly all of both files" rather than exactly all of it —
        // what matters is that it is BELOW 100%, where 447.06% was reported.
        assert!(
            (80.0..=100.0).contains(&report.duplication_percentage),
            "two identical files must read as almost fully duplicated, got {}%",
            report.duplication_percentage
        );
        for (path, stats) in &report.file_statistics {
            assert_eq!(stats.total_lines, 12, "{path}");
            assert!(
                stats.duplicate_lines <= 12,
                "{path}: {} of 12 lines",
                stats.duplicate_lines
            );
            assert!(stats.duplicate_lines >= 10, "{path}");
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
