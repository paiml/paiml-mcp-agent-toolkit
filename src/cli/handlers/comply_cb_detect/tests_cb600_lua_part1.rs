// CB-600 Lua Best Practices tests Part 1: Helpers, CB-600 Implicit Globals, CB-601 Nil-Unsafe Access
// Included from tests.rs via include!() - shares parent module scope

// =========================================================================
// Helper function tests
// =========================================================================

#[test]
fn test_is_lua_test_file() {
    assert!(is_lua_test_file(std::path::Path::new("foo_test.lua")));
    assert!(is_lua_test_file(std::path::Path::new("bar_spec.lua")));
    assert!(is_lua_test_file(std::path::Path::new("test_baz.lua")));
    assert!(is_lua_test_file(std::path::Path::new("tests/util.lua")));
    assert!(is_lua_test_file(std::path::Path::new("spec/helper.lua")));
    assert!(!is_lua_test_file(std::path::Path::new("app.lua")));
    assert!(!is_lua_test_file(std::path::Path::new("module.lua")));
}

#[test]
fn test_compute_lua_production_lines_filters_comments() {
    let content = "-- comment\nlocal x = 1\n--[[ block\ncomment ]]\nlocal y = 2\n";
    let lines = compute_lua_production_lines(content);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].1, "local x = 1");
    assert_eq!(lines[1].1, "local y = 2");
}

#[test]
fn test_walkdir_lua_files_skips_git() {
    let temp = TempDir::new().unwrap();
    let git_dir = temp.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    fs::write(git_dir.join("hook.lua"), "local x = 1").unwrap();
    fs::write(temp.path().join("app.lua"), "local y = 2").unwrap();
    let files = walkdir_lua_files(temp.path());
    assert_eq!(files.len(), 1);
    assert!(files[0].file_name().unwrap() == "app.lua");
}

// =========================================================================
// CB-600: Implicit Globals
// =========================================================================

#[test]
fn test_cb600_detects_implicit_global() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.lua"), "counter = 0\nlocal x = 1\n").unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-600");
    assert!(violations[0].description.contains("counter"));
}

#[test]
fn test_cb600_skips_local_vars() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local counter = 0\nlocal name = 'test'\n",
    )
    .unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb600_skips_std_globals() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.lua"), "print = custom_print\n").unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb600_skips_table_field_assignment() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "M.name = 'test'\nself.value = 42\ntbl[key] = true\n",
    )
    .unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb600_skips_table_constructor_fields() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local t = {\n  id = id,\n  name = name,\n  score = 100,\n}\n",
    )
    .unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(
        violations.is_empty(),
        "table constructor fields are not globals: {:?}",
        violations
    );
}

#[test]
fn test_cb600_skips_inline_table_constructor() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local t = { id = id, name = name }\n",
    )
    .unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(
        violations.is_empty(),
        "inline table constructor fields are not globals: {:?}",
        violations
    );
}

#[test]
fn test_cb600_skips_test_files() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app_test.lua"), "counter = 0\n").unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb600_no_lua_files_empty() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.rs"), "fn main() {}").unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb600_skips_function_params() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "function M.levenshtein(a, b)\n  a = a or \"\"\n  b = b or \"\"\nend\n",
    )
    .unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(
        violations.is_empty(),
        "function params should not be flagged: {:?}",
        violations
    );
}

#[test]
fn test_cb600_skips_for_loop_vars() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "for i = 1, 10 do\n  i = i + 1\nend\nfor k, v in pairs(t) do\n  k = tostring(k)\nend\n",
    )
    .unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(
        violations.is_empty(),
        "for-loop vars should not be flagged: {:?}",
        violations
    );
}

#[test]
fn test_cb600_skips_local_decl_reassignment() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local result = 0\nresult = result + 1\n",
    )
    .unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(
        violations.is_empty(),
        "local var reassignment should not be flagged: {:?}",
        violations
    );
}

#[test]
fn test_cb600_skips_multi_local_decl() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local a, b = 1, 2\na = a + 1\nb = b + 1\n",
    )
    .unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert!(
        violations.is_empty(),
        "multi-local reassignment should not be flagged: {:?}",
        violations
    );
}

#[test]
fn test_cb600_still_detects_true_globals() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "function foo(x)\n  x = x + 1\nend\nglobal_thing = 42\n",
    )
    .unwrap();
    let violations = detect_cb600_implicit_globals(temp.path());
    assert_eq!(violations.len(), 1, "true global should still be caught");
    assert!(violations[0].description.contains("global_thing"));
}

// =========================================================================
// CB-601: Nil-Unsafe Access
// =========================================================================

#[test]
fn test_cb601_detects_chained_call() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local name = get_user().name\n",
    )
    .unwrap();
    let violations = detect_cb601_nil_unsafe_access(temp.path());
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].pattern_id, "CB-601");
    assert!(violations[0].description.contains("chained"));
}

#[test]
fn test_cb601_detects_deep_chain() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local x = config.server.host.port\n",
    )
    .unwrap();
    let violations = detect_cb601_nil_unsafe_access(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("deep field"));
}

#[test]
fn test_cb601_shallow_access_passes() {
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        "local x = config.host\nlocal y = tbl.key\n",
    )
    .unwrap();
    let violations = detect_cb601_nil_unsafe_access(temp.path());
    assert!(violations.is_empty());
}

#[test]
fn test_cb601_string_key_with_dots_not_false_positive() {
    // ["H.N.S.W."] and ["C.I.C.D."] are string-literal table keys,
    // not deep field access chains
    let temp = TempDir::new().unwrap();
    fs::write(
        temp.path().join("app.lua"),
        concat!(
            "local corrections = {}\n",
            "corrections[\"H.N.S.W.\"] = \"HNSW\"\n",
            "corrections[\"C.I.C.D.\"] = \"CICD\"\n",
            "corrections['R.A.G.'] = \"RAG\"\n",
        ),
    )
    .unwrap();
    let violations = detect_cb601_nil_unsafe_access(temp.path());
    assert!(
        violations.is_empty(),
        "Dots inside string-literal table keys should not be flagged: {:?}",
        violations
    );
}

#[test]
fn test_cb601_real_deep_chain_still_detected() {
    // Ensure real deep chains are still caught after the string fix
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("app.lua"), "local x = a.b.c.d\n").unwrap();
    let violations = detect_cb601_nil_unsafe_access(temp.path());
    assert_eq!(violations.len(), 1);
    assert!(violations[0].description.contains("deep field"));
}

#[test]
fn test_count_consecutive_field_access_skips_strings() {
    // Dots inside strings don't count
    assert!(count_consecutive_field_access("tbl[\"H.N.S.W.\"] = 1") < 4);
    assert!(count_consecutive_field_access("x['a.b.c.d.e'] = 1") < 4);
    // Real chains still count
    assert_eq!(count_consecutive_field_access("a.b.c.d"), 4);
    assert_eq!(count_consecutive_field_access("a.b.c"), 3);
    // Mixed: bracket access counts as 1 level but its contents don't add depth
    assert!(count_consecutive_field_access("tbl[\"key\"].field") < 4);
}

