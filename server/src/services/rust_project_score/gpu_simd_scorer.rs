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
    pub fn new() -> Self {
        Self {
            name: "GPU/SIMD Quality".to_string(),
            max_points: MAX_POINTS,
            last_findings: Mutex::new(Vec::new()),
        }
    }

    /// Check if project has GPU/SIMD code
    fn has_gpu_simd_code(&self, project_path: &Path, cache: Option<&FileCache>) -> bool {
        // Check for CUDA files
        let cuda_extensions = ["cu", "cuh", "ptx"];
        let simd_patterns = [
            "std::arch::",
            "core::arch::",
            "_mm256_",
            "_mm512_",
            "arm_neon",
        ];
        let wgpu_patterns = ["wgpu::", "wgsl"];

        if let Some(file_cache) = cache {
            // Check cached files for SIMD/WGPU patterns
            for (path, content) in file_cache.get_rust_files_in_dir(&project_path.join("src")) {
                let path_str = path.to_string_lossy();
                for ext in &cuda_extensions {
                    if path_str.ends_with(ext) {
                        return true;
                    }
                }
                for pattern in &simd_patterns {
                    if content.contains(pattern) {
                        return true;
                    }
                }
                for pattern in &wgpu_patterns {
                    if content.contains(pattern) {
                        return true;
                    }
                }
            }
        }

        // Walk directory for CUDA files
        if let Ok(walker) = std::fs::read_dir(project_path) {
            for entry in walker.flatten() {
                if let Some(ext) = entry.path().extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if cuda_extensions.contains(&ext_str.as_str()) {
                        return true;
                    }
                    if ext_str == "wgsl" {
                        return true;
                    }
                }
            }
        }

        false
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
            // N/A: Project doesn't have GPU/SIMD code
            // Return full points (not penalized for not having GPU code)
            if let Ok(mut findings) = self.last_findings.lock() {
                findings.clear();
                findings.push("No GPU/SIMD code detected (N/A - full points)".to_string());
            }
            return Ok(CategoryScore::new(self.max_points, self.max_points));
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
        // Should get full points for N/A (no GPU code)
        assert_eq!(score.earned, 10.0);
    }

    #[test]
    fn test_clone() {
        let scorer = GpuSimdScorer::new();
        let cloned = scorer.clone();
        assert_eq!(scorer.name(), cloned.name());
        assert_eq!(scorer.max_points(), cloned.max_points());
    }
}
