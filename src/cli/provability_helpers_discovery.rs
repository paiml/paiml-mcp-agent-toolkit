/// Discover all functions in project
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn discover_project_functions(project_path: &Path) -> Result<Vec<FunctionId>> {
    crate::status_eprintln!("\u{1f4c2} Discovering functions in project...");

    let mut function_ids = Vec::new();
    let mut file_count = 0;

    // Use file discovery service for better performance
    use crate::services::file_discovery::{FileDiscoveryConfig, ProjectFileDiscovery};

    let discovery = ProjectFileDiscovery::new(project_path.to_path_buf())
        .with_config(FileDiscoveryConfig::default());

    let files = discovery.discover_files()?;

    for path in files {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        // Only process source files
        if matches!(ext, "rs" | "ts" | "js" | "py" | "c" | "cpp" | "h" | "hpp") {
            file_count += 1;
            let relative_path = path
                .strip_prefix(project_path)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            // Real functions, read from the file. This used to push a fixed
            // pair -- `main` at line 1 and `test` at line 10 -- for EVERY file,
            // with the comment "For stub implementation ... In real
            // implementation, would parse AST". So `analyze provability` on
            // pmat's own src/utils reported 34 functions: 17 files x the same
            // two phantoms, none of which existed, one of them at line 10 of a
            // one-line file.
            //
            // contracts/pmat-no-fabrication-v1.yaml, `output_derived_from_input`
            // and `source_location_fidelity`.
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            for (name, line) in extract_function_definitions(&content, ext) {
                function_ids.push(FunctionId {
                    file_path: relative_path.clone(),
                    function_name: name,
                    line_number: line,
                });
            }
        }
    }

    crate::status_eprintln!("\u{1f4ca} Found {file_count} source files");
    Ok(function_ids)
}

/// Function definitions in `content`, as `(name, 1-based line)`.
///
/// Deliberately conservative: a language whose definitions this cannot
/// recognise yields nothing. An empty result is a truthful "pmat did not find
/// functions here"; a synthesised one is not. C and C++ are omitted precisely
/// for that reason -- their declaration syntax is not separable from
/// expressions by line inspection, and guessing would recreate the bug.
fn extract_function_definitions(content: &str, ext: &str) -> Vec<(String, usize)> {
    let keyword = match ext {
        "rs" => "fn ",
        "py" => "def ",
        "ts" | "js" => "function ",
        _ => return Vec::new(),
    };

    let mut out = Vec::new();
    for (idx, raw) in content.lines().enumerate() {
        let trimmed = raw.trim();
        let Some(pos) = trimmed.find(keyword) else {
            continue;
        };
        // Skip definitions that only appear inside a comment.
        let before = trimmed.get(..pos).unwrap_or_default();
        if before.contains("//") || before.contains('#') || before.contains("/*") {
            continue;
        }
        let after = trimmed.get(pos + keyword.len()..).unwrap_or_default();
        let end = after
            .find(|c: char| c == '(' || c == '<' || c == ':' || c.is_whitespace())
            .unwrap_or(after.len());
        let name = after.get(..end).unwrap_or_default().trim();
        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            out.push((name.to_string(), idx + 1));
        }
    }
    out
}
