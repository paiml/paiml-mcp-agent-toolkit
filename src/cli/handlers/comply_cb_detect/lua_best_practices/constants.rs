#![cfg_attr(coverage_nightly, coverage(off))]
//! Constants used across Lua best practices detection modules.

/// Known Lua standard library globals that should not be flagged by CB-600.
pub(crate) const LUA_STD_GLOBALS: &[&str] = &[
    "assert",
    "collectgarbage",
    "dofile",
    "error",
    "getmetatable",
    "ipairs",
    "load",
    "loadfile",
    "next",
    "pairs",
    "pcall",
    "print",
    "rawequal",
    "rawget",
    "rawlen",
    "rawset",
    "require",
    "select",
    "setmetatable",
    "tonumber",
    "tostring",
    "type",
    "unpack",
    "xpcall",
    // Standard library tables
    "coroutine",
    "debug",
    "io",
    "math",
    "os",
    "package",
    "string",
    "table",
    "utf8",
    "bit32",
    "arg",
    // Common environment globals
    "self",
    "true",
    "false",
    "nil",
    "_G",
    "_ENV",
    "_VERSION",
];

/// Directories to skip when walking for Lua files.
pub(crate) const SKIP_DIRS: &[&str] = &[
    ".git",
    ".claude",
    "node_modules",
    "target",
    ".pmat",
    "vendor",
    "build",
    "dist",
];

/// Lua keywords and control flow prefixes that cannot be implicit globals.
pub(crate) const LUA_KEYWORD_PREFIXES: &[&str] = &[
    "local ",
    "function ",
    "if ",
    "for ",
    "while ",
    "repeat",
    "return",
    "end",
    "else",
    "elseif ",
    "until ",
    "break",
    "goto ",
    "::",
];

/// Standard library tables that commonly use dot notation (not colon).
pub(crate) const LUA_STD_TABLES: &[&str] = &[
    "math",
    "string",
    "table",
    "io",
    "os",
    "debug",
    "coroutine",
    "package",
    "utf8",
    "bit32",
];

/// Deprecated Lua APIs that have modern replacements.
pub(crate) const LUA_DEPRECATED_APIS: &[&str] = &["loadstring(", "setfenv(", "getfenv(", "module("];

/// Dangerous Lua APIs that enable command injection.
pub(crate) const LUA_DANGEROUS_APIS: &[&str] = &["os.execute(", "io.popen("];

/// Known Lua standard library functions that return `nil, err` on failure.
pub(crate) const NIL_ERR_FUNCTIONS: &[&str] = &[
    "io.open",
    "io.popen",
    "io.lines",
    "io.tmpfile",
    "os.execute",
    "os.rename",
    "os.remove",
    "load",
    "loadfile",
    "loadstring",
    "pcall",
    "xpcall",
    "require",
];

/// Generic status-check patterns for common naming conventions.
pub(crate) const STATUS_CHECK_PATTERNS: &[&str] = &[
    "if ok",
    "if not ok",
    "if success",
    "if not success",
    "if status",
    "if not status",
    "assert(ok",
    "assert(success",
];

/// Common Lua stdlib names that should be cached as locals in OpenResty hot paths.
pub(crate) const OPENRESTY_CACHEABLE_GLOBALS: &[&str] = &[
    "type",
    "pairs",
    "ipairs",
    "tostring",
    "tonumber",
    "select",
    "setmetatable",
    "getmetatable",
    "unpack",
    "error",
    "pcall",
    "string.byte",
    "string.char",
    "string.find",
    "string.format",
    "string.gsub",
    "string.len",
    "string.lower",
    "string.match",
    "string.rep",
    "string.sub",
    "string.upper",
    "table.insert",
    "table.remove",
    "table.concat",
    "table.sort",
    "math.floor",
    "math.ceil",
    "math.max",
    "math.min",
    "math.random",
];
