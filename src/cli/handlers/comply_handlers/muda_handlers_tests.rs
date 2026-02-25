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
}
