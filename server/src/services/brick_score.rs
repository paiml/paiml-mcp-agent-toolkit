//! ComputeBrick Profiling Score Service (PMAT-446)
//!
//! Reads BrickProfiler JSON output and calculates a 100-point score:
//! - Performance (40 pts): Throughput vs theoretical peak
//! - Efficiency (25 pts): Backend utilization, memory efficiency
//! - Correctness (20 pts): Assertions passing, numerical accuracy
//! - Stability (15 pts): CV < 5%, reproducibility
//!
//! Reference: aprender/docs/specifications/qwen2.5-coder-showcase-demo.md §2.5

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// BrickProfiler JSON input format (matches trueno::brick::BrickStats)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrickStats {
    /// Brick name
    pub name: String,
    /// Total samples collected
    pub count: u64,
    /// Total elapsed time (nanoseconds)
    pub total_ns: u64,
    /// Min elapsed time (nanoseconds)
    pub min_ns: u64,
    /// Max elapsed time (nanoseconds)
    pub max_ns: u64,
    /// Total elements processed
    pub total_elements: u64,
}

impl BrickStats {
    /// Calculate mean time in microseconds
    pub fn mean_us(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.total_ns as f64 / self.count as f64) / 1000.0
        }
    }

    /// Calculate throughput in elements/second
    pub fn throughput(&self) -> f64 {
        if self.total_ns == 0 {
            0.0
        } else {
            (self.total_elements as f64 * 1_000_000_000.0) / self.total_ns as f64
        }
    }

    /// Calculate coefficient of variation (CV)
    /// Approximated from min/max range
    pub fn cv_percent(&self) -> f64 {
        if self.count < 2 || self.min_ns == 0 {
            0.0
        } else {
            let mean = self.total_ns as f64 / self.count as f64;
            let range = (self.max_ns - self.min_ns) as f64;
            // CV approximation: range / (2 * sqrt(3) * mean) for uniform distribution
            // Using simpler heuristic: range / (4 * mean) * 100
            (range / (4.0 * mean)) * 100.0
        }
    }
}

/// BrickProfiler JSON output format
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrickProfilerOutput {
    /// Per-brick statistics
    pub bricks: Vec<BrickStats>,
    /// Total tokens processed
    #[serde(default)]
    pub total_tokens: u64,
    /// Total time in nanoseconds
    #[serde(default)]
    pub total_ns: u64,
    /// Model name (if applicable)
    #[serde(default)]
    pub model: Option<String>,
    /// Hardware info
    #[serde(default)]
    pub hardware: Option<String>,
}

/// Brick budget specification (microseconds)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BrickBudget {
    /// Brick name pattern (supports wildcards)
    pub name: String,
    /// Maximum allowed time in microseconds
    pub max_us: f64,
}

/// Default brick budgets from qwen2.5-coder-showcase-demo.md
pub fn default_brick_budgets() -> Vec<BrickBudget> {
    vec![
        BrickBudget {
            name: "RmsNorm".to_string(),
            max_us: 10.0,
        },
        BrickBudget {
            name: "QKV".to_string(),
            max_us: 15.0,
        },
        BrickBudget {
            name: "RoPE".to_string(),
            max_us: 5.0,
        },
        BrickBudget {
            name: "Attention".to_string(),
            max_us: 25.0,
        },
        BrickBudget {
            name: "OProj".to_string(),
            max_us: 10.0,
        },
        BrickBudget {
            name: "FFNGateUp".to_string(),
            max_us: 20.0,
        },
        BrickBudget {
            name: "SwiGLU".to_string(),
            max_us: 5.0,
        },
        BrickBudget {
            name: "FFNDown".to_string(),
            max_us: 15.0,
        },
        BrickBudget {
            name: "Residual".to_string(),
            max_us: 3.0,
        },
    ]
}

/// Category score
#[derive(Debug, Clone, Serialize)]
pub struct CategoryScore {
    /// Category name
    pub name: String,
    /// Points earned
    pub earned: f64,
    /// Maximum points available
    pub max_points: f64,
    /// Individual checks
    pub checks: Vec<BrickCheck>,
}

impl CategoryScore {
    pub fn percentage(&self) -> f64 {
        if self.max_points == 0.0 {
            100.0
        } else {
            (self.earned / self.max_points) * 100.0
        }
    }
}

/// Individual brick check result
#[derive(Debug, Clone, Serialize)]
pub struct BrickCheck {
    /// Brick name
    pub name: String,
    /// Check passed
    pub passed: bool,
    /// Points earned
    pub points: f64,
    /// Max points for this check
    pub max_points: f64,
    /// Actual value
    pub actual: f64,
    /// Threshold value
    pub threshold: f64,
    /// Unit (µs, %, elem/s, etc.)
    pub unit: String,
    /// Recommendation if failed
    pub recommendation: Option<String>,
}

/// Complete brick score
#[derive(Debug, Clone, Serialize)]
pub struct BrickScore {
    /// Performance category (40 points)
    pub performance: CategoryScore,
    /// Efficiency category (25 points)
    pub efficiency: CategoryScore,
    /// Correctness category (20 points)
    pub correctness: CategoryScore,
    /// Stability category (15 points)
    pub stability: CategoryScore,
    /// Total score (0-100)
    pub total_score: f64,
    /// Letter grade
    pub grade: char,
    /// Individual brick reports
    pub brick_reports: Vec<BrickReport>,
    /// Metadata
    pub metadata: BrickScoreMetadata,
}

/// Per-brick report
#[derive(Debug, Clone, Serialize)]
pub struct BrickReport {
    pub name: String,
    pub mean_us: f64,
    pub budget_us: Option<f64>,
    pub over_budget: bool,
    pub cv_percent: f64,
    pub throughput: f64,
    pub count: u64,
}

/// Score metadata
#[derive(Debug, Clone, Serialize)]
pub struct BrickScoreMetadata {
    pub version: String,
    pub project_path: String,
    pub model: Option<String>,
    pub hardware: Option<String>,
    pub total_bricks: usize,
    pub total_samples: u64,
}

/// Score a BrickProfiler output
pub fn score_brick_profiler(
    profiler_output: &BrickProfilerOutput,
    budgets: &[BrickBudget],
    project_path: &Path,
) -> BrickScore {
    let mut performance_checks = Vec::new();
    let mut efficiency_checks = Vec::new();
    let mut stability_checks = Vec::new();
    let mut brick_reports = Vec::new();

    // Calculate per-brick scores
    for brick in &profiler_output.bricks {
        let mean_us = brick.mean_us();
        let cv = brick.cv_percent();
        let throughput = brick.throughput();

        // Find budget for this brick
        let budget = budgets
            .iter()
            .find(|b| brick.name.contains(&b.name))
            .map(|b| b.max_us);

        let over_budget = budget.map(|b| mean_us > b).unwrap_or(false);

        // Performance check: within budget
        if let Some(budget_us) = budget {
            let budget_ratio = mean_us / budget_us;
            let perf_points = if budget_ratio <= 1.0 {
                4.0 // Full points for meeting budget
            } else if budget_ratio <= 1.5 {
                2.0 // Half points for 50% over
            } else if budget_ratio <= 2.0 {
                1.0 // Quarter points for 100% over
            } else {
                0.0 // No points for >2x over budget
            };

            performance_checks.push(BrickCheck {
                name: brick.name.clone(),
                passed: budget_ratio <= 1.0,
                points: perf_points,
                max_points: 4.0,
                actual: mean_us,
                threshold: budget_us,
                unit: "µs".to_string(),
                recommendation: if budget_ratio > 1.0 {
                    Some(format!(
                        "Optimize {} to meet {}µs budget (currently {:.1}µs, {:.0}% over)",
                        brick.name,
                        budget_us,
                        mean_us,
                        (budget_ratio - 1.0) * 100.0
                    ))
                } else {
                    None
                },
            });
        }

        // Efficiency check: throughput > 0
        let eff_points = if throughput > 1_000_000.0 {
            2.5 // >1M elem/s
        } else if throughput > 100_000.0 {
            1.5 // >100K elem/s
        } else if throughput > 0.0 {
            0.5
        } else {
            0.0
        };

        efficiency_checks.push(BrickCheck {
            name: brick.name.clone(),
            passed: throughput > 100_000.0,
            points: eff_points,
            max_points: 2.5,
            actual: throughput,
            threshold: 100_000.0,
            unit: "elem/s".to_string(),
            recommendation: if throughput < 100_000.0 {
                Some(format!(
                    "Improve {} throughput (currently {:.0} elem/s)",
                    brick.name, throughput
                ))
            } else {
                None
            },
        });

        // Stability check: CV < 15%
        let stability_points = if cv < 5.0 {
            1.5 // Excellent stability
        } else if cv < 10.0 {
            1.0 // Good stability
        } else if cv < 15.0 {
            0.5 // Acceptable stability
        } else {
            0.0 // Unstable
        };

        stability_checks.push(BrickCheck {
            name: brick.name.clone(),
            passed: cv < 15.0,
            points: stability_points,
            max_points: 1.5,
            actual: cv,
            threshold: 15.0,
            unit: "%".to_string(),
            recommendation: if cv >= 15.0 {
                Some(format!(
                    "Stabilize {} measurements (CV {:.1}% exceeds 15% threshold)",
                    brick.name, cv
                ))
            } else {
                None
            },
        });

        brick_reports.push(BrickReport {
            name: brick.name.clone(),
            mean_us,
            budget_us: budget,
            over_budget,
            cv_percent: cv,
            throughput,
            count: brick.count,
        });
    }

    // Calculate category scores, normalized to category max based on brick count
    let num_bricks = profiler_output.bricks.len() as f64;

    // Performance: normalize per-brick scores (4 pts per brick max) to 40 pt scale
    let perf_per_brick_max = 4.0;
    let perf_raw: f64 = performance_checks.iter().map(|c| c.points).sum();
    let perf_max_possible = num_bricks * perf_per_brick_max;
    let perf_normalized = if perf_max_possible > 0.0 {
        (perf_raw / perf_max_possible) * 40.0
    } else {
        0.0
    };

    let performance = CategoryScore {
        name: "Performance".to_string(),
        earned: perf_normalized.min(40.0),
        max_points: 40.0,
        checks: performance_checks,
    };

    // Efficiency: normalize per-brick scores (2.5 pts per brick max) to 25 pt scale
    let eff_per_brick_max = 2.5;
    let eff_raw: f64 = efficiency_checks.iter().map(|c| c.points).sum();
    let eff_max_possible = num_bricks * eff_per_brick_max;
    let eff_normalized = if eff_max_possible > 0.0 {
        (eff_raw / eff_max_possible) * 25.0
    } else {
        0.0
    };

    let efficiency = CategoryScore {
        name: "Efficiency".to_string(),
        earned: eff_normalized.min(25.0),
        max_points: 25.0,
        checks: efficiency_checks,
    };

    // Correctness: based on having samples (proxy for assertions passing)
    let correctness_earned = if profiler_output.bricks.iter().all(|b| b.count > 0) {
        20.0
    } else {
        10.0
    };

    let correctness = CategoryScore {
        name: "Correctness".to_string(),
        earned: correctness_earned,
        max_points: 20.0,
        checks: vec![BrickCheck {
            name: "All bricks executed".to_string(),
            passed: profiler_output.bricks.iter().all(|b| b.count > 0),
            points: correctness_earned,
            max_points: 20.0,
            actual: profiler_output.bricks.iter().filter(|b| b.count > 0).count() as f64,
            threshold: profiler_output.bricks.len() as f64,
            unit: "bricks".to_string(),
            recommendation: None,
        }],
    };

    // Stability: normalize per-brick scores (1.5 pts per brick max) to 15 pt scale
    let stab_per_brick_max = 1.5;
    let stab_raw: f64 = stability_checks.iter().map(|c| c.points).sum();
    let stab_max_possible = num_bricks * stab_per_brick_max;
    let stab_normalized = if stab_max_possible > 0.0 {
        (stab_raw / stab_max_possible) * 15.0
    } else {
        0.0
    };

    let stability = CategoryScore {
        name: "Stability".to_string(),
        earned: stab_normalized.min(15.0),
        max_points: 15.0,
        checks: stability_checks,
    };

    // Total score
    let total_score =
        performance.earned + efficiency.earned + correctness.earned + stability.earned;

    // Grade
    let grade = match total_score as u32 {
        90..=100 => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        60..=69 => 'D',
        _ => 'F',
    };

    BrickScore {
        performance,
        efficiency,
        correctness,
        stability,
        total_score,
        grade,
        brick_reports,
        metadata: BrickScoreMetadata {
            version: "1.0.0".to_string(),
            project_path: project_path.display().to_string(),
            model: profiler_output.model.clone(),
            hardware: profiler_output.hardware.clone(),
            total_bricks: profiler_output.bricks.len(),
            total_samples: profiler_output.bricks.iter().map(|b| b.count).sum(),
        },
    }
}

/// Load BrickProfiler JSON from file
pub fn load_profiler_json(path: &Path) -> anyhow::Result<BrickProfilerOutput> {
    let content = fs::read_to_string(path)?;
    let output: BrickProfilerOutput = serde_json::from_str(&content)?;
    Ok(output)
}

/// Scan project for brick profiler JSON files
pub fn find_profiler_files(project_path: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();

    // Common locations for profiler output
    let patterns = [
        "brick_profile.json",
        "profiler.json",
        ".pmat/brick_profile.json",
        "target/brick_profile.json",
        "results.json",
    ];

    for pattern in patterns {
        let path = project_path.join(pattern);
        if path.exists() {
            files.push(path);
        }
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brick_stats_calculations() {
        let stats = BrickStats {
            name: "TestBrick".to_string(),
            count: 100,
            total_ns: 1_000_000, // 1ms total
            min_ns: 8_000,
            max_ns: 12_000,
            total_elements: 1_000_000,
        };

        assert!((stats.mean_us() - 10.0).abs() < 0.01);
        assert!(stats.throughput() > 0.0);
        assert!(stats.cv_percent() < 100.0);
    }

    #[test]
    fn test_score_calculation() {
        let output = BrickProfilerOutput {
            bricks: vec![
                BrickStats {
                    name: "RmsNorm".to_string(),
                    count: 100,
                    total_ns: 800_000, // 8µs mean (within 10µs budget)
                    min_ns: 7_000,
                    max_ns: 9_000,
                    total_elements: 1_000_000,
                },
                BrickStats {
                    name: "Attention".to_string(),
                    count: 100,
                    total_ns: 2_000_000, // 20µs mean (within 25µs budget)
                    min_ns: 18_000,
                    max_ns: 22_000,
                    total_elements: 500_000,
                },
            ],
            total_tokens: 1000,
            total_ns: 2_800_000,
            model: Some("test-model".to_string()),
            hardware: Some("test-hw".to_string()),
        };

        let budgets = default_brick_budgets();
        let score = score_brick_profiler(&output, &budgets, Path::new("."));

        assert!(score.total_score > 0.0);
        assert!(score.total_score <= 100.0);
        assert!(score.grade != 'F');
    }
}
