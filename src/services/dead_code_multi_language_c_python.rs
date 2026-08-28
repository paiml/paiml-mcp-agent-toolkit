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
    /// Does this definition leave the tree — i.e. is it part of what a caller
    /// outside the analysed source could call?
    ///
    /// Set from the language's OWN visibility declaration and nothing else:
    /// absence of `static` in C, `pub` in Rust, membership of `__all__` or an
    /// `__init__.py` re-export in Python, a field of a returned module table in
    /// Lua. It is only honoured when [`LibraryTarget`] says the target is a
    /// library — in a program an exported symbol still has no outside caller.
    exported: bool,
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
                exported: has_external_linkage(line),
            });
        } else if line_idx + 1 < lines.len() {
            let combined = format!("{} {}", line, lines[line_idx + 1].trim());
            if let Some(name) = try_extract_c_func_name(&combined) {
                out.push(FunctionInfo {
                    name,
                    file: file_str.to_string(),
                    line: line_idx + 2,
                    exported: has_external_linkage(&combined),
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

/// Does this C definition have EXTERNAL linkage?
///
/// `static` is C's own visibility keyword: a `static` function cannot be named
/// from another translation unit, so it is not part of a library's API and an
/// un-called one is dead however the target is linked. Everything else is
/// visible to the linker and therefore callable from outside the analysed tree.
fn has_external_linkage(definition_line: &str) -> bool {
    !definition_line.trim_start().starts_with("static")
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

/// Analyze Python files.
///
/// `declared_exports` is what the tree states its public API to be
/// ([`python_declared_exports`]); a definition whose name is in it is marked
/// exported, which keeps a package's `__all__` out of the dead list once
/// [`detect_python_library_target`] has confirmed the target is a library.
fn analyze_python_files(
    files: &[std::path::PathBuf],
    declared_exports: &HashSet<String>,
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
                            exported: declared_exports.contains(&func_name),
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
