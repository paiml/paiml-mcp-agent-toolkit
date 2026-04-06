/// Baseline file for ratchet mechanism
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHealthBaseline {
    pub version: String,
    pub generated: String,
    pub files: HashMap<String, BaselineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    pub lines: usize,
    pub test_lines: usize,
    pub tlr: f32,
    pub health: u8,
    pub status: String,
}

impl FileHealthBaseline {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            generated: chrono::Utc::now().to_rfc3339(),
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, metrics: &FileHealthMetrics) {
        let key = metrics.path.to_string_lossy().to_string();
        self.files.insert(
            key,
            BaselineEntry {
                lines: metrics.lines,
                test_lines: metrics.test_lines,
                tlr: metrics.tlr,
                health: metrics.health_score,
                status: metrics.size_class.as_str().to_string(),
            },
        );
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Check if a file violates the ratchet (grew larger)
    pub fn check_ratchet(&self, path: &str, current_lines: usize) -> Option<RatchetViolation> {
        debug_assert!(!path.is_empty(), "path must not be empty");
        if let Some(baseline) = self.files.get(path) {
            if current_lines > baseline.lines {
                return Some(RatchetViolation {
                    path: path.to_string(),
                    baseline_lines: baseline.lines,
                    current_lines,
                    growth: current_lines - baseline.lines,
                });
            }
        }
        None
    }
}

impl Default for FileHealthBaseline {
    fn default() -> Self {
        Self::new()
    }
}

/// Ratchet violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetViolation {
    pub path: String,
    pub baseline_lines: usize,
    pub current_lines: usize,
    pub growth: usize,
}

/// Analyze a single file for health metrics
pub fn analyze_file(
    path: &Path,
    test_lines: usize,
    avg_complexity: f32,
    churn_30d: usize,
) -> Option<FileHealthMetrics> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let content = fs::read_to_string(path).ok()?;
    let lines = content.lines().count();

    Some(FileHealthMetrics::calculate(
        path.to_path_buf(),
        lines,
        test_lines,
        avg_complexity,
        churn_30d,
    ))
}

/// Count lines in a file
pub fn count_lines(path: &Path) -> Option<usize> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let content = fs::read_to_string(path).ok()?;
    Some(content.lines().count())
}

/// Scan a directory for source files and analyze health
pub fn scan_directory(root: &Path, extensions: &[&str], exclude_patterns: &[&str]) -> Vec<PathBuf> {
    debug_assert!(root.exists(), "root must exist: {}", root.display());
    let mut files = Vec::new();

    fn visit_dir(
        dir: &Path,
        extensions: &[&str],
        exclude_patterns: &[&str],
        files: &mut Vec<PathBuf>,
    ) {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let path_str = path.to_string_lossy();

                // Check exclusions
                let excluded = exclude_patterns.iter().any(|p| path_str.contains(p));
                if excluded {
                    continue;
                }

                if path.is_dir() {
                    visit_dir(&path, extensions, exclude_patterns, files);
                } else if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if extensions.iter().any(|e| ext == *e) {
                            files.push(path);
                        }
                    }
                }
            }
        }
    }

    visit_dir(root, extensions, exclude_patterns, &mut files);
    files
}

// ── Cross-Stack Health Types ───────────────────────────────────────────────

/// Stack-wide health report aggregating multiple projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackHealthReport {
    /// Per-project health reports
    pub projects: Vec<(String, FileHealthReport)>,
    /// Average health score across the stack
    pub stack_average_health: u8,
    /// Top-10 worst files across all projects
    pub stack_worst_files: Vec<(String, FileHealthMetrics)>,
    /// Overall stack grade
    pub stack_grade: HealthGrade,
}

impl StackHealthReport {
    /// Build a stack health report from individual project reports.
    pub fn from_projects(projects: Vec<(String, FileHealthReport)>) -> Self {
        debug_assert!(!projects.is_empty(), "projects must not be empty");
        let total_health: u64 = projects.iter().map(|(_, r)| r.average_health as u64).sum();
        let count = projects.len().max(1) as u64;
        let stack_average_health = (total_health / count) as u8;
        let stack_grade = HealthGrade::from_score(stack_average_health);

        // Collect all files across projects, sort by health ascending
        let mut all_files: Vec<(String, FileHealthMetrics)> = Vec::new();
        for (project_name, report) in &projects {
            for file in &report.critical_files {
                all_files.push((project_name.clone(), file.clone()));
            }
            for file in &report.problem_files {
                all_files.push((project_name.clone(), file.clone()));
            }
        }
        all_files.sort_by_key(|(_, f)| f.health_score);
        let stack_worst_files: Vec<_> = all_files.into_iter().take(10).collect();

        Self {
            projects,
            stack_average_health,
            stack_worst_files,
            stack_grade,
        }
    }
}

/// Stack-level baseline aggregating per-project baselines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackBaseline {
    pub version: String,
    pub generated: String,
    /// Per-project baselines keyed by project name
    pub projects: HashMap<String, FileHealthBaseline>,
}

impl StackBaseline {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            generated: chrono::Utc::now().to_rfc3339(),
            projects: HashMap::new(),
        }
    }

    pub fn add_project(&mut self, name: String, baseline: FileHealthBaseline) {
        self.projects.insert(name, baseline);
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

impl Default for StackBaseline {
    fn default() -> Self {
        Self::new()
    }
}

/// Default exclusion patterns for Rust projects
pub const DEFAULT_EXCLUDE_PATTERNS: &[&str] = &[
    "target/",
    ".git/",
    "node_modules/",
    "vendor/",
    ".pmat-cache/",
    "_generated",
    ".generated.",
    "generated_contracts.rs", // pv codegen output — build artifact
    ".lake/",  // Lean 4 package cache (submodule dependencies)
    ".elan/",  // Lean toolchain
    "generated/", // Code generated by pv scaffold
    "build/",  // Common build output
];

/// Default extensions for Rust projects
pub const RUST_EXTENSIONS: &[&str] = &["rs"];
