/// Collect `AssemblyScript` files (.as, .ts with AS context)
fn collect_assemblyscript_files(project_path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        process_assemblyscript_entry(entry.path(), &mut files);
    }

    Ok(files)
}

/// Process a single file entry for `AssemblyScript`
fn process_assemblyscript_entry(path: &Path, files: &mut Vec<PathBuf>) {
    let ext = match path.extension().and_then(|s| s.to_str()) {
        Some(ext) => ext,
        None => return,
    };

    match ext {
        "as" => add_assemblyscript_file(path, files),
        "ts" => check_and_add_typescript_file(path, files),
        _ => {}
    }
}

/// Add an `AssemblyScript` file to the collection
fn add_assemblyscript_file(path: &Path, files: &mut Vec<PathBuf>) {
    files.push(path.to_path_buf());
}

/// Check if TypeScript file is actually `AssemblyScript` and add if so
fn check_and_add_typescript_file(path: &Path, files: &mut Vec<PathBuf>) {
    if is_assemblyscript_typescript(path) {
        files.push(path.to_path_buf());
    }
}

/// Check if a TypeScript file contains `AssemblyScript` markers
fn is_assemblyscript_typescript(path: &Path) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return false,
    };

    contains_assemblyscript_markers(&content)
}

/// Check if content contains `AssemblyScript` markers
fn contains_assemblyscript_markers(content: &str) -> bool {
    content.contains("@global")
        || content.contains("@inline")
        || content.contains("i32")
        || content.contains("f64")
        || content.contains("memory.")
}

/// Collect WebAssembly files (.wasm, .wat)
fn collect_wasm_files(
    project_path: &Path,
    include_binary: bool,
    include_text: bool,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "wasm" if include_binary => files.push(path.to_path_buf()),
                "wat" if include_text => files.push(path.to_path_buf()),
                _ => {}
            }
        }
    }

    Ok(files)
}
