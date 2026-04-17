impl MakefileCompressor {
    fn parse_targets(&self, content: &str) -> HashMap<String, ParsedTarget> {
        let mut targets = HashMap::new();
        let mut current_target: Option<String> = None;
        let mut in_recipe = false;

        for line in content.lines() {
            // Skip comments
            if line.trim().starts_with('#') {
                continue;
            }

            // Check for target definition
            if let Some(caps) = self.target_pattern.captures(line) {
                if let Some(target_name_match) = caps.get(1) {
                    let target_name = target_name_match.as_str().to_string();

                    // Extract dependencies
                    let deps = line
                        .split(':')
                        .nth(1)
                        .map(|d| {
                            d.split_whitespace()
                                .map(std::string::ToString::to_string)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    targets.insert(
                        target_name.clone(),
                        ParsedTarget {
                            dependencies: deps,
                            recipe: Vec::new(),
                        },
                    );

                    current_target = Some(target_name);
                    in_recipe = true;
                }
            } else if in_recipe {
                // Check if line starts with tab or spaces (recipe line)
                if line.starts_with('\t') || line.starts_with("    ") {
                    if let Some(ref target) = current_target {
                        if let Some(t) = targets.get_mut(target) {
                            t.recipe.push(line.to_string());
                        }
                    }
                } else if !line.trim().is_empty() {
                    // Non-empty, non-indented line ends the recipe
                    in_recipe = false;
                    current_target = None;
                }
            }
        }

        targets
    }

    fn summarize_recipe(&self, recipe_lines: &[String]) -> String {
        // Find first meaningful command
        for line in recipe_lines {
            let trimmed = line.trim_start_matches('\t').trim_start_matches(' ');
            let clean = trimmed.trim_start_matches('@').trim_start_matches('-');

            // Skip echo, mkdir, and other non-meaningful commands
            if !clean.starts_with("echo ")
                && !clean.starts_with("mkdir ")
                && !clean.starts_with("rm ")
                && !clean.starts_with(':')
                && !clean.is_empty()
            {
                return crate::utils::string_truncate::truncate_with_ellipsis(clean, 97);
            }
        }

        "[complex recipe]".to_string()
    }

    fn detect_toolchain(&self, content: &str, targets: &[MakeTarget]) -> Option<String> {
        // Check for common toolchain indicators
        let content_lower = content.to_lowercase();

        // Rust
        if content_lower.contains("cargo ") || content_lower.contains("rustc") {
            return Some("rust".to_string());
        }

        // Python
        if content_lower.contains("python") || content_lower.contains("pip") {
            return Some("python".to_string());
        }

        // Node.js
        if content_lower.contains("npm ") || content_lower.contains("node ") {
            return Some("node".to_string());
        }

        // Go
        if content_lower.contains("go build") || content_lower.contains("go test") {
            return Some("go".to_string());
        }

        // C/C++
        if content_lower.contains("gcc")
            || content_lower.contains("g++")
            || content_lower.contains("clang")
        {
            return Some("c/c++".to_string());
        }

        // Java
        if content_lower.contains("javac")
            || content_lower.contains("mvn")
            || content_lower.contains("gradle")
        {
            return Some("java".to_string());
        }

        // Check target recipes for clues
        for target in targets {
            let recipe_lower = target.recipe_summary.to_lowercase();
            if recipe_lower.contains("cargo") {
                return Some("rust".to_string());
            }
            if recipe_lower.contains("python") {
                return Some("python".to_string());
            }
            if recipe_lower.contains("npm") || recipe_lower.contains("node") {
                return Some("node".to_string());
            }
        }

        None
    }

    fn extract_dependencies(&self, content: &str) -> Vec<String> {
        let mut deps = HashSet::new();

        // Look for common dependency patterns
        for line in content.lines() {
            let lower = line.to_lowercase();

            // Package managers
            if lower.contains("cargo install") {
                if let Some(pkg) = extract_package_name(&lower, "cargo install") {
                    deps.insert(pkg);
                }
            }
            if lower.contains("npm install") || lower.contains("npm i ") {
                if let Some(pkg) = extract_package_name(&lower, "npm install") {
                    deps.insert(pkg);
                }
            }
            if lower.contains("apt-get install") || lower.contains("apt install") {
                if let Some(pkg) = extract_package_name(&lower, "install") {
                    deps.insert(pkg);
                }
            }

            // Binary dependencies
            for cmd in &["docker", "kubectl", "terraform", "ansible", "make", "cmake"] {
                if lower.contains(&format!("command -v {cmd}"))
                    || lower.contains(&format!("which {cmd}"))
                {
                    deps.insert((*cmd).to_string());
                }
            }
        }

        let mut result: Vec<String> = deps.into_iter().collect();
        result.sort();
        result.truncate(10); // Limit to top 10 dependencies
        result
    }
}
