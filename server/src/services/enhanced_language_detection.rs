//! Enhanced Language Detection Module (BUG-011 Fix)
//!
//! This module provides multi-language detection with confidence scoring,
//! primary indicator recognition, and timeout handling.
//!
//! Fixes:
//! - BUG-011: Wrong language detection (python-uv instead of C++)
//! - BUG-012: Multi-language support missing

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use tracing::debug;

/// Language detection result with confidence scoring
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageDetection {
    pub language: String,
    pub confidence: f64,
}

/// Multi-language detection result
#[derive(Debug, Clone)]
pub struct MultiLanguageDetection {
    pub languages: Vec<LanguageInfo>,
    pub primary: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LanguageInfo {
    pub language: String,
    pub confidence: f64,
    pub file_count: usize,
    pub percentage: f64,
}

/// Detect primary project language with confidence scoring
pub fn detect_project_language_enhanced(path: &Path) -> LanguageDetection {
    debug!("Detecting project language at: {:?}", path);

    // Start with file extension counting
    let mut scores: HashMap<String, f64> = HashMap::new();

    // Primary indicators (high confidence boost)
    if path.join("Cargo.toml").exists() {
        *scores.entry("rust".to_string()).or_insert(0.0) += 90.0;
        debug!("Found Cargo.toml - boosting Rust confidence by 90");
    }

    if path.join("CMakeLists.txt").exists() {
        *scores.entry("cpp".to_string()).or_insert(0.0) += 85.0;
        debug!("Found CMakeLists.txt - boosting C++ confidence by 85");
    }

    if path.join("package.json").exists() {
        // Could be JavaScript or TypeScript, will be determined by file count
        *scores.entry("javascript".to_string()).or_insert(0.0) += 30.0;
        *scores.entry("typescript".to_string()).or_insert(0.0) += 30.0;
        debug!("Found package.json - boosting JS/TS confidence by 30");
    }

    if path.join("pyproject.toml").exists() {
        *scores.entry("python".to_string()).or_insert(0.0) += 50.0;
        debug!("Found pyproject.toml - boosting Python confidence by 50");
    }

    if path.join("go.mod").exists() {
        *scores.entry("go".to_string()).or_insert(0.0) += 90.0;
        debug!("Found go.mod - boosting Go confidence by 90");
    }

    // Count files by extension
    let file_counts = count_files_by_extension(path);
    let total_files: usize = file_counts.values().sum();

    if total_files == 0 {
        debug!("No files found, returning unknown");
        return LanguageDetection {
            language: "unknown".to_string(),
            confidence: 0.0,
        };
    }

    debug!("Total files: {}, counts: {:?}", total_files, file_counts);

    // Add percentage-based scoring
    for (ext, count) in file_counts.iter() {
        let percentage = (*count as f64 / total_files as f64) * 100.0;

        if let Some(lang) = extension_to_language(ext) {
            *scores.entry(lang.to_string()).or_insert(0.0) += percentage;
            debug!(
                "Extension {} ({} files, {:.1}%) maps to {}, adding {:.1} to score",
                ext, count, percentage, lang, percentage
            );
        }
    }

    // Find highest score
    let (best_lang, best_score) = scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(lang, score)| (lang.clone(), *score))
        .unwrap_or_else(|| ("unknown".to_string(), 0.0));

    debug!("Best language: {} with score: {:.1}", best_lang, best_score);

    // Cap confidence at 100%
    let confidence = best_score.min(100.0);

    LanguageDetection {
        language: best_lang,
        confidence,
    }
}

/// Detect all languages in a polyglot project
pub fn detect_all_languages(path: &Path) -> MultiLanguageDetection {
    debug!("Detecting all languages at: {:?}", path);

    let file_counts = count_files_by_extension(path);
    let total_files: usize = file_counts.values().sum();

    if total_files == 0 {
        return MultiLanguageDetection {
            languages: vec![],
            primary: "unknown".to_string(),
        };
    }

    let mut languages = Vec::new();

    // Group by language
    let mut lang_counts: HashMap<String, usize> = HashMap::new();
    for (ext, count) in file_counts.iter() {
        if let Some(lang) = extension_to_language(ext) {
            *lang_counts.entry(lang.to_string()).or_insert(0) += count;
        }
    }

    // Calculate percentages and filter >5%
    for (lang, count) in lang_counts.iter() {
        let percentage = (*count as f64 / total_files as f64) * 100.0;

        if percentage > 5.0 {
            // Calculate confidence (percentage + primary indicator boost)
            let mut confidence = percentage;

            // Add primary indicator boost
            if lang == "rust" && path.join("Cargo.toml").exists() {
                confidence += 10.0;
            } else if (lang == "cpp" || lang == "c") && path.join("CMakeLists.txt").exists() {
                confidence += 10.0;
            } else if (lang == "javascript" || lang == "typescript")
                && path.join("package.json").exists()
            {
                confidence += 5.0;
            }

            languages.push(LanguageInfo {
                language: lang.clone(),
                confidence: confidence.min(100.0),
                file_count: *count,
                percentage,
            });
        }
    }

    // Sort by percentage (descending)
    languages.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap());

    let primary = languages
        .first()
        .map(|l| l.language.clone())
        .unwrap_or_else(|| "unknown".to_string());

    debug!(
        "Detected {} languages, primary: {}",
        languages.len(),
        primary
    );

    MultiLanguageDetection { languages, primary }
}

/// Detect language with timeout
pub fn detect_project_language_with_timeout(
    path: &Path,
    _timeout: Duration,
) -> Result<LanguageDetection> {
    // For now, just use the regular detection
    // Timeout will be implemented at the call site using tokio::time::timeout
    Ok(detect_project_language_enhanced(path))
}

/// Override language detection manually
pub fn override_language_detection(_path: &Path, language: &str) -> LanguageDetection {
    LanguageDetection {
        language: language.to_string(),
        confidence: 100.0,
    }
}

/// Override with multiple languages
pub fn override_multiple_languages(path: &Path, languages: Vec<String>) -> MultiLanguageDetection {
    let file_counts = count_files_by_extension(path);
    let total_files: usize = file_counts.values().sum();

    let language_infos: Vec<LanguageInfo> = languages
        .into_iter()
        .map(|lang| {
            // Calculate actual file count for this language
            let count = file_counts
                .iter()
                .filter(|(ext, _)| {
                    extension_to_language(ext).map(|s| s.to_string()) == Some(lang.clone())
                })
                .map(|(_, c)| *c)
                .sum();

            let percentage = if total_files > 0 {
                (count as f64 / total_files as f64) * 100.0
            } else {
                0.0
            };

            LanguageInfo {
                language: lang,
                confidence: 100.0, // Manual override = 100% confidence
                file_count: count,
                percentage,
            }
        })
        .collect();

    let primary = language_infos
        .first()
        .map(|l| l.language.clone())
        .unwrap_or_else(|| "unknown".to_string());

    MultiLanguageDetection {
        languages: language_infos,
        primary,
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Count files by extension recursively
fn count_files_by_extension(path: &Path) -> HashMap<String, usize> {
    use walkdir::WalkDir;

    let mut counts: HashMap<String, usize> = HashMap::new();

    for entry in WalkDir::new(path)
        .max_depth(10)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                *counts.entry(ext.to_string()).or_insert(0) += 1;
            }
        }
    }

    counts
}

/// Map file extension to language
fn extension_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "js" | "jsx" => Some("javascript"),
        "ts" | "tsx" => Some("typescript"),
        "c" | "h" => Some("c"),
        "cc" | "cpp" | "cxx" | "hpp" | "hxx" | "h++" | "c++" => Some("cpp"),
        "go" => Some("go"),
        "java" => Some("java"),
        "kt" | "kts" => Some("kotlin"),
        "rb" => Some("ruby"),
        "php" => Some("php"),
        "swift" => Some("swift"),
        "cs" => Some("csharp"),
        "sh" | "bash" => Some("bash"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extension_to_language_rust() {
        assert_eq!(extension_to_language("rs"), Some("rust"));
    }

    #[test]
    fn test_extension_to_language_cpp() {
        assert_eq!(extension_to_language("cpp"), Some("cpp"));
        assert_eq!(extension_to_language("cc"), Some("cpp"));
        assert_eq!(extension_to_language("cxx"), Some("cpp"));
    }

    #[test]
    fn test_detect_rust_project_with_cargo_toml() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();

        let detection = detect_project_language_enhanced(temp.path());
        assert_eq!(detection.language, "rust");
        assert!(detection.confidence >= 90.0);
    }

    #[test]
    fn test_detect_cpp_project_with_cmake() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.10)\n",
        )
        .unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();
        std::fs::write(temp.path().join("src/main.cpp"), "int main() {}").unwrap();

        let detection = detect_project_language_enhanced(temp.path());
        assert_eq!(detection.language, "cpp");
        assert!(detection.confidence >= 85.0);
    }

    #[test]
    fn test_multi_language_detection() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir_all(temp.path().join("src")).unwrap();

        // Create 50 Rust files
        for i in 0..50 {
            std::fs::write(
                temp.path().join(format!("src/file_{}.rs", i)),
                "fn main() {}",
            )
            .unwrap();
        }

        // Create 30 Python files
        for i in 0..30 {
            std::fs::write(
                temp.path().join(format!("src/tool_{}.py", i)),
                "print('hello')",
            )
            .unwrap();
        }

        let detection = detect_all_languages(temp.path());
        assert_eq!(detection.languages.len(), 2);
        assert_eq!(detection.primary, "rust");
    }
}
