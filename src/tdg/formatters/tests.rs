#![cfg_attr(coverage_nightly, coverage(off))]

#[cfg(test)]
mod tests {
    use super::super::helpers::{
        format_metric_name, grade_delta, grade_description, grade_to_number, progress_bar,
    };
    use super::super::{
        format_comparison, format_human, format_json, format_markdown, format_project,
    };
    use crate::tdg::language_simple::Language;
    use crate::tdg::{
        Comparison, Grade, MetricCategory, PenaltyAttribution, ProjectScore, TdgScore,
    };
    use std::collections::HashSet;
    use std::path::PathBuf;

    /// GH #704: `analyze tdg` on a directory with nothing to grade printed
    /// `Average Score: 0.0/100 (F)` right above `Total Files: 0` — the struct
    /// default rendered as a measurement of a project never measured.
    #[test]
    fn empty_project_table_states_no_measurement_instead_of_zero_f() {
        let project = ProjectScore::aggregate(Vec::new());
        let rendered = format_project(&project);

        assert!(
            rendered.contains("not measured"),
            "the box must say the score was not measured, got:\n{rendered}"
        );
        assert!(
            !rendered.contains("0.0/100"),
            "the box must not print a 0.0 score for 0 analysed files, got:\n{rendered}"
        );
        assert!(
            !rendered.contains("(F)"),
            "the box must not grade a project it never measured, got:\n{rendered}"
        );
        assert!(rendered.contains("Total Files: 0"), "got:\n{rendered}");
    }

    /// Regression: every framed row must close in the same column.
    /// The renderers used to hand-pad each row with a literal run of spaces, so
    /// `│  Overall Score: 99.5/100 (A+)                  │` and
    /// `│  Total Files: 1593                               │` came out of the
    /// same fixed-width frame.
    fn assert_box_is_rectangular(rendered: &str, label: &str) {
        let widths: Vec<usize> = rendered
            .lines()
            .filter(|l| !l.is_empty())
            .map(super::super::boxdraw::visible_width)
            .collect();
        assert!(!widths.is_empty(), "{label}: nothing rendered");
        for (i, w) in widths.iter().enumerate() {
            assert_eq!(
                *w,
                widths[0],
                "{label}: line {i} is {w} cols wide, frame is {} — {:?}",
                widths[0],
                rendered.lines().nth(i)
            );
        }
    }

    #[test]
    fn test_rendered_boxes_are_rectangular() {
        // Contents of very different widths must not move the right border.
        for total in [0.0_f32, 99.5, 100.0] {
            let score = TdgScore {
                total,
                grade: Grade::from_score(total),
                file_path: Some(PathBuf::from("src/some/rather/long/path/module.rs")),
                penalties_applied: vec![PenaltyAttribution {
                    source_metric: MetricCategory::Coupling,
                    amount: 8.0,
                    applied_to: HashSet::new(),
                    issue: "Excessive coupling between modules detected in this analysis"
                        .to_string(),
                }],
                ..TdgScore::default()
            };
            assert_box_is_rectangular(&format_human(&score), &format!("format_human({total})"));
        }

        for count in [0_usize, 1, 1593] {
            let project = ProjectScore {
                // GH #704: 0 analysed files has no score and no grade at
                // all — the renderer must stay rectangular for that too.
                average_score: if count == 0 { None } else { Some(99.5) },
                average_grade: if count == 0 { None } else { Some(Grade::APlus) },
                not_measured: Vec::new(),
                total_files: count,
                files: Vec::new(),
                language_distribution: std::collections::BTreeMap::from([(Language::Rust, count)]),
                grade_distribution: std::collections::BTreeMap::from([(Grade::APlus, count)]),
                f_grade_count: 0,
                grade_capped: false,
                files_reported: 0,
                files_truncated: count > 0,
                list_filter: None,
                ungraded_files: Vec::new(),
                cross_file_duplication_ratio: None,
                cross_file_duplication_unmeasured: None,
                cross_file_duplication_coverage: None,
            };
            assert_box_is_rectangular(
                &format_project(&project),
                &format!("format_project({count})"),
            );
        }
    }

    #[test]
    fn test_format_human() {
        let score = TdgScore {
            total: 85.5,
            grade: Grade::AMinus,
            language: Language::Rust,
            confidence: 1.0,
            file_path: Some(PathBuf::from("src/test.rs")),
            ..TdgScore::default()
        };

        let output = format_human(&score);

        assert!(output.contains("85.5/100"));
        assert!(output.contains("A-"));
        assert!(output.contains("Rust"));
        assert!(output.contains("src/test.rs"));
        assert!(output.contains("📊 Breakdown"));
    }

    #[test]
    fn test_format_human_no_path() {
        let score = TdgScore {
            total: 90.0,
            grade: Grade::A,
            file_path: None,
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("Code Analysis"));
    }

    #[test]
    fn test_format_human_excellent_score() {
        let score = TdgScore {
            total: 95.0,
            grade: Grade::APlus,
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("Excellent code quality"));
    }

    #[test]
    fn test_format_human_good_score() {
        let score = TdgScore {
            total: 80.0,
            grade: Grade::B,
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("Good code quality"));
    }

    #[test]
    fn test_format_human_needs_improvement() {
        let score = TdgScore {
            total: 65.0,
            grade: Grade::CPlus,
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("needs improvement"));
    }

    #[test]
    fn test_format_human_needs_refactoring() {
        let score = TdgScore {
            total: 45.0,
            grade: Grade::F,
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("requires significant refactoring"));
    }

    #[test]
    fn test_format_human_with_penalties() {
        let score = TdgScore {
            total: 70.0,
            grade: Grade::BMinus,
            penalties_applied: vec![
                PenaltyAttribution {
                    source_metric: MetricCategory::StructuralComplexity,
                    amount: 5.0,
                    applied_to: HashSet::new(),
                    issue: "High complexity".to_string(),
                },
                PenaltyAttribution {
                    source_metric: MetricCategory::Documentation,
                    amount: 3.0,
                    applied_to: HashSet::new(),
                    issue: "This is a very long penalty description that should be truncated"
                        .to_string(),
                },
            ],
            ..TdgScore::default()
        };

        let output = format_human(&score);
        assert!(output.contains("Issues Found"));
        assert!(output.contains("High complexity"));
    }

    #[test]
    fn test_format_json() {
        let score = TdgScore::default();
        let output = format_json(&score);

        assert!(output.contains("structural_complexity"));
        assert!(output.contains("semantic_complexity"));
        assert!(output.contains("total"));
        assert!(output.contains("grade"));
    }

    #[test]
    fn test_format_markdown() {
        let score = TdgScore {
            total: 75.0,
            grade: Grade::B,
            ..TdgScore::default()
        };

        let output = format_markdown(&score);

        assert!(output.contains("# TDG Score Report"));
        assert!(output.contains("## Score Breakdown"));
        assert!(output.contains("| Metric | Score | Max"));
        assert!(output.contains("**B** (75-79)"));
    }

    #[test]
    fn test_format_markdown_with_path() {
        let score = TdgScore {
            total: 80.0,
            grade: Grade::B,
            file_path: Some(PathBuf::from("main.rs")),
            ..TdgScore::default()
        };

        let output = format_markdown(&score);
        assert!(output.contains("**File:** `main.rs`"));
    }

    #[test]
    fn test_format_markdown_with_penalties() {
        let score = TdgScore {
            total: 65.0,
            grade: Grade::CPlus,
            penalties_applied: vec![PenaltyAttribution {
                source_metric: MetricCategory::Documentation,
                amount: 10.0,
                applied_to: HashSet::new(),
                issue: "Missing docs".to_string(),
            }],
            ..TdgScore::default()
        };

        let output = format_markdown(&score);
        assert!(output.contains("## Issues Found"));
    }

    #[test]
    fn test_progress_bar() {
        assert_eq!(progress_bar(5.0, 10.0, 10), "█████░░░░░");
        assert_eq!(progress_bar(0.0, 10.0, 10), "░░░░░░░░░░");
        assert_eq!(progress_bar(10.0, 10.0, 10), "██████████");
        assert_eq!(progress_bar(15.0, 10.0, 10), "██████████"); // Clamped to max
    }

    #[test]
    fn test_progress_bar_negative() {
        assert_eq!(progress_bar(-5.0, 10.0, 10), "░░░░░░░░░░"); // Clamped to 0
    }

    #[test]
    fn test_grade_delta() {
        assert_eq!(grade_delta(Grade::B, Grade::A), "↑3");
        assert_eq!(grade_delta(Grade::A, Grade::B), "↓3");
        assert_eq!(grade_delta(Grade::B, Grade::B), "=");
    }

    #[test]
    fn test_grade_delta_extremes() {
        assert_eq!(grade_delta(Grade::F, Grade::APlus), "↑10");
        assert_eq!(grade_delta(Grade::APlus, Grade::F), "↓10");
    }

    #[test]
    fn test_grade_to_number() {
        assert_eq!(grade_to_number(Grade::APlus), 11);
        assert_eq!(grade_to_number(Grade::A), 10);
        assert_eq!(grade_to_number(Grade::AMinus), 9);
        assert_eq!(grade_to_number(Grade::BPlus), 8);
        assert_eq!(grade_to_number(Grade::B), 7);
        assert_eq!(grade_to_number(Grade::BMinus), 6);
        assert_eq!(grade_to_number(Grade::CPlus), 5);
        assert_eq!(grade_to_number(Grade::C), 4);
        assert_eq!(grade_to_number(Grade::CMinus), 3);
        assert_eq!(grade_to_number(Grade::D), 2);
        assert_eq!(grade_to_number(Grade::F), 1);
    }

    #[test]
    fn test_grade_description() {
        assert!(grade_description(Grade::APlus).contains("A+"));
        assert!(grade_description(Grade::A).contains("A"));
        assert!(grade_description(Grade::AMinus).contains("A-"));
        assert!(grade_description(Grade::BPlus).contains("B+"));
        assert!(grade_description(Grade::B).contains("B"));
        assert!(grade_description(Grade::BMinus).contains("B-"));
        assert!(grade_description(Grade::CPlus).contains("C+"));
        assert!(grade_description(Grade::C).contains("C"));
        assert!(grade_description(Grade::CMinus).contains("C-"));
        assert!(grade_description(Grade::D).contains("D"));
        assert!(grade_description(Grade::F).contains("F"));
    }

    #[test]
    fn test_format_metric_name() {
        assert_eq!(
            format_metric_name(&MetricCategory::StructuralComplexity),
            "Structural Complexity"
        );
        assert_eq!(
            format_metric_name(&MetricCategory::SemanticComplexity),
            "Semantic Complexity"
        );
        assert_eq!(
            format_metric_name(&MetricCategory::Duplication),
            "Code Duplication"
        );
        assert_eq!(format_metric_name(&MetricCategory::Coupling), "Coupling");
        assert_eq!(
            format_metric_name(&MetricCategory::Documentation),
            "Documentation"
        );
        assert_eq!(
            format_metric_name(&MetricCategory::Consistency),
            "Consistency"
        );
    }

    #[test]
    fn test_format_comparison() {
        let source1 = TdgScore {
            total: 70.0,
            grade: Grade::BMinus,
            ..TdgScore::default()
        };
        let source2 = TdgScore {
            total: 85.0,
            grade: Grade::AMinus,
            ..TdgScore::default()
        };
        let mut comparison = Comparison::new(source1, source2);
        comparison
            .improvements
            .push("Reduced complexity".to_string());

        let output = format_comparison(&comparison);
        assert!(output.contains("TDG Comparison"));
        assert!(output.contains("70.0"));
        assert!(output.contains("85.0"));
    }

    #[test]
    fn test_format_comparison_with_regressions() {
        let source1 = TdgScore {
            total: 90.0,
            grade: Grade::A,
            ..TdgScore::default()
        };
        let source2 = TdgScore {
            total: 85.0,
            grade: Grade::AMinus,
            ..TdgScore::default()
        };
        let mut comparison = Comparison::new(source1, source2);
        comparison
            .regressions
            .push("Increased complexity".to_string());

        let output = format_comparison(&comparison);
        assert!(output.contains("Regressions"));
    }

    #[test]
    fn test_format_project() {
        let mut language_distribution = std::collections::BTreeMap::new();
        language_distribution.insert(Language::Rust, 10);
        language_distribution.insert(Language::TypeScript, 5);

        // 15 files analysed, only the 2 worst listed (--top-files 2).
        let project = ProjectScore {
            average_score: Some(85.0),
            average_grade: Some(Grade::AMinus),
            not_measured: Vec::new(),
            total_files: 15,
            files: vec![
                TdgScore {
                    total: 90.0,
                    grade: Grade::A,
                    ..TdgScore::default()
                },
                TdgScore {
                    total: 80.0,
                    grade: Grade::B,
                    ..TdgScore::default()
                },
            ],
            language_distribution,
            // Populated on purpose: the assertions below check that the
            // distribution describes the ANALYSED POPULATION (15 files) rather
            // than the two-entry short list, which is the point of the
            // "Files Listed: 2 of 15" disclosure. A merge left this empty while
            // keeping the assertions, so the fixture contradicted the test.
            grade_distribution: std::collections::BTreeMap::from([(Grade::A, 10), (Grade::B, 5)]),
            f_grade_count: 0,
            grade_capped: false,
            files_reported: 2,
            files_truncated: true,
            list_filter: None,
            ungraded_files: Vec::new(),
            cross_file_duplication_ratio: None,
            cross_file_duplication_unmeasured: None,
            cross_file_duplication_coverage: None,
        };

        let output = format_project(&project);
        assert!(output.contains("Project TDG Score Report"));
        assert!(output.contains("85.0/100"));
        assert!(output.contains("Language Distribution"));
        assert!(output.contains("Grade Distribution"));
        // A truncated list must name both numbers.
        assert!(output.contains("Files Listed: 2 of 15"));
        // Distributions describe the analysed population, not the short list.
        assert!(output.contains("A:  10 files"));
        assert!(output.contains("B:   5 files"));
    }

    /// R22: a file the walker refused must be disclosed in the human report,
    /// next to the average it is missing from. The refusal was an `eprintln!`
    /// only, so the box printed `Average Score: 100.0/100 (A+)` over the single
    /// survivor of a crate whose only Rust file failed to parse.
    #[test]
    fn the_project_box_discloses_files_that_were_not_graded() {
        let mut project = ProjectScore::aggregate(vec![TdgScore {
            total: 100.0,
            grade: Grade::APlus,
            language: Language::Python,
            file_path: Some(PathBuf::from("./src/bad.py")),
            ..TdgScore::default()
        }]);
        project.ungraded_files.push(crate::tdg::UngradedFile {
            path: "./src/main.rs".to_string(),
            reason: "cannot parse string into token stream".to_string(),
        });

        let output = format_project(&project);
        assert!(
            output.contains("Not Graded: 1 file(s)"),
            "a walked-but-unmeasured file must be disclosed: {output}"
        );
    }

    /// R23: the #279 waiver was disclosed only by `check-quality --format json`.
    /// Every surface that reports the verdict has to report the waiver.
    #[test]
    fn the_project_box_discloses_critical_defect_waivers() {
        let mut waived = TdgScore {
            file_path: Some(PathBuf::from("./src/new.rs")),
            has_critical_defects: true,
            critical_defects_count: 3,
            critical_defects_suppressed: Some("file is not tracked by git (#279)".to_string()),
            ..TdgScore::default()
        };
        waived.calculate_total();
        let project = ProjectScore::aggregate(vec![waived.clone()]);

        let output = format_project(&project);
        assert!(
            output.contains("Waived (#279): 1 file(s)"),
            "the project box must disclose the waiver: {output}"
        );

        let file_report = format_human(&waived);
        assert!(
            file_report.contains("Critical Defects: 3"),
            "the per-file box must report the defects: {file_report}"
        );
        assert!(
            file_report.contains("waived"),
            "the per-file box must disclose the waiver: {file_report}"
        );
    }
}
