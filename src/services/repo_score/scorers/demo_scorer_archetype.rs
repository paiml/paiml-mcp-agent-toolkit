// DemoScorer archetype detection - RepoArchetype enum, detection, and file counting helpers

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

}
