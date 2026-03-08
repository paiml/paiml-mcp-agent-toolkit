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

/// Generate memory usage recommendations
fn generate_memory_recommendations(
    stats: &crate::services::memory_manager::MemoryStats,
) -> Vec<String> {
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
        c::pct(stats.allocation_pressure * 100.0, 80.0, 60.0)
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
