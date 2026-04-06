/// Include paths extracted from compile_commands.json.
///
/// Reads `compile_commands.json` (or `build/compile_commands.json`) to extract
/// `-I` include paths, enabling better header file discovery and linking.
#[derive(Debug, Default)]
pub(super) struct CompileCommands {
    /// Unique include directories from -I flags
    pub include_dirs: Vec<String>,
    /// Number of compilation entries parsed
    pub entry_count: usize,
}

/// Load compile_commands.json from project root or common build directories.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn load_compile_commands(project_root: &std::path::Path) -> CompileCommands {
    debug_assert!(project_root.exists(), "project_root must exist: {}", project_root.display());
    let candidates = [
        project_root.join("compile_commands.json"),
        project_root.join("build/compile_commands.json"),
        project_root.join("cmake-build-debug/compile_commands.json"),
        project_root.join("cmake-build-release/compile_commands.json"),
    ];

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Some(cmds) = parse_compile_commands(&content) {
                eprintln!(
                    "  compile_commands.json: {} entries, {} include dirs (from {})",
                    cmds.entry_count,
                    cmds.include_dirs.len(),
                    path.display()
                );
                return cmds;
            }
        }
    }

    CompileCommands::default()
}

/// Parse compile_commands.json and extract include directories.
///
/// Format: [{"directory": "...", "command": "... -I/path ...", "file": "..."}]
fn parse_compile_commands(content: &str) -> Option<CompileCommands> {
    let entries: Vec<serde_json::Value> = serde_json::from_str(content).ok()?;
    let mut include_dirs = std::collections::HashSet::new();
    let entry_count = entries.len();

    for entry in &entries {
        // Extract from "command" field
        if let Some(command) = entry.get("command").and_then(|c| c.as_str()) {
            extract_include_dirs(command, &mut include_dirs);
        }
        // Extract from "arguments" field (alternative format)
        if let Some(args) = entry.get("arguments").and_then(|a| a.as_array()) {
            let command = args
                .iter()
                .filter_map(|a| a.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            extract_include_dirs(&command, &mut include_dirs);
        }
    }

    let mut dirs: Vec<String> = include_dirs.into_iter().collect();
    dirs.sort();

    Some(CompileCommands {
        include_dirs: dirs,
        entry_count,
    })
}

/// Extract -I and -isystem include paths from a compiler command string.
fn extract_include_dirs(command: &str, dirs: &mut std::collections::HashSet<String>) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let mut iter = parts.iter().peekable();
    while let Some(&part) = iter.next() {
        // -I path or -isystem path (space-separated)
        if part == "-I" || part == "-isystem" {
            if let Some(&&next) = iter.peek() {
                dirs.insert(next.to_string());
                iter.next();
            }
            continue;
        }
        // -I/path or -isystem/path (concatenated)
        for prefix in &["-I", "-isystem"] {
            if let Some(path) = part.strip_prefix(prefix) {
                if !path.is_empty() {
                    dirs.insert(path.to_string());
                }
            }
        }
    }
}
