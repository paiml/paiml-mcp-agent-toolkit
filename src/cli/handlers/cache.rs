#![cfg_attr(coverage_nightly, coverage(off))]
//! Cache Management Commands
//!
//! This module provides CLI commands for managing and optimizing cache strategies.
//!
//! ## Available Commands
//!
//! - `pmat cache stats` - Display current cache statistics

use crate::services::cache::{CacheOrchestrator, OrchestratorConfig};
use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Cache management commands
#[derive(Debug, Clone, Subcommand)]
pub enum CacheCommand {
    /// Display cache statistics and performance metrics
    Stats {
        /// Show detailed statistics
        #[arg(long)]
        detailed: bool,
        /// Output format (json, table)
        #[arg(long, default_value = "table")]
        format: String,
        /// Include historical data
        #[arg(long)]
        history: bool,
    },
}

/// Handle cache management commands
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_cache_command(command: &CacheCommand) -> Result<()> {
    match command {
        CacheCommand::Stats {
            detailed,
            format,
            history,
        } => handle_cache_stats(*detailed, format, *history).await,
    }
}

/// Build the report from the live orchestrator plus whatever cache state is on
/// disk under `root`.
///
/// This used to fetch `get_performance_metrics()` into `let _stats`, throw it
/// away, and print literals — 85.0% effectiveness next to 0 evaluations,
/// 100.0 req/sec, a 64.0 MB working set and 30.0% pressure — so an empty
/// directory and this 4000-file repo produced byte-identical output. Every
/// number below is now either read off the orchestrator or measured from the
/// on-disk cache directories; anything that cannot be computed is `None`
/// ("not measured"), never a plausible constant.
fn build_cache_stats(orchestrator: &CacheOrchestrator, root: &Path) -> CacheStatsOutput {
    let stats = orchestrator.get_orchestrator_stats();

    // Effectiveness is an average over evaluations. With zero evaluations
    // there is nothing to average, so we report nothing.
    let overall_effectiveness = if stats.evaluations_performed > 0 {
        Some(stats.current_metrics.effectiveness_score)
    } else {
        None
    };

    CacheStatsOutput {
        orchestrator_stats: OrchestratorStatsOutput {
            strategy_switches: stats.strategy_switches,
            evaluations_performed: stats.evaluations_performed,
            recommendations_generated: stats.recommendations_generated,
            performance_improvements: stats.performance_improvements,
            overall_effectiveness,
        },
        on_disk_caches: scan_on_disk_caches(root),
    }
}

/// Cache directories pmat writes under a project root.
const ON_DISK_CACHE_DIRS: [&str; 2] = [".pmat", ".pmat-cache"];

fn scan_on_disk_caches(root: &Path) -> Vec<OnDiskCacheOutput> {
    ON_DISK_CACHE_DIRS
        .iter()
        .filter_map(|name| {
            let dir = root.join(name);
            if !dir.is_dir() {
                return None;
            }
            let (entries, bytes) = measure_dir(&dir);
            Some(OnDiskCacheOutput {
                path: dir.display().to_string(),
                entries,
                bytes,
            })
        })
        .collect()
}

/// Count files and total bytes under `dir`, iteratively (cache trees nest).
fn measure_dir(dir: &Path) -> (u64, u64) {
    let mut entries = 0u64;
    let mut bytes = 0u64;
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in read.flatten() {
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() {
                stack.push(entry.path());
            } else {
                entries += 1;
                bytes += meta.len();
            }
        }
    }

    (entries, bytes)
}

async fn handle_cache_stats(detailed: bool, format: &str, history: bool) -> Result<()> {
    let config = OrchestratorConfig::default();
    let orchestrator = CacheOrchestrator::new(config);
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let output = build_cache_stats(&orchestrator, &root);

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&output)?),
        "table" => print_cache_stats_table(&output, &root, detailed, history),
        _ => return Err(anyhow::anyhow!("Unknown format: {format}")),
    }

    Ok(())
}

fn print_cache_stats_table(output: &CacheStatsOutput, root: &Path, detailed: bool, history: bool) {
    use crate::cli::colors as c;

    println!("{}", c::header("PMAT Cache Statistics"));
    println!("{}", c::rule());
    println!();
    println!("{}", c::subheader("Orchestrator (this process):"));
    let stats = &output.orchestrator_stats;
    println!(
        "   {}: {}",
        c::dim("Strategy Switches"),
        c::number(&stats.strategy_switches.to_string())
    );
    println!(
        "   {}: {}",
        c::dim("Evaluations"),
        c::number(&stats.evaluations_performed.to_string())
    );
    match stats.overall_effectiveness {
        Some(pct) => println!(
            "   {}: {}",
            c::dim("Overall Effectiveness"),
            c::pct(pct * 100.0, 70.0, 50.0)
        ),
        None => println!(
            "   {}: {}",
            c::dim("Overall Effectiveness"),
            c::dim("not measured (no cache evaluations in this process)")
        ),
    }
    println!();

    println!(
        "{}",
        c::subheader(&format!("On-disk caches under {}:", root.display()))
    );
    if output.on_disk_caches.is_empty() {
        println!("   {}", c::dim("none found"));
    } else {
        for cache in &output.on_disk_caches {
            println!(
                "   {}: {} file(s), {:.1} MB",
                c::dim(&cache.path),
                c::number(&cache.entries.to_string()),
                cache.bytes as f64 / (1024.0 * 1024.0)
            );
        }
    }
    println!();

    if detailed {
        println!("{}", c::subheader("Detailed Analysis:"));
        println!(
            "   {}",
            c::dim("Per-tier hit rates require a live cache session; the CLI process has none.")
        );
    }
    if history {
        println!("{}", c::dim("Historical Data: not recorded"));
    }
}

// Output structures for JSON serialization
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CacheStatsOutput {
    orchestrator_stats: OrchestratorStatsOutput,
    on_disk_caches: Vec<OnDiskCacheOutput>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct OrchestratorStatsOutput {
    strategy_switches: u64,
    evaluations_performed: u64,
    recommendations_generated: u64,
    performance_improvements: u64,
    /// `None` when no evaluation has run — an effectiveness score cannot be
    /// averaged over zero samples.
    overall_effectiveness: Option<f64>,
}

/// A cache directory that actually exists on disk, with what it really holds.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct OnDiskCacheOutput {
    path: String,
    entries: u64,
    bytes: u64,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // === CacheCommand enum tests ===

    #[test]
    fn test_cache_command_stats_default() {
        let cmd = CacheCommand::Stats {
            detailed: false,
            format: "table".to_string(),
            history: false,
        };

        match cmd {
            CacheCommand::Stats {
                detailed,
                format,
                history,
            } => {
                assert!(!detailed);
                assert_eq!(format, "table");
                assert!(!history);
            }
        }
    }

    #[test]
    fn test_cache_command_stats_detailed() {
        let cmd = CacheCommand::Stats {
            detailed: true,
            format: "json".to_string(),
            history: true,
        };

        match cmd {
            CacheCommand::Stats {
                detailed,
                format,
                history,
            } => {
                assert!(detailed);
                assert_eq!(format, "json");
                assert!(history);
            }
        }
    }

    #[test]
    fn test_cache_command_debug() {
        let cmd = CacheCommand::Stats {
            detailed: false,
            format: "table".to_string(),
            history: false,
        };

        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("Stats"));
        assert!(debug_str.contains("table"));
    }

    #[test]
    fn test_cache_command_clone() {
        let cmd = CacheCommand::Stats {
            detailed: true,
            format: "json".to_string(),
            history: true,
        };

        let cloned = cmd.clone();
        match cloned {
            CacheCommand::Stats {
                detailed,
                format,
                history,
            } => {
                assert!(detailed);
                assert_eq!(format, "json");
                assert!(history);
            }
        }
    }

    // === Output struct tests ===

    #[test]
    fn test_orchestrator_stats_output_serialization() {
        let stats = OrchestratorStatsOutput {
            strategy_switches: 5,
            evaluations_performed: 100,
            recommendations_generated: 10,
            performance_improvements: 8,
            overall_effectiveness: Some(0.92),
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("strategy_switches"));
        assert!(json.contains("5"));
        assert!(json.contains("0.92"));
    }

    #[test]
    fn test_cache_stats_output_full() {
        let output = CacheStatsOutput {
            orchestrator_stats: OrchestratorStatsOutput {
                strategy_switches: 3,
                evaluations_performed: 50,
                recommendations_generated: 5,
                performance_improvements: 4,
                overall_effectiveness: Some(0.88),
            },
            on_disk_caches: vec![OnDiskCacheOutput {
                path: ".pmat".to_string(),
                entries: 7,
                bytes: 2048,
            }],
        };

        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("orchestrator_stats"));
        assert!(json.contains("on_disk_caches"));
        assert!(json.contains(".pmat"));
    }

    // === Regression: cache stats must not fabricate ===

    #[test]
    fn test_effectiveness_is_not_measured_without_evaluations() {
        // `pmat cache stats` used to print "Overall Effectiveness: 85.0%"
        // next to "Evaluations: 0" — a percentage averaged over nothing.
        let orchestrator = CacheOrchestrator::new(OrchestratorConfig::default());
        let dir = tempfile::TempDir::new().unwrap();

        let output = build_cache_stats(&orchestrator, dir.path());
        assert_eq!(output.orchestrator_stats.evaluations_performed, 0);
        assert!(
            output.orchestrator_stats.overall_effectiveness.is_none(),
            "effectiveness cannot be computed from 0 evaluations"
        );
    }

    #[test]
    fn test_cache_stats_move_with_the_directory() {
        // An empty directory and a directory holding a real .pmat cache used
        // to produce byte-identical output.
        let orchestrator = CacheOrchestrator::new(OrchestratorConfig::default());

        let empty = tempfile::TempDir::new().unwrap();
        let populated = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(populated.path().join(".pmat")).unwrap();
        std::fs::write(
            populated.path().join(".pmat").join("context.idx"),
            b"0123456789",
        )
        .unwrap();

        let empty_stats = build_cache_stats(&orchestrator, empty.path());
        let populated_stats = build_cache_stats(&orchestrator, populated.path());

        assert!(empty_stats.on_disk_caches.is_empty());
        assert_eq!(populated_stats.on_disk_caches.len(), 1);
        assert_eq!(populated_stats.on_disk_caches[0].entries, 1);
        assert_eq!(populated_stats.on_disk_caches[0].bytes, 10);
        assert_ne!(
            empty_stats, populated_stats,
            "cache stats must depend on the directory being reported on"
        );
    }

    // === Handler tests ===

    #[tokio::test]
    async fn test_handle_cache_command_table_format() {
        let cmd = CacheCommand::Stats {
            detailed: false,
            format: "table".to_string(),
            history: false,
        };

        let result = handle_cache_command(&cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cache_command_json_format() {
        let cmd = CacheCommand::Stats {
            detailed: false,
            format: "json".to_string(),
            history: false,
        };

        let result = handle_cache_command(&cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cache_command_detailed() {
        let cmd = CacheCommand::Stats {
            detailed: true,
            format: "table".to_string(),
            history: false,
        };

        let result = handle_cache_command(&cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cache_command_with_history() {
        let cmd = CacheCommand::Stats {
            detailed: false,
            format: "table".to_string(),
            history: true,
        };

        let result = handle_cache_command(&cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_cache_command_invalid_format() {
        let cmd = CacheCommand::Stats {
            detailed: false,
            format: "invalid".to_string(),
            history: false,
        };

        let result = handle_cache_command(&cmd).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown format"));
    }

    // === Deserialization tests ===

    #[test]
    fn test_orchestrator_stats_output_deserialize() {
        let json = r#"{
            "strategy_switches": 10,
            "evaluations_performed": 200,
            "recommendations_generated": 20,
            "performance_improvements": 15,
            "overall_effectiveness": 0.95
        }"#;

        let stats: OrchestratorStatsOutput = serde_json::from_str(json).unwrap();
        assert_eq!(stats.strategy_switches, 10);
        assert_eq!(stats.overall_effectiveness, Some(0.95));
    }

    #[test]
    fn test_on_disk_cache_output_deserialize() {
        let json = r#"{ "path": ".pmat-cache", "entries": 42, "bytes": 8192 }"#;

        let cache: OnDiskCacheOutput = serde_json::from_str(json).unwrap();
        assert_eq!(cache.path, ".pmat-cache");
        assert_eq!(cache.entries, 42);
        assert_eq!(cache.bytes, 8192);
    }
}
