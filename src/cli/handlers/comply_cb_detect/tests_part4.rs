#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::fs;

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
        fs::write(temp.path().join("test_coro.lua"), "coroutine.resume(co)\n").unwrap();
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
        assert!(
            !uncached.is_empty(),
            "Expected uncached type() violation, got {:?}",
            violations
        );
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
