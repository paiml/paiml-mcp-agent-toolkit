//! Language Override Module (BUG-012 Fix)
//!
//! Provides language override functionality for CLI commands.
//! Integrates with enhanced_language_detection (BUG-011).

use anyhow::Result;
use std::path::Path;

use crate::services::enhanced_language_detection::{
    detect_project_language_enhanced, LanguageDetection,
};

/// Language override options
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct LanguageOverride {
    /// Single language override (--language)
    pub language: Option<String>,
    /// Multiple languages (--languages)
    pub languages: Option<Vec<String>>,
}


/// Get effective language(s) for analysis
///
/// Priority:
/// 1. --language flag (single language override)
/// 2. --languages flag (multiple languages)
/// 3. Auto-detection using enhanced_language_detection (BUG-011)
pub fn get_effective_languages(
    override_opts: &LanguageOverride,
    path: &Path,
) -> Result<Vec<String>> {
    // Case 1: Single language override
    if let Some(lang) = &override_opts.language {
        let normalized = normalize_language_name(lang)?;
        validate_language_support(&normalized)?;
        return Ok(vec![normalized]);
    }

    // Case 2: Multiple languages override
    if let Some(langs) = &override_opts.languages {
        let mut normalized_langs = Vec::new();
        for lang in langs {
            let normalized = normalize_language_name(lang)?;
            validate_language_support(&normalized)?;
            normalized_langs.push(normalized);
        }
        return Ok(normalized_langs);
    }

    // Case 3: Auto-detection using BUG-011 enhanced detection
    let detection = detect_project_language_enhanced(path);
    Ok(vec![detection.language])
}

/// Normalize language name (case-insensitive)
///
/// Examples:
/// - "Python" -> "python"
/// - "CPP" -> "cpp"
/// - "TypeScript" -> "typescript"
pub fn normalize_language_name(name: &str) -> Result<String> {
    let normalized = name.to_lowercase().trim().to_string();

    if normalized.is_empty() {
        anyhow::bail!("Language name cannot be empty");
    }

    Ok(normalized)
}

/// Validate that a language is supported
///
/// Supported languages (from BUG-011 enhanced detection):
/// - rust, python, javascript, typescript, go, cpp, c, java, kotlin,
///   swift, ruby, php, bash, wasm, etc.
pub fn validate_language_support(language: &str) -> Result<()> {
    let supported = [
        "rust",
        "python",
        "javascript",
        "typescript",
        "go",
        "cpp",
        "c",
        "java",
        "kotlin",
        "swift",
        "ruby",
        "php",
        "bash",
        "sh",
        "shell",
        "wasm",
        "wat",
    ];

    if supported.contains(&language) {
        Ok(())
    } else {
        anyhow::bail!(
            "Language '{}' is not supported. Supported languages: {}",
            language,
            supported.join(", ")
        )
    }
}

/// Check if language override is specified
pub fn has_override(override_opts: &LanguageOverride) -> bool {
    override_opts.language.is_some() || override_opts.languages.is_some()
}

/// Get language detection with override applied
pub fn get_language_detection_with_override(
    override_opts: &LanguageOverride,
    path: &Path,
) -> Result<LanguageDetection> {
    if let Some(lang) = &override_opts.language {
        let normalized = normalize_language_name(lang)?;
        validate_language_support(&normalized)?;

        // Return synthetic detection result
        return Ok(LanguageDetection {
            language: normalized.clone(),
            confidence: 100.0, // Manual override has 100% confidence
        });
    }

    // For multiple languages or no override, use auto-detection
    Ok(detect_project_language_enhanced(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_normalize_language_name() {
        assert_eq!(normalize_language_name("Python").unwrap(), "python");
        assert_eq!(normalize_language_name("PYTHON").unwrap(), "python");
        assert_eq!(normalize_language_name("python").unwrap(), "python");
        assert_eq!(normalize_language_name("  Rust  ").unwrap(), "rust");
    }

    #[test]
    fn test_normalize_empty_name() {
        assert!(normalize_language_name("").is_err());
        assert!(normalize_language_name("  ").is_err());
    }

    #[test]
    fn test_validate_supported_languages() {
        assert!(validate_language_support("rust").is_ok());
        assert!(validate_language_support("python").is_ok());
        assert!(validate_language_support("cpp").is_ok());
        assert!(validate_language_support("c").is_ok());
    }

    #[test]
    fn test_validate_unsupported_language() {
        let result = validate_language_support("fortran");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not supported"));
    }

    #[test]
    fn test_single_language_override() {
        let temp = create_test_project();

        let override_opts = LanguageOverride {
            language: Some("python".to_string()),
            languages: None,
        };

        let langs = get_effective_languages(&override_opts, temp.path()).unwrap();
        assert_eq!(langs, vec!["python"]);
    }

    #[test]
    fn test_multiple_languages_override() {
        let temp = create_test_project();

        let override_opts = LanguageOverride {
            language: None,
            languages: Some(vec!["rust".to_string(), "python".to_string()]),
        };

        let langs = get_effective_languages(&override_opts, temp.path()).unwrap();
        assert_eq!(langs, vec!["rust", "python"]);
    }

    #[test]
    fn test_auto_detection_fallback() {
        let temp = create_rust_project();

        let override_opts = LanguageOverride::default();

        let langs = get_effective_languages(&override_opts, temp.path()).unwrap();
        assert_eq!(langs.len(), 1);
        assert_eq!(langs[0], "rust");
    }

    #[test]
    fn test_single_override_takes_precedence() {
        let temp = create_test_project();

        let override_opts = LanguageOverride {
            language: Some("python".to_string()),
            languages: Some(vec!["rust".to_string()]), // Should be ignored
        };

        let langs = get_effective_languages(&override_opts, temp.path()).unwrap();
        assert_eq!(langs, vec!["python"]); // Only single override
    }

    #[test]
    fn test_case_insensitive_override() {
        let temp = create_test_project();

        let override_opts = LanguageOverride {
            language: Some("PYTHON".to_string()),
            languages: None,
        };

        let langs = get_effective_languages(&override_opts, temp.path()).unwrap();
        assert_eq!(langs, vec!["python"]); // Normalized to lowercase
    }

    fn create_test_project() -> TempDir {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("test.txt"), "test").unwrap();
        temp
    }

    fn create_rust_project() -> TempDir {
        use std::fs;

        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        temp
    }
}
