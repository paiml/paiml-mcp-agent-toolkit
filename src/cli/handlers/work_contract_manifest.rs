/// Immutable file manifest captured at work start
///
/// Detects file hiding/exclusion gaming by tracking ALL source files.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileManifest {
    /// All source files with metadata
    pub files: HashMap<PathBuf, FileEntry>,

    /// Files that MUST be included in coverage (no exclusion allowed)
    pub coverage_required: Vec<PathBuf>,

    /// Checksum of entire manifest (tamper detection)
    pub manifest_hash: String,
}

impl FileManifest {
    /// Build manifest from project directory
    pub fn build(project_path: &Path) -> Result<Self> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut files = HashMap::new();
        let mut coverage_required = Vec::new();

        // Walk source directories
        for entry in walkdir::WalkDir::new(project_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !Self::is_excluded(e.path()))
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_entry) = FileEntry::from_path(path)? {
                    let rel_path = path
                        .strip_prefix(project_path)
                        .unwrap_or(path)
                        .to_path_buf();

                    // Mark CUDA/AVX files as coverage-required
                    if matches!(
                        file_entry.category,
                        FileCategory::CudaKernel | FileCategory::SimdAvx | FileCategory::RustSource
                    ) {
                        coverage_required.push(rel_path.clone());
                    }

                    files.insert(rel_path, file_entry);
                }
            }
        }

        // Compute manifest hash
        let mut hasher = Sha256::new();
        let mut sorted_paths: Vec<_> = files.keys().collect();
        sorted_paths.sort();
        for path in sorted_paths {
            hasher.update(path.to_string_lossy().as_bytes());
            if let Some(entry) = files.get(path) {
                hasher.update(entry.content_hash.as_bytes());
            }
        }
        let manifest_hash = format!("{:x}", hasher.finalize());

        Ok(Self {
            files,
            coverage_required,
            manifest_hash,
        })
    }

    /// Check if path should be excluded from manifest
    fn is_excluded(path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let path_str = path.to_string_lossy();

        // Exclude common non-source directories
        path_str.contains("/target/")
            || path_str.contains("/.git/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/.pmat-")
            || path_str.ends_with(".lock")
    }

    /// Verify manifest integrity (find missing files)
    pub fn verify_integrity(&self, project_path: &Path) -> Vec<PathBuf> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let mut missing = Vec::new();

        for rel_path in self.files.keys() {
            let full_path = project_path.join(rel_path);
            if !full_path.exists() {
                missing.push(rel_path.clone());
            }
        }

        missing
    }
}

/// File entry in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// SHA256 of file content at baseline
    pub content_hash: String,

    /// Line count at baseline
    pub lines: usize,

    /// Function count at baseline (approximate)
    pub functions: usize,

    /// Maximum complexity at baseline
    pub max_complexity: u32,

    /// File category for coverage requirements
    pub category: FileCategory,
}

impl FileEntry {
    /// Create file entry from path
    pub fn from_path(path: &Path) -> Result<Option<Self>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let extension = path.extension().and_then(|e| e.to_str());

        // Only process source files
        let category = match extension {
            Some("rs") => Self::categorize_rust_file(path)?,
            Some("cu" | "cuh") => FileCategory::CudaKernel,
            Some("c" | "cpp" | "cc" | "h" | "hpp") => FileCategory::CSource,
            Some("py") => FileCategory::PythonSource,
            Some("ts" | "tsx" | "js" | "jsx") => FileCategory::TypeScriptSource,
            Some("go") => FileCategory::GoSource,
            Some("lean") => FileCategory::LeanSource,
            _ => return Ok(None),
        };

        let content = std::fs::read_to_string(path)?;
        let lines = content.lines().count();

        // Compute content hash
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = format!("{:x}", hasher.finalize());

        // Approximate function count (simple heuristic)
        let functions = Self::count_functions(&content, &category);

        Ok(Some(Self {
            content_hash,
            lines,
            functions,
            max_complexity: 0, // Will be computed by complexity analyzer
            category,
        }))
    }

    /// Categorize Rust file (detect SIMD, tests, etc.)
    fn categorize_rust_file(path: &Path) -> Result<FileCategory> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = std::fs::read_to_string(path)?;

        // Check for test files
        let path_str = path.to_string_lossy();
        if path_str.contains("/tests/")
            || path_str.contains("_test.rs")
            || path_str.ends_with("tests.rs")
        {
            return Ok(FileCategory::TestCode);
        }

        // Check for build scripts
        if path_str.ends_with("build.rs") {
            return Ok(FileCategory::BuildScript);
        }

        // Check for SIMD patterns
        if Self::contains_simd_patterns(&content) {
            return Ok(FileCategory::SimdAvx);
        }

        Ok(FileCategory::RustSource)
    }

    /// Check if content contains SIMD patterns
    fn contains_simd_patterns(content: &str) -> bool {
        debug_assert!(!content.is_empty(), "content must not be empty");
        // Use concat! to avoid CB-021 self-detection when scanning this file
        let patterns = [
            "#[target_feature(enable",
            "std::arch::x86_64",
            "std::arch::aarch64",
            concat!("_mm", "256_"),
            concat!("_mm", "512_"),
            concat!("_mm", "_"),
            concat!("vld", "1q_"),
            concat!("vst", "1q_"),
            "is_x86_feature_detected!",
            "core::arch::",
        ];
        patterns.iter().any(|p| content.contains(p))
    }

    /// Simple function count heuristic
    fn count_functions(content: &str, category: &FileCategory) -> usize {
        debug_assert!(!content.is_empty(), "content must not be empty");
        match category {
            FileCategory::RustSource | FileCategory::SimdAvx => {
                // Count `fn ` occurrences (simple heuristic)
                content.matches("fn ").count()
            }
            FileCategory::CudaKernel => {
                // Count __global__ and __device__ functions
                content.matches("__global__").count() + content.matches("__device__").count()
            }
            FileCategory::PythonSource => content.matches("def ").count(),
            FileCategory::TypeScriptSource => {
                content.matches("function ").count() + content.matches("=> {").count()
            }
            FileCategory::GoSource => content.matches("func ").count(),
            FileCategory::LeanSource => {
                content.matches("def ").count()
                    + content.matches("theorem ").count()
                    + content.matches("lemma ").count()
            }
            _ => 0,
        }
    }
}

/// File category for coverage requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileCategory {
    /// Standard Rust code - must be covered
    RustSource,
    /// CUDA kernels - must be covered (no hiding allowed)
    CudaKernel,
    /// SIMD/AVX code - must be covered (no hiding allowed)
    SimdAvx,
    /// C/C++ source
    CSource,
    /// Python source
    PythonSource,
    /// TypeScript/JavaScript source
    TypeScriptSource,
    /// Go source
    GoSource,
    /// Lean source (proof assistant)
    LeanSource,
    /// Test code - excluded from coverage
    TestCode,
    /// Build scripts - optional coverage
    BuildScript,
    /// Generated code - excluded
    Generated,
}
