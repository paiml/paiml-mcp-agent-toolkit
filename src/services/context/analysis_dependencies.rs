// Dependency reading for Rust (Cargo.toml), Deno (deno.json/package.json),
// and Python (pyproject.toml/requirements.txt).

async fn read_dependencies(root_path: &Path, toolchain: &str) -> Vec<String> {
    debug_assert!(root_path.exists(), "root_path must exist: {}", root_path.display());
    match toolchain {
        "rust" => read_rust_dependencies(root_path).await,
        "deno" => read_deno_dependencies(root_path).await,
        "python-uv" => read_python_dependencies(root_path).await,
        _ => Vec::new(),
    }
}

async fn read_rust_dependencies(root_path: &Path) -> Vec<String> {
    debug_assert!(root_path.exists(), "root_path must exist: {}", root_path.display());
    if let Ok(cargo_content) = tokio::fs::read_to_string(root_path.join("Cargo.toml")).await {
        if let Ok(cargo_toml) = cargo_content.parse::<toml::Value>() {
            if let Some(deps) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
                return deps.keys().cloned().collect();
            }
        }
    }
    Vec::new()
}

async fn read_deno_dependencies(root_path: &Path) -> Vec<String> {
    debug_assert!(root_path.exists(), "root_path must exist: {}", root_path.display());
    let mut dependencies = Vec::new();

    // Check deno.json
    if let Ok(deno_json) = tokio::fs::read_to_string(root_path.join("deno.json")).await {
        if let Ok(deno_config) = serde_json::from_str::<serde_json::Value>(&deno_json) {
            if let Some(imports) = deno_config.get("imports").and_then(|i| i.as_object()) {
                dependencies.extend(imports.keys().cloned());
            }
        }
    }

    // Check package.json
    if let Ok(package_json) = tokio::fs::read_to_string(root_path.join("package.json")).await {
        if let Ok(package) = serde_json::from_str::<serde_json::Value>(&package_json) {
            if let Some(deps) = package.get("dependencies").and_then(|d| d.as_object()) {
                dependencies.extend(deps.keys().cloned());
            }
        }
    }

    dependencies
}

async fn read_python_dependencies(root_path: &Path) -> Vec<String> {
    debug_assert!(root_path.exists(), "root_path must exist: {}", root_path.display());
    let mut dependencies = Vec::new();

    // Check pyproject.toml
    if let Ok(pyproject_content) = tokio::fs::read_to_string(root_path.join("pyproject.toml")).await
    {
        if let Ok(pyproject) = pyproject_content.parse::<toml::Value>() {
            if let Some(deps) = pyproject
                .get("project")
                .and_then(|p| p.get("dependencies"))
                .and_then(|d| d.as_array())
            {
                dependencies.extend(
                    deps.iter()
                        .filter_map(|d| d.as_str())
                        .map(|s| s.split_whitespace().next().unwrap_or(s).to_string()),
                );
            }
        }
    }

    // Check requirements.txt
    if let Ok(requirements) = tokio::fs::read_to_string(root_path.join("requirements.txt")).await {
        for line in requirements.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                let dep_name = line
                    .split(['=', '>', '<', '~'])
                    .next()
                    .unwrap_or(line)
                    .trim();
                dependencies.push(dep_name.to_string());
            }
        }
    }

    dependencies
}
