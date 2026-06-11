// Language detection helpers for determining the primary programming language
// of a project based on marker files and file extensions.

/// Detects the primary programming language of a project based on marker files
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::cli::detect_primary_language;
/// use std::path::Path;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let dir = tempdir().unwrap();
/// fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
///
/// let lang = detect_primary_language(dir.path());
/// assert_eq!(lang, Some("rust".to_string()));
/// ```
/// Returns `true` only when Ruchy is the *dominant* source language.
///
/// Ruchy transpiles to Rust, so a genuine Ruchy project can also carry a
/// `Cargo.toml` and generated `.rs` files. We therefore let Ruchy win over the
/// `Cargo.toml` marker — but ONLY when `.ruchy`/`.rh` sources are at least as
/// numerous as `.rs` files. This stops a single test fixture (e.g.
/// `tests/fixtures/foo.ruchy`) from misclassifying an entire Rust project as
/// "ruchy" (which previously made `pmat analyze complexity` mislabel pmat's
/// own repo).
fn ruchy_is_primary_language(path: &Path) -> bool {
    use walkdir::WalkDir;
    let mut ruchy = 0usize;
    let mut rust = 0usize;
    for entry in WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            // Never exclude the root, even if it starts with a dot.
            e.depth() == 0 || !should_exclude_dir(e.file_name().to_str().unwrap_or(""))
        })
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        match entry.path().extension().and_then(|ext| ext.to_str()) {
            Some("ruchy" | "rh") => ruchy += 1,
            Some("rs") => rust += 1,
            _ => {}
        }
    }
    ruchy > 0 && ruchy >= rust
}

fn detect_by_project_files(path: &Path) -> Option<String> {
    // Project marker files in priority order
    const MARKERS: &[(&str, &str)] = &[
        ("Cargo.toml", "rust"),
        ("pyproject.toml", "python-uv"),
        ("setup.py", "python-uv"),
        ("build.gradle", "kotlin"),
        ("build.gradle.kts", "kotlin"),
    ];

    for (file, lang) in MARKERS {
        if path.join(file).exists() {
            return Some((*lang).to_string());
        }
    }

    // Special handling for JS/TS projects
    if path.join("package.json").exists() {
        if path.join("deno.json").exists() || path.join("deno.jsonc").exists() {
            return Some("deno".to_string());
        }
        // Don't assume deno - let file extension counting determine JS/TS
        return None;
    }

    None
}

fn should_exclude_dir(name: &str) -> bool {
    name.starts_with('.')
        || matches!(
            name,
            "target" | "node_modules" | "build" | "dist" | "archive"
        )
}

fn count_extension(ext: &str, lang_counts: &mut std::collections::HashMap<&'static str, usize>) {
    match ext {
        "rs" => *lang_counts.entry("rust").or_insert(0) += 1,
        "ts" | "tsx" => *lang_counts.entry("typescript").or_insert(0) += 1,
        "js" | "jsx" => *lang_counts.entry("javascript").or_insert(0) += 1,
        "py" => *lang_counts.entry("python-uv").or_insert(0) += 1,
        "c" | "h" => *lang_counts.entry("c").or_insert(0) += 1, // PMAT-BUG-003 fix
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "cu" | "cuh" => *lang_counts.entry("cpp").or_insert(0) += 1, // PMAT-BUG-004 fix
        "kt" | "kts" => *lang_counts.entry("kotlin").or_insert(0) += 1,
        "sh" | "bash" => *lang_counts.entry("bash").or_insert(0) += 1,
        "lua" => *lang_counts.entry("lua").or_insert(0) += 1,
        _ => {}
    }
}

fn detect_by_file_extensions(path: &Path) -> Option<String> {
    use walkdir::WalkDir;
    let mut lang_counts = std::collections::HashMap::new();

    for entry in WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_str().unwrap_or("");
            // Don't exclude the root directory, even if it starts with a dot
            if e.depth() == 0 {
                return true;
            }
            !should_exclude_dir(file_name)
        })
        .flatten()
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                count_extension(ext, &mut lang_counts);
            }
        }
    }

    lang_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(lang, _)| lang.to_string())
}

#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect primary language.
pub fn detect_primary_language(path: &Path) -> Option<String> {
    // Ruchy transpiles to Rust and can ship alongside a Cargo.toml, so it must
    // be able to win over the Rust marker — but ONLY when Ruchy actually
    // dominates the source tree (see `ruchy_is_primary_language`).
    if ruchy_is_primary_language(path) {
        return Some("ruchy".to_string());
    }

    // Check project marker files
    if let Some(lang) = detect_by_project_files(path) {
        return Some(lang);
    }

    // Fall back to file extension counting
    detect_by_file_extensions(path)
}

/// Detect primary language with confidence score
fn detect_with_confidence_by_markers(path: &Path) -> Option<(String, f64)> {
    // Project markers with 100% confidence
    const CONFIDENT_MARKERS: &[(&str, &str)] = &[
        ("Cargo.toml", "rust"),
        ("pyproject.toml", "python-uv"),
        ("setup.py", "python-uv"),
        ("build.gradle", "kotlin"),
        ("build.gradle.kts", "kotlin"),
    ];

    for (file, lang) in CONFIDENT_MARKERS {
        if path.join(file).exists() {
            return Some(((*lang).to_string(), 100.0));
        }
    }

    // Special JS/TS handling
    if path.join("package.json").exists() {
        if path.join("deno.json").exists() || path.join("deno.jsonc").exists() {
            return Some(("deno".to_string(), 100.0));
        } else {
            // Check if it's TypeScript or JavaScript based on file extensions
            let (lang, _) = count_files_by_extension(path)?;
            return Some((lang, 90.0));
        }
    }

    None
}

fn count_files_by_extension(path: &Path) -> Option<(String, f64)> {
    use walkdir::WalkDir;
    let mut lang_counts = std::collections::HashMap::new();
    let mut total_files = 0;

    for entry in WalkDir::new(path)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_str().unwrap_or("");
            // Don't exclude the root directory, even if it starts with a dot
            if e.depth() == 0 {
                return true;
            }
            !should_exclude_dir(file_name)
        })
        .flatten()
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                let lang = match ext {
                    "rs" => Some("rust"),
                    "ts" | "tsx" => Some("typescript"),
                    "js" | "jsx" => Some("javascript"),
                    "py" => Some("python-uv"),
                    "kt" | "kts" => Some("kotlin"),
                    "sh" | "bash" => Some("bash"),
                    "lua" => Some("lua"),
                    _ => None,
                };

                if let Some(l) = lang {
                    *lang_counts.entry(l).or_insert(0) += 1;
                    total_files += 1;
                }
            }
        }
    }

    if total_files == 0 {
        return None;
    }

    lang_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(lang, count)| {
            let confidence = (f64::from(count) / f64::from(total_files)) * 100.0;
            (lang.to_string(), confidence)
        })
}

#[must_use]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
/// Detect primary language with confidence.
pub fn detect_primary_language_with_confidence(path: &Path) -> Option<(String, f64)> {
    // Try project markers first
    if let Some(result) = detect_with_confidence_by_markers(path) {
        return Some(result);
    }

    // Fall back to file counting
    count_files_by_extension(path)
}

#[cfg(test)]
mod ruchy_detection_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Regression: a single `.ruchy` test fixture must NOT reclassify a Rust
    /// project as "ruchy". Previously `has_ruchy_files` returned `true` on any
    /// stray fixture, so `pmat analyze complexity` mislabeled its own repo.
    #[test]
    fn lone_ruchy_fixture_does_not_override_cargo() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"").unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("lib.rs"), "pub fn a() {}").unwrap();
        let fixtures = dir.path().join("tests").join("fixtures");
        fs::create_dir_all(&fixtures).unwrap();
        fs::write(fixtures.join("sample.ruchy"), "let x = 1").unwrap();

        assert_eq!(
            detect_primary_language(dir.path()),
            Some("rust".to_string())
        );
    }

    /// A genuinely Ruchy-dominant tree (more `.ruchy` than `.rs`) still wins,
    /// even when a `Cargo.toml` is present (Ruchy transpiles to Rust).
    #[test]
    fn ruchy_dominant_tree_wins_over_cargo() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"").unwrap();
        fs::write(dir.path().join("generated.rs"), "fn main() {}").unwrap();
        for i in 0..5 {
            fs::write(dir.path().join(format!("m{i}.ruchy")), "let x = 1").unwrap();
        }

        assert_eq!(
            detect_primary_language(dir.path()),
            Some("ruchy".to_string())
        );
    }

    /// No Ruchy at all → marker detection drives the result.
    #[test]
    fn no_ruchy_uses_marker() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname=\"x\"").unwrap();
        assert_eq!(
            detect_primary_language(dir.path()),
            Some("rust".to_string())
        );
    }
}
