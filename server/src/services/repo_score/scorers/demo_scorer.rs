// DemoScorer - Category G: Demo Quality (10 points)
//
// Scores based on:
// - G1: Time-to-Interaction (3 points) - Demo starts quickly, quick-start guide present
// - G2: Error Gracefulness (3 points) - Proper error handling, no raw panics/stack traces
// - G3: Visual Stability (2 points) - Rich output formatting, consistent UX patterns
// - G4: "Wow" Factor (2 points) - Uses rich terminal UI or interactive components
//
// References (Primary):
// - Storey et al. (2017) - Interactive demos reduce cognitive barriers
// - Lavie & Tractinsky (2004) - Visual aesthetics correlate with perceived usability
// - Miller (1968) - Response time thresholds for user perception
//
// References (Review Additions - Toyota Way):
// - Nasehi et al. (2012) - Code example quality in StackOverflow
// - Steinmacher et al. (2015) - Barriers for newcomers to OSS projects
// - Barik et al. (2017) - Error message recoverability
// - Posnett et al. (2011) - Ecological fallacy in software metrics
// - Treude et al. (2011) - Social impact of badges (diminishing returns)
// - Uddin & Robillard (2015) - API documentation failure modes

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Repository archetype for calibrated scoring (Toyota Way - Standardized Work)
///
/// Based on Uddin et al. (2017) distinction between API usage patterns (Cookbooks)
/// and project scaffolds (Boilerplates), with additional archetypes for
/// demo applications and tutorial content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepoArchetype {
    /// Documentation-heavy, examples are markdown/config (e.g., apr-cookbook)
    Cookbook,
    /// Executable demonstration application with runnable code
    DemoApp,
    /// Code library with API examples
    Library,
    /// Step-by-step learning content
    Tutorial,
    /// Project scaffold/boilerplate for cloning
    Boilerplate,
}

impl RepoArchetype {
    /// Determine the effective max score for G2 based on archetype
    /// Returns None if G2 should be N/A (removed from denominator)
    pub fn g2_max_score(&self) -> Option<f64> {
        match self {
            // Cookbooks are documentation-heavy; G2 is N/A
            RepoArchetype::Cookbook => None,
            // Tutorial content may not have executable demos
            RepoArchetype::Tutorial => Some(1.5),
            // All others should have proper error handling
            _ => Some(3.0),
        }
    }

    /// Get human-readable archetype name
    pub fn name(&self) -> &'static str {
        match self {
            RepoArchetype::Cookbook => "Cookbook",
            RepoArchetype::DemoApp => "Demo Application",
            RepoArchetype::Library => "Library",
            RepoArchetype::Tutorial => "Tutorial",
            RepoArchetype::Boilerplate => "Boilerplate",
        }
    }
}

pub struct DemoScorer;

impl DemoScorer {
    pub fn new() -> Self {
        Self
    }

    /// Detect repository archetype for calibrated scoring
    /// Based on Uddin et al. (2017) and Steinmacher et al. (2015)
    pub async fn detect_archetype(&self, repo_path: &Path) -> RepoArchetype {
        let name = repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Name-based heuristics
        if name.contains("cookbook") || name.contains("recipes") {
            return RepoArchetype::Cookbook;
        }
        if name.contains("boilerplate")
            || name.contains("starter")
            || name.contains("template")
            || name.contains("scaffold")
        {
            return RepoArchetype::Boilerplate;
        }
        if name.contains("tutorial") || name.contains("learn") || name.contains("course") {
            return RepoArchetype::Tutorial;
        }
        if name.contains("demo") || name.contains("example") {
            return RepoArchetype::DemoApp;
        }

        // Content-based detection
        let has_src = repo_path.join("src").exists();
        let has_lib_rs = repo_path.join("src/lib.rs").exists();
        let has_main_rs = repo_path.join("src/main.rs").exists();
        let demo_files = self.find_demo_files(repo_path).await;

        // Count markdown files vs code files
        let md_count = self.count_files_by_extension(repo_path, "md").await;
        let code_count = self.count_code_files(repo_path).await;

        // Cookbook: primarily markdown, few code files
        if md_count > 5 && code_count < 10 {
            return RepoArchetype::Cookbook;
        }

        // Library: has lib.rs
        if has_lib_rs && !has_main_rs {
            return RepoArchetype::Library;
        }

        // DemoApp: has main.rs and demo files
        if has_main_rs && !demo_files.is_empty() {
            return RepoArchetype::DemoApp;
        }

        // Library with binary: has both lib.rs and main.rs
        if has_lib_rs && has_main_rs {
            return RepoArchetype::Library;
        }

        // Check for Cargo.toml to determine if it's a Rust project
        if has_src {
            return RepoArchetype::Library;
        }

        // Default to DemoApp if unclear
        RepoArchetype::DemoApp
    }

    /// Count files with specific extension
    async fn count_files_by_extension(&self, repo_path: &Path, ext: &str) -> usize {
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(e) = path.extension().and_then(|e| e.to_str()) {
                        if e == ext {
                            count += 1;
                        }
                    }
                } else if path.is_dir()
                    && !path
                        .file_name()
                        .is_some_and(|n| n.to_str().is_some_and(|s| s.starts_with('.')))
                {
                    // Recursively count (but skip hidden dirs)
                    count += Box::pin(self.count_files_by_extension(&path, ext)).await;
                }
            }
        }
        count
    }

    /// Count code files (rs, py, js, ts, go)
    async fn count_code_files(&self, repo_path: &Path) -> usize {
        let mut count = 0;
        let code_exts = ["rs", "py", "js", "ts", "go", "rb", "java", "c", "cpp"];
        for ext in code_exts {
            count += self.count_files_by_extension(repo_path, ext).await;
        }
        count
    }

    /// Score Time-to-Interaction (G1: 3 points)
    /// Checks for quick-start guides, simple examples, and fast demo execution
    async fn score_time_to_interaction(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let mut score: f64 = 0.0;
        let mut findings = vec![];

        // Check for examples directory
        let examples_dirs = ["examples", "demos", "demo", "samples"];
        let mut has_examples = false;
        for dir in examples_dirs {
            if repo_path.join(dir).exists() {
                has_examples = true;
                score += 1.0;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Demo Quality".to_string(),
                    message: format!("{} directory found", dir),
                    location: Some(dir.to_string()),
                    impact_points: 1.0,
                });
                break;
            }
        }

        if !has_examples {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Demo Quality".to_string(),
                message: "No examples/ or demos/ directory found".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        // Check for quick-start in README
        let readme_path = repo_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&readme_path).await {
                let quick_start_patterns = [
                    r"(?i)##?\s*quick\s*start",
                    r"(?i)##?\s*getting\s*started",
                    r"(?i)##?\s*try\s*it\s*(out|now)",
                    r"(?i)##?\s*5[\s-]minute",
                    r"(?i)##?\s*tldr",
                ];

                for pattern in quick_start_patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if re.is_match(&content) {
                            score += 1.0;
                            findings.push(Finding {
                                severity: Severity::Success,
                                category: "Demo Quality".to_string(),
                                message: "Quick-start section found in README".to_string(),
                                location: Some("README.md".to_string()),
                                impact_points: 1.0,
                            });
                            break;
                        }
                    }
                }

                // Check for one-liner install/run command
                let one_liner_patterns = [
                    r"```(?:bash|sh)?\n(?:cargo install|pip install|npm install|npx)[^\n]+\n```",
                    r"```(?:bash|sh)?\n[^\n]{1,80}\n```", // Simple one-liner command
                ];

                for pattern in one_liner_patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if re.is_match(&content) {
                            score += 1.0;
                            findings.push(Finding {
                                severity: Severity::Success,
                                category: "Demo Quality".to_string(),
                                message: "One-liner install/run command found".to_string(),
                                location: Some("README.md".to_string()),
                                impact_points: 1.0,
                            });
                            break;
                        }
                    }
                }
            }
        }

        score = score.min(3.0_f64);

        if findings.is_empty() {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Demo Quality".to_string(),
                message: "No quick-start documentation found".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "G1".to_string(),
            name: "Time-to-Interaction".to_string(),
            score,
            max_score: 3.0,
            findings,
        })
    }

    /// Score Error Gracefulness (G2: 3 points)
    /// Checks for proper error handling in demo code (no raw panics/unwraps in user-facing code)
    ///
    /// Context-aware scoring based on Barik et al. (2017):
    /// - Don't penalize unwrap() in test/setup/proof-of-concept functions
    /// - Focus on error message recoverability, not just mechanism
    async fn score_error_gracefulness(
        &self,
        repo_path: &Path,
        archetype: RepoArchetype,
    ) -> Result<SubcategoryScore> {
        // Handle N/A state for cookbooks (Toyota Way: remove from denominator, not pity score)
        if archetype.g2_max_score().is_none() {
            return Ok(SubcategoryScore {
                id: "G2".to_string(),
                name: "Error Gracefulness (N/A for Cookbook)".to_string(),
                score: 0.0,
                max_score: 0.0, // N/A: removed from denominator
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "Demo Quality".to_string(),
                    message: "G2 scoring not applicable for documentation-heavy repositories"
                        .to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        let max_score = archetype.g2_max_score().unwrap_or(3.0);
        let mut score: f64 = max_score; // Start with full score, deduct for issues
        let mut findings = vec![];

        // Find demo/example files
        let demo_files = self.find_demo_files(repo_path).await;

        if demo_files.is_empty() {
            // Check if README has error handling documentation
            let readme_path = repo_path.join("README.md");
            let has_error_section = if readme_path.exists() {
                if let Ok(content) = tokio::fs::read_to_string(&readme_path).await {
                    let error_section_patterns = [
                        r"(?i)##?\s*error\s*handling",
                        r"(?i)##?\s*troubleshoot",
                        r"(?i)##?\s*common\s*(errors|issues|problems)",
                    ];
                    error_section_patterns.iter().any(|p| {
                        regex::Regex::new(p)
                            .map(|re| re.is_match(&content))
                            .unwrap_or(false)
                    })
                } else {
                    false
                }
            } else {
                false
            };

            // Partial credit: 2.0 if has error handling docs, 1.5 otherwise
            let partial_score: f64 = if has_error_section { 2.0 } else { 1.5 };
            return Ok(SubcategoryScore {
                id: "G2".to_string(),
                name: "Error Gracefulness".to_string(),
                score: partial_score.min(max_score),
                max_score,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "Demo Quality".to_string(),
                    message: if has_error_section {
                        "No demo files to analyze, but error handling documentation found"
                            .to_string()
                    } else {
                        "No demo files found to analyze for error handling".to_string()
                    },
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        let mut raw_unwrap_count = 0;
        let mut contextual_unwrap_count = 0; // unwraps in test/setup (acceptable)
        let mut raw_panic_count = 0;
        let mut expect_with_message_count = 0;
        let mut proper_error_handling_count = 0;

        // Pre-compile regex patterns outside the loop (clippy::regex_creation_in_loops)
        let contextual_fn_pattern = regex::Regex::new(
            r"(?s)fn\s+(test_|setup|init|proof_of_concept|example_)[^{]*\{[^}]*\.unwrap\(\)",
        )
        .unwrap();
        let unwrap_pattern = regex::Regex::new(r"\.unwrap\(\)").unwrap();
        let panic_pattern = regex::Regex::new(r"panic!\(").unwrap();
        let expect_pattern = regex::Regex::new(r#"\.expect\("[^"]+"\)"#).unwrap();

        for file_path in &demo_files {
            if let Ok(content) = tokio::fs::read_to_string(file_path).await {
                // Context-aware unwrap detection (Barik et al. 2017)
                // Don't penalize unwraps in test/setup/proof_of_concept functions
                contextual_unwrap_count += contextual_fn_pattern.find_iter(&content).count();

                // Count all unwraps
                let total_unwraps = unwrap_pattern.find_iter(&content).count();
                // User-facing unwraps = total - contextual
                raw_unwrap_count += total_unwraps.saturating_sub(contextual_unwrap_count);

                // Count raw panics (still bad even in demos - prefer better error messages)
                raw_panic_count += panic_pattern.find_iter(&content).count();

                // Count expect with message (acceptable - communicates intent)
                expect_with_message_count += expect_pattern.find_iter(&content).count();

                // Count proper error handling (good)
                let error_handling_patterns = [
                    r"\?;", // Question mark operator
                    r"match\s+.*\{[^}]*Err\(",
                    r"if\s+let\s+Err\(",
                    r"\.map_err\(",
                    r"anyhow::|thiserror::|eyre::",
                ];

                for pattern in error_handling_patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        proper_error_handling_count += re.find_iter(&content).count();
                    }
                }
            }
        }

        // Scoring logic
        // Deduct for raw unwraps (0.5 points per 5 unwraps, max -1.5)
        let unwrap_penalty = (raw_unwrap_count as f64 / 5.0 * 0.5).min(1.5);
        if raw_unwrap_count > 0 {
            score -= unwrap_penalty;
            findings.push(Finding {
                severity: if raw_unwrap_count > 10 {
                    Severity::Error
                } else {
                    Severity::Warning
                },
                category: "Demo Quality".to_string(),
                message: format!(
                    "{} raw .unwrap() calls in user-facing demo code (use .expect() with message or proper error handling)",
                    raw_unwrap_count
                ),
                location: None,
                impact_points: -unwrap_penalty,
            });
        }

        // Note contextual unwraps (not penalized)
        if contextual_unwrap_count > 0 {
            findings.push(Finding {
                severity: Severity::Info,
                category: "Demo Quality".to_string(),
                message: format!(
                    "{} .unwrap() calls in test/setup functions (acceptable)",
                    contextual_unwrap_count
                ),
                location: None,
                impact_points: 0.0,
            });
        }

        // Deduct for raw panics (0.5 points per panic, max -1.0)
        let panic_penalty = (raw_panic_count as f64 * 0.5).min(1.0);
        if raw_panic_count > 0 {
            score -= panic_penalty;
            findings.push(Finding {
                severity: Severity::Error,
                category: "Demo Quality".to_string(),
                message: format!(
                    "{} panic!() calls in demo code (prefer graceful error messages)",
                    raw_panic_count
                ),
                location: None,
                impact_points: -panic_penalty,
            });
        }

        // Bonus for proper error handling
        if proper_error_handling_count > 5 && raw_unwrap_count < 5 {
            findings.push(Finding {
                severity: Severity::Success,
                category: "Demo Quality".to_string(),
                message: "Good error handling patterns detected in demo code".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        if expect_with_message_count > 0 {
            findings.push(Finding {
                severity: Severity::Info,
                category: "Demo Quality".to_string(),
                message: format!(
                    "{} .expect() calls with messages (acceptable for demos)",
                    expect_with_message_count
                ),
                location: None,
                impact_points: 0.0,
            });
        }

        score = score.max(0.0);

        if findings.is_empty() {
            findings.push(Finding {
                severity: Severity::Success,
                category: "Demo Quality".to_string(),
                message: "Demo code has graceful error handling".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "G2".to_string(),
            name: "Error Gracefulness".to_string(),
            score,
            max_score,
            findings,
        })
    }

    /// Score Visual Stability (G3: 2 points)
    /// Checks for consistent output formatting, progress indicators, structured output
    ///
    /// Based on Posnett et al. (2011) - avoids ecological fallacy:
    /// - Don't just check manifest for dependencies
    /// - Verify actual usage in source code (Genchi Genbutsu)
    async fn score_visual_stability(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let mut score: f64 = 0.0;
        let mut findings = vec![];

        let demo_files = self.find_demo_files(repo_path).await;

        // Check for rich output libraries in manifest
        let cargo_toml = repo_path.join("Cargo.toml");
        let package_json = repo_path.join("package.json");
        let pyproject = repo_path.join("pyproject.toml");

        // Track detected libraries for usage verification
        let mut detected_libs: Vec<&str> = vec![];

        if cargo_toml.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&cargo_toml).await {
                let rich_patterns = [
                    "indicatif",
                    "console",
                    "colored",
                    "termcolor",
                    "ratatui",
                    "crossterm",
                    "comfy-table",
                    "prettytable",
                    "dialoguer",
                    "owo-colors",
                ];
                for p in rich_patterns {
                    if content.contains(p) {
                        detected_libs.push(p);
                    }
                }
            }
        } else if package_json.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&package_json).await {
                let rich_patterns = [
                    "chalk",
                    "ora",
                    "ink",
                    "blessed",
                    "cli-table",
                    "boxen",
                    "figlet",
                ];
                for p in rich_patterns {
                    if content.contains(p) {
                        detected_libs.push(p);
                    }
                }
            }
        } else if pyproject.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&pyproject).await {
                let rich_patterns = ["rich", "tqdm", "colorama", "click", "typer"];
                for p in rich_patterns {
                    if content.contains(p) {
                        detected_libs.push(p);
                    }
                }
            }
        }

        // Genchi Genbutsu: Verify actual usage in src/ files (Posnett et al. 2011)
        let src_path = repo_path.join("src");
        let mut verified_usage = false;
        if !detected_libs.is_empty() && src_path.exists() {
            verified_usage = self.verify_library_usage(&src_path, &detected_libs).await;
        }

        // Scoring: Manifest detection = 0.5, Verified usage = 1.0
        if !detected_libs.is_empty() {
            if verified_usage {
                score += 1.0;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Demo Quality".to_string(),
                    message: format!(
                        "Rich output libraries detected and verified in use: {}",
                        detected_libs.join(", ")
                    ),
                    location: Some("src/".to_string()),
                    impact_points: 1.0,
                });
            } else {
                // Partial credit for having the dependency (may be used in demos)
                score += 0.5;
                findings.push(Finding {
                    severity: Severity::Info,
                    category: "Demo Quality".to_string(),
                    message: format!(
                        "Rich output libraries in manifest but usage not verified: {}",
                        detected_libs.join(", ")
                    ),
                    location: None,
                    impact_points: 0.5,
                });
            }
        }

        // Check for structured output patterns in demo files
        let mut has_structured_output = false;
        for file_path in &demo_files {
            if let Ok(content) = tokio::fs::read_to_string(file_path).await {
                let structured_patterns = [
                    r#"println!\s*\(\s*"\s*\{"#,       // Structured println with format
                    r#"eprintln!\s*\("#,               // Proper stderr usage
                    r#"serde_json::to_string_pretty"#, // Pretty JSON output
                    r#"format!\s*\("#,                 // Formatted output
                    r#"table\.add_row"#,               // Table output
                    r#"ProgressBar::new"#,             // Progress bar
                    r#"spinner"#,                      // Spinner animation
                ];

                for pattern in structured_patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if re.is_match(&content) {
                            has_structured_output = true;
                            break;
                        }
                    }
                }
            }
        }

        if has_structured_output {
            score += 1.0;
            findings.push(Finding {
                severity: Severity::Success,
                category: "Demo Quality".to_string(),
                message: "Structured/formatted output detected in demos".to_string(),
                location: None,
                impact_points: 1.0,
            });
        }

        score = score.min(2.0);

        if findings.is_empty() {
            findings.push(Finding {
                severity: Severity::Warning,
                category: "Demo Quality".to_string(),
                message: "Consider adding rich terminal output (indicatif, colored, etc.)"
                    .to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "G3".to_string(),
            name: "Visual Stability".to_string(),
            score,
            max_score: 2.0,
            findings,
        })
    }

    /// Verify that detected libraries are actually used in source code
    /// Implements Genchi Genbutsu principle - go and see the actual usage
    async fn verify_library_usage(&self, src_path: &Path, libs: &[&str]) -> bool {
        let usage_patterns: std::collections::HashMap<&str, &[&str]> = [
            (
                "indicatif",
                &["ProgressBar", "MultiProgress", "ProgressStyle"][..],
            ),
            ("console", &["Term", "Style", "Color"][..]),
            (
                "colored",
                &[".red()", ".green()", ".blue()", "Colorize"][..],
            ),
            (
                "termcolor",
                &["StandardStream", "ColorChoice", "WriteColor"][..],
            ),
            ("ratatui", &["Terminal", "Frame", "Widget"][..]),
            ("crossterm", &["execute!", "queue!", "cursor::"][..]),
            ("comfy-table", &["Table", "Row", "Cell"][..]),
            ("prettytable", &["Table", "row!"][..]),
            ("dialoguer", &["Select", "Input", "Confirm"][..]),
            ("owo-colors", &["OwoColorize", ".style("][..]),
            ("chalk", &["chalk.red", "chalk.green", "chalk.blue"][..]),
            ("ora", &["Ora", "ora("][..]),
            ("ink", &["render", "<Box", "<Text"][..]),
            ("rich", &["Console", "Table", "Progress"][..]),
            ("tqdm", &["tqdm(", "trange"][..]),
            ("colorama", &["Fore.", "Back.", "Style."][..]),
            ("click", &["@click.command", "click.echo"][..]),
            ("typer", &["typer.Typer", "@app.command"][..]),
        ]
        .into_iter()
        .collect();

        // Scan src/ files recursively
        if let Ok(entries) = std::fs::read_dir(src_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ["rs", "py", "js", "ts"].contains(&ext) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for lib in libs {
                                if let Some(patterns) = usage_patterns.get(lib) {
                                    for pattern in *patterns {
                                        if content.contains(pattern) {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if path.is_dir() {
                    // Recurse into subdirectories
                    if Box::pin(self.verify_library_usage(&path, libs)).await {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Score "Wow" Factor (G4: 2 points)
    /// Checks for interactive components, web demos, ASCII art, badges
    ///
    /// Based on Treude et al. (2011) - badges have diminishing returns:
    /// - Cap badge points at 2 badges (0.25 pts each = 0.5 max)
    /// - Prioritize interactive elements over static badges
    async fn score_wow_factor(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let mut score: f64 = 0.0;
        let mut findings = vec![];

        // Check README for demo GIF/video
        let readme_path = repo_path.join("README.md");
        if readme_path.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&readme_path).await {
                // Check for demo GIF/video (case-insensitive) - HIGH VALUE
                let demo_media_patterns = [
                    r#"(?i)!\[.*demo.*\]\([^)]+\.gif\)"#,
                    r#"(?i)!\[.*demo.*\]\([^)]+\.mp4\)"#,
                    r#"(?i)!\[.*demo.*\]\([^)]+\.webm\)"#,
                    r#"(?i)<video[^>]+>"#,
                    r#"(?i)asciinema\.org"#,
                    r#"(?i)!\[.*\]\([^)]+asciicast[^)]+\)"#,
                ];

                for pattern in demo_media_patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if re.is_match(&content) {
                            score += 1.0;
                            findings.push(Finding {
                                severity: Severity::Success,
                                category: "Demo Quality".to_string(),
                                message: "Demo GIF/video/screencast found in README".to_string(),
                                location: Some("README.md".to_string()),
                                impact_points: 1.0,
                            });
                            break;
                        }
                    }
                }

                // Check for interactive playground links - HIGH VALUE (Treude et al. 2011)
                let playground_patterns = [
                    r#"(?i)replit\.com"#,
                    r#"(?i)codesandbox\.io"#,
                    r#"(?i)stackblitz\.com"#,
                    r#"(?i)play\.rust-lang\.org"#,
                    r#"(?i)playground"#,
                    r#"(?i)try\s+it\s+(online|now|live)"#,
                ];

                for pattern in playground_patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if re.is_match(&content) {
                            score += 0.75;
                            findings.push(Finding {
                                severity: Severity::Success,
                                category: "Demo Quality".to_string(),
                                message: "Interactive playground/demo link detected".to_string(),
                                location: Some("README.md".to_string()),
                                impact_points: 0.75,
                            });
                            break;
                        }
                    }
                }

                // Check for badges - CAPPED at 2 badges (Treude et al. 2011: diminishing returns)
                let badge_count = content.matches("![").count();
                // Award 0.25 per badge, max 0.5 (2 badges worth)
                let badge_score = (badge_count.min(2) as f64) * 0.25;
                if badge_count > 0 {
                    if badge_count <= 2 {
                        score += badge_score;
                        findings.push(Finding {
                            severity: Severity::Success,
                            category: "Demo Quality".to_string(),
                            message: format!(
                                "{} badges detected (professional appearance)",
                                badge_count
                            ),
                            location: Some("README.md".to_string()),
                            impact_points: badge_score,
                        });
                    } else {
                        // Over-badging creates noise (Heijunka violation)
                        score += 0.5; // Still cap at 2 badges worth
                        findings.push(Finding {
                            severity: Severity::Info,
                            category: "Demo Quality".to_string(),
                            message: format!(
                                "{} badges detected (excessive - consider reducing to 2-4 essential badges)",
                                badge_count
                            ),
                            location: Some("README.md".to_string()),
                            impact_points: 0.5,
                        });
                    }
                }

                // Check for ASCII art/logo - MODERATE VALUE
                let ascii_art_patterns = [
                    r#"```\n[^\n]*[|/\\─━═╔╗╚╝][^\n]*\n"#,
                    r#"<pre>[^<]*[|/\\─━═][^<]*</pre>"#,
                    r#"<img[^>]+logo[^>]+>"#,
                    r#"<img[^>]+hero[^>]+>"#,
                ];

                for pattern in ascii_art_patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if re.is_match(&content) {
                            score += 0.25;
                            findings.push(Finding {
                                severity: Severity::Success,
                                category: "Demo Quality".to_string(),
                                message: "Logo/ASCII art detected".to_string(),
                                location: Some("README.md".to_string()),
                                impact_points: 0.25,
                            });
                            break;
                        }
                    }
                }
            }
        }

        // Check for interactive web demo - HIGH VALUE
        let web_demo_paths = [
            "docs/index.html",
            "demo/index.html",
            "public/index.html",
            "www/index.html",
        ];

        for path in web_demo_paths {
            if repo_path.join(path).exists() {
                score += 0.75;
                findings.push(Finding {
                    severity: Severity::Success,
                    category: "Demo Quality".to_string(),
                    message: format!("Web demo found at {}", path),
                    location: Some(path.to_string()),
                    impact_points: 0.75,
                });
                break;
            }
        }

        score = score.min(2.0_f64);

        if findings.is_empty() {
            findings.push(Finding {
                severity: Severity::Info,
                category: "Demo Quality".to_string(),
                message: "Consider adding demo GIF/video or interactive web demo".to_string(),
                location: None,
                impact_points: 0.0,
            });
        }

        Ok(SubcategoryScore {
            id: "G4".to_string(),
            name: "Wow Factor".to_string(),
            score,
            max_score: 2.0,
            findings,
        })
    }

    /// Find demo/example files in the repository
    async fn find_demo_files(&self, repo_path: &Path) -> Vec<std::path::PathBuf> {
        let mut demo_files = vec![];

        // Check examples directory
        let examples_dirs = ["examples", "demos", "demo", "samples"];
        for dir in examples_dirs {
            let dir_path = repo_path.join(dir);
            if dir_path.exists() {
                if let Ok(entries) = std::fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                            if ["rs", "py", "js", "ts", "go", "rb", "sh"].contains(&ext) {
                                demo_files.push(path);
                            }
                        }
                    }
                }
            }
        }

        // Check for main demo file in root
        let main_demo_names = [
            "demo.rs",
            "demo.py",
            "demo.js",
            "demo.ts",
            "example.rs",
            "example.py",
        ];
        for name in main_demo_names {
            let path = repo_path.join(name);
            if path.exists() {
                demo_files.push(path);
            }
        }

        demo_files
    }
}

#[async_trait]
impl Scorer for DemoScorer {
    fn category_name(&self) -> &str {
        "Demo Quality"
    }

    fn max_score(&self) -> f64 {
        10.0
    }

    async fn score(&self, repo_path: &Path, _config: &ScorerConfig) -> Result<CategoryScore> {
        // Detect repository archetype for calibrated scoring
        let archetype = self.detect_archetype(repo_path).await;

        let g1 = self.score_time_to_interaction(repo_path).await?;
        let g2 = self.score_error_gracefulness(repo_path, archetype).await?;
        let g3 = self.score_visual_stability(repo_path).await?;
        let g4 = self.score_wow_factor(repo_path).await?;

        // Calculate dynamic max score based on archetype (N/A categories removed from denominator)
        let effective_max = g1.max_score + g2.max_score + g3.max_score + g4.max_score;
        let total_score = g1.score + g2.score + g3.score + g4.score;

        let mut findings = vec![Finding {
            severity: Severity::Info,
            category: "Demo Quality".to_string(),
            message: format!("Repository detected as: {} archetype", archetype.name()),
            location: None,
            impact_points: 0.0,
        }];
        findings.extend(g1.findings.clone());
        findings.extend(g2.findings.clone());
        findings.extend(g3.findings.clone());
        findings.extend(g4.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            effective_max,
            vec![g1, g2, g3, g4],
            findings,
        ))
    }
}

impl Default for DemoScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        TempDir::new().unwrap()
    }

    fn create_readme(repo_path: &std::path::Path, content: &str) {
        let readme_path = repo_path.join("README.md");
        fs::write(readme_path, content).unwrap();
    }

    fn create_examples_dir(repo_path: &std::path::Path) {
        let examples_dir = repo_path.join("examples");
        fs::create_dir_all(&examples_dir).unwrap();

        // Create a sample example file
        fs::write(
            examples_dir.join("basic.rs"),
            r#"
fn main() {
    let result = do_something().expect("Failed to do something");
    println!("Result: {:?}", result);
}

fn do_something() -> Result<i32, String> {
    Ok(42)
}
"#,
        )
        .unwrap();
    }

    fn create_cargo_toml(repo_path: &std::path::Path, content: &str) {
        fs::write(repo_path.join("Cargo.toml"), content).unwrap();
    }

    const PROFESSIONAL_README: &str = r#"# Project

![Build](https://img.shields.io/badge/build-passing-green)
![Tests](https://img.shields.io/badge/tests-100%25-green)
![Coverage](https://img.shields.io/badge/coverage-85%25-green)
![License](https://img.shields.io/badge/license-MIT-blue)

<img src="docs/logo.svg" alt="Logo" width="200">

## Quick Start

```bash
cargo install myproject
```

## Demo

![Demo](docs/demo.gif)

## Getting Started

1. Install the project
2. Run `myproject --help`
"#;

    const MINIMAL_README: &str = r#"# Project

A project.
"#;

    #[tokio::test]
    async fn test_demo_scorer_professional_repo() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, PROFESSIONAL_README);
        create_examples_dir(repo_path);
        create_cargo_toml(
            repo_path,
            r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
indicatif = "0.17"
colored = "2.0"
"#,
        );

        let scorer = DemoScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should get a good score with professional setup
        assert!(
            result.score >= 6.0,
            "Professional repo should score >= 6.0, got {}",
            result.score
        );
        assert_eq!(result.subcategories.len(), 4);
    }

    #[tokio::test]
    async fn test_demo_scorer_minimal_repo() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(repo_path, MINIMAL_README);

        let scorer = DemoScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Minimal repo should get lower score
        assert!(
            result.score < 5.0,
            "Minimal repo should score < 5.0, got {}",
            result.score
        );
    }

    #[tokio::test]
    async fn test_time_to_interaction_with_examples() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_examples_dir(repo_path);
        create_readme(
            repo_path,
            "# Project\n\n## Quick Start\n\n```bash\ncargo run\n```",
        );

        let scorer = DemoScorer::new();
        let result = scorer.score_time_to_interaction(repo_path).await.unwrap();

        assert!(
            result.score >= 2.0,
            "Should score >= 2.0 with examples and quick-start"
        );
        assert_eq!(result.id, "G1");
    }

    #[tokio::test]
    async fn test_error_gracefulness_with_unwraps() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        let examples_dir = repo_path.join("examples");
        fs::create_dir_all(&examples_dir).unwrap();

        // Create file with many unwraps
        fs::write(
            examples_dir.join("bad.rs"),
            r#"
fn main() {
    let x = get_value().unwrap();
    let y = parse().unwrap();
    let z = read().unwrap();
    let a = write().unwrap();
    let b = compute().unwrap();
    let c = process().unwrap();
    panic!("Something went wrong");
}
"#,
        )
        .unwrap();

        let scorer = DemoScorer::new();
        // Use DemoApp archetype for standard error gracefulness scoring
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::DemoApp)
            .await
            .unwrap();

        // Should be penalized for raw unwraps and panic
        assert!(result.score < 3.0, "Should lose points for raw unwraps");
        assert!(
            result.findings.iter().any(|f| f.message.contains("unwrap")),
            "Should warn about unwraps"
        );
    }

    #[tokio::test]
    async fn test_error_gracefulness_cookbook_na() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = DemoScorer::new();
        // Cookbook archetype should have N/A for G2
        let result = scorer
            .score_error_gracefulness(repo_path, RepoArchetype::Cookbook)
            .await
            .unwrap();

        // G2 should be N/A for cookbooks (max_score = 0)
        assert_eq!(
            result.max_score, 0.0,
            "Cookbook G2 max_score should be 0 (N/A)"
        );
        assert_eq!(result.score, 0.0, "Cookbook G2 score should be 0 (N/A)");
        assert!(
            result.name.contains("N/A"),
            "Cookbook G2 should indicate N/A"
        );
    }

    #[tokio::test]
    async fn test_visual_stability_with_rich_libs() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_cargo_toml(
            repo_path,
            r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
indicatif = "0.17"
"#,
        );

        let scorer = DemoScorer::new();
        let result = scorer.score_visual_stability(repo_path).await.unwrap();

        // Should at least get partial credit for having the library in manifest
        assert!(result.score >= 0.5, "Should detect rich output library");
        assert!(
            result
                .findings
                .iter()
                .any(|f| f.message.contains("indicatif")),
            "Should mention the library"
        );
    }

    #[tokio::test]
    async fn test_archetype_detection_cookbook() {
        let temp_dir = TempDir::with_prefix("my-cookbook").unwrap();
        let repo_path = temp_dir.path();

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;

        assert_eq!(
            archetype,
            RepoArchetype::Cookbook,
            "Should detect cookbook by name"
        );
    }

    #[tokio::test]
    async fn test_archetype_detection_library() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        let src_dir = repo_path.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();

        let scorer = DemoScorer::new();
        let archetype = scorer.detect_archetype(repo_path).await;

        assert_eq!(
            archetype,
            RepoArchetype::Library,
            "Should detect library by src/lib.rs"
        );
    }

    #[tokio::test]
    async fn test_wow_factor_with_demo_gif() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();
        create_readme(
            repo_path,
            r#"# Project

![Build](https://img.shields.io/badge/build-passing-green)
![Tests](https://img.shields.io/badge/tests-100%25-green)
![Coverage](https://img.shields.io/badge/coverage-85%25-green)
![License](https://img.shields.io/badge/license-MIT-blue)

## Demo

![Demo](docs/demo.gif)
"#,
        );

        let scorer = DemoScorer::new();
        let result = scorer.score_wow_factor(repo_path).await.unwrap();

        assert!(result.score >= 1.0, "Should detect demo GIF");
        assert!(
            result.findings.iter().any(|f| f.message.contains("GIF")),
            "Should mention demo GIF"
        );
    }

    #[tokio::test]
    async fn test_category_name_and_max_score() {
        let scorer = DemoScorer::new();
        assert_eq!(scorer.category_name(), "Demo Quality");
        assert_eq!(scorer.max_score(), 10.0);
    }

    #[tokio::test]
    async fn test_empty_repo() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = DemoScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Empty repo should still return a valid score
        assert!(result.score >= 0.0);
        assert_eq!(result.max_score, 10.0);
    }
}
