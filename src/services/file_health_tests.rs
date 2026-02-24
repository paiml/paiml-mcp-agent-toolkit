#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_classification() {
        assert_eq!(FileSizeClass::from_lines(100), FileSizeClass::Ideal);
        assert_eq!(FileSizeClass::from_lines(200), FileSizeClass::Ideal);
        assert_eq!(FileSizeClass::from_lines(201), FileSizeClass::Acceptable);
        assert_eq!(FileSizeClass::from_lines(500), FileSizeClass::Acceptable);
        assert_eq!(FileSizeClass::from_lines(501), FileSizeClass::Warning);
        assert_eq!(FileSizeClass::from_lines(1000), FileSizeClass::Warning);
        assert_eq!(FileSizeClass::from_lines(1001), FileSizeClass::Problem);
        assert_eq!(FileSizeClass::from_lines(2000), FileSizeClass::Problem);
        assert_eq!(FileSizeClass::from_lines(2001), FileSizeClass::Critical);
        assert_eq!(FileSizeClass::from_lines(12000), FileSizeClass::Critical);
    }

    #[test]
    fn test_health_grade_from_score() {
        assert_eq!(HealthGrade::from_score(100), HealthGrade::A);
        assert_eq!(HealthGrade::from_score(90), HealthGrade::A);
        assert_eq!(HealthGrade::from_score(89), HealthGrade::B);
        assert_eq!(HealthGrade::from_score(80), HealthGrade::B);
        assert_eq!(HealthGrade::from_score(79), HealthGrade::C);
        assert_eq!(HealthGrade::from_score(70), HealthGrade::C);
        assert_eq!(HealthGrade::from_score(69), HealthGrade::D);
        assert_eq!(HealthGrade::from_score(60), HealthGrade::D);
        assert_eq!(HealthGrade::from_score(59), HealthGrade::E);
        assert_eq!(HealthGrade::from_score(50), HealthGrade::E);
        assert_eq!(HealthGrade::from_score(49), HealthGrade::F);
        assert_eq!(HealthGrade::from_score(0), HealthGrade::F);
    }

    #[test]
    fn test_required_tlr_scaling() {
        assert_eq!(FileHealthMetrics::required_tlr_for_size(50), 0.3);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(100), 0.3);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(150), 0.5);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(300), 0.5);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(400), 0.7);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(500), 0.7);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(600), 1.0);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(1000), 1.0);
        assert_eq!(FileHealthMetrics::required_tlr_for_size(1500), 1.5);
    }

    #[test]
    fn test_health_score_calculation_ideal_file() {
        // 100 lines, 50 test lines, low complexity, stable
        let metrics = FileHealthMetrics::calculate(
            PathBuf::from("test.rs"),
            100, // lines
            50,  // test_lines (TLR = 0.5, required = 0.3)
            3.0, // avg_complexity
            1,   // churn_30d
        );

        assert_eq!(metrics.size_class, FileSizeClass::Ideal);
        // size: 30, tlr: 40 (0.5/0.3 > 1.0), complexity: 20, stability: 10 = 100
        assert_eq!(metrics.health_score, 100);
        assert_eq!(metrics.grade, HealthGrade::A);
    }

    #[test]
    fn test_health_score_calculation_critical_file() {
        // 5000 lines, 100 test lines, high complexity, volatile
        let metrics = FileHealthMetrics::calculate(
            PathBuf::from("monolith.rs"),
            5000, // lines (critical)
            100,  // test_lines (TLR = 0.02, required = 1.5)
            25.0, // avg_complexity (very high)
            15,   // churn_30d (hot file)
        );

        assert_eq!(metrics.size_class, FileSizeClass::Critical);
        // size: 0, tlr: ~0.5 (0.02/1.5 * 40), complexity: 0, stability: 0
        assert!(metrics.health_score < 10);
        assert_eq!(metrics.grade, HealthGrade::F);
    }

    #[test]
    fn test_health_score_calculation_medium_file() {
        // 400 lines, 200 test lines, medium complexity
        let metrics = FileHealthMetrics::calculate(
            PathBuf::from("service.rs"),
            400, // lines (acceptable)
            200, // test_lines (TLR = 0.5, required = 0.7)
            8.0, // avg_complexity
            3,   // churn_30d
        );

        assert_eq!(metrics.size_class, FileSizeClass::Acceptable);
        // size: 25, tlr: ~28 (0.5/0.7 * 40), complexity: 15, stability: 7 = ~75
        assert!(metrics.health_score >= 70 && metrics.health_score <= 80);
        assert!(matches!(metrics.grade, HealthGrade::B | HealthGrade::C));
    }

    #[test]
    fn test_baseline_ratchet_violation() {
        let mut baseline = FileHealthBaseline::new();
        baseline.files.insert(
            "src/big.rs".to_string(),
            BaselineEntry {
                lines: 1000,
                test_lines: 200,
                tlr: 0.2,
                health: 45,
                status: "warning".to_string(),
            },
        );

        // File grew - violation
        let violation = baseline.check_ratchet("src/big.rs", 1050);
        assert!(violation.is_some());
        let v = violation.unwrap();
        assert_eq!(v.baseline_lines, 1000);
        assert_eq!(v.current_lines, 1050);
        assert_eq!(v.growth, 50);

        // File shrunk - no violation
        let no_violation = baseline.check_ratchet("src/big.rs", 950);
        assert!(no_violation.is_none());

        // New file - no violation (not in baseline)
        let new_file = baseline.check_ratchet("src/new.rs", 600);
        assert!(new_file.is_none());
    }

    #[test]
    fn test_report_compliance() {
        let files = vec![
            FileHealthMetrics::calculate(PathBuf::from("good.rs"), 100, 50, 3.0, 1),
            FileHealthMetrics::calculate(PathBuf::from("ok.rs"), 300, 100, 8.0, 2),
        ];

        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        assert!(report.is_compliant);
        assert!(report.critical_files.is_empty());
    }

    #[test]
    fn test_report_non_compliance() {
        let files = vec![
            FileHealthMetrics::calculate(PathBuf::from("good.rs"), 100, 50, 3.0, 1),
            FileHealthMetrics::calculate(PathBuf::from("bad.rs"), 5000, 50, 25.0, 20),
        ];

        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        assert!(!report.is_compliant);
        assert_eq!(report.critical_files.len(), 1);
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_grade_is_passing() {
        assert!(HealthGrade::A.is_passing());
        assert!(HealthGrade::B.is_passing());
        assert!(HealthGrade::C.is_passing());
        assert!(!HealthGrade::D.is_passing());
        assert!(!HealthGrade::E.is_passing());
        assert!(!HealthGrade::F.is_passing());
    }

    #[test]
    fn test_file_size_class_as_str() {
        assert_eq!(FileSizeClass::Ideal.as_str(), "ideal");
        assert_eq!(FileSizeClass::Acceptable.as_str(), "acceptable");
        assert_eq!(FileSizeClass::Warning.as_str(), "warning");
        assert_eq!(FileSizeClass::Problem.as_str(), "problem");
        assert_eq!(FileSizeClass::Critical.as_str(), "critical");
    }

    #[test]
    fn test_health_grade_as_str() {
        assert_eq!(HealthGrade::A.as_str(), "A");
        assert_eq!(HealthGrade::B.as_str(), "B");
        assert_eq!(HealthGrade::C.as_str(), "C");
        assert_eq!(HealthGrade::D.as_str(), "D");
        assert_eq!(HealthGrade::E.as_str(), "E");
        assert_eq!(HealthGrade::F.as_str(), "F");
    }

    #[test]
    fn test_empty_report() {
        let report = FileHealthReport::from_files(PathBuf::from("."), vec![]);
        assert_eq!(report.total_files, 0);
        assert_eq!(report.total_lines, 0);
        assert_eq!(report.average_health, 100);
        assert!(report.is_compliant);
    }

    #[test]
    fn test_report_with_problem_files() {
        // A file that lands in problem range (50-69):
        // - 600 lines (warning range) = 15 pts
        // - 400 test lines, TLR=0.67, required=1.0, ratio=0.67, score ≈ 27 pts
        // - complexity 8.0 = 15 pts
        // - churn 4 = 7 pts
        // Total = 15+27+15+7 = 64 (problem range)
        let files = vec![FileHealthMetrics::calculate(
            PathBuf::from("medium.rs"),
            600,
            400,
            8.0,
            4,
        )];

        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        // Should have problem files but still be compliant (no critical)
        assert!(report.is_compliant);
        assert!(!report.problem_files.is_empty() || !report.warning_files.is_empty());
    }

    #[test]
    fn test_baseline_add_file() {
        let mut baseline = FileHealthBaseline::new();
        let metrics = FileHealthMetrics::calculate(PathBuf::from("test.rs"), 200, 100, 5.0, 1);

        baseline.add_file(&metrics);

        assert!(baseline.files.contains_key("test.rs"));
        let entry = baseline.files.get("test.rs").unwrap();
        assert_eq!(entry.lines, 200);
        assert_eq!(entry.test_lines, 100);
    }

    #[test]
    fn test_baseline_default() {
        let baseline = FileHealthBaseline::default();
        assert!(baseline.files.is_empty());
        assert_eq!(baseline.version, "1.0");
    }

    #[test]
    fn test_baseline_save_and_load() {
        let mut baseline = FileHealthBaseline::new();
        let metrics = FileHealthMetrics::calculate(PathBuf::from("src/lib.rs"), 150, 75, 4.0, 2);
        baseline.add_file(&metrics);

        // Save to temp file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_baseline.json");

        baseline.save(&temp_path).expect("Failed to save");

        // Load it back
        let loaded = FileHealthBaseline::load(&temp_path).expect("Failed to load");

        assert_eq!(loaded.version, baseline.version);
        assert!(loaded.files.contains_key("src/lib.rs"));

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_analyze_file_function() {
        // Create a temp file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_analyze.rs");
        std::fs::write(&temp_path, "fn main() {\n    println!(\"Hello\");\n}\n").unwrap();

        let metrics = analyze_file(&temp_path, 10, 3.0, 1);
        assert!(metrics.is_some());

        let m = metrics.unwrap();
        assert_eq!(m.lines, 3); // 3 lines (trailing newline doesn't count as line)
        assert_eq!(m.test_lines, 10);

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_analyze_file_nonexistent() {
        let result = analyze_file(Path::new("/nonexistent/file.rs"), 0, 0.0, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_count_lines_function() {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_count.rs");
        std::fs::write(&temp_path, "line1\nline2\nline3\n").unwrap();

        let count = count_lines(&temp_path);
        assert_eq!(count, Some(3));

        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_count_lines_nonexistent() {
        let count = count_lines(Path::new("/nonexistent/file.rs"));
        assert!(count.is_none());
    }

    #[test]
    fn test_scan_directory() {
        // Use CARGO_MANIFEST_DIR for deterministic path
        let project_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let files = scan_directory(project_dir, &["rs"], DEFAULT_EXCLUDE_PATTERNS);

        // Should find some Rust files in the project
        assert!(
            !files.is_empty(),
            "Should find Rust files in project directory"
        );
    }

    #[test]
    fn test_health_zero_lines_tlr() {
        // Edge case: zero lines should give TLR of 1.0
        let metrics = FileHealthMetrics::calculate(PathBuf::from("empty.rs"), 0, 0, 0.0, 0);
        assert_eq!(metrics.tlr, 1.0);
    }

    #[test]
    fn test_report_with_low_tlr_files() {
        let files = vec![
            // Low TLR file
            FileHealthMetrics::calculate(PathBuf::from("untested.rs"), 500, 50, 10.0, 3),
        ];

        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        // Should have TLR recommendation
        assert!(report.recommendations.iter().any(|r| r.contains("TLR")));
    }

    // ── StackHealthReport tests ────────────────────────────────────────────

    #[test]
    fn test_stack_health_report_empty() {
        let report = StackHealthReport::from_projects(vec![]);
        assert!(report.projects.is_empty());
        assert_eq!(report.stack_average_health, 0);
        assert!(report.stack_worst_files.is_empty());
        assert_eq!(report.stack_grade, HealthGrade::F);
    }

    #[test]
    fn test_stack_health_report_single_healthy_project() {
        let files = vec![FileHealthMetrics::calculate(
            PathBuf::from("good.rs"),
            100,
            50,
            3.0,
            1,
        )];
        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        let stack = StackHealthReport::from_projects(vec![("myproject".to_string(), report)]);

        assert_eq!(stack.projects.len(), 1);
        assert_eq!(stack.stack_average_health, 100);
        assert_eq!(stack.stack_grade, HealthGrade::A);
    }

    #[test]
    fn test_stack_health_report_mixed_projects() {
        let good_files = vec![FileHealthMetrics::calculate(
            PathBuf::from("good.rs"),
            100,
            50,
            3.0,
            1,
        )];
        let bad_files = vec![FileHealthMetrics::calculate(
            PathBuf::from("bad.rs"),
            5000,
            50,
            25.0,
            20,
        )];
        let good_report = FileHealthReport::from_files(PathBuf::from("."), good_files);
        let bad_report = FileHealthReport::from_files(PathBuf::from("."), bad_files);

        let stack = StackHealthReport::from_projects(vec![
            ("good_project".to_string(), good_report),
            ("bad_project".to_string(), bad_report),
        ]);

        assert_eq!(stack.projects.len(), 2);
        // Average of 100 and a low score
        assert!(stack.stack_average_health < 100);
        // Worst files should include the bad file
        assert!(!stack.stack_worst_files.is_empty());
    }

    #[test]
    fn test_stack_health_report_worst_files_capped_at_10() {
        // Create a project with many critical files
        let mut files = Vec::new();
        for i in 0..15 {
            files.push(FileHealthMetrics::calculate(
                PathBuf::from(format!("file_{}.rs", i)),
                5000,
                10,
                25.0,
                20,
            ));
        }
        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        let stack = StackHealthReport::from_projects(vec![("big".to_string(), report)]);

        assert!(stack.stack_worst_files.len() <= 10);
    }

    #[test]
    fn test_stack_health_report_serialization() {
        let files = vec![FileHealthMetrics::calculate(
            PathBuf::from("a.rs"),
            100,
            50,
            3.0,
            1,
        )];
        let report = FileHealthReport::from_files(PathBuf::from("."), files);
        let stack = StackHealthReport::from_projects(vec![("proj".to_string(), report)]);

        let json = serde_json::to_string(&stack).expect("serialize");
        let deser: StackHealthReport = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.stack_average_health, stack.stack_average_health);
        assert_eq!(deser.stack_grade, stack.stack_grade);
        assert_eq!(deser.projects.len(), 1);
    }

    // ── StackBaseline tests ────────────────────────────────────────────────

    #[test]
    fn test_stack_baseline_new() {
        let baseline = StackBaseline::new();
        assert_eq!(baseline.version, "1.0");
        assert!(baseline.projects.is_empty());
        assert!(!baseline.generated.is_empty());
    }

    #[test]
    fn test_stack_baseline_default() {
        let baseline = StackBaseline::default();
        assert_eq!(baseline.version, "1.0");
        assert!(baseline.projects.is_empty());
    }

    #[test]
    fn test_stack_baseline_add_project() {
        let mut stack = StackBaseline::new();
        let project_baseline = FileHealthBaseline::new();
        stack.add_project("aprender".to_string(), project_baseline);

        assert_eq!(stack.projects.len(), 1);
        assert!(stack.projects.contains_key("aprender"));
    }

    #[test]
    fn test_stack_baseline_add_multiple_projects() {
        let mut stack = StackBaseline::new();
        stack.add_project("project_a".to_string(), FileHealthBaseline::new());
        stack.add_project("project_b".to_string(), FileHealthBaseline::new());
        stack.add_project("project_c".to_string(), FileHealthBaseline::new());

        assert_eq!(stack.projects.len(), 3);
    }

    #[test]
    fn test_stack_baseline_save_and_load() {
        let mut stack = StackBaseline::new();
        let mut proj_baseline = FileHealthBaseline::new();
        let metrics = FileHealthMetrics::calculate(PathBuf::from("lib.rs"), 200, 100, 5.0, 1);
        proj_baseline.add_file(&metrics);
        stack.add_project("test_proj".to_string(), proj_baseline);

        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_stack_baseline.json");

        stack.save(&temp_path).expect("save");
        let loaded = StackBaseline::load(&temp_path).expect("load");

        assert_eq!(loaded.version, "1.0");
        assert_eq!(loaded.projects.len(), 1);
        assert!(loaded.projects.contains_key("test_proj"));

        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_stack_baseline_overwrite_project() {
        let mut stack = StackBaseline::new();
        stack.add_project("proj".to_string(), FileHealthBaseline::new());

        let mut updated = FileHealthBaseline::new();
        let metrics = FileHealthMetrics::calculate(PathBuf::from("new.rs"), 300, 150, 5.0, 2);
        updated.add_file(&metrics);
        stack.add_project("proj".to_string(), updated);

        assert_eq!(stack.projects.len(), 1);
        assert!(stack
            .projects
            .get("proj")
            .unwrap()
            .files
            .contains_key("new.rs"));
    }
}
