impl MakefileCompressor {
    #[must_use]
    pub fn new() -> Self {
        let mut critical_targets = HashSet::new();
        critical_targets.insert("all");
        critical_targets.insert("build");
        critical_targets.insert("test");
        critical_targets.insert("install");
        critical_targets.insert("clean");
        critical_targets.insert("release");
        critical_targets.insert("fmt");
        critical_targets.insert("format");
        critical_targets.insert("lint");
        critical_targets.insert("check");
        critical_targets.insert("coverage");
        critical_targets.insert("docs");
        critical_targets.insert("deploy");
        critical_targets.insert("run");
        critical_targets.insert("serve");
        critical_targets.insert("dev");
        critical_targets.insert("prod");
        critical_targets.insert("dist");
        critical_targets.insert("package");

        let mut critical_vars = HashSet::new();
        critical_vars.insert("PROJECT_NAME");
        critical_vars.insert("VERSION");
        critical_vars.insert("CC");
        critical_vars.insert("CXX");
        critical_vars.insert("CARGO");
        critical_vars.insert("RUSTC");
        critical_vars.insert("PYTHON");
        critical_vars.insert("NODE");
        critical_vars.insert("NPM");
        critical_vars.insert("DOCKER");
        critical_vars.insert("KUBECTL");
        critical_vars.insert("CFLAGS");
        critical_vars.insert("LDFLAGS");
        critical_vars.insert("TARGET");
        critical_vars.insert("ARCH");
        critical_vars.insert("OS");

        Self {
            critical_targets,
            critical_vars,
            var_pattern: Regex::new(r"^([A-Z_][A-Z0-9_]*)\s*[:?]?=").expect("Invalid regex"),
            target_pattern: Regex::new(r"^([a-zA-Z0-9_\-\.]+):").expect("Invalid regex"),
        }
    }

    pub fn compress(&self, content: &str) -> CompressedMakefile {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut result = CompressedMakefile::default();

        // Phase 1: Extract variables
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(caps) = self.var_pattern.captures(trimmed) {
                if let Some(var_name_match) = caps.get(1) {
                    let var_name = var_name_match.as_str();
                    if self.critical_vars.contains(var_name) || var_name.starts_with("PROJECT_") {
                        result.variables.push(line.to_string());
                    }
                }
            }
        }

        // Phase 2: Parse and extract targets
        let targets = self.parse_targets(content);
        for (name, target) in targets {
            if self.is_critical_target(&name) {
                result.targets.push(MakeTarget {
                    name: name.clone(),
                    deps: target.dependencies,
                    recipe_summary: self.summarize_recipe(&target.recipe),
                });
            }
        }

        // Phase 3: Detect toolchain
        result.detected_toolchain = self.detect_toolchain(content, &result.targets);

        // Phase 4: Extract key dependencies
        result.key_dependencies = self.extract_dependencies(content);

        debug!(
            "Compressed Makefile: {} variables, {} targets",
            result.variables.len(),
            result.targets.len()
        );

        result
    }

    fn is_critical_target(&self, name: &str) -> bool {
        self.critical_targets.contains(name)
            || name.starts_with("docker")
            || name.starts_with("test-")
            || name.starts_with("build-")
            || name.contains("deploy")
            || name.contains("install")
    }
}
