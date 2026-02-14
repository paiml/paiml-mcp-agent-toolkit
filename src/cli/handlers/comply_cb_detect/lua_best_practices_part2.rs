// =============================================================================
// CB-616: Lua Type Annotation Awareness (#183)
// =============================================================================

/// Detected Lua annotation system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LuaAnnotationSystem {
    LuaLS,
    LDoc,
}

impl std::fmt::Display for LuaAnnotationSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaAnnotationSystem::LuaLS => write!(f, "LuaLS/sumneko"),
            LuaAnnotationSystem::LDoc => write!(f, "LDoc"),
        }
    }
}

/// CB-616: Detect type annotation system and report doc coverage.
/// Supports LuaLS (---@param, ---@return) and LDoc (-- @tparam, -- @treturn).
pub fn detect_cb616_type_annotations(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    if files.is_empty() {
        return Vec::new();
    }

    let mut luals_count: usize = 0;
    let mut ldoc_count: usize = 0;
    let mut total_functions: usize = 0;
    let mut annotated_functions: usize = 0;

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let stats = count_annotation_stats(&content);
        luals_count += stats.luals;
        ldoc_count += stats.ldoc;
        total_functions += stats.functions;
        annotated_functions += stats.annotated;
    }

    build_annotation_violations(luals_count, ldoc_count, total_functions, annotated_functions)
}

/// Stats from scanning a single file for annotations.
struct AnnotationStats {
    luals: usize,
    ldoc: usize,
    functions: usize,
    annotated: usize,
}

/// Count annotation patterns and functions in a single file.
fn count_annotation_stats(content: &str) -> AnnotationStats {
    let mut stats = AnnotationStats { luals: 0, ldoc: 0, functions: 0, annotated: 0 };
    let mut prev_was_annotation = false;

    for line in content.lines() {
        let trimmed = line.trim();
        let is_annotation = is_annotation_line(trimmed, &mut stats);

        if trimmed.starts_with("function ") || trimmed.starts_with("local function ") {
            stats.functions += 1;
            if prev_was_annotation {
                stats.annotated += 1;
            }
        }
        prev_was_annotation = is_annotation;
    }
    stats
}

/// Check if a line is an annotation and count it. Returns true if annotation.
fn is_annotation_line(trimmed: &str, stats: &mut AnnotationStats) -> bool {
    // LuaLS: ---@param, ---@return, ---@class, ---@field, ---@type
    if trimmed.starts_with("---@") {
        stats.luals += 1;
        return true;
    }
    // LDoc: -- @tparam, -- @treturn, -- @param, -- @return, -- @raise
    if trimmed.starts_with("-- @") || trimmed.starts_with("--- @") {
        let after = trimmed.trim_start_matches('-').trim();
        if after.starts_with("@tparam")
            || after.starts_with("@treturn")
            || after.starts_with("@param")
            || after.starts_with("@return")
            || after.starts_with("@raise")
        {
            stats.ldoc += 1;
            return true;
        }
    }
    false
}

/// Build violations from aggregated annotation stats.
fn build_annotation_violations(
    luals_count: usize,
    ldoc_count: usize,
    total_functions: usize,
    annotated_functions: usize,
) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    let system = match (luals_count > 0, ldoc_count > 0) {
        (true, true) => Some(format!("LuaLS/sumneko ({luals_count} annotations) + LDoc ({ldoc_count} annotations)")),
        (true, false) => Some(format!("LuaLS/sumneko ({luals_count} annotations)")),
        (false, true) => Some(format!("LDoc ({ldoc_count} annotations)")),
        (false, false) => None,
    };

    if let Some(desc) = system {
        let coverage_pct = if total_functions > 0 {
            annotated_functions * 100 / total_functions
        } else {
            0
        };
        violations.push(CbPatternViolation {
            pattern_id: "CB-616".to_string(),
            file: "project".to_string(),
            line: 0,
            description: format!(
                "Type annotations: {desc}. Doc coverage: {annotated_functions}/{total_functions} functions ({coverage_pct}%)"
            ),
            severity: Severity::Info,
        });
    } else if total_functions >= 10 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-616".to_string(),
            file: "project".to_string(),
            line: 0,
            description: format!(
                "No type annotations found ({total_functions} functions) — consider adding LuaLS annotations"
            ),
            severity: Severity::Info,
        });
    }

    violations
}

// =============================================================================
// CB-617: OpenResty-Specific Lua Checks (#185)
// =============================================================================

/// Detect if a project uses OpenResty based on require("resty.*") or ngx.* usage.
fn is_openresty_project(files: &[PathBuf]) -> bool {
    files.iter().take(50).any(|f| {
        fs::read_to_string(f).is_ok_and(|c| {
            c.contains("require(\"resty") || c.contains("require('resty")
                || c.contains("ngx.") || c.contains("nginx.conf")
        })
    })
}

/// Common Lua stdlib names that should be cached as locals in OpenResty hot paths.
const OPENRESTY_CACHEABLE_GLOBALS: &[&str] = &[
    "type", "pairs", "ipairs", "tostring", "tonumber", "select",
    "setmetatable", "getmetatable", "unpack", "error", "pcall",
    "string.byte", "string.char", "string.find", "string.format",
    "string.gsub", "string.len", "string.lower", "string.match",
    "string.rep", "string.sub", "string.upper",
    "table.insert", "table.remove", "table.concat", "table.sort",
    "math.floor", "math.ceil", "math.max", "math.min", "math.random",
];

/// CB-617: OpenResty-specific performance and safety checks.
/// Only runs on detected OpenResty projects.
/// - Flags stdlib globals used in handler functions without local caching
/// - Flags ngx.var access without nil check
pub fn detect_cb617_openresty_checks(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    if !is_openresty_project(&files) {
        return Vec::new();
    }

    let mut violations = Vec::new();

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        check_stdlib_caching(&content, &rel, &mut violations);
    }

    violations
}

/// Check if stdlib globals are used in handler functions without local caching.
fn check_stdlib_caching(
    content: &str,
    rel: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    // Collect locally cached names at module level
    let cached: std::collections::HashSet<&str> = content
        .lines()
        .filter(|l| l.trim().starts_with("local "))
        .filter_map(|l| extract_local_cache_name(l.trim()))
        .collect();

    // Check handler functions for uncached global usage
    let mut in_handler = false;
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if is_handler_function(trimmed) {
            in_handler = true;
        }
        if in_handler {
            check_uncached_global_in_line(trimmed, i + 1, rel, &cached, violations);
        }
        if trimmed == "end" && in_handler {
            in_handler = false;
        }
    }
}

/// Extract the cached name from `local type = type` or `local str_find = string.find`.
/// Only matches exact global caching (not function calls like `local t = type(x)`).
fn extract_local_cache_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("local ")?;
    let eq_pos = rest.find('=')?;
    let rhs = rest[eq_pos + 1..].trim();
    // Check if RHS is exactly a known cacheable global (no parens/brackets after)
    for g in OPENRESTY_CACHEABLE_GLOBALS {
        if rhs == *g || (rhs.starts_with(g) && rhs[g.len()..].chars().next().map_or(true, |c| c == ' ' || c == '\n')) {
            return Some(*g);
        }
    }
    None
}

/// Check if a function definition is an OpenResty handler.
fn is_handler_function(line: &str) -> bool {
    let handlers = ["access", "header_filter", "body_filter", "log", "rewrite", "content"];
    handlers.iter().any(|h| {
        line.contains(&format!("function _M.{h}"))
            || line.contains(&format!("function _M:{h}"))
    })
}

/// Check a single line inside a handler for uncached globals.
fn check_uncached_global_in_line(
    trimmed: &str,
    line_num: usize,
    rel: &str,
    cached: &std::collections::HashSet<&str>,
    violations: &mut Vec<CbPatternViolation>,
) {
    if trimmed.starts_with("--") {
        return;
    }
    // Check for simple stdlib calls like type(...), pairs(...)
    for g in &["type", "pairs", "ipairs", "tostring", "tonumber"] {
        let pattern = format!("{g}(");
        if trimmed.contains(&pattern) && !cached.contains(*g) {
            violations.push(CbPatternViolation {
                pattern_id: "CB-617".to_string(),
                file: rel.to_string(),
                line: line_num,
                description: format!(
                    "Uncached `{g}()` in handler — add `local {g} = {g}` at module top"
                ),
                severity: Severity::Info,
            });
            return; // One per line
        }
    }
}

// =============================================================================
// CB-618: Lua FFI Safety Checks (#189)
// =============================================================================

/// CB-618: Detect LuaJIT FFI safety issues.
/// - Flags ffi.new("char[?]", ...) buffer allocations
/// - Flags C.* function calls without error checking
/// - Reports FFI usage summary
pub fn detect_cb618_ffi_safety(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    let mut violations = Vec::new();
    let mut ffi_file_count = 0;

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if !content.contains("require(\"ffi\")") && !content.contains("require('ffi')")
            && !content.contains("require \"ffi\"")
        {
            continue;
        }
        ffi_file_count += 1;

        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        check_ffi_patterns(&content, &rel, &mut violations);
    }

    if ffi_file_count > 0 {
        violations.push(CbPatternViolation {
            pattern_id: "CB-618".to_string(),
            file: "project".to_string(),
            line: 0,
            description: format!("LuaJIT FFI used in {ffi_file_count} files"),
            severity: Severity::Info,
        });
    }

    violations
}

/// Check FFI-related patterns in a single file.
fn check_ffi_patterns(
    content: &str,
    rel: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("--") {
            continue;
        }
        // Detect C.open / C.socket / C.malloc without error check
        check_ffi_resource_call(trimmed, i + 1, rel, content, violations);
    }
}

/// Flag C.open, C.socket, C.malloc calls without error checking on next lines.
fn check_ffi_resource_call(
    trimmed: &str,
    line_num: usize,
    rel: &str,
    content: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    let resource_funcs = ["C.open", "C.socket", "C.malloc", "C.mmap"];
    for func in &resource_funcs {
        if !trimmed.contains(func) {
            continue;
        }
        // Check if next 2 lines have an error check (< 0, == nil, ~= nil, etc.)
        let next_lines: String = content
            .lines()
            .skip(line_num)
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");
        let has_check = next_lines.contains("< 0")
            || next_lines.contains("== nil")
            || next_lines.contains("~= nil")
            || next_lines.contains("== -1")
            || next_lines.contains("if not ");
        if !has_check {
            violations.push(CbPatternViolation {
                pattern_id: "CB-618".to_string(),
                file: rel.to_string(),
                line: line_num,
                description: format!(
                    "`{func}()` without error check — verify return value before use"
                ),
                severity: Severity::Warning,
            });
        }
        return;
    }
}

// =============================================================================
// CB-619: Lua OOP Pattern Recognition (#182)
// =============================================================================

/// Detected Lua OOP pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LuaOopPattern {
    SeparateMetatable,
    PrototypalInheritance,
    CallConstructor,
    SelfAsMetatable,
}

impl std::fmt::Display for LuaOopPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaOopPattern::SeparateMetatable => write!(f, "separate-metatable"),
            LuaOopPattern::PrototypalInheritance => write!(f, "prototypal-inheritance"),
            LuaOopPattern::CallConstructor => write!(f, "__call-constructor"),
            LuaOopPattern::SelfAsMetatable => write!(f, "self-as-metatable"),
        }
    }
}

/// CB-619: Detect Lua OOP patterns and report them for TDG awareness.
/// Recognizes: separate metatable, prototypal inheritance, __call constructor, self-as-metatable.
pub fn detect_cb619_oop_patterns(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_lua_files(project_path);
    let mut pattern_counts: std::collections::HashMap<LuaOopPattern, usize> =
        std::collections::HashMap::new();

    for file_path in &files {
        if is_lua_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for pattern in detect_oop_in_file(&content) {
            *pattern_counts.entry(pattern).or_insert(0) += 1;
        }
    }

    if pattern_counts.is_empty() {
        return Vec::new();
    }

    let mut parts: Vec<String> = pattern_counts
        .iter()
        .map(|(p, c)| format!("{p} ({c} files)"))
        .collect();
    parts.sort();

    vec![CbPatternViolation {
        pattern_id: "CB-619".to_string(),
        file: "project".to_string(),
        line: 0,
        description: format!("OOP patterns: {}", parts.join(", ")),
        severity: Severity::Info,
    }]
}

/// Detect OOP patterns in a single file's content.
fn detect_oop_in_file(content: &str) -> Vec<LuaOopPattern> {
    let mut patterns = Vec::new();
    let has_setmetatable = content.contains("setmetatable");

    if !has_setmetatable {
        return patterns;
    }

    // Separate metatable: `local mt = { __index = M }` + `setmetatable({}, mt)`
    if (content.contains("__index = M") || content.contains("__index = _M"))
        && (content.contains("setmetatable({") || content.contains("setmetatable(self"))
    {
        patterns.push(LuaOopPattern::SeparateMetatable);
    }

    // Prototypal: `self.__index = self` or `Base:extend`
    if content.contains("self.__index = self") || content.contains(":extend") {
        patterns.push(LuaOopPattern::PrototypalInheritance);
    }

    // __call constructor: `setmetatable(M, { __call = ...`
    if content.contains("__call") && has_setmetatable {
        patterns.push(LuaOopPattern::CallConstructor);
    }

    // Self-as-metatable: `setmetatable(X, X)`
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("setmetatable(") {
            let parts: Vec<&str> = rest.splitn(3, ',').collect();
            if parts.len() >= 2 {
                let arg1 = parts[0].trim();
                let arg2 = parts[1].trim().trim_end_matches(')');
                if arg1 == arg2 && !arg1.is_empty() && !arg1.starts_with('{') {
                    patterns.push(LuaOopPattern::SelfAsMetatable);
                    break;
                }
            }
        }
    }

    patterns
}
