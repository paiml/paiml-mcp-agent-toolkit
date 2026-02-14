// =============================================================================
// CB-608: Unchecked nil, err Return Pattern (#181)
// =============================================================================

mod cb608_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cb608_detects_unchecked_io_open() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local f = io.open('data.txt')\n",
        )
        .unwrap();
        let violations = detect_cb608_unchecked_nil_err(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-608");
        assert!(violations[0].description.contains("io.open"));
    }

    #[test]
    fn test_cb608_passes_when_error_captured() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local f, err = io.open('data.txt')\n",
        )
        .unwrap();
        let violations = detect_cb608_unchecked_nil_err(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb608_detects_unchecked_pcall() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "pcall(dangerous_function)\n",
        )
        .unwrap();
        let violations = detect_cb608_unchecked_nil_err(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("pcall"));
    }

    #[test]
    fn test_cb608_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app_test.lua"),
            "local f = io.open('test.txt')\n",
        )
        .unwrap();
        let violations = detect_cb608_unchecked_nil_err(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-609: assert() in Library Code (#193)
// =============================================================================

mod cb609_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cb609_detects_assert_in_library() {
        let temp = TempDir::new().unwrap();
        // assert on line 8 (past the 5-line require-guard threshold)
        let code = "local M = {}\nlocal utils = require('utils')\n\
                     local fmt = string.format\nlocal insert = table.insert\n\
                     local pairs = pairs\nlocal type = type\n\
                     function M.process(data)\n  assert(type(data) == 'table')\n\
                     return data\nend\nreturn M\n";
        fs::write(temp.path().join("lib.lua"), code).unwrap();
        let violations = detect_cb609_assert_in_library(temp.path());
        assert!(!violations.is_empty(), "Should detect assert() past line 5");
        assert_eq!(violations[0].pattern_id, "CB-609");
    }

    #[test]
    fn test_cb609_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("test_app.lua"),
            "assert(result == expected)\n",
        )
        .unwrap();
        let violations = detect_cb609_assert_in_library(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb609_skips_module_require_guards() {
        let temp = TempDir::new().unwrap();
        // assert in first 5 lines (module-level guards) should be skipped
        fs::write(
            temp.path().join("lib.lua"),
            "local ok = pcall(require, 'ffi')\nassert(ok, 'FFI required')\nlocal M = {}\nreturn M\n",
        )
        .unwrap();
        let violations = detect_cb609_assert_in_library(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb609_skips_test_framework_assertions() {
        let temp = TempDir::new().unwrap();
        // assert.is_true() is a test framework method, not plain assert()
        fs::write(
            temp.path().join("lib.lua"),
            "local M = {}\nfunction M.run()\n  -- noop\nend\nlocal x = assert.is_true\nreturn M\n",
        )
        .unwrap();
        let violations = detect_cb609_assert_in_library(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-610: String Accumulator in Loop (#190)
// =============================================================================

mod cb610_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_cb610_detects_string_accumulator() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("app.lua"),
            "local result = ''\nfor _, item in ipairs(items) do\n  result = result .. item\nend\n",
        )
        .unwrap();
        let violations = detect_cb610_string_accumulator_in_loop(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-610");
    }

    #[test]
    fn test_cb610_skips_single_use_concat() {
        let temp = TempDir::new().unwrap();
        // Single-use concat in loop is O(n), not O(n²) — should not be flagged
        fs::write(
            temp.path().join("app.lua"),
            "for _, item in ipairs(items) do\n  log('Processing: ' .. item.name)\nend\n",
        )
        .unwrap();
        let violations = detect_cb610_string_accumulator_in_loop(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb610_skips_outside_loop() {
        let temp = TempDir::new().unwrap();
        // Accumulator outside loop is fine (not O(n²))
        fs::write(
            temp.path().join("app.lua"),
            "local msg = prefix .. suffix\n",
        )
        .unwrap();
        let violations = detect_cb610_string_accumulator_in_loop(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb610_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("test_builder.lua"),
            "local result = ''\nfor _, item in ipairs(items) do\n  result = result .. item\nend\n",
        )
        .unwrap();
        let violations = detect_cb610_string_accumulator_in_loop(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-611: Weak Table Misuse (#186)
// =============================================================================

mod cb611_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb611_detects_string_key_on_weak_key_table() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("cache.lua"),
            r#"local cache = setmetatable({}, { __mode = "k" })
cache["my_key"] = some_value
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-611");
        assert!(violations[0].description.contains("string"));
    }

    #[test]
    fn test_cb611_detects_numeric_key_on_weak_key_table() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("pool.lua"),
            r#"local pool = setmetatable({}, { __mode = "k" })
pool[123] = conn
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("numeric"));
    }

    #[test]
    fn test_cb611_ignores_weak_value_tables() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("cache.lua"),
            r#"local cache = setmetatable({}, { __mode = "v" })
cache["my_key"] = some_value
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb611_ignores_weak_kv_tables() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("cache.lua"),
            r#"local cache = setmetatable({}, { __mode = "kv" })
cache["my_key"] = some_value
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb611_allows_table_key_on_weak_key_table() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("refs.lua"),
            r#"local refs = setmetatable({}, { __mode = "k" })
refs[obj] = true
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb611_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("test_cache.lua"),
            r#"local cache = setmetatable({}, { __mode = "k" })
cache["key"] = val
"#,
        )
        .unwrap();
        let violations = detect_cb611_weak_table_misuse(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-612: Test Framework Detection (#184)
// =============================================================================

mod cb612_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb612_detects_busted_via_spec_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join("app_spec.lua"), "describe('app', function() end)\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-612");
        assert!(violations[0].description.contains("busted"));
    }

    #[test]
    fn test_cb612_detects_busted_via_config() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join(".busted"), "return { default = { verbose = true } }\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("busted"));
    }

    #[test]
    fn test_cb612_detects_test_nginx() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        fs::create_dir(temp.path().join("t")).unwrap();
        fs::write(temp.path().join("t/001-basic.t"), "use Test::Nginx::Socket;\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("Test::Nginx"));
    }

    #[test]
    fn test_cb612_detects_hybrid_busted_and_test_nginx() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join("handler_spec.lua"), "describe('handler', function() end)\n").unwrap();
        fs::create_dir(temp.path().join("t")).unwrap();
        fs::write(temp.path().join("t/001.t"), "use Test::Nginx;\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("busted"));
        assert!(violations[0].description.contains("Test::Nginx"));
    }

    #[test]
    fn test_cb612_no_framework_few_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("init.lua"), "return {}\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        // Less than 3 Lua files, no warning
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb612_no_framework_many_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("a.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join("b.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join("c.lua"), "return {}\n").unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("No Lua test framework"));
    }

    #[test]
    fn test_cb612_detects_luaunit() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        fs::write(
            temp.path().join("test_app.lua"),
            "local lu = require('luaunit')\nfunction test_foo() end\n",
        )
        .unwrap();
        let violations = detect_cb612_test_framework(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("LuaUnit"));
    }
}

// =============================================================================
// CB-613: Require Cycle Detection (#187)
// =============================================================================

mod cb613_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb613_detects_simple_cycle() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("a.lua"),
            "local b = require('b')\nreturn {}\n",
        )
        .unwrap();
        fs::write(
            temp.path().join("b.lua"),
            "local a = require('a')\nreturn {}\n",
        )
        .unwrap();
        let violations = detect_cb613_require_cycles(temp.path());
        assert!(!violations.is_empty(), "Should detect a -> b -> a cycle");
        assert_eq!(violations[0].pattern_id, "CB-613");
        assert!(violations[0].description.contains("Circular"));
    }

    #[test]
    fn test_cb613_no_cycle() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("a.lua"),
            "local b = require('b')\nreturn {}\n",
        )
        .unwrap();
        fs::write(temp.path().join("b.lua"), "return {}\n").unwrap();
        let violations = detect_cb613_require_cycles(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb613_ignores_function_scoped_require() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("a.lua"),
            "local b = require('b')\nreturn {}\n",
        )
        .unwrap();
        fs::write(
            temp.path().join("b.lua"),
            "local M = {}\nfunction M.init()\n  local a = require('a')\nend\nreturn M\n",
        )
        .unwrap();
        let violations = detect_cb613_require_cycles(temp.path());
        // b's require('a') is inside a function, so no cycle
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb613_single_file_no_crash() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("a.lua"), "return {}\n").unwrap();
        let violations = detect_cb613_require_cycles(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb613_handles_dotted_requires() {
        let temp = TempDir::new().unwrap();
        // a requires foo.b, b requires foo.a — but cycle detection uses normalized names
        fs::write(
            temp.path().join("a.lua"),
            "local b = require(\"b\")\nreturn {}\n",
        )
        .unwrap();
        fs::write(
            temp.path().join("b.lua"),
            "local a = require(\"a\")\nreturn {}\n",
        )
        .unwrap();
        let violations = detect_cb613_require_cycles(temp.path());
        assert!(!violations.is_empty());
    }
}

// =============================================================================
// CB-614: Global Protection and Sandbox Detection (#191)
// =============================================================================

mod cb614_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb614_detects_full_protection() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("a.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join("b.lua"), "return {}\n").unwrap();
        fs::write(
            temp.path().join("strict.lua"),
            r#"local strict = {}
strict.__newindex = function(t, k, v) error("cannot set: " .. k) end
strict.__index = function(t, k) error("cannot get undefined: " .. k) end
setmetatable(_G, strict)
"#,
        )
        .unwrap();
        let violations = detect_cb614_global_protection(temp.path());
        let info: Vec<_> = violations.iter().filter(|v| v.file == "project").collect();
        assert!(!info.is_empty());
        assert!(info[0].description.contains("full"));
    }

    #[test]
    fn test_cb614_detects_partial_protection() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("a.lua"), "return {}\n").unwrap();
        fs::write(temp.path().join("b.lua"), "return {}\n").unwrap();
        fs::write(
            temp.path().join("strict.lua"),
            "setmetatable(_G, { __newindex = error })\n",
        )
        .unwrap();
        let violations = detect_cb614_global_protection(temp.path());
        let project: Vec<_> = violations.iter().filter(|v| v.file == "project").collect();
        assert!(!project.is_empty());
        assert!(project[0].description.contains("partial"));
    }

    #[test]
    fn test_cb614_flags_loadfile_without_t_mode() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("loader.lua"),
            "local chunk = loadfile(\"plugin.lua\")\n",
        )
        .unwrap();
        let violations = detect_cb614_global_protection(temp.path());
        let load_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.description.contains("loadfile"))
            .collect();
        assert!(!load_violations.is_empty());
        assert!(load_violations[0].description.contains("bytecode"));
    }

    #[test]
    fn test_cb614_allows_loadfile_with_t_mode() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("loader.lua"),
            "local chunk = loadfile(\"plugin.lua\", \"t\", env)\n",
        )
        .unwrap();
        let violations = detect_cb614_global_protection(temp.path());
        let load_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.description.contains("loadfile"))
            .collect();
        assert!(load_violations.is_empty());
    }

    #[test]
    fn test_cb614_no_lua_files() {
        let temp = TempDir::new().unwrap();
        let violations = detect_cb614_global_protection(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-615: Coroutine Complexity Checks (#188)
// =============================================================================

mod cb615_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb615_detects_resume_without_pcall() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("scheduler.lua"),
            "local co = coroutine.create(fn)\ncoroutine.resume(co)\n",
        )
        .unwrap();
        let violations = detect_cb615_coroutine_checks(temp.path());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].pattern_id, "CB-615");
        assert!(violations[0].description.contains("pcall"));
    }

    #[test]
    fn test_cb615_allows_resume_with_pcall() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("scheduler.lua"),
            "local ok, err = pcall(coroutine.resume, co)\n",
        )
        .unwrap();
        let violations = detect_cb615_coroutine_checks(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb615_allows_resume_with_ok_err_capture() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("worker.lua"),
            "local ok, err = coroutine.resume(co, arg)\n",
        )
        .unwrap();
        let violations = detect_cb615_coroutine_checks(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb615_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("test_coro.lua"),
            "coroutine.resume(co)\n",
        )
        .unwrap();
        let violations = detect_cb615_coroutine_checks(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb615_no_coroutines() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        let violations = detect_cb615_coroutine_checks(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-616: Type Annotation Awareness (#183)
// =============================================================================

mod cb616_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb616_detects_luals_annotations() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("module.lua"),
            r#"---@param name string
---@return boolean
function M.check(name)
  return true
end
function M.other() end
"#,
        )
        .unwrap();
        let violations = detect_cb616_type_annotations(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("LuaLS"));
        assert!(violations[0].description.contains("1/2"));
    }

    #[test]
    fn test_cb616_detects_ldoc_annotations() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("widget.lua"),
            r#"-- @tparam string name Widget name
-- @treturn Widget The new widget
function M.new(name) end
"#,
        )
        .unwrap();
        let violations = detect_cb616_type_annotations(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("LDoc"));
    }

    #[test]
    fn test_cb616_no_annotations_few_functions() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("small.lua"), "function init() end\n").unwrap();
        let violations = detect_cb616_type_annotations(temp.path());
        // Only 1 function, below threshold of 10
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb616_no_annotations_many_functions() {
        let temp = TempDir::new().unwrap();
        let mut content = String::new();
        for i in 0..12 {
            content.push_str(&format!("function fn_{i}() end\n"));
        }
        fs::write(temp.path().join("big.lua"), &content).unwrap();
        let violations = detect_cb616_type_annotations(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("No type annotations"));
    }

    #[test]
    fn test_cb616_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("test_mod.lua"),
            "---@param x number\nfunction test_add(x) end\n",
        )
        .unwrap();
        let violations = detect_cb616_type_annotations(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-617: OpenResty-Specific Checks (#185)
// =============================================================================

mod cb617_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb617_skips_non_openresty_project() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        let violations = detect_cb617_openresty_checks(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb617_detects_uncached_type_in_handler() {
        let temp = TempDir::new().unwrap();
        // Handler file with ngx.* usage (triggers OpenResty detection) and uncached type()
        fs::write(
            temp.path().join("handler.lua"),
            "local M = {}\nfunction _M.access(conf)\n    local h = ngx.var.host\n    local t = type(conf.name)\nend\nreturn M\n",
        )
        .unwrap();
        let violations = detect_cb617_openresty_checks(temp.path());
        let uncached: Vec<_> = violations
            .iter()
            .filter(|v| v.description.contains("Uncached"))
            .collect();
        assert!(!uncached.is_empty(), "Expected uncached type() violation, got {:?}", violations);
        assert!(uncached[0].description.contains("type"));
    }

    #[test]
    fn test_cb617_allows_cached_type_in_handler() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("handler.lua"),
            "local type = type\nlocal M = {}\nfunction _M.access(conf)\n    local h = ngx.var.host\n    local t = type(conf.name)\nend\nreturn M\n",
        )
        .unwrap();
        let violations = detect_cb617_openresty_checks(temp.path());
        let uncached: Vec<_> = violations
            .iter()
            .filter(|v| v.description.contains("Uncached"))
            .collect();
        assert!(uncached.is_empty());
    }
}

// =============================================================================
// CB-618: FFI Safety Checks (#189)
// =============================================================================

mod cb618_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb618_detects_ffi_usage() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("crypto.lua"),
            r#"local ffi = require("ffi")
local C = ffi.C
ffi.cdef[[int open(const char *path, int flags);]]
local fd = C.open("/dev/urandom", 0)
"#,
        )
        .unwrap();
        let violations = detect_cb618_ffi_safety(temp.path());
        // Should have the info message about FFI usage + warning about unchecked C.open
        let info: Vec<_> = violations.iter().filter(|v| v.file == "project").collect();
        assert!(!info.is_empty());
        assert!(info[0].description.contains("FFI used in 1 files"));
    }

    #[test]
    fn test_cb618_flags_unchecked_c_open() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("io.lua"),
            r#"local ffi = require("ffi")
local C = ffi.C
local fd = C.open(path, 0)
C.read(fd, buf, size)
"#,
        )
        .unwrap();
        let violations = detect_cb618_ffi_safety(temp.path());
        let warnings: Vec<_> = violations
            .iter()
            .filter(|v| v.description.contains("C.open"))
            .collect();
        assert!(!warnings.is_empty());
    }

    #[test]
    fn test_cb618_allows_checked_c_open() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("io.lua"),
            r#"local ffi = require("ffi")
local C = ffi.C
local fd = C.open(path, 0)
if fd < 0 then error("open failed") end
"#,
        )
        .unwrap();
        let violations = detect_cb618_ffi_safety(temp.path());
        let warnings: Vec<_> = violations
            .iter()
            .filter(|v| v.description.contains("C.open"))
            .collect();
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_cb618_no_ffi() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("app.lua"), "return {}\n").unwrap();
        let violations = detect_cb618_ffi_safety(temp.path());
        assert!(violations.is_empty());
    }
}

// =============================================================================
// CB-619: OOP Pattern Recognition (#182)
// =============================================================================

mod cb619_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cb619_detects_separate_metatable() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("class.lua"),
            r#"local M = {}
local mt = { __index = M }
function M.new(opts)
    return setmetatable({}, mt)
end
"#,
        )
        .unwrap();
        let violations = detect_cb619_oop_patterns(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("separate-metatable"));
    }

    #[test]
    fn test_cb619_detects_prototypal_inheritance() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("base.lua"),
            r#"local Base = {}
function Base:extend(o)
    o = o or {}
    setmetatable(o, self)
    self.__index = self
    return o
end
"#,
        )
        .unwrap();
        let violations = detect_cb619_oop_patterns(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("prototypal"));
    }

    #[test]
    fn test_cb619_detects_call_constructor() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("widget.lua"),
            r#"local M = {}
setmetatable(M, { __call = function(_, ...) return M.new(...) end })
"#,
        )
        .unwrap();
        let violations = detect_cb619_oop_patterns(temp.path());
        assert_eq!(violations.len(), 1);
        assert!(violations[0].description.contains("__call"));
    }

    #[test]
    fn test_cb619_no_oop() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("util.lua"), "local M = {}\nreturn M\n").unwrap();
        let violations = detect_cb619_oop_patterns(temp.path());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_cb619_skips_test_files() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("test_class.lua"),
            "local M = {}\nlocal mt = { __index = M }\nsetmetatable({}, mt)\n",
        )
        .unwrap();
        let violations = detect_cb619_oop_patterns(temp.path());
        assert!(violations.is_empty());
    }
}
