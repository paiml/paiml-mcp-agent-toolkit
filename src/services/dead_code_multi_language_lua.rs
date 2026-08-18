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
                    exported: is_exported,
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
                    exported: is_exported,
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
                            // `local` is Lua's privacy keyword: a local function
                            // is not reachable from another chunk, so it is not
                            // part of any module's API.
                            exported: false,
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
                            // A global is ambient, not declared: nothing in the
                            // tree states that it is a module's API, so it is
                            // not treated as one. That gap is what
                            // `detect_lua_library_target` discloses.
                            exported: false,
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

/// How many of `files` end in `return <table>` — Lua's one declaration that a
/// file is a module whose fields are its API.
///
/// Read separately from [`analyze_lua_files`] rather than threaded through its
/// return type: that signature is what four existing tests exercise, and a
/// count for the report is not worth changing what they call.
fn count_lua_module_returns(files: &[std::path::PathBuf]) -> usize {
    files
        .iter()
        .filter_map(|file| std::fs::read_to_string(file).ok())
        .filter(|content| detect_lua_module_return(content).is_some())
        .count()
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
