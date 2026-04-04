// Pre-compiled regex patterns using std::sync::LazyLock (infallible for constant patterns)
//
// All patterns are hardcoded string literals validated by tests.
// LazyLock replaces legacy lazy_static! with zero-overhead std-only initialization.

use std::sync::LazyLock;

// C function definition regex
static C_DEF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^\s*(?:static\s+)?(?:inline\s+)?(?:void|int|char|float|double|long|short|unsigned|struct\s+\w+|enum\s+\w+|\w+\s+\*?)\s+(\w+)\s*\([^)]*\)\s*(?:\{|$)"
    ).expect("C_DEF_REGEX: hardcoded constant pattern")
});

// C function call regex
static C_CALL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\w+)\s*\(").expect("C_CALL_REGEX: hardcoded constant pattern")
});

// C function declaration regex
static C_DECLARATION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*(?:static\s+)?(?:inline\s+)?(?:extern\s+)?(?:void|int|char|float|double|long|short|unsigned|struct\s+\w+|enum\s+\w+|\w+\s+\*?)\s+\w+\s*\("
    ).expect("C_DECLARATION_REGEX: hardcoded constant pattern")
});

// Python function definition regex
static PY_DEF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*def\s+(\w+)\s*\(").expect("PY_DEF_REGEX: hardcoded constant pattern")
});

// Python function call regex
static PY_CALL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\w+)\s*\(").expect("PY_CALL_REGEX: hardcoded constant pattern")
});

// Python def check regex
static PY_DEF_CHECK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*def\s+\w+\s*\(").expect("PY_DEF_CHECK_REGEX: hardcoded constant pattern")
});

// Rust function definition regex
static RUST_DEF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*[<(]")
        .expect("RUST_DEF_REGEX: hardcoded constant pattern")
});

// Rust function call regex
static RUST_CALL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\w+)\s*[!]?\(").expect("RUST_CALL_REGEX: hardcoded constant pattern")
});

// Rust fn definition check regex
static RUST_FN_DEF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+\w+")
        .expect("RUST_FN_DEF_REGEX: hardcoded constant pattern")
});

// Lua local function definition: local function name(...)
static LUA_LOCAL_FUNC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*local\s+function\s+(\w+)\s*\(")
        .expect("LUA_LOCAL_FUNC_REGEX: hardcoded constant pattern")
});

// Lua global function definition: function name(...)
static LUA_GLOBAL_FUNC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*function\s+(\w+)\s*\(")
        .expect("LUA_GLOBAL_FUNC_REGEX: hardcoded constant pattern")
});

// Lua module function: function M.name(...) or function M:name(...)
static LUA_MODULE_FUNC_NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*function\s+(\w+)[.:](\w+)\s*\(")
        .expect("LUA_MODULE_FUNC_NAME_REGEX: hardcoded constant pattern")
});

// Lua function call regex
static LUA_CALL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\w+)\s*\(").expect("LUA_CALL_REGEX: hardcoded constant pattern")
});

// Lua module return: return M (last non-empty line)
static LUA_RETURN_MODULE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*return\s+(\w+)\s*$")
        .expect("LUA_RETURN_MODULE_REGEX: hardcoded constant pattern")
});

// Lua table field function: M.name = function(...)
static LUA_TABLE_FUNC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*(\w+)\.(\w+)\s*=\s*function\s*\(")
        .expect("LUA_TABLE_FUNC_REGEX: hardcoded constant pattern")
});
