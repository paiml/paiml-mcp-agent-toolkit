// memory_display.rs - Display, formatting, and recommendation logic for memory stats
// Included by memory.rs via include!() - shares parent module scope

/// Build pool statistics output data
fn build_pool_stats_output(
    pool_stats: &rustc_hash::FxHashMap<
        crate::services::memory_manager::PoolType,
        crate::services::memory_manager::PoolStats,
    >,
) -> HashMap<String, PoolStatsOutput> {
    let mut pool_stats_output = HashMap::new();

    for (pool_type, pool_stats) in pool_stats {
        let efficiency_rating = calculate_pool_efficiency_rating(pool_stats.reuse_ratio);

        pool_stats_output.insert(
            format!("{pool_type:?}"),
            PoolStatsOutput {
                buffer_count: pool_stats.buffer_count,
                total_size: pool_stats.total_size,
                allocation_count: pool_stats.allocation_count,
                reuse_count: pool_stats.reuse_count,
                reuse_ratio: pool_stats.reuse_ratio,
                efficiency_rating: efficiency_rating.to_string(),
            },
        );
    }

    pool_stats_output
}

/// True when the in-process pool manager has recorded nothing at all.
///
/// `pmat memory` inspects `global_memory_manager()`, which is created fresh per
/// process and is never exercised by the `memory` subcommand itself, so every
/// counter is structurally zero on every invocation. Zeros from a manager that
/// was never used are not a measurement of this machine's memory.
fn nothing_was_recorded(stats: &crate::services::memory_manager::MemoryStats) -> bool {
    stats.total_allocated == 0
        && stats.peak_usage == 0
        && stats
            .pool_stats
            .values()
            .all(|p| p.allocation_count == 0 && p.reuse_count == 0)
}

/// The honest line to print instead of tuning advice derived from zeros.
const NO_MEMORY_DATA_NOTE: &str =
    "No allocations were recorded in this process: `pmat memory` reads the in-process pool \
     manager, which starts empty, so these counters measure nothing.";

/// Generate memory usage recommendations
fn generate_memory_recommendations(
    stats: &crate::services::memory_manager::MemoryStats,
) -> Vec<String> {
    // The zeros used to produce five "Pool X has low reuse efficiency (0.0%).
    // Consider adjusting pool size." lines on every run — tuning advice
    // synthesised from a pool that had never been asked for a buffer.
    if nothing_was_recorded(stats) {
        return vec![NO_MEMORY_DATA_NOTE.to_string()];
    }

    let mut recommendations = Vec::new();

    add_pressure_recommendations(&mut recommendations, stats.allocation_pressure);
    add_pool_efficiency_recommendations(&mut recommendations, &stats.pool_stats);

    if recommendations.is_empty() {
        recommendations.push("Memory usage is optimal.".to_string());
    }

    recommendations
}

/// Add memory pressure recommendations
fn add_pressure_recommendations(recommendations: &mut Vec<String>, allocation_pressure: f64) {
    if allocation_pressure > 0.9 {
        recommendations.push(
            "CRITICAL: Memory pressure very high. Consider reducing workload or increasing limits."
                .to_string(),
        );
    } else if allocation_pressure > 0.8 {
        recommendations.push("WARNING: High memory pressure. Monitor usage closely.".to_string());
    }
}

/// Add pool efficiency recommendations
fn add_pool_efficiency_recommendations(
    recommendations: &mut Vec<String>,
    pool_stats: &rustc_hash::FxHashMap<
        crate::services::memory_manager::PoolType,
        crate::services::memory_manager::PoolStats,
    >,
) {
    for (pool_type, pool_stats) in pool_stats {
        // A pool that was never asked for a buffer has no reuse efficiency to
        // be low: its 0.0% is the absence of a measurement, not a finding.
        if pool_stats.allocation_count == 0 {
            continue;
        }
        if pool_stats.reuse_ratio < 0.3 {
            recommendations.push(format!(
                "Pool {:?} has low reuse efficiency ({:.1}%). Consider adjusting pool size.",
                pool_type,
                pool_stats.reuse_ratio * 100.0
            ));
        }
    }
}

/// Output memory statistics in requested format
fn output_memory_stats(output: &MemoryStatsOutput, format: &str, detailed: bool) -> Result<()> {
    match format {
        "json" => output_json_format(output),
        "csv" => output_csv_format(output),
        "table" => print_memory_stats_table(output, detailed),
        _ => print_memory_stats_table(output, detailed),
    }
}

/// Output statistics in JSON format
fn output_json_format(output: &MemoryStatsOutput) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(output)?);
    Ok(())
}

/// Output statistics in CSV format
fn output_csv_format(output: &MemoryStatsOutput) -> Result<()> {
    println!("metric,value");
    println!("total_allocated,{}", output.total_allocated);
    println!("peak_usage,{}", output.peak_usage);
    println!("allocation_pressure,{:.3}", output.allocation_pressure);
    println!("string_intern_size,{}", output.string_intern_size);
    Ok(())
}

fn print_memory_stats_table(stats: &MemoryStatsOutput, detailed: bool) -> Result<()> {
    print_header();
    print_overall_stats(stats);

    if detailed {
        print_pool_stats(&stats.pool_stats);
    }

    print_recommendations(&stats.recommendations);

    Ok(())
}

/// Print the memory statistics header
fn print_header() {
    use crate::cli::colors as c;

    println!("{}", c::header("PMAT Memory Statistics"));
    println!();
}

/// Print overall memory usage statistics
fn print_overall_stats(stats: &MemoryStatsOutput) {
    use crate::cli::colors as c;

    println!("{}", c::subheader("Overall Memory Usage:"));
    println!("  {}: {}", c::label("Total Allocated"), c::number(&format_bytes(stats.total_allocated)));
    println!("  {}: {}", c::label("Peak Usage"), c::number(&format_bytes(stats.peak_usage)));
    println!(
        "  {}: {}",
        c::label("Pressure"),
        c::pct_inverse(stats.allocation_pressure * 100.0, 30.0, 60.0)
    );
    println!(
        "  {}: {}",
        c::label("String Intern"),
        c::number(&format_bytes(stats.string_intern_size))
    );
    println!();
}

/// Print pool-specific statistics
fn print_pool_stats(pool_stats: &HashMap<String, PoolStatsOutput>) {
    use crate::cli::colors as c;

    println!("{}", c::subheader("Pool Statistics:"));
    for (pool_name, stats) in pool_stats {
        print_single_pool_stats(pool_name, stats);
    }
}

/// Print statistics for a single pool
fn print_single_pool_stats(pool_name: &str, stats: &PoolStatsOutput) {
    use crate::cli::colors as c;

    println!("  {}:", c::label(pool_name));
    println!("    {}: {}", c::label("Buffers"), c::number(&stats.buffer_count.to_string()));
    println!("    {}: {}", c::label("Size"), c::number(&format_bytes(stats.total_size)));
    println!("    {}: {}", c::label("Allocations"), c::number(&stats.allocation_count.to_string()));
    println!("    {}: {}", c::label("Reuses"), c::number(&stats.reuse_count.to_string()));

    let colored_rating = match stats.efficiency_rating.as_str() {
        "Excellent" | "Good" => format!("{}{}{}", c::GREEN, stats.efficiency_rating, c::RESET),
        "Fair" => format!("{}{}{}", c::YELLOW, stats.efficiency_rating, c::RESET),
        _ => format!("{}{}{}", c::RED, stats.efficiency_rating, c::RESET),
    };
    println!(
        "    {}: {} ({})",
        c::label("Efficiency"),
        colored_rating,
        c::pct(stats.reuse_ratio * 100.0, 80.0, 60.0)
    );
    println!();
}

/// Print recommendations
fn print_recommendations(recommendations: &[String]) {
    use crate::cli::colors as c;

    println!("{}", c::subheader("Recommendations:"));
    for rec in recommendations {
        if rec.starts_with("CRITICAL:") {
            println!("  {}", c::fail(rec));
        } else if rec.starts_with("WARNING:") {
            println!("  {}", c::warn(rec));
        } else {
            println!("  {}", c::dim(rec));
        }
    }
}

/// Calculate efficiency rating from reuse ratio
fn calculate_pool_efficiency_rating(reuse_ratio: f64) -> &'static str {
    if reuse_ratio > 0.8 {
        "Excellent"
    } else if reuse_ratio > 0.6 {
        "Good"
    } else if reuse_ratio > 0.4 {
        "Fair"
    } else {
        "Poor"
    }
}

/// Format bytes in human-readable format (delegates to batuta-common)
fn format_bytes(bytes: usize) -> String {
    batuta_common::fmt::format_bytes(bytes as u64)
}

#[cfg(test)]
mod unrecorded_memory_tests {
    //! Regression tests for `pmat memory stats`: the pool manager is fresh per
    //! process and the subcommand never allocates through it, so every counter
    //! was zero — and five "Pool X has low reuse efficiency (0.0%)" tuning
    //! recommendations were synthesised from those zeros on every run.
    use super::{generate_memory_recommendations, nothing_was_recorded};
    use crate::services::memory_manager::{MemoryStats, PoolStats, PoolType};

    fn stats_with(pool: PoolStats, total_allocated: usize) -> MemoryStats {
        let mut pool_stats = rustc_hash::FxHashMap::default();
        pool_stats.insert(PoolType::AstParsing, pool);
        MemoryStats {
            total_allocated,
            pool_stats,
            string_intern_size: 0,
            peak_usage: total_allocated,
            allocation_pressure: 0.0,
        }
    }

    fn empty_pool() -> PoolStats {
        PoolStats {
            buffer_count: 0,
            total_size: 0,
            allocation_count: 0,
            reuse_count: 0,
            reuse_ratio: 0.0,
        }
    }

    #[test]
    fn test_untouched_pools_yield_a_not_measured_note_not_tuning_advice() {
        let stats = stats_with(empty_pool(), 0);
        assert!(nothing_was_recorded(&stats));

        let recommendations = generate_memory_recommendations(&stats);
        assert_eq!(recommendations.len(), 1);
        assert!(
            !recommendations[0].contains("low reuse efficiency"),
            "a pool that was never used cannot have low reuse efficiency: {:?}",
            recommendations[0]
        );
        assert!(recommendations[0].contains("measure nothing"));
    }

    #[test]
    fn test_a_used_pool_with_poor_reuse_still_gets_advice() {
        let used = PoolStats {
            buffer_count: 4,
            total_size: 4096,
            allocation_count: 100,
            reuse_count: 5,
            reuse_ratio: 0.05,
        };
        let stats = stats_with(used, 4096);
        assert!(!nothing_was_recorded(&stats));

        let recommendations = generate_memory_recommendations(&stats);
        assert!(
            recommendations
                .iter()
                .any(|r| r.contains("low reuse efficiency")),
            "measured poor reuse must still be reported: {recommendations:?}"
        );
    }
}
