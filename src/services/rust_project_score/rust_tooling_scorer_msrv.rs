// MSRV tracking and release profile optimization scoring
// Included into rust_tooling_scorer.rs

/// Extract the MSRV version string from Cargo.toml content
fn extract_msrv(content: &str) -> Option<String> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    content
        .lines()
        .find(|line| line.contains("rust-version"))
        .and_then(|line| {
            line.split('=')
                .nth(1)?
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .split_once('.')
                .map(|(major, minor)| {
                    format!("{}.{}", major, minor.split('.').next().unwrap_or(minor))
                })
        })
}

/// Check if any CI workflow tests against a given MSRV version
fn ci_tests_msrv(project_path: &Path, msrv: &str) -> bool {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let workflows_dir = project_path.join(".github/workflows");
    let entries = match std::fs::read_dir(&workflows_dir) {
        Ok(e) => e,
        Err(_) => return false,
    };
    entries.filter_map(|e| e.ok()).any(|entry| {
        std::fs::read_to_string(entry.path())
            .map(|content| content.contains(msrv) && content.contains("rust:"))
            .unwrap_or(false)
    })
}

/// Extract a named profile section from Cargo.toml content
fn extract_profile_section<'a>(content: &'a str, header: &str) -> Option<&'a str> {
    debug_assert!(true, "contract: extract_profile_section");
    let start = content.find(header)?;
    let section = &content[start..];
    let end = section[1..]
        .find("\n[")
        .map(|i| i + 1)
        .unwrap_or(section.len());
    Some(&section[..end])
}

/// Score release profile settings
fn score_release_section(section: &str) -> f64 {
    debug_assert!(!section.is_empty(), "section must not be empty");
    let mut score = 0.0;
    let has_lto = section.contains("lto = true")
        || section.contains("lto = \"thin\"")
        || section.contains("lto = \"fat\"")
        || section.contains("lto = 'thin'")
        || section.contains("lto = 'fat'");
    if has_lto {
        score += 4.0;
    }
    if section.contains("codegen-units = 1") {
        score += 3.0;
    }
    if section.contains("panic = \"abort\"") || section.contains("panic = 'abort'") {
        score += 2.0;
    }
    score
}

/// Check if a profile section has LTO enabled (penalty for dev/test)
fn profile_has_lto(section: &str) -> bool {
    debug_assert!(!section.is_empty(), "section must not be empty");
    section.contains("lto = true")
        || section.contains("lto = \"")
        || section.contains("lto = '")
}

impl RustToolingScorer {
    /// Score MSRV (Minimum Supported Rust Version) tracking (10pts)
    pub(super) fn score_msrv_tracking(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(0.0);
        }

        let cargo_toml_content = if let Some(cache) = cache {
            cache
                .get(&cargo_toml_path)
                .ok_or_else(|| ScorerError::IoError("Cargo.toml not in cache".to_string()))?
                .clone()
        } else {
            std::fs::read_to_string(&cargo_toml_path)
                .map_err(|e| ScorerError::IoError(e.to_string()))?
        };

        if !cargo_toml_content.contains("rust-version") {
            return Ok(0.0);
        }

        let mut score = 5.0; // rust-version exists

        if let Some(msrv) = extract_msrv(&cargo_toml_content) {
            if ci_tests_msrv(project_path, &msrv) {
                score += 3.0;
            }
        }

        let readme_has_msrv = project_path.join("README.md").exists()
            && std::fs::read_to_string(project_path.join("README.md"))
                .map(|c| c.to_lowercase().contains("msrv"))
                .unwrap_or(false);
        if readme_has_msrv {
            score += 2.0;
        }

        Ok(score)
    }

    /// Score release profile optimizations (11pts)
    ///
    /// Based on "Learn from Rust Giants" specification (TPS-reviewed):
    /// - +4pts: [profile.release] with LTO enabled
    /// - +3pts: codegen-units = 1 for maximum optimization (release only)
    /// - +2pts: panic = "abort" for smaller binaries (release)
    /// - +2pts: [profile.dev] with panic = "abort" (faster testing)
    /// - -3pts penalty if LTO enabled in dev or test profiles (slows TDD loop)
    pub(super) fn score_release_profiles(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<f64> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(0.0);
        }

        let cargo_toml_content = if let Some(cache) = cache {
            cache
                .get(&cargo_toml_path)
                .ok_or_else(|| ScorerError::IoError("Cargo.toml not in cache".to_string()))?
                .clone()
        } else {
            std::fs::read_to_string(&cargo_toml_path)
                .map_err(|e| ScorerError::IoError(e.to_string()))?
        };

        let mut score = 0.0;

        if let Some(release) =
            extract_profile_section(&cargo_toml_content, "[profile.release]")
        {
            score += score_release_section(release);
        }

        if let Some(dev) = extract_profile_section(&cargo_toml_content, "[profile.dev]") {
            if dev.contains("panic = \"abort\"") || dev.contains("panic = 'abort'") {
                score += 2.0;
            }
            if profile_has_lto(dev) {
                score -= 3.0;
            }
        }

        if let Some(test) = extract_profile_section(&cargo_toml_content, "[profile.test]") {
            if profile_has_lto(test) {
                score -= 3.0;
            }
        }

        Ok(score)
    }
}
