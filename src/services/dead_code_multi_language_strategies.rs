// =============================================================================
// Rust Dead Code Strategy
// =============================================================================

struct RustDeadCodeStrategy;

impl DeadCodeStrategy for RustDeadCodeStrategy {
    fn language(&self) -> &str {
        "rust"
    }

    fn analyze(&self, path: &Path) -> Result<DeadCodeResult> {
        debug!("Running Rust dead code analysis (no cargo)");

        // `pmat analyze dead-code` routes Rust to `cargo check`, so this is the
        // engine that answers when there is no cargo to ask — and it must reach
        // the same verdict about a library's `pub` API that rustc does.
        let library_target = detect_rust_library_target(path);
        let rust_files = find_files_by_extension(path, &["rs"]);
        let (defined_functions, mut called_functions) = analyze_rust_files(&rust_files)?;
        let exported_roots =
            seed_exported_roots(&library_target, &defined_functions, &mut called_functions);
        let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

        let total_functions = defined_functions.len();
        let dead_percentage = dead_percentage_of(dead_functions.len(), total_functions);

        Ok(DeadCodeResult {
            language: "rust".to_string(),
            dead_functions,
            total_functions,
            total_files: rust_files.len(),
            dead_code_percentage: dead_percentage,
            library_target,
            exported_roots,
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
        let (_, mut called_functions) = analyze_c_files(&c_all_files)?;

        let library_target = detect_c_family_library_target(path);
        let exported_roots =
            seed_exported_roots(&library_target, &defined_functions, &mut called_functions);

        // Find dead functions (defined but never called)
        let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

        let total_functions = defined_functions.len();
        let dead_percentage = dead_percentage_of(dead_functions.len(), total_functions);

        Ok(DeadCodeResult {
            language: "c".to_string(),
            dead_functions,
            total_functions,
            // Every .c/.h file walked for call analysis (#720).
            total_files: c_all_files.len(),
            dead_code_percentage: dead_percentage,
            library_target,
            exported_roots,
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

        // Find all C++ source files.
        //
        // `.c` is in this list because a CMakeLists.txt scores `cpp` +85
        // whatever the sources are (`enhanced_language_detection.rs`), so a
        // pure-C project with a CMake build is dispatched HERE — and this list
        // omitting `.c` meant it analysed no files at all: `add_executable(app
        // src.c)` with a dead function in `src.c` reported "0 files analyzed, 0
        // with dead code". A C++ project containing C translation units is
        // ordinary besides; `analyze_cpp_files` is `analyze_c_files`, so there
        // is nothing to dispatch differently.
        let cpp_files =
            find_files_by_extension(path, &["cpp", "cc", "cxx", "hpp", "hxx", "h", "c"]);

        // Extract function definitions and calls (similar to C)
        let (defined_functions, mut called_functions) = analyze_cpp_files(&cpp_files)?;

        let library_target = detect_c_family_library_target(path);
        let exported_roots =
            seed_exported_roots(&library_target, &defined_functions, &mut called_functions);

        // Find dead functions
        let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

        let total_functions = defined_functions.len();
        let dead_percentage = dead_percentage_of(dead_functions.len(), total_functions);

        Ok(DeadCodeResult {
            language: "cpp".to_string(),
            dead_functions,
            total_functions,
            // Every C++ source/header file walked (#720).
            total_files: cpp_files.len(),
            dead_code_percentage: dead_percentage,
            library_target,
            exported_roots,
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

        // What the tree DECLARES as its public API, read before the definitions
        // so each one can be marked exported as it is found.
        let declared_exports = python_declared_exports(&py_files);
        let library_target = detect_python_library_target(path, &declared_exports);

        // Extract function definitions and calls
        let (defined_functions, mut called_functions) =
            analyze_python_files(&py_files, &declared_exports)?;
        let exported_roots =
            seed_exported_roots(&library_target, &defined_functions, &mut called_functions);

        // Find dead functions
        let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

        let total_functions = defined_functions.len();
        let dead_percentage = dead_percentage_of(dead_functions.len(), total_functions);

        Ok(DeadCodeResult {
            language: "python".to_string(),
            dead_functions,
            total_functions,
            // Every .py file walked (#720): a 2-file fixture with 4 functions
            // used to report 4 files analyzed.
            total_files: py_files.len(),
            dead_code_percentage: dead_percentage,
            library_target,
            exported_roots,
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
        let (defined_functions, mut called_functions) = analyze_lua_files(&lua_files)?;

        // Lua already treated a returned module's fields as reachable; this
        // names the verdict so the report can publish it, and seeds the same
        // roots through the one mechanism every language now uses.
        let library_target = detect_lua_library_target(count_lua_module_returns(&lua_files));
        let exported_roots =
            seed_exported_roots(&library_target, &defined_functions, &mut called_functions);

        // Find dead functions (defined but never called AND not exported)
        let dead_functions = find_uncalled_functions(&defined_functions, &called_functions);

        let total_functions = defined_functions.len();
        let dead_percentage = dead_percentage_of(dead_functions.len(), total_functions);

        Ok(DeadCodeResult {
            language: "lua".to_string(),
            dead_functions,
            total_functions,
            // Every .lua file walked (#720).
            total_files: lua_files.len(),
            dead_code_percentage: dead_percentage,
            library_target,
            exported_roots,
        })
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Dead functions as a percentage of defined ones, and `0.0` over no functions
/// at all rather than a division by zero.
///
/// One implementation: this was written out five times, once per strategy.
fn dead_percentage_of(dead: usize, total: usize) -> f64 {
    if total > 0 {
        #[allow(clippy::cast_precision_loss)]
        {
            (dead as f64 / total as f64) * 100.0
        }
    } else {
        0.0
    }
}
