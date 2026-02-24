/// Score a single brick's performance against its budget.
fn score_performance(mean_us: f64, budget_us: f64, brick_name: &str) -> BrickCheck {
    let budget_ratio = mean_us / budget_us;
    let perf_points = match () {
        _ if budget_ratio <= 1.0 => 4.0,
        _ if budget_ratio <= 1.5 => 2.0,
        _ if budget_ratio <= 2.0 => 1.0,
        _ => 0.0,
    };
    let recommendation = (budget_ratio > 1.0).then(|| {
        format!(
            "Optimize {} to meet {}µs budget (currently {:.1}µs, {:.0}% over)",
            brick_name,
            budget_us,
            mean_us,
            (budget_ratio - 1.0) * 100.0
        )
    });

    BrickCheck {
        name: brick_name.to_string(),
        passed: budget_ratio <= 1.0,
        points: perf_points,
        max_points: 4.0,
        actual: mean_us,
        threshold: budget_us,
        unit: "µs".to_string(),
        recommendation,
    }
}

/// Score a single brick's throughput efficiency.
fn score_efficiency(throughput: f64, brick_name: &str) -> BrickCheck {
    let eff_points = if throughput > 1_000_000.0 {
        2.5 // >1M elem/s
    } else if throughput > 100_000.0 {
        1.5 // >100K elem/s
    } else if throughput > 0.0 {
        0.5
    } else {
        0.0
    };

    BrickCheck {
        name: brick_name.to_string(),
        passed: throughput > 100_000.0,
        points: eff_points,
        max_points: 2.5,
        actual: throughput,
        threshold: 100_000.0,
        unit: "elem/s".to_string(),
        recommendation: if throughput < 100_000.0 {
            Some(format!(
                "Improve {} throughput (currently {:.0} elem/s)",
                brick_name, throughput
            ))
        } else {
            None
        },
    }
}

/// Score a single brick's measurement stability via coefficient of variation.
fn score_stability(cv: f64, brick_name: &str) -> BrickCheck {
    let stability_points = if cv < 5.0 {
        1.5 // Excellent stability
    } else if cv < 10.0 {
        1.0 // Good stability
    } else if cv < 15.0 {
        0.5 // Acceptable stability
    } else {
        0.0 // Unstable
    };

    BrickCheck {
        name: brick_name.to_string(),
        passed: cv < 15.0,
        points: stability_points,
        max_points: 1.5,
        actual: cv,
        threshold: 15.0,
        unit: "%".to_string(),
        recommendation: if cv >= 15.0 {
            Some(format!(
                "Stabilize {} measurements (CV {:.1}% exceeds 15% threshold)",
                brick_name, cv
            ))
        } else {
            None
        },
    }
}

/// Score a BrickProfiler output
///
/// PMAT-448: If hardware is provided, the score metadata will include
/// detailed hardware info for reproducibility.
pub fn score_brick_profiler(
    profiler_output: &BrickProfilerOutput,
    budgets: &[BrickBudget],
    project_path: &Path,
    hardware: Option<&HardwareCapability>,
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
            performance_checks.push(score_performance(mean_us, budget_us, &brick.name));
        }

        // Efficiency check: throughput
        efficiency_checks.push(score_efficiency(throughput, &brick.name));

        // Stability check: CV < 15%
        stability_checks.push(score_stability(cv, &brick.name));

        // PMAT-449: Estimate arithmetic intensity for roofline analysis
        // AI = FLOP / bytes_transferred
        // For typical ML operations: ~2 FLOPs per element (multiply-add)
        // Memory: 4 bytes per f32 element (read) + 4 bytes (write) = 8 bytes
        // Baseline AI ≈ 2 / 8 = 0.25 FLOP/byte
        let ai = estimate_arithmetic_intensity(&brick.name);
        let bottleneck_class = hardware.map(|hw| hw.bottleneck(ai, false));

        brick_reports.push(BrickReport {
            name: brick.name.clone(),
            mean_us,
            budget_us: budget,
            over_budget,
            cv_percent: cv,
            throughput,
            count: brick.count,
            arithmetic_intensity: Some(ai),
            bottleneck: bottleneck_class,
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
            actual: profiler_output
                .bricks
                .iter()
                .filter(|b| b.count > 0)
                .count() as f64,
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

    // PMAT-448: Calculate budget scale factor if hardware is provided
    let (simd_str, mem_bw, peak_gflops, scale_factor) = if let Some(hw) = hardware {
        let simd = format!("{:?}", hw.cpu.simd);
        let simd_factor = hw.cpu.simd.compute_speedup();
        let mem_bw_factor = hw.cpu.memory_bw_gbps / 25.0;
        let scale = (simd_factor * mem_bw_factor).sqrt();
        (
            Some(simd),
            Some(hw.cpu.memory_bw_gbps),
            Some(hw.cpu.peak_gflops),
            Some(scale),
        )
    } else {
        (None, None, None, None)
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
            hardware: profiler_output
                .hardware
                .clone()
                .or_else(|| hardware.map(|hw| format!("{} ({})", hw.cpu.model, hw.hostname))),
            total_bricks: profiler_output.bricks.len(),
            total_samples: profiler_output.bricks.iter().map(|b| b.count).sum(),
            simd: simd_str,
            memory_bw_gbps: mem_bw,
            peak_gflops,
            budget_scale_factor: scale_factor,
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
