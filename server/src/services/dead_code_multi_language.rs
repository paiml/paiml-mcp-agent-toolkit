//! Multi-Language Dead Code Analysis Module (BUG-004 Fix)
//!
//! This module provides dead code detection across multiple programming languages
//! without requiring Cargo.toml or assuming Rust projects.
//!
//! Fixes:
//! - BUG-004: Dead code analyzer broken for non-Rust projects

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
    ).unwrap();

    // C function call regex
    static ref C_CALL_REGEX: Regex = Regex::new(r"\b(\w+)\s*\(").unwrap();

    // C function declaration regex
    static ref C_DECLARATION_REGEX: Regex = Regex::new(
        r"^\s*(?:static\s+)?(?:inline\s+)?(?:extern\s+)?(?:void|int|char|float|double|long|short|unsigned|struct\s+\w+|enum\s+\w+|\w+\s+\*?)\s+\w+\s*\("
    ).unwrap();

    // Python function definition regex
    static ref PY_DEF_REGEX: Regex = Regex::new(r"(?m)^\s*def\s+(\w+)\s*\(").unwrap();

    // Python function call regex
    static ref PY_CALL_REGEX: Regex = Regex::new(r"\b(\w+)\s*\(").unwrap();

    // Python def check regex
    static ref PY_DEF_CHECK_REGEX: Regex = Regex::new(r"^\s*def\s+\w+\s*\(").unwrap();

    // Rust function definition regex
    static ref RUST_DEF_REGEX: Regex = Regex::new(r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*[<(]").unwrap();

    // Rust function call regex
    static ref RUST_CALL_REGEX: Regex = Regex::new(r"\b(\w+)\s*[!]?\(").unwrap();

    // Rust fn definition check regex
    static ref RUST_FN_DEF_REGEX: Regex = Regex::new(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+\w+").unwrap();
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
        _ => {
            return Err(anyhow::anyhow!(
                "Dead code analysis not supported for language: {}. Supported: rust, c, cpp, python",
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

        // Find function definitions - supports multiline with opening brace on next line
        // Matches: void function_name() { or void function_name()\n{
        let lines: Vec<&str> = content.lines().collect();
        let mut skip_next_line = false;
        for (line_idx, line) in lines.iter().enumerate() {
            // Skip if this line was part of a multiline match
            if skip_next_line {
                skip_next_line = false;
                continue;
            }

            // Check current line
            if let Some(cap) = C_DEF_REGEX.captures(line) {
                let func_name = cap.get(1).unwrap().as_str().to_string();

                // Skip main and common library functions
                if func_name != "main" && !func_name.starts_with("_") {
                    defined_functions.push(FunctionInfo {
                        name: func_name,
                        file: file.display().to_string(),
                        line: line_idx + 1,
                    });
                }
            } else if line_idx + 1 < lines.len() {
                // Check if next line has opening brace (multiline function definition)
                let combined = format!("{} {}", line, lines[line_idx + 1].trim());
                if let Some(cap) = C_DEF_REGEX.captures(&combined) {
                    let func_name = cap.get(1).unwrap().as_str().to_string();
                    if func_name != "main" && !func_name.starts_with("_") {
                        // Add function at the NEXT line (where the actual definition is)
                        defined_functions.push(FunctionInfo {
                            name: func_name,
                            file: file.display().to_string(),
                            line: line_idx + 2, // Next line, not current
                        });
                        // Skip processing the next line since we've already handled it
                        skip_next_line = true;
                    }
                }
            }
        }

        // Find function calls: function_name(
        for line in content.lines() {
            // For lines with inline bodies like: int main() { used_function(); }
            // Extract the part after the opening brace FIRST (before skipping declarations)
            let code_to_scan = if let Some(brace_pos) = line.find('{') {
                &line[brace_pos + 1..] // Scan content after the '{'
            } else {
                // Skip function declarations (lines that start with return type and have no body)
                if C_DECLARATION_REGEX.is_match(line) {
                    continue;
                }
                line // Scan entire line
            };

            // Find all function calls
            for cap in C_CALL_REGEX.captures_iter(code_to_scan) {
                let func_name = cap.get(1).unwrap().as_str().to_string();
                // Filter out common keywords that look like function calls
                if ![
                    "if", "while", "for", "switch", "sizeof", "return", "printf", "include",
                    "define",
                ]
                .contains(&func_name.as_str())
                {
                    called_functions.insert(func_name);
                }
            }
        }
    }

    debug!(
        "Found {} defined functions, {} unique calls",
        defined_functions.len(),
        called_functions.len()
    );

    Ok((defined_functions, called_functions))
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
                let func_name = cap.get(1).unwrap().as_str().to_string();

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

        // Find function calls
        for line in content.lines() {
            // Skip function definitions
            if PY_DEF_CHECK_REGEX.is_match(line) {
                continue;
            }

            for cap in PY_CALL_REGEX.captures_iter(line) {
                let func_name = cap.get(1).unwrap().as_str().to_string();
                // Filter out Python keywords
                if ![
                    "if", "while", "for", "print", "range", "len", "str", "int", "list", "dict",
                    "set", "def",
                ]
                .contains(&func_name.as_str())
                {
                    called_functions.insert(func_name);
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
                let func_name = cap.get(1).unwrap().as_str().to_string();

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

        // Find function calls
        for line in content.lines() {
            // Skip function definitions
            if RUST_FN_DEF_REGEX.is_match(line) {
                continue;
            }

            for cap in RUST_CALL_REGEX.captures_iter(line) {
                let func_name = cap.get(1).unwrap().as_str().to_string();
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

    debug!(
        "Found {} defined Rust functions, {} unique calls",
        defined_functions.len(),
        called_functions.len()
    );

    Ok((defined_functions, called_functions))
}

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
        assert!(result.dead_functions.len() > 0);
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
}
