#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod check_handlers_tests {
    use super::*;

    #[test]
    fn test_is_heavily_cfg_gated_true() {
        let content = r#"
#[cfg(target_arch = "x86_64")]
#[cfg(target_arch = "aarch64")]
#[cfg(feature = "simd")]
#[target_feature(enable = "avx2")]
fn simd_function() {}
"#;
        assert!(is_heavily_cfg_gated(content));
    }

    #[test]
    fn test_is_heavily_cfg_gated_false() {
        let content = r#"
fn regular_function() {}
fn another_function() {}
"#;
        assert!(!is_heavily_cfg_gated(content));
    }

    #[test]
    fn test_is_heavily_cfg_gated_boundary() {
        // Exactly 3 cfg attributes should NOT trigger
        let content = r#"
#[cfg(target_arch = "x86_64")]
#[cfg(feature = "simd")]
#[target_feature(enable = "avx2")]
"#;
        assert!(!is_heavily_cfg_gated(content));
    }

    #[test]
    fn test_filter_production_lines_basic() {
        let lines = vec![
            "fn main() {}",
            "let x = 1;",
            "#[cfg(test)]",
            "mod tests {",
            "    #[test]",
            "    fn test_something() {}",
            "}",
        ];
        let result = filter_production_lines(&lines);
        // Should exclude everything after #[cfg(test)]
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "fn main() {}");
        assert_eq!(result[1], "let x = 1;");
    }

    #[test]
    fn test_filter_production_lines_no_test_module() {
        let lines = vec!["fn main() {}", "let x = 1;"];
        let result = filter_production_lines(&lines);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_format_violation_list_empty() {
        let issues: Vec<String> = vec![];
        let result = format_violation_list(&issues);
        assert!(result.is_empty() || result.trim().is_empty());
    }

    #[test]
    fn test_format_violation_list_single() {
        let issues = vec!["CB-001: test issue".to_string()];
        let result = format_violation_list(&issues);
        assert!(result.contains("CB-001"));
    }

    #[test]
    fn test_format_violation_list_multiple() {
        let issues = vec![
            "CB-001: issue 1".to_string(),
            "CB-002: issue 2".to_string(),
        ];
        let result = format_violation_list(&issues);
        assert!(result.contains("CB-001"));
        assert!(result.contains("CB-002"));
    }

    #[test]
    fn test_check_version_currency_current() {
        let check = check_version_currency(PMAT_VERSION);
        assert_eq!(check.status, CheckStatus::Pass);
    }

    #[test]
    fn test_check_version_currency_old() {
        let check = check_version_currency("1.0.0");
        // Should warn or fail for old version
        assert!(check.status == CheckStatus::Warn || check.status == CheckStatus::Fail);
    }

    #[test]
    fn test_count_dead_items_no_dead() {
        let lines = vec![
            "pub fn active() {}",
            "pub struct Active {}",
        ];
        let (total, dead, _) = count_dead_items(&lines);
        assert!(total >= 2);
        assert_eq!(dead, 0);
    }

    #[test]
    fn test_count_dead_items_with_dead() {
        let lines = vec![
            "#[allow(dead_code)]",
            "fn unused() {}",
            "pub fn active() {}",
        ];
        let (total, dead, _) = count_dead_items(&lines);
        assert!(total >= 2);
        assert!(dead >= 1);
    }

    #[test]
    fn test_classify_item_line_function() {
        let mut total = 0;
        let mut dead = 0;
        let mut annotations = 0;
        let mut next_is_dead = false;

        classify_item_line("pub fn test() {}", &mut total, &mut dead, &mut annotations, &mut next_is_dead);
        assert_eq!(total, 1);
        assert_eq!(dead, 0);
    }

    #[test]
    fn test_classify_item_line_dead_annotation() {
        let mut total = 0;
        let mut dead = 0;
        let mut annotations = 0;
        let mut next_is_dead = false;

        classify_item_line("#[allow(dead_code)]", &mut total, &mut dead, &mut annotations, &mut next_is_dead);
        assert_eq!(annotations, 1);
        assert!(next_is_dead);
    }
}
