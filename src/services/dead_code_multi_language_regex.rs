// Pre-compiled regex patterns (compiled once at module load time)
lazy_static! {
    // C function definition regex
    static ref C_DEF_REGEX: Regex = Regex::new(
        r"(?m)^\s*(?:static\s+)?(?:inline\s+)?(?:void|int|char|float|double|long|short|unsigned|struct\s+\w+|enum\s+\w+|\w+\s+\*?)\s+(\w+)\s*\([^)]*\)\s*(?:\{|$)"
    ).expect("Invalid regex");

    // C function call regex
    static ref C_CALL_REGEX: Regex = Regex::new(r"\b(\w+)\s*\(").expect("Invalid regex");

    // C function declaration regex
    static ref C_DECLARATION_REGEX: Regex = Regex::new(
        r"^\s*(?:static\s+)?(?:inline\s+)?(?:extern\s+)?(?:void|int|char|float|double|long|short|unsigned|struct\s+\w+|enum\s+\w+|\w+\s+\*?)\s+\w+\s*\("
    ).expect("Invalid regex");

    // Python function definition regex
    static ref PY_DEF_REGEX: Regex = Regex::new(r"(?m)^\s*def\s+(\w+)\s*\(").expect("Invalid regex");

    // Python function call regex
    static ref PY_CALL_REGEX: Regex = Regex::new(r"\b(\w+)\s*\(").expect("Invalid regex");

    // Python def check regex
    static ref PY_DEF_CHECK_REGEX: Regex = Regex::new(r"^\s*def\s+\w+\s*\(").expect("Invalid regex");

    // Rust function definition regex
    static ref RUST_DEF_REGEX: Regex = Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*[<(]").expect("Invalid regex");

    // Rust function call regex
    static ref RUST_CALL_REGEX: Regex = Regex::new(r"\b(\w+)\s*[!]?\(").expect("Invalid regex");

    // Rust fn definition check regex
    static ref RUST_FN_DEF_REGEX: Regex = Regex::new(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+\w+").expect("Invalid regex");

    // Lua local function definition: local function name(...)
    static ref LUA_LOCAL_FUNC_REGEX: Regex = Regex::new(
        r"(?m)^\s*local\s+function\s+(\w+)\s*\("
    ).expect("Invalid regex");

    // Lua global function definition: function name(...)
    static ref LUA_GLOBAL_FUNC_REGEX: Regex = Regex::new(
        r"(?m)^\s*function\s+(\w+)\s*\("
    ).expect("Invalid regex");

    // Lua module function: function M.name(...) or function M:name(...)
    static ref LUA_MODULE_FUNC_NAME_REGEX: Regex = Regex::new(
        r"(?m)^\s*function\s+(\w+)[.:](\w+)\s*\("
    ).expect("Invalid regex");

    // Lua function call regex
    static ref LUA_CALL_REGEX: Regex = Regex::new(r"\b(\w+)\s*\(").expect("Invalid regex");

    // Lua module return: return M (last non-empty line)
    static ref LUA_RETURN_MODULE_REGEX: Regex = Regex::new(
        r"^\s*return\s+(\w+)\s*$"
    ).expect("Invalid regex");

    // Lua table field function: M.name = function(...)
    static ref LUA_TABLE_FUNC_REGEX: Regex = Regex::new(
        r"(?m)^\s*(\w+)\.(\w+)\s*=\s*function\s*\("
    ).expect("Invalid regex");
}
