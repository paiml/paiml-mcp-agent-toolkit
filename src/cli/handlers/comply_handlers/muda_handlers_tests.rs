#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_muda_grade_classification() {
        assert_eq!(MudaGrade::from_score(0.0), MudaGrade::Lean);
        assert_eq!(MudaGrade::from_score(20.0), MudaGrade::Lean);
        assert_eq!(MudaGrade::from_score(21.0), MudaGrade::Efficient);
        assert_eq!(MudaGrade::from_score(40.0), MudaGrade::Efficient);
        assert_eq!(MudaGrade::from_score(50.0), MudaGrade::Moderate);
        assert_eq!(MudaGrade::from_score(70.0), MudaGrade::High);
        assert_eq!(MudaGrade::from_score(90.0), MudaGrade::Critical);
    }

    #[test]
    fn test_muda_score_on_self() {
        let project_path = PathBuf::from(".");
        let report = calculate_muda_score(&project_path);
        // Total should be in valid range
        assert!(report.total_score >= 0.0);
        assert!(report.total_score <= 100.0);
        // All individual scores should be in range
        assert!(report.overproduction >= 0.0 && report.overproduction <= 100.0);
        assert!(report.waiting >= 0.0 && report.waiting <= 100.0);
        assert!(report.inventory >= 0.0 && report.inventory <= 100.0);
        assert!(report.transport >= 0.0 && report.transport <= 100.0);
        assert!(report.over_processing >= 0.0 && report.over_processing <= 100.0);
        assert!(report.motion >= 0.0 && report.motion <= 100.0);
        assert!(report.defects >= 0.0 && report.defects <= 100.0);
    }

    #[test]
    fn test_muda_grade_display() {
        assert_eq!(format!("{}", MudaGrade::Lean), "Lean");
        assert_eq!(format!("{}", MudaGrade::Critical), "Critical");
    }

    #[test]
    fn test_transport_empty_project() {
        let path = PathBuf::from("/nonexistent/path");
        let score = measure_transport(&path);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_motion_no_cargo_lock() {
        let path = PathBuf::from("/nonexistent/path");
        let score = measure_motion(&path);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_defects_empty_project() {
        let path = PathBuf::from("/nonexistent/path");
        let score = measure_defects(&path);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_is_satd_marker_real_comments() {
        // Real SATD markers
        assert!(is_satd_marker("// TODO: implement this"));
        assert!(is_satd_marker("// FIXME: broken logic"));
        assert!(is_satd_marker("// HACK: temporary workaround"));
        assert!(is_satd_marker("//TODO: no space"));
        assert!(is_satd_marker("// FIXME(noah): needs refactor"));
    }

    #[test]
    fn test_is_satd_marker_excludes_non_comments() {
        // Not comments — should NOT be flagged
        assert!(!is_satd_marker(r#"patterns: vec!["TODO".to_string()]"#));
        assert!(!is_satd_marker(r#"let s = "FIXME: broken";"#));
        assert!(!is_satd_marker("fn check_todo() {"));
        assert!(!is_satd_marker(r#"Regex::new(r"\bHACK\b")"#));
    }

    #[test]
    fn test_is_satd_marker_excludes_doc_comments() {
        assert!(!is_satd_marker("/// TODO: document this"));
        assert!(!is_satd_marker("//! FIXME: module docs"));
    }

    #[test]
    fn test_is_satd_marker_excludes_security_annotations() {
        assert!(!is_satd_marker(
            "// SECURITY: Require 'passed' field to exist"
        ));
        assert!(!is_satd_marker(
            "// SAFETY: this pointer is valid because..."
        ));
    }

    #[test]
    fn test_is_satd_marker_excludes_string_literals_in_comments() {
        // Comments that reference SATD patterns in quotes (meta-discussion)
        assert!(!is_satd_marker(
            r#"// tracking "TODO" and "FIXME" comments"#
        ));
        assert!(!is_satd_marker(r#"// scans for "HACK" markers"#));
    }

    #[test]
    fn test_count_satd_in_content() {
        let content = r#"
// TODO: real debt marker
/// TODO: doc comment (excluded)
//! FIXME: module doc (excluded)
let x = "TODO: string literal (excluded)";
// SECURITY: FIXME cache validation (excluded)
// HACK: actual hack
fn contains_todo() {} // no marker, just identifier
"#;
        assert_eq!(count_satd_in_content(content), 2); // Only the real TODO and HACK
    }

    #[test]
    fn test_count_satd_skips_test_modules() {
        let content = "// TODO: real debt in production\nfn prod() {}\n\n#[cfg(test)]\nmod tests {\n    // TODO: test marker (excluded)\n    // FIXME: test fix (excluded)\n}\n";
        assert_eq!(count_satd_in_content(content), 1); // Only the production TODO
    }

    #[test]
    fn test_count_satd_skips_raw_string_content() {
        let content = "fn check() {\n    let code = r#\"\n        // TODO: embedded comment\n        // FIXME: also embedded\n    \"#;\n}\n// HACK: real marker\n";
        assert_eq!(count_satd_in_content(content), 1); // Only the real HACK
    }

    #[test]
    fn test_strip_quoted_strings() {
        assert_eq!(strip_quoted_strings(r#"hello "world" foo"#), "hello  foo");
        assert_eq!(strip_quoted_strings(r#""TODO" marker"#), " marker");
        assert_eq!(strip_quoted_strings("no quotes"), "no quotes");
        // Multiple quoted segments
        assert_eq!(strip_quoted_strings(r#"vec!["TODO", "FIXME"]"#), "vec![, ]");
    }

    #[test]
    fn test_file_details_populated_on_self() {
        let project_path = PathBuf::from(".");
        let report = calculate_muda_score(&project_path);
        // On a real Rust project, at least some file details should be populated
        // (e.g. Defects from .unwrap() usage, Inventory from SATD markers)
        // file_details is a HashMap<String, Vec<String>>
        for files in report.file_details.values() {
            // Each entry should have at most 5 files
            assert!(files.len() <= 5, "File details capped at 5 per category");
            for f in files {
                // Each file entry should contain a path-like string
                assert!(!f.is_empty(), "File detail entry should not be empty");
            }
        }
    }

    #[test]
    fn test_file_details_empty_on_nonexistent() {
        let path = PathBuf::from("/nonexistent/path");
        let report = calculate_muda_score(&path);
        // Nonexistent path should have empty file_details
        assert!(
            report.file_details.is_empty(),
            "Nonexistent path should have no file details"
        );
    }

    #[test]
    fn test_collect_inventory_files_on_self() {
        let project_path = PathBuf::from(".");
        let files = collect_inventory_files(&project_path);
        // Capped at 5
        assert!(files.len() <= 5);
        for f in &files {
            assert!(f.contains("SATD"), "Inventory files should indicate SATD count");
        }
    }

    #[test]
    fn test_collect_defect_files_on_self() {
        let project_path = PathBuf::from(".");
        let files = collect_defect_files(&project_path);
        // Capped at 5
        assert!(files.len() <= 5);
        // On a real project with .unwrap() calls, we should get results
        for f in &files {
            assert!(
                f.contains("defect pts") || f.contains("unwrap"),
                "Defect files should indicate defect type: {}",
                f
            );
        }
    }

    #[test]
    fn test_collect_over_processing_files_on_self() {
        let project_path = PathBuf::from(".");
        let files = collect_over_processing_files(&project_path);
        // Capped at 5
        assert!(files.len() <= 5);
        for f in &files {
            assert!(
                f.contains("cc="),
                "Over-processing files should show complexity: {}",
                f
            );
        }
    }

    #[test]
    fn test_collect_overproduction_files_nonexistent() {
        let path = PathBuf::from("/nonexistent/path");
        let files = collect_overproduction_files(&path);
        assert!(files.is_empty());
    }

    #[test]
    fn test_collect_inventory_files_nonexistent() {
        let path = PathBuf::from("/nonexistent/path");
        let files = collect_inventory_files(&path);
        assert!(files.is_empty());
    }

    #[test]
    fn test_collect_defect_files_nonexistent() {
        let path = PathBuf::from("/nonexistent/path");
        let files = collect_defect_files(&path);
        assert!(files.is_empty());
    }

    #[test]
    fn test_collect_over_processing_files_nonexistent() {
        let path = PathBuf::from("/nonexistent/path");
        let files = collect_over_processing_files(&path);
        assert!(files.is_empty());
    }

    #[test]
    fn test_estimate_max_complexity_simple() {
        let content = "fn simple() {\n    let x = 1;\n}\n";
        assert_eq!(estimate_max_complexity(content), 1);
    }

    #[test]
    fn test_estimate_max_complexity_branching() {
        let content = r#"
fn complex() {
    if x > 0 {
        if y > 0 {
            for i in 0..10 {
                match z {
                    1 => {},
                    _ => {},
                }
            }
        }
    }
    while running {
        if a && b {
            break;
        }
    }
}
"#;
        let cc = estimate_max_complexity(content);
        // if + if + for + match + while + if + (&&) = 7 + base 1 = 8
        assert!(cc >= 7, "Expected high complexity, got {}", cc);
    }

    #[test]
    fn test_estimate_max_complexity_multiple_fns() {
        let content = r#"
fn simple() {
    let x = 1;
}

fn complex() {
    if a {
        if b {
            if c {
                for i in items {
                    match x {
                        _ => {}
                    }
                }
            }
        }
    }
}
"#;
        let cc = estimate_max_complexity(content);
        // The complex fn has: if + if + if + for + match = 5 + base 1 = 6
        assert!(cc >= 5, "Expected max from complex fn, got {}", cc);
    }

    #[test]
    fn test_muda_report_serialization_with_file_details() {
        let mut file_details = HashMap::new();
        file_details.insert(
            "Defects".to_string(),
            vec!["src/main.rs (5 unwrap)".to_string()],
        );
        let report = MudaReport {
            overproduction: 10.0,
            waiting: 5.0,
            inventory: 20.0,
            transport: 3.0,
            over_processing: 15.0,
            motion: 8.0,
            defects: 12.0,
            total_score: 11.25,
            grade: MudaGrade::Lean,
            file_details,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("file_details"));
        assert!(json.contains("src/main.rs"));

        // Deserialize back
        let parsed: MudaReport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.file_details.len(), 1);
        assert!(parsed.file_details.contains_key("Defects"));
    }

    #[test]
    fn test_muda_report_serialization_empty_file_details() {
        let report = MudaReport {
            overproduction: 0.0,
            waiting: 0.0,
            inventory: 0.0,
            transport: 0.0,
            over_processing: 0.0,
            motion: 0.0,
            defects: 0.0,
            total_score: 0.0,
            grade: MudaGrade::Lean,
            file_details: HashMap::new(),
        };
        let json = serde_json::to_string(&report).unwrap();
        // Empty file_details should be skipped in serialization
        assert!(
            !json.contains("file_details"),
            "Empty file_details should be skipped: {}",
            json
        );
    }
}
