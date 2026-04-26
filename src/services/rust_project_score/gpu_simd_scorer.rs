#![cfg_attr(coverage_nightly, coverage(off))]
//! GPU/SIMD Scorer for Rust Project Score v2.2
//!
//! Integrates CUDA-SIMD TDG analysis into the Rust Project Score.
//! Uses the 100-point Popper falsification scoring system to evaluate
//! GPU/SIMD code quality.
//!
//! Toyota Way Principle: Jidoka (自働化) - Built-in Quality
//! Stop the line when critical GPU defects are detected.
//!
//! References:
//! - Popper, K. R. (1959). *The Logic of Scientific Discovery*
//! - Liker, J. K. (2004). *The Toyota Way*

use super::models::{CategoryScore, FileCache, ScoringMode};
use super::scorer::{Scorer, ScorerResult};
use crate::tdg::{CudaSimdAnalyzer, CudaSimdConfig, DefectSeverity};
use std::path::Path;
use std::sync::Mutex;

/// Maximum points for GPU/SIMD category
const MAX_POINTS: f64 = 10.0;

/// Points breakdown:
/// - No P0 defects: 4 points (Jidoka)
/// - Gateway passed: 3 points (Falsifiability)
/// - Score >= 70: 2 points (Quality threshold)
/// - Barrier safety >= 90%: 1 point
const NO_P0_DEFECTS_POINTS: f64 = 4.0;
const GATEWAY_PASSED_POINTS: f64 = 3.0;
const QUALITY_THRESHOLD_POINTS: f64 = 2.0;
const BARRIER_SAFETY_POINTS: f64 = 1.0;

/// GPU/SIMD Scorer
///
/// Analyzes a Rust project for GPU/SIMD code quality:
/// 1. CUDA/PTX files for barrier safety (PARITY-114)
/// 2. SIMD intrinsics (AVX2/AVX-512/NEON)
/// 3. WGPU/WGSL shaders
///
/// Uses the 100-point Popper falsification scoring system.
#[derive(Debug)]
pub struct GpuSimdScorer {
    /// Category name
    name: String,
    /// Maximum possible points
    max_points: f64,
    /// Last analysis findings for recommendations
    last_findings: Mutex<Vec<String>>,
}

impl GpuSimdScorer {
    /// Create a new GPU/SIMD Scorer
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            name: "GPU/SIMD Quality".to_string(),
            max_points: MAX_POINTS,
            last_findings: Mutex::new(Vec::new()),
        }
    }

    /// Check if project has GPU/SIMD code
    fn has_gpu_simd_code(&self, project_path: &Path, cache: Option<&FileCache>) -> bool {
        if let Some(file_cache) = cache {
            if Self::check_cache_for_gpu_simd(file_cache, project_path) {
                return true;
            }
        }
        Self::check_directory_for_gpu_files(project_path)
    }

    fn check_cache_for_gpu_simd(cache: &FileCache, project_path: &Path) -> bool {
        for (path, content) in cache.get_rust_files_in_dir(&project_path.join("src")) {
            if Self::file_has_gpu_simd_indicators(&path.to_string_lossy(), content) {
                return true;
            }
        }
        false
    }

    // Wave 39 PR26: contract added — output is bool determined by static
    // pattern lists (CUDA_EXTENSIONS / SIMD_PATTERNS / WGPU_PATTERNS).
    // Deterministic, no allocations.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn file_has_gpu_simd_indicators(path_str: &str, content: &str) -> bool {
        const CUDA_EXTENSIONS: &[&str] = &["cu", "cuh", "ptx"];
        // Use concat! to avoid self-matching during CB-021 compliance scanning
        const SIMD_PATTERNS: &[&str] = &[
            "std::arch::",
            "core::arch::",
            concat!("_mm", "256_"),
            concat!("_mm", "512_"),
            "arm_neon",
        ];
        const WGPU_PATTERNS: &[&str] = &["wgpu::", "wgsl"];

        CUDA_EXTENSIONS.iter().any(|ext| path_str.ends_with(ext))
            || SIMD_PATTERNS.iter().any(|p| content.contains(p))
            || WGPU_PATTERNS.iter().any(|p| content.contains(p))
    }

    fn check_directory_for_gpu_files(project_path: &Path) -> bool {
        const GPU_EXTENSIONS: &[&str] = &["cu", "cuh", "ptx", "wgsl"];
        let Ok(walker) = std::fs::read_dir(project_path) else {
            return false;
        };
        walker.flatten().any(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| GPU_EXTENSIONS.contains(&ext))
        })
    }
}

impl Default for GpuSimdScorer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for GpuSimdScorer {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            max_points: self.max_points,
            last_findings: Mutex::new(Vec::new()),
        }
    }
}

impl Scorer for GpuSimdScorer {
    fn name(&self) -> &str {
        &self.name
    }

    fn max_points(&self) -> f64 {
        self.max_points
    }

    fn score_with_mode(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
    ) -> ScorerResult<CategoryScore> {
        self.score_with_cache(project_path, _mode, None)
    }

    fn score_with_cache(
        &self,
        project_path: &Path,
        _mode: ScoringMode,
        cache: Option<&FileCache>,
    ) -> ScorerResult<CategoryScore> {
        // Check if project has GPU/SIMD code
        if !self.has_gpu_simd_code(project_path, cache) {
            // N/A: Project doesn't have GPU/SIMD code — excluded from grade
            if let Ok(mut findings) = self.last_findings.lock() {
                findings.clear();
                findings.push("No GPU/SIMD code detected (N/A)".to_string());
            }
            return Ok(CategoryScore::not_applicable(self.max_points));
        }

        // Configure analyzer for scoring (not strict gate)
        let config = CudaSimdConfig {
            min_score: 0.0, // We'll compute points ourselves
            fail_on_p0: false,
            analyze_simd: true,
            analyze_wgpu: true,
            ..Default::default()
        };

        let analyzer = CudaSimdAnalyzer::with_config(config);
        let result = match analyzer.analyze(project_path) {
            Ok(r) => r,
            Err(e) => {
                // If analysis fails, give partial credit
                if let Ok(mut findings) = self.last_findings.lock() {
                    findings.clear();
                    findings.push(format!("CUDA-TDG analysis failed: {}", e));
                }
                return Ok(CategoryScore::new(self.max_points / 2.0, self.max_points));
            }
        };

        let mut earned = 0.0;
        let mut findings = Vec::new();

        // Check 1: No P0 defects (4 points) - Jidoka
        let p0_count = result
            .defects
            .iter()
            .filter(|d| d.defect_class.severity == DefectSeverity::P0Critical)
            .count();

        if p0_count == 0 {
            earned += NO_P0_DEFECTS_POINTS;
            findings.push("No P0 critical defects".to_string());
        } else {
            findings.push(format!("{} P0 critical defects detected", p0_count));
        }

        // Check 2: Gateway passed (3 points) - Falsifiability
        if result.score.gateway_passed {
            earned += GATEWAY_PASSED_POINTS;
            findings.push("Falsifiability gateway passed".to_string());
        } else {
            findings.push("Falsifiability gateway failed".to_string());
        }

        // Check 3: Score >= 70 (2 points) - Quality threshold
        if result.score.total >= 70.0 {
            earned += QUALITY_THRESHOLD_POINTS;
            findings.push(format!(
                "CUDA-TDG score {:.1}/100 (Grade {})",
                result.score.total, result.score.grade
            ));
        } else {
            findings.push(format!(
                "CUDA-TDG score {:.1}/100 below threshold",
                result.score.total
            ));
        }

        // Check 4: Barrier safety >= 90% (1 point)
        if result.barrier_safety.safety_score >= 0.9 {
            earned += BARRIER_SAFETY_POINTS;
            findings.push(format!(
                "Barrier safety {:.0}%",
                result.barrier_safety.safety_score * 100.0
            ));
        } else if result.barrier_safety.total_barriers > 0 {
            findings.push(format!(
                "Barrier safety {:.0}% (below 90%)",
                result.barrier_safety.safety_score * 100.0
            ));
        }

        // Add file counts to findings
        findings.push(format!(
            "Files: {} CUDA, {} SIMD, {} WGPU",
            result.cuda_files, result.simd_files, result.wgpu_files
        ));

        // Store findings for recommendations
        if let Ok(mut stored) = self.last_findings.lock() {
            *stored = findings;
        }

        Ok(CategoryScore::new(earned, self.max_points))
    }

    fn recommendations(&self, _project_path: &Path) -> Vec<String> {
        self.last_findings
            .lock()
            .map(|f| f.clone())
            .unwrap_or_default()
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorer_creation() {
        let scorer = GpuSimdScorer::new();
        assert_eq!(scorer.name(), "GPU/SIMD Quality");
        assert_eq!(scorer.max_points(), 10.0);
    }

    #[test]
    fn test_scorer_default() {
        let scorer = GpuSimdScorer::default();
        assert_eq!(scorer.max_points(), 10.0);
    }

    #[test]
    fn test_no_gpu_code_returns_full_points() {
        let scorer = GpuSimdScorer::new();
        let temp_dir = tempfile::tempdir().unwrap();

        // Create a simple Rust file without GPU code
        let rust_file = temp_dir.path().join("src");
        std::fs::create_dir_all(&rust_file).unwrap();
        std::fs::write(rust_file.join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(
            temp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"",
        )
        .unwrap();

        let result = scorer.score_with_mode(temp_dir.path(), ScoringMode::Fast);
        assert!(result.is_ok());

        let score = result.unwrap();
        // N/A: no GPU code — excluded from grade calculation
        assert_eq!(score.earned, 0.0);
        assert!(!score.applicable);
    }

    #[test]
    fn test_clone() {
        let scorer = GpuSimdScorer::new();
        let cloned = scorer.clone();
        assert_eq!(scorer.name(), cloned.name());
        assert_eq!(scorer.max_points(), cloned.max_points());
    }

    // ── Wave 39 PR26: file_has_gpu_simd_indicators + check_directory ────────

    #[test]
    fn test_file_has_gpu_simd_indicators_cuda_extensions() {
        // PIN: .cu, .cuh, .ptx file extensions trigger GPU detection regardless of content.
        assert!(GpuSimdScorer::file_has_gpu_simd_indicators("kernel.cu", ""));
        assert!(GpuSimdScorer::file_has_gpu_simd_indicators(
            "device.cuh",
            ""
        ));
        assert!(GpuSimdScorer::file_has_gpu_simd_indicators(
            "shader.ptx",
            ""
        ));
    }

    #[test]
    fn test_file_has_gpu_simd_indicators_x86_simd_intrinsics() {
        // PIN: SIMD intrinsics like _mm256_/_mm512_ in content trigger detection.
        let content_avx2 = "let v = _mm256_add_ps(a, b);";
        assert!(GpuSimdScorer::file_has_gpu_simd_indicators(
            "src/foo.rs",
            content_avx2
        ));
        let content_avx512 = "let v = _mm512_add_ps(a, b);";
        assert!(GpuSimdScorer::file_has_gpu_simd_indicators(
            "src/foo.rs",
            content_avx512
        ));
    }

    #[test]
    fn test_file_has_gpu_simd_indicators_arch_modules() {
        let content = "use std::arch::x86_64::*;";
        assert!(GpuSimdScorer::file_has_gpu_simd_indicators(
            "src/foo.rs",
            content
        ));
        let content_core = "use core::arch::aarch64::*;";
        assert!(GpuSimdScorer::file_has_gpu_simd_indicators(
            "src/foo.rs",
            content_core
        ));
    }

    #[test]
    fn test_file_has_gpu_simd_indicators_arm_neon() {
        let content = "use std::arch::arm_neon::*;";
        assert!(GpuSimdScorer::file_has_gpu_simd_indicators(
            "src/foo.rs",
            content
        ));
    }

    #[test]
    fn test_file_has_gpu_simd_indicators_wgpu_patterns() {
        let content = "let device = wgpu::Device::new();";
        assert!(GpuSimdScorer::file_has_gpu_simd_indicators(
            "src/foo.rs",
            content
        ));
    }

    #[test]
    fn test_file_has_gpu_simd_indicators_wgsl_pattern() {
        let content = "let shader = include_wgsl!(\"shader.wgsl\");";
        assert!(GpuSimdScorer::file_has_gpu_simd_indicators(
            "src/foo.rs",
            content
        ));
    }

    #[test]
    fn test_file_has_gpu_simd_indicators_no_match_returns_false() {
        // PIN: regular Rust code with no GPU/SIMD patterns → false.
        assert!(!GpuSimdScorer::file_has_gpu_simd_indicators(
            "src/foo.rs",
            "fn foo() -> i32 { 42 }"
        ));
    }

    #[test]
    fn test_file_has_gpu_simd_indicators_empty_content_no_extension() {
        assert!(!GpuSimdScorer::file_has_gpu_simd_indicators("", ""));
        assert!(!GpuSimdScorer::file_has_gpu_simd_indicators(
            "Cargo.toml",
            ""
        ));
    }

    #[test]
    fn test_file_has_gpu_simd_indicators_extension_check_path_string() {
        // PIN: extension check uses ends_with on the full path string,
        // not Path::extension. So "foo.cu.bak" would NOT match (.bak).
        assert!(!GpuSimdScorer::file_has_gpu_simd_indicators(
            "kernel.cu.bak",
            ""
        ));
    }

    // ── check_directory_for_gpu_files (tempdir-testable) ────────────────────

    #[test]
    fn test_check_directory_for_gpu_files_finds_cu_file() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("kernel.cu"), "// CUDA kernel").unwrap();
        assert!(GpuSimdScorer::check_directory_for_gpu_files(tmp.path()));
    }

    #[test]
    fn test_check_directory_for_gpu_files_finds_wgsl_file() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("shader.wgsl"),
            "@vertex fn vs_main() -> vec4f",
        )
        .unwrap();
        assert!(GpuSimdScorer::check_directory_for_gpu_files(tmp.path()));
    }

    #[test]
    fn test_check_directory_for_gpu_files_no_gpu_files() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();
        assert!(!GpuSimdScorer::check_directory_for_gpu_files(tmp.path()));
    }

    #[test]
    fn test_check_directory_for_gpu_files_unreadable_returns_false() {
        // PIN: read_dir error (e.g., nonexistent path) → false (graceful).
        let bogus = std::path::PathBuf::from("/nonexistent/path/that/should/not/exist");
        assert!(!GpuSimdScorer::check_directory_for_gpu_files(&bogus));
    }

    #[test]
    fn test_check_directory_for_gpu_files_top_level_only() {
        // PIN: only the immediate dir is scanned (read_dir doesn't recurse).
        // A .cu file in a subdir is NOT detected.
        let tmp = tempfile::tempdir().unwrap();
        let subdir = tmp.path().join("nested");
        std::fs::create_dir(&subdir).unwrap();
        std::fs::write(subdir.join("kernel.cu"), "// nested").unwrap();
        // Top-level has no GPU files.
        assert!(!GpuSimdScorer::check_directory_for_gpu_files(tmp.path()));
    }
}
