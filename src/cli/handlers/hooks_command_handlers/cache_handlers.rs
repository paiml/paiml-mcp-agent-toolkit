//! Cache management handlers for O(1) hooks cache (PMAT-453)

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::cli::commands::HooksCacheAction;
use crate::cli::OutputFormat;
use crate::tdg::hooks_cache::{CacheCheckResult, HooksCacheManager};
use anyhow::Result;

/// Handle hooks cache subcommand
pub(super) async fn handle_cache(action: &HooksCacheAction) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let manager = HooksCacheManager::new(&project_root);

    match action {
        HooksCacheAction::Init => handle_cache_init(&manager).await,
        HooksCacheAction::Status { format } => handle_cache_status(&manager, format).await,
        HooksCacheAction::Clear { gate } => handle_cache_clear(&manager, gate.as_deref()).await,
        HooksCacheAction::Metrics { format } => handle_cache_metrics(&manager, format).await,
    }
}

/// Initialize cache directory structure
async fn handle_cache_init(manager: &HooksCacheManager) -> Result<()> {
    println!("📁 Initializing hooks cache...");

    manager.init()?;

    println!("✅ Cache directory structure created:");
    println!("   .pmat/hooks-cache/");
    println!("   ├── tree-hash.json    (Level 0: repo-wide cache)");
    println!("   ├── gates/            (Level 1: per-gate cache)");
    println!("   ├── files/            (Level 2: per-file cache)");
    println!("   └── metrics.json      (CB-021: health monitoring)");

    Ok(())
}

/// Show cache status and check result
async fn handle_cache_status(manager: &HooksCacheManager, format: &OutputFormat) -> Result<()> {
    let check_result = manager.check()?;
    let metrics = manager.get_metrics().unwrap_or_default();
    let hit_rate = manager.hit_rate().unwrap_or(0.0);

    match format {
        OutputFormat::Json | OutputFormat::Yaml => {
            let status = serde_json::json!({
                "cache_status": match &check_result {
                    CacheCheckResult::Hit { result, cached_at } => serde_json::json!({
                        "type": "hit",
                        "result": format!("{:?}", result),
                        "cached_at": cached_at.to_rfc3339()
                    }),
                    CacheCheckResult::Miss { reason } => serde_json::json!({
                        "type": "miss",
                        "reason": reason.to_string()
                    }),
                    CacheCheckResult::Partial { cached_gates, uncached_gates } => serde_json::json!({
                        "type": "partial",
                        "cached_gates": cached_gates,
                        "uncached_gates": uncached_gates
                    }),
                },
                "metrics": {
                    "total_runs": metrics.total_runs,
                    "cache_hits": metrics.cache_hits,
                    "cache_misses": metrics.cache_misses,
                    "hit_rate": hit_rate,
                    "avg_hit_time_ms": metrics.avg_cache_hit_time_ms,
                    "avg_miss_time_ms": metrics.avg_cache_miss_time_ms,
                    "cache_size_bytes": metrics.cache_size_bytes
                }
            });
            println!("{}", serde_json::to_string_pretty(&status)?);
        }
        _ => {
            println!("📊 Hooks Cache Status");
            println!("====================");
            println!();

            match &check_result {
                CacheCheckResult::Hit { result, cached_at } => {
                    println!("🎯 Cache Status: HIT");
                    println!("   Result: {:?}", result);
                    println!(
                        "   Cached at: {}",
                        cached_at.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    println!();
                    println!("   ✅ O(1) skip available - no full analysis needed");
                }
                CacheCheckResult::Miss { reason } => {
                    println!("❌ Cache Status: MISS");
                    println!("   Reason: {}", reason);
                    println!();
                    println!("   Full analysis required on next hook run");
                }
                CacheCheckResult::Partial {
                    cached_gates,
                    uncached_gates,
                } => {
                    println!("⚡ Cache Status: PARTIAL");
                    println!("   Cached gates: {}", cached_gates.join(", "));
                    println!("   Uncached gates: {}", uncached_gates.join(", "));
                }
            }

            println!();
            println!("📈 Metrics:");
            println!("   Total runs: {}", metrics.total_runs);
            println!("   Hit rate: {:.1}%", hit_rate * 100.0);
            println!("   Avg hit time: {:.1}ms", metrics.avg_cache_hit_time_ms);
            println!("   Avg miss time: {:.1}ms", metrics.avg_cache_miss_time_ms);
            println!("   Cache size: {} bytes", metrics.cache_size_bytes);
        }
    }

    Ok(())
}

/// Clear cache
async fn handle_cache_clear(manager: &HooksCacheManager, gate: Option<&str>) -> Result<()> {
    if let Some(gate_name) = gate {
        println!("🗑️  Clearing cache for gate: {}", gate_name);
        manager.clear_gate(gate_name)?;
        println!("✅ Gate cache cleared");
    } else {
        println!("🗑️  Clearing all hooks cache...");
        manager.clear()?;
        println!("✅ All cache cleared - next commit will run full analysis");
    }

    Ok(())
}

/// Show detailed metrics
async fn handle_cache_metrics(manager: &HooksCacheManager, format: &OutputFormat) -> Result<()> {
    let metrics = manager.get_metrics()?;
    let hit_rate = manager.hit_rate()?;
    let is_healthy = manager.is_healthy()?;

    match format {
        OutputFormat::Json | OutputFormat::Yaml => {
            let output = serde_json::json!({
                "total_runs": metrics.total_runs,
                "cache_hits": metrics.cache_hits,
                "cache_misses": metrics.cache_misses,
                "hit_rate": hit_rate,
                "avg_cache_hit_time_ms": metrics.avg_cache_hit_time_ms,
                "avg_cache_miss_time_ms": metrics.avg_cache_miss_time_ms,
                "cache_size_bytes": metrics.cache_size_bytes,
                "last_full_rebuild": metrics.last_full_rebuild.map(|t| t.to_rfc3339()),
                "health_status": if is_healthy { "healthy" } else { "degraded" }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!("📊 Hooks Cache Metrics (CB-021)");
            println!("==============================");
            println!();
            println!(
                "Health Status: {}",
                if is_healthy {
                    "✅ Healthy"
                } else {
                    "⚠️  Degraded"
                }
            );
            println!();
            println!("📈 Performance:");
            println!("   Total runs: {}", metrics.total_runs);
            println!("   Cache hits: {}", metrics.cache_hits);
            println!("   Cache misses: {}", metrics.cache_misses);
            println!("   Hit rate: {:.1}%", hit_rate * 100.0);
            println!();
            println!("⏱️  Timing:");
            println!(
                "   Avg cache hit: {:.2}ms (target: <5ms)",
                metrics.avg_cache_hit_time_ms
            );
            println!("   Avg cache miss: {:.2}ms", metrics.avg_cache_miss_time_ms);
            if let Some(last_rebuild) = metrics.last_full_rebuild {
                println!(
                    "   Last full rebuild: {}",
                    last_rebuild.format("%Y-%m-%d %H:%M:%S UTC")
                );
            }
            println!();
            println!("💾 Storage:");
            println!("   Cache size: {} bytes", metrics.cache_size_bytes);

            // Show health recommendation if degraded
            if !is_healthy {
                println!();
                println!("⚠️  Cache health is degraded (hit rate < 60%)");
                println!("   Consider running 'pmat hooks cache clear' to reset");
            }
        }
    }

    Ok(())
}
