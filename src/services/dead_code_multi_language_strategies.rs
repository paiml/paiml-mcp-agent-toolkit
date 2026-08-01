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

        let (total_functions, total_files) = count_rust_functions_and_files(path)?;
        let dead_percentage = if total_functions > 0 {
            (dead_functions.len() as f64 / total_functions as f64) * 100.0
        } else {
            0.0
        };

        Ok(DeadCodeResult {
            language: "rust".to_string(),
            dead_functions,
            total_functions,
            total_files,
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
            // Every .c/.h file walked for call analysis (#720).
            total_files: c_all_files.len(),
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
            // Every C++ source/header file walked (#720).
            total_files: cpp_files.len(),
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
            // Every .py file walked (#720): a 2-file fixture with 4 functions
            // used to report 4 files analyzed.
            total_files: py_files.len(),
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
            // Every .lua file walked (#720).
            total_files: lua_files.len(),
            dead_code_percentage: dead_percentage,
        })
    }
}

// =============================================================================
// Helper Functions
// =============================================================================
