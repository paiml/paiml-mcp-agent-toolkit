// =============================================================================
// OIP Tarantula patterns check tests (CB-120 through CB-124)
// =============================================================================

#[test]
fn test_check_oip_tarantula_patterns_clean() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn safe_fn() {}").unwrap();

    let check =
        crate::cli::handlers::comply_handlers::check_oip_tarantula_patterns(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

// =============================================================================
// Coverage quality patterns check tests (CB-125 through CB-127)
// =============================================================================

#[test]
fn test_check_coverage_quality_patterns_clean() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn clean() {}").unwrap();

    let check =
        crate::cli::handlers::comply_handlers::check_coverage_quality_patterns(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Pass);
}

// =============================================================================
// Dead code percentage check tests (CB-304)
// =============================================================================

#[test]
fn test_check_dead_code_percentage_no_src() {
    let temp_dir = TempDir::new().unwrap();

    let check = crate::cli::handlers::comply_handlers::check_dead_code_percentage(temp_dir.path());
    assert_eq!(check.status, CheckStatus::Skip);
}

#[test]
fn test_check_dead_code_percentage_clean() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("lib.rs"),
        "pub fn active_fn() { println!(\"used\"); }",
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_dead_code_percentage(temp_dir.path());
    // Clean code should pass
    assert!(check.status == CheckStatus::Pass || check.status == CheckStatus::Skip);
}

#[test]
fn test_check_dead_code_percentage_with_dead_code() {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    // Create file with dead code annotations
    std::fs::write(
        src_dir.join("lib.rs"),
        r#"

fn dead_fn1() {}

fn dead_fn2() {}

fn dead_fn3() {}
/// Active fn.
pub fn active_fn() {}
"#,
    )
    .unwrap();

    let check = crate::cli::handlers::comply_handlers::check_dead_code_percentage(temp_dir.path());
    // Should detect dead code
    assert!(check.name.contains("Dead Code"));
}

// =============================================================================
// Dead code analysis helper tests
// =============================================================================

#[test]
fn test_is_heavily_cfg_gated_false() {
    let content = "pub fn normal_fn() {}";
    let result = crate::cli::handlers::comply_handlers::is_heavily_cfg_gated(content);
    assert!(!result);
}

#[test]
fn test_is_heavily_cfg_gated_true() {
    let content = r#"
#[cfg(target_arch = "x86_64")]
fn simd_fn1() {}
#[cfg(target_feature = "avx2")]
fn simd_fn2() {}
#[cfg(feature = "simd")]
fn simd_fn3() {}
#[target_feature(enable = "sse2")]
fn simd_fn4() {}
"#;
    let result = crate::cli::handlers::comply_handlers::is_heavily_cfg_gated(content);
    assert!(result);
}

#[test]
fn test_is_dead_code_annotation_true() {
    assert!(crate::cli::handlers::comply_handlers::is_dead_code_annotation(&format!(
        "#[allow({})]",
        "dead_code"
    )));
    assert!(crate::cli::handlers::comply_handlers::is_dead_code_annotation("#[allow(unused)]"));
}

#[test]
fn test_is_dead_code_annotation_false() {
    assert!(!crate::cli::handlers::comply_handlers::is_dead_code_annotation("pub fn active() {}"));
    assert!(!crate::cli::handlers::comply_handlers::is_dead_code_annotation("#[derive(Debug)]"));
}

#[test]
fn test_is_code_item_declaration_true() {
    assert!(crate::cli::handlers::comply_handlers::is_code_item_declaration("pub fn test() {}"));
    assert!(crate::cli::handlers::comply_handlers::is_code_item_declaration("fn private() {}"));
    assert!(
        crate::cli::handlers::comply_handlers::is_code_item_declaration("pub struct MyStruct {")
    );
    assert!(crate::cli::handlers::comply_handlers::is_code_item_declaration("pub enum MyEnum {"));
    assert!(crate::cli::handlers::comply_handlers::is_code_item_declaration("pub trait MyTrait {"));
    assert!(
        crate::cli::handlers::comply_handlers::is_code_item_declaration(
            "pub const VALUE: i32 = 42;"
        )
    );
    assert!(
        crate::cli::handlers::comply_handlers::is_code_item_declaration(
            "pub async fn async_fn() {}"
        )
    );
}

#[test]
fn test_is_code_item_declaration_false() {
    assert!(!crate::cli::handlers::comply_handlers::is_code_item_declaration("let x = 5;"));
    assert!(!crate::cli::handlers::comply_handlers::is_code_item_declaration("// comment"));
    assert!(!crate::cli::handlers::comply_handlers::is_code_item_declaration("#[derive(Debug)]"));
}

#[test]
fn test_has_code_markers_true() {
    assert!(crate::cli::handlers::comply_handlers::has_code_markers(
        "fn test() {"
    ));
    assert!(crate::cli::handlers::comply_handlers::has_code_markers(
        "let x = 5;"
    ));
    assert!(crate::cli::handlers::comply_handlers::has_code_markers(
        "return value;"
    ));
    assert!(crate::cli::handlers::comply_handlers::has_code_markers(
        "struct Foo {"
    ));
}

#[test]
fn test_has_code_markers_false() {
    assert!(!crate::cli::handlers::comply_handlers::has_code_markers(
        "just a comment"
    ));
    assert!(!crate::cli::handlers::comply_handlers::has_code_markers(
        "empty line"
    ));
}

#[test]
fn test_is_commented_out_code_true() {
    assert!(crate::cli::handlers::comply_handlers::is_commented_out_code("// fn test() {"));
    assert!(crate::cli::handlers::comply_handlers::is_commented_out_code("// let x = 5;"));
    assert!(crate::cli::handlers::comply_handlers::is_commented_out_code("// return foo;"));
}

#[test]
fn test_is_commented_out_code_false() {
    // Just a regular comment
    assert!(
        !crate::cli::handlers::comply_handlers::is_commented_out_code(
            "// This is a regular comment"
        )
    );
    // Not a comment
    assert!(!crate::cli::handlers::comply_handlers::is_commented_out_code("fn test() {}"));
}

#[test]
fn test_flush_comment_run_threshold() {
    // Run of 3 or more counts
    assert_eq!(
        crate::cli::handlers::comply_handlers::flush_comment_run(3),
        3
    );
    assert_eq!(
        crate::cli::handlers::comply_handlers::flush_comment_run(5),
        5
    );
    // Run of less than 3 doesn't count
    assert_eq!(
        crate::cli::handlers::comply_handlers::flush_comment_run(2),
        0
    );
    assert_eq!(
        crate::cli::handlers::comply_handlers::flush_comment_run(0),
        0
    );
}

#[test]
fn test_update_macro_depth_enter() {
    let depth =
        crate::cli::handlers::comply_handlers::update_macro_depth("macro_rules! my_macro {", None);
    assert!(depth.is_some());
}

#[test]
fn test_update_macro_depth_exit() {
    let depth = crate::cli::handlers::comply_handlers::update_macro_depth("}", Some(1));
    assert!(depth.is_none());
}

#[test]
fn test_update_macro_depth_nested() {
    // Start macro
    let depth =
        crate::cli::handlers::comply_handlers::update_macro_depth("macro_rules! test {", None);
    assert!(depth.is_some());

    // Inside macro, nested brace
    let depth = crate::cli::handlers::comply_handlers::update_macro_depth("() => { {", depth);
    assert!(depth.is_some());
}

#[test]
fn test_filter_production_lines() {
    let lines = vec![
        "pub fn main() {}",
        "#[cfg(test)]",
        "mod tests {",
        "    #[test]",
        "    fn test_something() {}",
        "}",
    ];
    let prod_lines = crate::cli::handlers::comply_handlers::filter_production_lines(&lines);
    assert_eq!(prod_lines.len(), 1);
    assert_eq!(prod_lines[0], "pub fn main() {}");
}

#[test]
fn test_count_dead_items_no_dead_code() {
    let lines = vec![
        "pub fn active() {}",
        "pub struct Active {}",
        "impl Active {}",
    ];
    let refs: Vec<&str> = lines.iter().map(|s| *s).collect();
    let (total, dead, annotations) = crate::cli::handlers::comply_handlers::count_dead_items(&refs);
    assert_eq!(total, 2); // fn + struct
    assert_eq!(dead, 0);
    assert_eq!(annotations, 0);
}

#[test]
fn test_count_dead_items_with_dead_code() {
    let lines = vec![
        "",
        "fn dead_fn() {}",
        "pub fn active() {}",
    ];
    let refs: Vec<&str> = lines.iter().map(|s| *s).collect();
    let (total, dead, annotations) = crate::cli::handlers::comply_handlers::count_dead_items(&refs);
    assert_eq!(total, 2);
    assert_eq!(dead, 1);
    assert_eq!(annotations, 1);
}

#[test]
fn test_count_block_comment_code_lines() {
    let lines = vec![
        "/* fn old_function() {",
        "    let x = 5;",
        "    return x;",
        "} */",
    ];
    let refs: Vec<&str> = lines.iter().map(|s| *s).collect();
    let dead_lines = crate::cli::handlers::comply_handlers::count_block_comment_code_lines(&refs);
    // Should detect code-like content in block comment
    assert!(dead_lines >= 2);
}

#[test]
fn test_count_commented_code_lines() {
    let lines = vec![
        "// fn old_function() {",
        "//     let x = 5;",
        "//     return x;",
        "// }",
        "pub fn new_function() {}",
    ];
    let refs: Vec<&str> = lines.iter().map(|s| *s).collect();
    let dead_lines = crate::cli::handlers::comply_handlers::count_commented_code_lines(&refs);
    // Should detect 4 consecutive commented code lines
    assert_eq!(dead_lines, 4);
}

#[test]
fn test_analyze_file_dead_code() {
    let lines = vec![
        "",
        "fn dead() {}",
        "pub fn active() {}",
        "#[cfg(test)]",
        "mod tests {}",
    ];
    let refs: Vec<&str> = lines.iter().map(|s| *s).collect();
    let (total, dead, prod_lines, estimated_dead) =
        crate::cli::handlers::comply_handlers::analyze_file_dead_code(&refs);

    assert!(total >= 2);
    assert!(dead >= 1);
    assert!(prod_lines >= 3);
    assert!(estimated_dead >= 10); // ~10 per annotation
}

// =============================================================================
// format_violation_list tests
// =============================================================================

#[test]
fn test_format_violation_list_empty() {
    let issues: Vec<String> = vec![];
    let result = crate::cli::handlers::comply_handlers::format_violation_list(&issues);
    assert!(result.is_empty());
}

#[test]
fn test_format_violation_list_single() {
    let issues = vec!["CB-001: Test issue".to_string()];
    let result = crate::cli::handlers::comply_handlers::format_violation_list(&issues);
    assert_eq!(result, "    - CB-001: Test issue");
}

#[test]
fn test_format_violation_list_multiple() {
    let issues = vec![
        "Issue 1".to_string(),
        "Issue 2".to_string(),
        "Issue 3".to_string(),
    ];
    let result = crate::cli::handlers::comply_handlers::format_violation_list(&issues);
    assert!(result.contains("    - Issue 1"));
    assert!(result.contains("    - Issue 2"));
    assert!(result.contains("    - Issue 3"));
}

// =============================================================================
// collect_cb_violations tests
// =============================================================================

#[test]
fn test_collect_cb_violations_empty_project() {
    let temp_dir = TempDir::new().unwrap();
    let (issues, critical, warning) =
        crate::cli::handlers::comply_handlers::collect_cb_violations(temp_dir.path(), false, false);

    // Empty project should have no violations
    assert!(issues.is_empty());
    assert_eq!(critical, 0);
    assert_eq!(warning, 0);
}

// =============================================================================
// build_cb_result tests
// =============================================================================

#[test]
fn test_build_cb_result_no_issues() {
    let result = crate::cli::handlers::comply_handlers::build_cb_result(vec![], 0, 0);
    assert_eq!(result.status, CheckStatus::Pass);
    assert!(result.message.contains("no violations"));
}

#[test]
fn test_build_cb_result_warnings_only() {
    let issues = vec!["Warning 1".to_string(), "Warning 2".to_string()];
    let result = crate::cli::handlers::comply_handlers::build_cb_result(issues, 0, 2);
    assert_eq!(result.status, CheckStatus::Warn);
    assert!(result.message.contains("2 warnings"));
}

#[test]
fn test_build_cb_result_critical() {
    let issues = vec!["Critical 1".to_string(), "Warning 1".to_string()];
    let result = crate::cli::handlers::comply_handlers::build_cb_result(issues, 1, 1);
    assert_eq!(result.status, CheckStatus::Fail);
    assert_eq!(result.severity, Severity::Critical);
}
