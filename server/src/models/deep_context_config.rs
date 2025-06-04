use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeepContextConfig {
    #[serde(default)]
    pub entry_points: Vec<String>,

    #[serde(default = "default_dead_code_threshold")]
    pub dead_code_threshold: f64,

    #[serde(default)]
    pub complexity_thresholds: ComplexityThresholds,

    #[serde(default)]
    pub include_tests: bool,

    #[serde(default)]
    pub include_benches: bool,

    #[serde(default)]
    pub cross_language_detection: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComplexityThresholds {
    #[serde(default = "default_cyclomatic_warning")]
    pub cyclomatic_warning: u32,

    #[serde(default = "default_cyclomatic_error")]
    pub cyclomatic_error: u32,

    #[serde(default = "default_cognitive_warning")]
    pub cognitive_warning: u32,

    #[serde(default = "default_cognitive_error")]
    pub cognitive_error: u32,
}

impl Default for ComplexityThresholds {
    fn default() -> Self {
        Self {
            cyclomatic_warning: 10,
            cyclomatic_error: 20,
            cognitive_warning: 15,
            cognitive_error: 30,
        }
    }
}

impl Default for DeepContextConfig {
    fn default() -> Self {
        Self {
            entry_points: Vec::new(),
            dead_code_threshold: 0.15,
            complexity_thresholds: ComplexityThresholds::default(),
            include_tests: false,
            include_benches: false,
            cross_language_detection: true,
        }
    }
}

impl DeepContextConfig {
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

    pub fn detect_entry_points(&self) -> Vec<String> {
        let mut entry_points = Vec::new();

        // Check for binary targets
        if Path::new("src/main.rs").exists() {
            entry_points.push("main".into());
        }

        // Check for library
        if Path::new("src/lib.rs").exists() {
            entry_points.push("lib".into());
        }

        // Check for multiple binaries
        if let Ok(entries) = std::fs::read_dir("src/bin") {
            for entry in entries.flatten() {
                if let Some(name) = entry.path().file_stem() {
                    entry_points.push(format!("bin/{}", name.to_string_lossy()));
                }
            }
        }

        // Check for WASM entry points
        if Path::new("Cargo.toml").exists() {
            if let Ok(content) = std::fs::read_to_string("Cargo.toml") {
                if content.contains("wasm-bindgen") || content.contains("wasm-pack") {
                    entry_points.push("wasm_bindgen".into());
                }
            }
        }

        // Check for FFI entry points
        if let Ok(entries) = std::fs::read_dir("src") {
            for entry in entries.flatten() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if content.contains("#[no_mangle]") {
                        entry_points.push("no_mangle".into());
                        break;
                    }
                }
            }
        }

        entry_points
    }

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

    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
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

    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

// Default value functions for serde
fn default_dead_code_threshold() -> f64 {
    0.15
}

fn default_cyclomatic_warning() -> u32 {
    10
}

fn default_cyclomatic_error() -> u32 {
    20
}

fn default_cognitive_warning() -> u32 {
    15
}

fn default_cognitive_error() -> u32 {
    30
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config_validation() {
        let config = DeepContextConfig::default();
        // Default config with auto-detection should validate (or provide clear error)
        let result = config.validate();

        // It's ok if validation fails due to no detected entry points in test env
        if let Err(errors) = result {
            assert!(errors.iter().any(|e| e.contains("No entry points")));
        }
    }

    #[test]
    fn test_entry_point_validation() {
        // Standard entry points should pass
        let mut config = DeepContextConfig {
            entry_points: vec!["main".to_string()],
            ..Default::default()
        };
        assert!(config.validate().is_ok());

        config.entry_points = vec!["lib".to_string()];
        assert!(config.validate().is_ok());

        config.entry_points = vec!["bin/pmat".to_string()];
        assert!(config.validate().is_ok());

        // Non-standard entry points should generate warning
        config.entry_points = vec!["custom_entry".to_string()];
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err()[0].contains("No standard entry point"));
    }

    #[test]
    fn test_threshold_validation() {
        let mut config = DeepContextConfig {
            entry_points: vec!["main".to_string()],
            dead_code_threshold: -0.1,
            ..Default::default()
        };

        // Invalid dead code threshold
        assert!(config.validate().is_err());

        config.dead_code_threshold = 1.5;
        assert!(config.validate().is_err());

        config.dead_code_threshold = 0.5;
        assert!(config.validate().is_ok());

        // Invalid complexity thresholds
        config.complexity_thresholds.cyclomatic_warning = 20;
        config.complexity_thresholds.cyclomatic_error = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_entry_point_detection() {
        let temp_dir = TempDir::new().unwrap();
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create main.rs
        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

        // Create lib.rs
        fs::write(src_dir.join("lib.rs"), "pub fn lib_func() {}").unwrap();

        // Create bin directory with binary
        let bin_dir = src_dir.join("bin");
        fs::create_dir(&bin_dir).unwrap();
        fs::write(bin_dir.join("pmat.rs"), "fn main() {}").unwrap();

        // Change to temp directory for detection
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let config = DeepContextConfig::default();
        let detected = config.detect_entry_points();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert!(detected.contains(&"main".to_string()));
        assert!(detected.contains(&"lib".to_string()));
        assert!(detected.contains(&"bin/pmat".to_string()));
    }

    #[test]
    fn test_config_serialization() {
        let config = DeepContextConfig {
            entry_points: vec!["main".to_string(), "lib".to_string()],
            dead_code_threshold: 0.1,
            complexity_thresholds: ComplexityThresholds {
                cyclomatic_warning: 8,
                cyclomatic_error: 15,
                cognitive_warning: 12,
                cognitive_error: 25,
            },
            include_tests: true,
            include_benches: false,
            cross_language_detection: true,
        };

        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: DeepContextConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.entry_points, deserialized.entry_points);
        assert_eq!(config.dead_code_threshold, deserialized.dead_code_threshold);
        assert_eq!(config.include_tests, deserialized.include_tests);
    }
}
