// CB-600 Lua Best Practices tests Part 2: CB-602 pcall, CB-603 API, CB-604 Unused, CB-605 Concat, CB-606 Return, CB-607 Colon
// Included from tests.rs via include!() - shares parent module scope

// =========================================================================
// CB-602: pcall Error Handling
// =========================================================================

#[test]
fn test_cb602_uncaptured_pcall() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.lua"), "pcall(dangerous_fn)\n").unwrap();
    let violations = detect_cb602_pcall_error_handling(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("not captured"));
}

#[test]
fn test_cb602_unchecked_status() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local ok, result = pcall(fn_call)\nlocal x = result\n",
    )
    .unwrap();
    let violations = detect_cb602_pcall_error_handling(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("not checked"));
}

#[test]
fn test_cb602_checked_pcall_passes() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local ok, result = pcall(fn_call)\nif not ok then\n  error(result)\nend\n",
    )
    .unwrap();
    let violations = detect_cb602_pcall_error_handling(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb602_prefixed_variable_not_false_positive() {
    // Pattern: local wrap_ok = pcall(obj.method, obj, ...) / if wrap_ok then
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        concat!(
            "local wrap_ok, wrap_err = pcall(obj.wrap, obj, config)\n",
            "if wrap_ok then\n",
            "  config = {applied = true}\n",
            "end\n",
        ),
    )
    .unwrap();
    let violations = detect_cb602_pcall_error_handling(temp.path());
    assert!(
        violations.is_empty(),
        "pcall with prefixed var checked on next line should not be flagged: {:?}",
        violations
    );
}

#[test]
fn test_cb602_multiple_prefixed_vars_pass() {
    // All 4 false positive patterns from CB-602 audit
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        concat!(
            "local lint_ok, lint_result = pcall(obj.lint, obj, source)\n",
            "if lint_ok then\n",
            "  process(lint_result)\n",
            "end\n",
            "\n",
            "local qe_ok, qe_result = pcall(query_engine.execute, query_engine, q)\n",
            "if not qe_ok then\n",
            "  error(qe_result)\n",
            "end\n",
            "\n",
            "local export_ok, data = pcall(obj.export, obj, fmt)\n",
            "if export_ok then\n",
            "  save(data)\n",
            "end\n",
        ),
    )
    .unwrap();
    let violations = detect_cb602_pcall_error_handling(temp.path());
    assert!(
        violations.is_empty(),
        "All prefixed pcall vars checked on next line: {:?}",
        violations
    );
}

#[test]
fn test_cb602_extract_pcall_status_var() {
    assert_eq!(
        extract_pcall_status_var("local ok, err = pcall(fn)"),
        Some("ok".to_string())
    );
    assert_eq!(
        extract_pcall_status_var("local wrap_ok, wrap_err = pcall(obj.method, obj)"),
        Some("wrap_ok".to_string())
    );
    assert_eq!(
        extract_pcall_status_var("status = pcall(fn)"),
        Some("status".to_string())
    );
    assert_eq!(extract_pcall_status_var("pcall(fn)"), None);
}

// =========================================================================
// CB-603: Deprecated/Dangerous API
// =========================================================================

#[test]
fn test_cb603_detects_loadstring() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.lua"), "local fn = loadstring(code)\n").unwrap();
    let violations = detect_cb603_deprecated_dangerous_api(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("loadstring"));
}

#[test]
fn test_cb603_detects_os_execute() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.lua"), "os.execute(cmd)\n").unwrap();
    let violations = detect_cb603_deprecated_dangerous_api(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("os.execute"));
}

#[test]
fn test_cb603_skips_comments() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "-- loadstring(code)\nlocal x = 1\n",
    )
    .unwrap();
    let violations = detect_cb603_deprecated_dangerous_api(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb603_hardcoded_string_is_info_severity() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.lua"), "os.execute(\"make clean\")\n").unwrap();
    let violations = detect_cb603_deprecated_dangerous_api(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(
        violations[0].severity,
        Severity::Info,
        "Hardcoded string arg should be Info"
    );
    assert!(violations[0].description.contains("hardcoded"));
}

#[test]
fn test_cb603_concatenation_is_warning() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "os.execute(\"rm -rf \" .. user_input)\n",
    )
    .unwrap();
    let violations = detect_cb603_deprecated_dangerous_api(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(
        violations[0].severity,
        Severity::Warning,
        "Concatenation should be Warning"
    );
    assert!(violations[0].description.contains("command injection"));
}

#[test]
fn test_cb603_variable_arg_is_warning() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.lua"), "os.execute(cmd)\n").unwrap();
    let violations = detect_cb603_deprecated_dangerous_api(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].severity, Severity::Warning);
}

#[test]
fn test_cb603_inline_suppression() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        concat!(
            "os.execute(\"make build\") -- pmat:ignore CB-603\n",
            "os.execute(\"make test\")\n",
        ),
    )
    .unwrap();
    let violations = detect_cb603_deprecated_dangerous_api(temp.path());
    assert_eq!(violations.len(), 1, "Suppressed line should not be flagged");
    assert_eq!(
        violations[0].line, 2,
        "Only unsuppressed line should be flagged"
    );
}

#[test]
fn test_cb603_bare_pmat_ignore_suppresses_all() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "io.popen(\"ls\") -- pmat:ignore\n",
    )
    .unwrap();
    let violations = detect_cb603_deprecated_dangerous_api(temp.path());
    assert!(
        violations.is_empty(),
        "Bare pmat:ignore should suppress all"
    );
}

// =========================================================================
// CB-604: Unused Variables
// =========================================================================

#[test]
fn test_cb604_detects_unused_var() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local unused = compute()\nlocal used = 1\nprint(used)\n",
    )
    .unwrap();
    let violations = detect_cb604_unused_variables(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("unused"));
}

#[test]
fn test_cb604_underscore_prefix_skipped() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.lua"), "local _ignored = compute()\n").unwrap();
    let violations = detect_cb604_unused_variables(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb604_used_var_passes() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local name = get_name()\nprint(name)\n",
    )
    .unwrap();
    let violations = detect_cb604_unused_variables(temp.path());
    assert!(violations.is_empty());
}

// =========================================================================
// CB-605: String Concat in Loop
// =========================================================================

#[test]
fn test_cb605_detects_concat_in_loop() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "for i = 1, 10 do\n  result = result .. item\nend\n",
    )
    .unwrap();
    let violations = detect_cb605_string_concat_in_loop(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("concatenation"));
}

#[test]
fn test_cb605_concat_outside_loop_passes() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local msg = greeting .. name\n",
    )
    .unwrap();
    let violations = detect_cb605_string_concat_in_loop(temp.path());
    assert!(violations.is_empty());
}

// =========================================================================
// CB-606: Missing Module Return
// =========================================================================

#[test]
fn test_cb606_detects_missing_return() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("module.lua"),
        "local M = {}\nfunction M.hello()\n  print('hi')\nend\n",
    )
    .unwrap();
    let violations = detect_cb606_missing_module_return(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("return M"));
}

#[test]
fn test_cb606_return_present_passes() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("module.lua"),
        "local M = {}\nfunction M.hello()\n  print('hi')\nend\nreturn M\n",
    )
    .unwrap();
    let violations = detect_cb606_missing_module_return(temp.path());
    assert!(violations.is_empty());
}

// =========================================================================
// CB-607: Colon/Dot Confusion
// =========================================================================

#[test]
fn test_cb607_detects_mixed_style() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "player:move()\nplayer.jump()\n",
    )
    .unwrap();
    let violations = detect_cb607_colon_dot_confusion(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("player"));
}

#[test]
fn test_cb607_consistent_usage_passes() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "player:move()\nplayer:jump()\n",
    )
    .unwrap();
    let violations = detect_cb607_colon_dot_confusion(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb607_std_library_skipped() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "math.floor(x)\nmath.ceil(y)\nstring.format('hi')\n",
    )
    .unwrap();
    let violations = detect_cb607_colon_dot_confusion(temp.path());
    assert!(violations.is_empty());
}
