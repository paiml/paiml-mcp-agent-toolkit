// Falsifiability scoring helper functions
// Included from falsifiability.rs — no `use` imports or `#!` attributes allowed

fn dir_contains_test_markers(src_path: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(src_path) else { return false };
    entries.flatten().any(|entry| {
        entry.path().is_file()
            && std::fs::read_to_string(entry.path())
                .is_ok_and(|c| c.contains("#[test]") || c.contains("#[cfg(test)]"))
    })
}

fn score_bench_directory(project_path: &Path, earned: &mut f64, description: &mut Vec<String>) {
    if !workspace::any_member_has_dir(project_path, "benches") {
        return;
    }
    *earned += 1.0;
    description.push("benches/ exists".to_string());

    let bench_content = workspace::read_member_dir_content(project_path, "benches", "rs");
    if bench_content.contains("criterion") || bench_content.contains("Criterion") {
        *earned += 2.0;
        description.push("Criterion.rs found".to_string());
    }
}

fn score_bench_dependencies(project_path: &Path, earned: &mut f64, description: &mut Vec<String>) {
    let has_bench_dep = workspace::get_code_paths(project_path).iter().any(|member| {
        let cargo_path = member.join("Cargo.toml");
        cargo_path.exists()
            && std::fs::read_to_string(&cargo_path)
                .is_ok_and(|c| c.contains("criterion") || c.contains("divan"))
    });
    let root_cargo = project_path.join("Cargo.toml");
    let has_root_bench_dep = root_cargo.exists()
        && std::fs::read_to_string(&root_cargo)
            .is_ok_and(|c| c.contains("criterion") || c.contains("divan"));
    if has_bench_dep || has_root_bench_dep {
        *earned += 1.0;
        description.push("benchmark dependency found".to_string());
    }
}

fn score_readme_hardware(project_path: &Path, earned: &mut f64, description: &mut Vec<String>) {
    let readme_path = project_path.join("README.md");
    let Ok(content) = std::fs::read_to_string(&readme_path) else { return };
    let hw_patterns = ["CPU", "RAM", "Intel", "AMD", "i7", "i9", "Ryzen", "GB"];
    if hw_patterns.iter().any(|p| content.to_uppercase().contains(p)) {
        *earned += 2.0;
        description.push("hardware specs documented".to_string());
    }
    if content.contains("95%") || content.contains("confidence") || content.contains("CI") || content.contains("±") {
        *earned += 1.0;
        description.push("confidence intervals mentioned".to_string());
    }
}

fn ci_path_has_tests(project_path: &Path, ci_path: &str) -> bool {
    let full_path = project_path.join(ci_path);
    if !full_path.exists() {
        return false;
    }
    if full_path.is_dir() {
        return ci_dir_has_test_commands(&full_path);
    }
    std::fs::read_to_string(&full_path).is_ok_and(|c| c.contains("test"))
}

fn ci_dir_has_test_commands(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else { return false };
    entries.flatten().any(|entry| {
        std::fs::read_to_string(entry.path()).is_ok_and(|content| {
            content.contains("cargo test") || content.contains("pytest") || content.contains("npm test")
        })
    })
}

fn makefile_has_tests(project_path: &Path) -> bool {
    let makefile = project_path.join("Makefile");
    makefile.exists()
        && std::fs::read_to_string(&makefile)
            .is_ok_and(|c| c.contains("test:") || c.contains("test-"))
}
