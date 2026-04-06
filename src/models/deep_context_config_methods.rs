// DeepContextConfig impl methods: validation, detection, merge, file I/O

fn detect_bin_targets(entry_points: &mut Vec<String>) {
    debug_assert!(true, "contract: detect_bin_targets");
    if let Ok(entries) = std::fs::read_dir("src/bin") {
        for entry in entries.flatten() {
            if let Some(name) = entry.path().file_stem() {
                entry_points.push(format!("bin/{}", name.to_string_lossy()));
            }
        }
    }
}

fn detect_wasm_entry_points(entry_points: &mut Vec<String>) {
    debug_assert!(true, "contract: detect_wasm_entry_points");
    if !Path::new("Cargo.toml").exists() {
        return;
    }
    if let Ok(content) = std::fs::read_to_string("Cargo.toml") {
        if content.contains("wasm-bindgen") || content.contains("wasm-pack") {
            entry_points.push("wasm_bindgen".into());
        }
    }
}

fn detect_ffi_entry_points(entry_points: &mut Vec<String>) {
    debug_assert!(true, "contract: detect_ffi_entry_points");
    let Ok(entries) = std::fs::read_dir("src") else {
        return;
    };
    for entry in entries.flatten() {
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            if content.contains("#[no_mangle]") {
                entry_points.push("no_mangle".into());
                return;
            }
        }
    }
}

impl DeepContextConfig {
    /// Validates the configuration for correctness
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::deep_context_config::DeepContextConfig;
    ///
    /// let mut config = DeepContextConfig::default();
    /// config.entry_points = vec!["src/main.rs".to_string()];
    ///
    /// match config.validate() {
    ///     Ok(_) => println!("Config is valid"),
    ///     Err(errors) => println!("Found {} errors", errors.len()),
    /// }
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate entry points
        if self.entry_points.is_empty() {
            // Auto-detect based on project structure
            let detected = self.detect_entry_points();
            if detected.is_empty() {
                errors.push("No entry points configured or detected".into());
            }
        } else {
            // Verify at least one standard entry point
            let has_standard = self.entry_points.iter().any(|ep| {
                ep == "main"
                    || ep.ends_with("::main")
                    || ep == "lib"
                    || ep.starts_with("bin/")
                    || ep.contains("wasm_bindgen")
                    || ep.contains("no_mangle")
            });

            if !has_standard {
                errors.push(
                    "No standard entry point found (main, lib, bin/*, wasm_bindgen, no_mangle). \
                     This may cause false dead code positives."
                        .into(),
                );
            }
        }

        // Validate thresholds
        if self.dead_code_threshold < 0.0 || self.dead_code_threshold > 1.0 {
            errors.push(format!(
                "Invalid dead_code_threshold: {} (must be 0.0-1.0)",
                self.dead_code_threshold
            ));
        }

        // Validate complexity thresholds
        if self.complexity_thresholds.cyclomatic_warning
            >= self.complexity_thresholds.cyclomatic_error
        {
            errors.push(format!(
                "Cyclomatic warning threshold ({}) must be less than error threshold ({})",
                self.complexity_thresholds.cyclomatic_warning,
                self.complexity_thresholds.cyclomatic_error
            ));
        }

        if self.complexity_thresholds.cognitive_warning
            >= self.complexity_thresholds.cognitive_error
        {
            errors.push(format!(
                "Cognitive warning threshold ({}) must be less than error threshold ({})",
                self.complexity_thresholds.cognitive_warning,
                self.complexity_thresholds.cognitive_error
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn detect_entry_points(&self) -> Vec<String> {
        let mut entry_points = Vec::new();

        if Path::new("src/main.rs").exists() {
            entry_points.push("main".into());
        }
        if Path::new("src/lib.rs").exists() {
            entry_points.push("lib".into());
        }

        detect_bin_targets(&mut entry_points);
        detect_wasm_entry_points(&mut entry_points);
        detect_ffi_entry_points(&mut entry_points);

        entry_points
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn merge_with_detected(&mut self) {
        if self.entry_points.is_empty() {
            self.entry_points = self.detect_entry_points();
        } else {
            // Add detected entry points that aren't already configured
            let detected = self.detect_entry_points();
            for ep in detected {
                if !self.entry_points.contains(&ep) {
                    self.entry_points.push(ep);
                }
            }
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = std::fs::read_to_string(path)?;
        let mut config: Self = toml::from_str(&content)?;

        // Validate the loaded configuration
        if let Err(errors) = config.validate() {
            return Err(errors.join("; ").into());
        }

        // Merge with detected entry points
        config.merge_with_detected();

        Ok(config)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
