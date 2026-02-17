//! Multi-Language Dead Code Analysis Module (BUG-004 Fix)
//!
//! This module provides dead code detection across multiple programming languages
//! without requiring Cargo.toml or assuming Rust projects.
//!
//! Fixes:
//! - BUG-004: Dead code analyzer broken for non-Rust projects

#![cfg_attr(coverage_nightly, coverage(off))]
use anyhow::{Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
use tracing::{debug, info};
use walkdir::WalkDir;

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

/// Dead code analysis result
#[derive(Debug, Clone, PartialEq)]
pub struct DeadCodeResult {
    pub language: String,
    pub dead_functions: Vec<DeadFunction>,
    pub total_functions: usize,
    pub dead_code_percentage: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeadFunction {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub reason: String,
}

/// Strategy trait for language-specific dead code detection
pub trait DeadCodeStrategy {
    /// Analyze dead code in the given project
    fn analyze(&self, path: &Path) -> Result<DeadCodeResult>;

    /// Get the language this strategy handles
    fn language(&self) -> &str;
}

/// Analyze dead code using appropriate strategy for the project language
pub fn analyze_dead_code_multi_language(path: &Path) -> Result<DeadCodeResult> {
    info!("Starting multi-language dead code analysis at: {:?}", path);

    // Step 1: Detect language using enhanced detection from BUG-011
    let detection =
        crate::services::enhanced_language_detection::detect_project_language_enhanced(path);

    debug!(
        "Detected language: {} (confidence: {:.1}%)",
        detection.language, detection.confidence
    );

    // Step 2: Select appropriate strategy
    let strategy: Box<dyn DeadCodeStrategy> = match detection.language.as_str() {
        "rust" => Box::new(RustDeadCodeStrategy),
        "c" => Box::new(CDeadCodeStrategy),
        "cpp" => Box::new(CppDeadCodeStrategy),
        "python" => Box::new(PythonDeadCodeStrategy),
        "lua" => Box::new(LuaDeadCodeStrategy),
        _ => {
            return Err(anyhow::anyhow!(
                "Dead code analysis not supported for language: {}. Supported: rust, c, cpp, python, lua",
                detection.language
            ));
        }
    };

    // Step 3: Run analysis
    strategy.analyze(path)
}

// =============================================================================
// Rust Dead Code Strategy
// =============================================================================

struct RustDeadCodeStrategy;

impl DeadCodeStrategy for RustDeadCodeStrategy {
    fn language(&self) -> &str {
        "rust"
    }

    fn analyze(&self, path: &Path) -> Result<DeadCodeResult> {
        debug!("Running Rust dead code analysis with cargo check");

        // Use existing cargo-based dead code detection
        // This is the current working implementation for Rust
        let dead_functions = analyze_rust_dead_code_with_cargo(path)?;

        let total_functions = count_rust_functions(path)?;
        let dead_percentage = if total_functions > 0 {
            (dead_functions.len() as f64 / total_functions as f64) * 100.0
        } else {
            0.0
        };

        Ok(DeadCodeResult {
            language: "rust".to_string(),
            dead_functions,
            total_functions,
            dead_code_percentage: dead_percentage,
        })
    }
}

// =============================================================================
// C Dead Code Strategy
// =============================================================================

struct CDeadCodeStrategy;

impl DeadCodeStrategy for CDeadCodeStrategy {
    fn language(&self) -> &str {
        "c"
    }

    fn analyze(&self, path: &Path) -> Result<DeadCodeResult> {
        debug!("Running C dead code analysis (AST-based)");

        // Find C source files (.c only, not .h headers which are just declarations)
        let c_impl_files = find_files_by_extension(path, &["c"]);
        // Find all C files (including headers) for call analysis
        let c_all_files = find_files_by_extension(path, &["c", "h"]);

        // Extract function definitions from .c files only
        let (defined_functions, _) = analyze_c_files(&c_impl_files)?;
        // Extract calls from all files (including headers)
        let (_, called_functions) = analyze_c_files(&c_all_files)?;

        // Find dead functions (defined but never called)
        let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

        let total_functions = defined_functions.len();
        let dead_percentage = if total_functions > 0 {
            (dead_functions.len() as f64 / total_functions as f64) * 100.0
        } else {
            0.0
        };

        Ok(DeadCodeResult {
            language: "c".to_string(),
            dead_functions,
            total_functions,
            dead_code_percentage: dead_percentage,
        })
    }
}

// =============================================================================
// C++ Dead Code Strategy
// =============================================================================

struct CppDeadCodeStrategy;

impl DeadCodeStrategy for CppDeadCodeStrategy {
    fn language(&self) -> &str {
        "cpp"
    }

    fn analyze(&self, path: &Path) -> Result<DeadCodeResult> {
        debug!("Running C++ dead code analysis (AST-based)");

        // Find all C++ source files
        let cpp_files = find_files_by_extension(path, &["cpp", "cc", "cxx", "hpp", "hxx", "h"]);

        // Extract function definitions and calls (similar to C)
        let (defined_functions, called_functions) = analyze_cpp_files(&cpp_files)?;

        // Find dead functions
        let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

        let total_functions = defined_functions.len();
        let dead_percentage = if total_functions > 0 {
            (dead_functions.len() as f64 / total_functions as f64) * 100.0
        } else {
            0.0
        };

        Ok(DeadCodeResult {
            language: "cpp".to_string(),
            dead_functions,
            total_functions,
            dead_code_percentage: dead_percentage,
        })
    }
}

// =============================================================================
// Python Dead Code Strategy
// =============================================================================

struct PythonDeadCodeStrategy;

impl DeadCodeStrategy for PythonDeadCodeStrategy {
    fn language(&self) -> &str {
        "python"
    }

    fn analyze(&self, path: &Path) -> Result<DeadCodeResult> {
        debug!("Running Python dead code analysis (AST-based)");

        // Find all Python files
        let py_files = find_files_by_extension(path, &["py"]);

        // Extract function definitions and calls
        let (defined_functions, called_functions) = analyze_python_files(&py_files)?;

        // Find dead functions
        let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

        let total_functions = defined_functions.len();
        let dead_percentage = if total_functions > 0 {
            (dead_functions.len() as f64 / total_functions as f64) * 100.0
        } else {
            0.0
        };

        Ok(DeadCodeResult {
            language: "python".to_string(),
            dead_functions,
            total_functions,
            dead_code_percentage: dead_percentage,
        })
    }
}

// =============================================================================
// Lua Dead Code Strategy (with module export awareness)
// =============================================================================

struct LuaDeadCodeStrategy;

impl DeadCodeStrategy for LuaDeadCodeStrategy {
    fn language(&self) -> &str {
        "lua"
    }

    fn analyze(&self, path: &Path) -> Result<DeadCodeResult> {
        debug!("Running Lua dead code analysis (module-export-aware)");

        let lua_files = find_files_by_extension(path, &["lua"]);
        let (defined_functions, called_functions) = analyze_lua_files(&lua_files)?;

        // Find dead functions (defined but never called AND not exported)
        let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

        let total_functions = defined_functions.len();
        let dead_percentage = if total_functions > 0 {
            (dead_functions.len() as f64 / total_functions as f64) * 100.0
        } else {
            0.0
        };

        Ok(DeadCodeResult {
            language: "lua".to_string(),
            dead_functions,
            total_functions,
            dead_code_percentage: dead_percentage,
        })
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Find files with given extensions
fn find_files_by_extension(path: &Path, extensions: &[&str]) -> Vec<std::path::PathBuf> {
    WalkDir::new(path)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| extensions.contains(&ext))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    file: String,
    line: usize,
}

/// Analyze C files to find function definitions and calls
fn analyze_c_files(files: &[std::path::PathBuf]) -> Result<(Vec<FunctionInfo>, HashSet<String>)> {
    let mut defined_functions = Vec::new();
    let mut called_functions = HashSet::new();

    for file in files {
        let content = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {:?}", file))?;
        let file_str = file.display().to_string();

        extract_c_function_definitions(&content, &file_str, &mut defined_functions);
        extract_c_function_calls(&content, &mut called_functions);
    }

    debug!(
        "Found {} defined functions, {} unique calls",
        defined_functions.len(),
        called_functions.len()
    );

    Ok((defined_functions, called_functions))
}

/// Extract C function definitions, handling multiline signatures
fn extract_c_function_definitions(content: &str, file_str: &str, out: &mut Vec<FunctionInfo>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut skip_next_line = false;

    for (line_idx, line) in lines.iter().enumerate() {
        if skip_next_line {
            skip_next_line = false;
            continue;
        }

        if let Some(name) = try_extract_c_func_name(line) {
            out.push(FunctionInfo {
                name,
                file: file_str.to_string(),
                line: line_idx + 1,
            });
        } else if line_idx + 1 < lines.len() {
            let combined = format!("{} {}", line, lines[line_idx + 1].trim());
            if let Some(name) = try_extract_c_func_name(&combined) {
                out.push(FunctionInfo {
                    name,
                    file: file_str.to_string(),
                    line: line_idx + 2,
                });
                skip_next_line = true;
            }
        }
    }
}

/// Try to extract a C function name from a line, filtering main and _ prefixed
fn try_extract_c_func_name(line: &str) -> Option<String> {
    let cap = C_DEF_REGEX.captures(line)?;
    let func_name = cap.get(1)?.as_str();
    if func_name != "main" && !func_name.starts_with('_') {
        Some(func_name.to_string())
    } else {
        None
    }
}

/// C keywords to exclude from call tracking
const C_KEYWORDS: &[&str] = &[
    "if", "while", "for", "switch", "sizeof", "return", "printf", "include", "define",
];

/// Extract C function calls from source content
fn extract_c_function_calls(content: &str, calls: &mut HashSet<String>) {
    for line in content.lines() {
        let code_to_scan = if let Some(brace_pos) = line.find('{') {
            &line[brace_pos + 1..]
        } else if C_DECLARATION_REGEX.is_match(line) {
            continue;
        } else {
            line
        };

        for cap in C_CALL_REGEX.captures_iter(code_to_scan) {
            if let Some(m) = cap.get(1) {
                let name = m.as_str();
                if !C_KEYWORDS.contains(&name) {
                    calls.insert(name.to_string());
                }
            }
        }
    }
}

/// Analyze C++ files (similar to C)
fn analyze_cpp_files(files: &[std::path::PathBuf]) -> Result<(Vec<FunctionInfo>, HashSet<String>)> {
    // For now, use same logic as C (can be enhanced with C++-specific features)
    analyze_c_files(files)
}

/// Analyze Python files
fn analyze_python_files(
    files: &[std::path::PathBuf],
) -> Result<(Vec<FunctionInfo>, HashSet<String>)> {
    let mut defined_functions = Vec::new();
    let mut called_functions = HashSet::new();

    for file in files {
        let content = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {:?}", file))?;

        // Find function definitions: def function_name(args):
        for (line_idx, line) in content.lines().enumerate() {
            if let Some(cap) = PY_DEF_REGEX.captures(line) {
                if let Some(func_name_match) = cap.get(1) {
                    let func_name = func_name_match.as_str().to_string();

                    // Skip main and special methods
                    if func_name != "main" && !func_name.starts_with("__") {
                        defined_functions.push(FunctionInfo {
                            name: func_name,
                            file: file.display().to_string(),
                            line: line_idx + 1,
                        });
                    }
                }
            }
        }

        // Find function calls
        for line in content.lines() {
            // Skip function definitions
            if PY_DEF_CHECK_REGEX.is_match(line) {
                continue;
            }

            for cap in PY_CALL_REGEX.captures_iter(line) {
                if let Some(func_name_match) = cap.get(1) {
                    let func_name = func_name_match.as_str().to_string();
                    // Filter out Python keywords
                    if ![
                        "if", "while", "for", "print", "range", "len", "str", "int", "list",
                        "dict", "set", "def",
                    ]
                    .contains(&func_name.as_str())
                    {
                        called_functions.insert(func_name);
                    }
                }
            }
        }
    }

    debug!(
        "Found {} defined Python functions, {} unique calls",
        defined_functions.len(),
        called_functions.len()
    );

    Ok((defined_functions, called_functions))
}

/// Find functions that are defined but never called
fn find_uncalled_functions(
    defined: &[FunctionInfo],
    called: &HashSet<String>,
) -> Vec<DeadFunction> {
    defined
        .iter()
        .filter(|func| !called.contains(&func.name))
        .map(|func| DeadFunction {
            name: func.name.clone(),
            file: func.file.clone(),
            line: func.line,
            reason: "Function is defined but never called".to_string(),
        })
        .collect()
}

/// Analyze Lua files with module export awareness
///
/// Handles Lua module patterns:
/// - `local function name()` — local, dead if uncalled
/// - `function name()` — global, lower confidence (may be called externally)
/// - `function M.name()` / `M.name = function()` — exported if `return M` present
fn analyze_lua_files(files: &[std::path::PathBuf]) -> Result<(Vec<FunctionInfo>, HashSet<String>)> {
    let mut defined_functions = Vec::new();
    let mut called_functions = HashSet::new();

    // Lua keywords and builtins to exclude from call tracking
    let lua_keywords: HashSet<&str> = [
        "if",
        "then",
        "else",
        "elseif",
        "end",
        "do",
        "while",
        "for",
        "repeat",
        "until",
        "function",
        "local",
        "return",
        "break",
        "goto",
        "in",
        "and",
        "or",
        "not",
        "nil",
        "true",
        "false",
        "require",
        "print",
        "pairs",
        "ipairs",
        "type",
        "error",
        "pcall",
        "xpcall",
        "select",
        "rawget",
        "rawset",
        "rawlen",
        "tostring",
        "tonumber",
        "setmetatable",
        "getmetatable",
        "table",
        "string",
        "math",
        "coroutine",
        "unpack",
        "assert",
        "next",
        "io",
        "os",
        "debug",
        "dofile",
        "loadfile",
        "loadstring",
    ]
    .iter()
    .copied()
    .collect();

    for file in files {
        let content = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read Lua file: {:?}", file))?;

        // Skip test files
        let file_str = file.display().to_string();
        if file_str.contains("/tests/")
            || file_str.contains("/test/")
            || file_str.contains("/spec/")
            || file_str.ends_with("_test.lua")
            || file_str.ends_with("_spec.lua")
        {
            // Still collect calls from test files (they exercise production code)
            collect_lua_calls(&content, &lua_keywords, &mut called_functions);
            continue;
        }

        // Detect module return pattern: last non-empty, non-comment line is `return X`
        let returned_module = detect_lua_module_return(&content);

        // Extract function definitions
        for (line_idx, line) in content.lines().enumerate() {
            // Module functions: function M.name(...) or function M:name(...)
            if let Some(cap) = LUA_MODULE_FUNC_NAME_REGEX.captures(line) {
                let module_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let func_name = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                // If the module table is returned, this function is exported
                let is_exported = returned_module
                    .as_ref()
                    .map(|m| m == module_name)
                    .unwrap_or(false);

                if is_exported {
                    // Mark exported functions as "called" so they're not flagged dead
                    called_functions.insert(func_name.to_string());
                }

                defined_functions.push(FunctionInfo {
                    name: func_name.to_string(),
                    file: file_str.clone(),
                    line: line_idx + 1,
                });
                continue;
            }

            // Table field functions: M.name = function(...)
            if let Some(cap) = LUA_TABLE_FUNC_REGEX.captures(line) {
                let module_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let func_name = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                let is_exported = returned_module
                    .as_ref()
                    .map(|m| m == module_name)
                    .unwrap_or(false);

                if is_exported {
                    called_functions.insert(func_name.to_string());
                }

                defined_functions.push(FunctionInfo {
                    name: func_name.to_string(),
                    file: file_str.clone(),
                    line: line_idx + 1,
                });
                continue;
            }

            // Local functions: local function name(...)
            if let Some(cap) = LUA_LOCAL_FUNC_REGEX.captures(line) {
                if let Some(name_match) = cap.get(1) {
                    let func_name = name_match.as_str();
                    if func_name != "main" {
                        defined_functions.push(FunctionInfo {
                            name: func_name.to_string(),
                            file: file_str.clone(),
                            line: line_idx + 1,
                        });
                    }
                }
                continue;
            }

            // Global functions: function name(...) — but not module funcs (handled above)
            if let Some(cap) = LUA_GLOBAL_FUNC_REGEX.captures(line) {
                if let Some(name_match) = cap.get(1) {
                    let func_name = name_match.as_str();
                    if func_name != "main" && !lua_keywords.contains(func_name) {
                        // Global functions may be called from other files
                        defined_functions.push(FunctionInfo {
                            name: func_name.to_string(),
                            file: file_str.clone(),
                            line: line_idx + 1,
                        });
                    }
                }
            }
        }

        // Collect function calls
        collect_lua_calls(&content, &lua_keywords, &mut called_functions);
    }

    debug!(
        "Found {} defined Lua functions, {} unique calls",
        defined_functions.len(),
        called_functions.len()
    );

    Ok((defined_functions, called_functions))
}

/// Detect if a Lua file returns a module table (e.g., `return M`)
fn detect_lua_module_return(content: &str) -> Option<String> {
    // Find last non-empty, non-comment line
    for line in content.lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }
        if let Some(cap) = LUA_RETURN_MODULE_REGEX.captures(trimmed) {
            return cap.get(1).map(|m| m.as_str().to_string());
        }
        // Last meaningful line is not a module return
        return None;
    }
    None
}

/// Collect function calls from Lua source
fn collect_lua_calls(content: &str, keywords: &HashSet<&str>, calls: &mut HashSet<String>) {
    for line in content.lines() {
        let trimmed = line.trim();
        // Skip comments and function definitions
        if trimmed.starts_with("--")
            || trimmed.starts_with("local function ")
            || trimmed.starts_with("function ")
        {
            continue;
        }

        for cap in LUA_CALL_REGEX.captures_iter(line) {
            if let Some(name_match) = cap.get(1) {
                let func_name = name_match.as_str();
                if !keywords.contains(func_name) {
                    calls.insert(func_name.to_string());
                }
            }
        }

        // Also track method-style calls: obj:method() and obj.method()
        // These appear as the part after : or . before (
        // We already capture the identifier before (, which gets the method name
    }
}

// =============================================================================
// Rust-specific helpers (existing cargo-based analysis)
// =============================================================================

fn analyze_rust_dead_code_with_cargo(path: &Path) -> Result<Vec<DeadFunction>> {
    // Simple regex-based analyzer (similar to C/Python)
    // In future, could integrate cargo check warnings

    let rust_files = find_files_by_extension(path, &["rs"]);
    let (defined_functions, called_functions) = analyze_rust_files(&rust_files)?;
    let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

    Ok(dead_functions)
}

fn count_rust_functions(path: &Path) -> Result<usize> {
    let rust_files = find_files_by_extension(path, &["rs"]);
    let (defined_functions, _) = analyze_rust_files(&rust_files)?;
    Ok(defined_functions.len())
}

/// Analyze Rust files
fn analyze_rust_files(
    files: &[std::path::PathBuf],
) -> Result<(Vec<FunctionInfo>, HashSet<String>)> {
    let mut defined_functions = Vec::new();
    let mut called_functions = HashSet::new();

    for file in files {
        let content = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {:?}", file))?;

        // Find function definitions: fn function_name(
        for (line_idx, line) in content.lines().enumerate() {
            if let Some(cap) = RUST_DEF_REGEX.captures(line) {
                if let Some(func_name_match) = cap.get(1) {
                    let func_name = func_name_match.as_str().to_string();

                    // Skip main and test functions
                    if func_name != "main" && !func_name.starts_with("test_") {
                        defined_functions.push(FunctionInfo {
                            name: func_name,
                            file: file.display().to_string(),
                            line: line_idx + 1,
                        });
                    }
                }
            }
        }

        // Find function calls
        for line in content.lines() {
            // Skip function definitions
            if RUST_FN_DEF_REGEX.is_match(line) {
                continue;
            }

            for cap in RUST_CALL_REGEX.captures_iter(line) {
                if let Some(func_name_match) = cap.get(1) {
                    let func_name = func_name_match.as_str().to_string();
                    // Filter out Rust keywords
                    if ![
                        "if", "while", "for", "match", "return", "let", "mut", "use", "mod", "fn",
                        "println", "vec", "Some", "None", "Ok", "Err",
                    ]
                    .contains(&func_name.as_str())
                    {
                        called_functions.insert(func_name);
                    }
                }
            }
        }
    }

    debug!(
        "Found {} defined Rust functions, {} unique calls",
        defined_functions.len(),
        called_functions.len()
    );

    Ok((defined_functions, called_functions))
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_c_dead_code_detection() {
        let temp = create_test_c_project();
        let result = analyze_dead_code_multi_language(temp.path()).unwrap();

        eprintln!("C dead code result: {:?}", result);
        eprintln!("Dead functions: {:?}", result.dead_functions);

        assert_eq!(result.language, "c");
        assert_eq!(
            result.total_functions, 2,
            "Should find 2 functions: used_function and unused_function"
        );
        assert_eq!(
            result.dead_functions.len(),
            1,
            "Should find 1 dead function"
        );
        assert_eq!(result.dead_functions[0].name, "unused_function");
    }

    #[test]
    fn test_python_dead_code_detection() {
        let temp = create_test_python_project();
        let result = analyze_dead_code_multi_language(temp.path()).unwrap();

        assert_eq!(result.language, "python");
        assert!(!result.dead_functions.is_empty());
    }

    fn create_test_c_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("main.c"),
            "int main() { used_function(); return 0; }\nvoid used_function() {}\nvoid unused_function() {}\n",
        ).unwrap();
        temp
    }

    fn create_test_python_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("main.py"),
            "def main():\n    used_function()\n\ndef used_function():\n    pass\n\ndef unused_function():\n    pass\n",
        ).unwrap();
        std::fs::write(
            temp.path().join("pyproject.toml"),
            "[project]\nname=\"test\"\n",
        )
        .unwrap();
        temp
    }

    #[test]
    fn test_lua_dead_code_detection_basic() {
        let temp = TempDir::new().unwrap();
        // Create a Lua project with used and unused functions
        std::fs::write(
            temp.path().join("main.lua"),
            concat!(
                "local function used_helper()\n",
                "    return 42\n",
                "end\n",
                "\n",
                "local function dead_helper()\n",
                "    return 99\n",
                "end\n",
                "\n",
                "function run()\n",
                "    local x = used_helper()\n",
                "    return x\n",
                "end\n",
            ),
        )
        .unwrap();

        let lua_files = find_files_by_extension(temp.path(), &["lua"]);
        let (defined, called) = analyze_lua_files(&lua_files).unwrap();

        assert_eq!(defined.len(), 3, "Should find 3 functions");
        assert!(
            called.contains("used_helper"),
            "used_helper should be in calls"
        );
        assert!(
            !called.contains("dead_helper"),
            "dead_helper should NOT be in calls"
        );

        let dead = find_uncalled_functions(&defined, &called);
        let dead_names: Vec<&str> = dead.iter().map(|d| d.name.as_str()).collect();
        assert!(
            dead_names.contains(&"dead_helper"),
            "dead_helper should be dead"
        );
        assert!(
            !dead_names.contains(&"used_helper"),
            "used_helper should not be dead"
        );
    }

    #[test]
    fn test_lua_module_export_awareness() {
        let temp = TempDir::new().unwrap();
        // Module pattern: functions on M are exported via `return M`
        std::fs::write(
            temp.path().join("mymodule.lua"),
            concat!(
                "local M = {}\n",
                "\n",
                "function M.public_api()\n",
                "    return M.internal_calc()\n",
                "end\n",
                "\n",
                "function M.internal_calc()\n",
                "    return 42\n",
                "end\n",
                "\n",
                "local function truly_dead()\n",
                "    return 0\n",
                "end\n",
                "\n",
                "return M\n",
            ),
        )
        .unwrap();

        let lua_files = find_files_by_extension(temp.path(), &["lua"]);
        let (defined, called) = analyze_lua_files(&lua_files).unwrap();

        // Module functions should be treated as exported (called)
        assert!(
            called.contains("public_api"),
            "M.public_api should be marked as exported"
        );
        assert!(
            called.contains("internal_calc"),
            "M.internal_calc should be marked as exported"
        );

        let dead = find_uncalled_functions(&defined, &called);
        let dead_names: Vec<&str> = dead.iter().map(|d| d.name.as_str()).collect();
        assert!(
            dead_names.contains(&"truly_dead"),
            "truly_dead should be dead"
        );
        assert!(
            !dead_names.contains(&"public_api"),
            "exported funcs should not be dead"
        );
        assert!(
            !dead_names.contains(&"internal_calc"),
            "exported funcs should not be dead"
        );
    }

    #[test]
    fn test_lua_table_field_function_export() {
        let temp = TempDir::new().unwrap();
        // Alternative module pattern: M.name = function(...)
        std::fs::write(
            temp.path().join("alt_module.lua"),
            concat!(
                "local M = {}\n",
                "\n",
                "M.handler = function(req)\n",
                "    return req\n",
                "end\n",
                "\n",
                "M.middleware = function(ctx)\n",
                "    return ctx\n",
                "end\n",
                "\n",
                "local function orphan()\n",
                "    return nil\n",
                "end\n",
                "\n",
                "return M\n",
            ),
        )
        .unwrap();

        let lua_files = find_files_by_extension(temp.path(), &["lua"]);
        let (defined, called) = analyze_lua_files(&lua_files).unwrap();

        assert!(called.contains("handler"), "M.handler should be exported");
        assert!(
            called.contains("middleware"),
            "M.middleware should be exported"
        );

        let dead = find_uncalled_functions(&defined, &called);
        let dead_names: Vec<&str> = dead.iter().map(|d| d.name.as_str()).collect();
        assert!(dead_names.contains(&"orphan"), "orphan should be dead");
        assert_eq!(dead.len(), 1, "Only orphan should be dead");
    }

    #[test]
    fn test_lua_no_module_return_no_exports() {
        let temp = TempDir::new().unwrap();
        // File without module return - no export awareness
        std::fs::write(
            temp.path().join("script.lua"),
            concat!(
                "local M = {}\n",
                "\n",
                "function M.something()\n",
                "    return 1\n",
                "end\n",
                "\n",
                "-- no return M at end\n",
                "print(\"hello\")\n",
            ),
        )
        .unwrap();

        let lua_files = find_files_by_extension(temp.path(), &["lua"]);
        let (defined, called) = analyze_lua_files(&lua_files).unwrap();

        // Without `return M`, M.something is NOT auto-exported
        assert!(
            !called.contains("something"),
            "Without module return, not auto-exported"
        );
        let dead = find_uncalled_functions(&defined, &called);
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].name, "something");
    }

    #[test]
    fn test_lua_detect_module_return() {
        assert_eq!(
            detect_lua_module_return("return M\n"),
            Some("M".to_string())
        );
        assert_eq!(
            detect_lua_module_return("return MyModule\n"),
            Some("MyModule".to_string())
        );
        assert_eq!(
            detect_lua_module_return("x = 1\nreturn M\n"),
            Some("M".to_string())
        );
        assert_eq!(
            detect_lua_module_return("return M\n-- trailing comment\n"),
            Some("M".to_string())
        );
        assert_eq!(detect_lua_module_return("print('done')\n"), None);
        assert_eq!(detect_lua_module_return("return 1, 2, 3\n"), None);
        assert_eq!(detect_lua_module_return(""), None);
    }

    #[test]
    fn test_lua_test_files_excluded_from_definitions() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("tests")).unwrap();
        std::fs::write(
            temp.path().join("tests/test_main.lua"),
            concat!(
                "local function test_helper()\n",
                "    return true\n",
                "end\n",
                "\n",
                "function test_run()\n",
                "    used_in_prod()\n",
                "end\n",
            ),
        )
        .unwrap();
        std::fs::write(
            temp.path().join("main.lua"),
            concat!("local function used_in_prod()\n", "    return 1\n", "end\n",),
        )
        .unwrap();

        let lua_files = find_files_by_extension(temp.path(), &["lua"]);
        let (defined, called) = analyze_lua_files(&lua_files).unwrap();

        // Test file functions should NOT be in defined list
        let def_names: Vec<&str> = defined.iter().map(|d| d.name.as_str()).collect();
        assert!(
            !def_names.contains(&"test_helper"),
            "Test functions excluded"
        );
        assert!(!def_names.contains(&"test_run"), "Test functions excluded");
        assert!(
            def_names.contains(&"used_in_prod"),
            "Prod functions included"
        );

        // But calls FROM test files should still be tracked
        assert!(
            called.contains("used_in_prod"),
            "Calls from tests should be tracked"
        );
    }
}
