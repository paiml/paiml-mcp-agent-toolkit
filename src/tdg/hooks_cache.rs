#![cfg_attr(coverage_nightly, coverage(off))]
//! O(1) Hooks Cache Manager (PMAT-453)
//!
//! Implements the 3-level hash hierarchy for O(1) pre-commit hooks:
//! - Level 0: Git tree hash (repo-wide cache)
//! - Level 1: Per-gate hash (staged files by type)
//! - Level 2: Per-file hash (individual file results)
//!
//! Phase 2: Per-gate caching for partial cache hits
//! Phase 3: Parallel gate execution with rayon
//!
//! Spec: server/docs/specifications/pmat-hooks-v2-spec.md

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Cache directory relative to project root
const CACHE_DIR: &str = ".pmat/hooks-cache";

/// Maximum cache age before forced re-run (hours)
const MAX_CACHE_AGE_HOURS: i64 = 24;

/// O(1) Hooks Cache Manager
#[derive(Debug)]
pub struct HooksCacheManager {
    project_path: PathBuf,
    cache_dir: PathBuf,
}

/// Level 0: Repo-wide cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeHashCache {
    /// Git tree hash of HEAD
    pub tree_hash: String,
    /// Overall result (pass/fail)
    pub result: CacheResult,
    /// Per-gate results
    pub gates: HashMap<String, GateCacheEntry>,
    /// Timestamp of cache creation
    pub timestamp: DateTime<Utc>,
    /// PMAT version that created this cache
    pub pmat_version: String,
    /// Config hash (invalidate on config change)
    pub config_hash: String,
}

/// Level 1: Per-gate cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCacheEntry {
    /// Hash of files relevant to this gate
    pub files_hash: String,
    /// Gate result
    pub result: CacheResult,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Number of files checked
    pub files_checked: usize,
    /// Warnings (if any)
    pub warnings: Vec<String>,
}

/// Cache result enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheResult {
    Pass,
    Fail,
    Warn,
}

/// Metrics for cache health monitoring (CB-021)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksCacheMetrics {
    pub total_runs: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_cache_hit_time_ms: f64,
    pub avg_cache_miss_time_ms: f64,
    pub last_full_rebuild: Option<DateTime<Utc>>,
    pub cache_size_bytes: u64,
}

/// Result of cache check
#[derive(Debug)]
pub enum CacheCheckResult {
    /// Cache hit - can skip all gates
    Hit {
        result: CacheResult,
        cached_at: DateTime<Utc>,
    },
    /// Cache miss - need to run gates
    Miss { reason: CacheMissReason },
    /// Partial hit - some gates can be skipped
    Partial {
        cached_gates: Vec<String>,
        uncached_gates: Vec<String>,
    },
}

/// Reason for cache miss
#[derive(Debug, Clone)]
pub enum CacheMissReason {
    /// No cache file exists
    NoCacheFile,
    /// Tree hash changed
    TreeHashChanged { old: String, new: String },
    /// Config hash changed
    ConfigHashChanged,
    /// Cache is stale (too old)
    CacheStale { age_hours: i64 },
    /// PMAT version changed
    VersionChanged { old: String, new: String },
    /// Cache file corrupted
    CacheCorrupted(String),
}

impl std::error::Error for CacheMissReason {}

// =========================================================================
// Phase 2 & 3: Gate Definition and Results
// =========================================================================

/// Definition of a quality gate for Phase 2/3
#[derive(Debug, Clone)]
pub struct GateDefinition {
    /// Gate name (e.g., "complexity", "satd", "format")
    pub name: String,
    /// Files this gate operates on
    pub files: Vec<PathBuf>,
    /// File patterns (e.g., "*.rs", "*.py")
    pub patterns: Vec<String>,
}

impl GateDefinition {
    /// Create a new gate definition
    pub fn new(name: impl Into<String>, files: Vec<PathBuf>) -> Self {
        Self {
            name: name.into(),
            files,
            patterns: vec![],
        }
    }

    /// Create with file patterns
    pub fn with_patterns(name: impl Into<String>, patterns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            files: vec![],
            patterns,
        }
    }
}

/// Result of running a single gate
#[derive(Debug, Clone)]
pub struct GateRunResult {
    /// Gate result
    pub result: CacheResult,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Warnings generated
    pub warnings: Vec<String>,
    /// Whether result came from cache
    pub from_cache: bool,
}

/// Result of checking which gates are cached (Phase 2)
#[derive(Debug)]
pub struct GateCheckResult {
    /// Gates with valid cache entries
    pub cached: Vec<(String, GateCacheEntry)>,
    /// Gates that need to run
    pub uncached: Vec<GateDefinition>,
}

/// Results of parallel gate execution (Phase 3)
#[derive(Debug)]
pub struct ParallelGateResults {
    /// Overall result
    pub overall: CacheResult,
    /// Individual gate results
    pub results: Vec<(String, GateRunResult)>,
    /// Errors from failed gates
    pub errors: Vec<(String, String)>,
    /// Total execution time
    pub total_duration_ms: u64,
}

/// Results of smart gate execution (Phase 2 + 3 combined)
#[derive(Debug)]
pub struct SmartGateResults {
    /// Overall result
    pub overall: CacheResult,
    /// Individual gate results (cached + freshly run)
    pub results: Vec<(String, GateRunResult)>,
    /// Errors from failed gates
    pub errors: Vec<(String, String)>,
    /// Number of gates that used cache
    pub gates_cached: usize,
    /// Number of gates that had to run
    pub gates_run: usize,
    /// Total execution time
    pub total_duration_ms: u64,
}

impl HooksCacheManager {
    /// Create new cache manager for project
    pub fn new(project_path: &Path) -> Self {
        let cache_dir = project_path.join(CACHE_DIR);
        Self {
            project_path: project_path.to_path_buf(),
            cache_dir,
        }
    }

    /// Initialize cache directory structure
    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.cache_dir)?;
        fs::create_dir_all(self.cache_dir.join("gates"))?;
        fs::create_dir_all(self.cache_dir.join("files"))?;

        // Create initial metrics file
        let metrics = HooksCacheMetrics::default();
        self.save_metrics(&metrics)?;

        Ok(())
    }

    /// Check cache for O(1) decision
    ///
    /// Returns immediately if cache hit (5ms target)
    pub fn check(&self) -> Result<CacheCheckResult> {
        let cache_path = self.cache_dir.join("tree-hash.json");

        // Check if cache file exists
        if !cache_path.exists() {
            return Ok(CacheCheckResult::Miss {
                reason: CacheMissReason::NoCacheFile,
            });
        }

        // Load cache
        let cache: TreeHashCache = match fs::read_to_string(&cache_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(c) => c,
                Err(e) => {
                    return Ok(CacheCheckResult::Miss {
                        reason: CacheMissReason::CacheCorrupted(e.to_string()),
                    });
                }
            },
            Err(e) => {
                return Ok(CacheCheckResult::Miss {
                    reason: CacheMissReason::CacheCorrupted(e.to_string()),
                })
            }
        };

        // Check tree hash
        let current_hash = self.get_tree_hash()?;
        if cache.tree_hash != current_hash {
            return Ok(CacheCheckResult::Miss {
                reason: CacheMissReason::TreeHashChanged {
                    old: cache.tree_hash,
                    new: current_hash,
                },
            });
        }

        // Check config hash
        let current_config_hash = self.get_config_hash()?;
        if cache.config_hash != current_config_hash {
            return Ok(CacheCheckResult::Miss {
                reason: CacheMissReason::ConfigHashChanged,
            });
        }

        // Check PMAT version
        let current_version = env!("CARGO_PKG_VERSION");
        if cache.pmat_version != current_version {
            return Ok(CacheCheckResult::Miss {
                reason: CacheMissReason::VersionChanged {
                    old: cache.pmat_version,
                    new: current_version.to_string(),
                },
            });
        }

        // Check cache staleness
        let age = Utc::now() - cache.timestamp;
        if age.num_hours() > MAX_CACHE_AGE_HOURS {
            return Ok(CacheCheckResult::Miss {
                reason: CacheMissReason::CacheStale {
                    age_hours: age.num_hours(),
                },
            });
        }

        // Cache hit!
        Ok(CacheCheckResult::Hit {
            result: cache.result,
            cached_at: cache.timestamp,
        })
    }

    /// Update cache after successful gate run
    pub fn update(
        &self,
        result: CacheResult,
        gates: HashMap<String, GateCacheEntry>,
    ) -> Result<()> {
        let cache = TreeHashCache {
            tree_hash: self.get_tree_hash()?,
            result,
            gates,
            timestamp: Utc::now(),
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            config_hash: self.get_config_hash()?,
        };

        let cache_path = self.cache_dir.join("tree-hash.json");
        let content = serde_json::to_string_pretty(&cache)?;
        fs::write(cache_path, content)?;

        Ok(())
    }

    /// Clear all caches
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)?;
        }
        self.init()?;
        Ok(())
    }

    /// Clear specific gate cache
    pub fn clear_gate(&self, gate_name: &str) -> Result<()> {
        let gate_path = self
            .cache_dir
            .join("gates")
            .join(format!("{}.json", gate_name));
        if gate_path.exists() {
            fs::remove_file(gate_path)?;
        }
        Ok(())
    }

    /// Get cache metrics
    pub fn get_metrics(&self) -> Result<HooksCacheMetrics> {
        let metrics_path = self.cache_dir.join("metrics.json");
        if !metrics_path.exists() {
            return Ok(HooksCacheMetrics::default());
        }

        let content = fs::read_to_string(metrics_path)?;
        let metrics: HooksCacheMetrics = serde_json::from_str(&content)?;
        Ok(metrics)
    }

    /// Update metrics after hook run
    pub fn record_run(&self, hit: bool, duration_ms: u64) -> Result<()> {
        let mut metrics = self.get_metrics()?;

        metrics.total_runs += 1;
        if hit {
            metrics.cache_hits += 1;
            // Update rolling average for hits
            let n = metrics.cache_hits as f64;
            metrics.avg_cache_hit_time_ms =
                ((n - 1.0) * metrics.avg_cache_hit_time_ms + duration_ms as f64) / n;
        } else {
            metrics.cache_misses += 1;
            metrics.last_full_rebuild = Some(Utc::now());
            // Update rolling average for misses
            let n = metrics.cache_misses as f64;
            metrics.avg_cache_miss_time_ms =
                ((n - 1.0) * metrics.avg_cache_miss_time_ms + duration_ms as f64) / n;
        }

        // Update cache size
        metrics.cache_size_bytes = self.calculate_cache_size()?;

        self.save_metrics(&metrics)?;
        Ok(())
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> Result<f64> {
        let metrics = self.get_metrics()?;
        if metrics.total_runs == 0 {
            return Ok(0.0);
        }
        Ok(metrics.cache_hits as f64 / metrics.total_runs as f64)
    }

    /// Check if cache is healthy (CB-021)
    pub fn is_healthy(&self) -> Result<bool> {
        let metrics = self.get_metrics()?;
        // Need at least 10 runs to assess health
        if metrics.total_runs < 10 {
            return Ok(true);
        }
        // Healthy if hit rate > 60%
        Ok(self.hit_rate()? > 0.60)
    }

    // =========================================================================
    // Phase 2: Per-Gate Caching
    // =========================================================================

    /// Check if a specific gate can be skipped (Level 1 cache)
    pub fn check_gate(&self, gate_name: &str, files: &[PathBuf]) -> Result<Option<GateCacheEntry>> {
        let gate_path = self
            .cache_dir
            .join("gates")
            .join(format!("{}.json", gate_name));

        if !gate_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&gate_path)?;
        let entry: GateCacheEntry = serde_json::from_str(&content)?;

        // Check if files hash matches
        let current_hash = self.hash_files(files)?;
        if entry.files_hash == current_hash {
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// Update a specific gate's cache
    pub fn update_gate(
        &self,
        gate_name: &str,
        files: &[PathBuf],
        result: CacheResult,
        duration_ms: u64,
        warnings: Vec<String>,
    ) -> Result<()> {
        let entry = GateCacheEntry {
            files_hash: self.hash_files(files)?,
            result,
            duration_ms,
            files_checked: files.len(),
            warnings,
        };

        let gate_path = self
            .cache_dir
            .join("gates")
            .join(format!("{}.json", gate_name));
        let content = serde_json::to_string_pretty(&entry)?;
        fs::write(gate_path, content)?;

        Ok(())
    }

    /// Check which gates need to run (partial cache check)
    pub fn check_gates(&self, gates: &[GateDefinition]) -> Result<GateCheckResult> {
        let mut cached = Vec::new();
        let mut uncached = Vec::new();

        for gate in gates {
            match self.check_gate(&gate.name, &gate.files)? {
                Some(entry) => cached.push((gate.name.clone(), entry)),
                None => uncached.push(gate.clone()),
            }
        }

        Ok(GateCheckResult { cached, uncached })
    }

    // =========================================================================
    // Phase 3: Parallel Execution
    // =========================================================================

    /// Run gates in parallel using rayon
    pub fn run_gates_parallel<F>(
        &self,
        gates: Vec<GateDefinition>,
        runner: F,
    ) -> Result<ParallelGateResults>
    where
        F: Fn(&GateDefinition) -> Result<GateRunResult> + Sync + Send,
    {
        let start = std::time::Instant::now();
        let results = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));

        // Run gates in parallel
        gates.par_iter().for_each(|gate| match runner(gate) {
            Ok(result) => {
                results
                    .lock()
                    .expect("mutex not poisoned")
                    .push((gate.name.clone(), result));
            }
            Err(e) => {
                errors
                    .lock()
                    .expect("mutex not poisoned")
                    .push((gate.name.clone(), e.to_string()));
            }
        });

        let results = Arc::try_unwrap(results)
            .expect("all parallel tasks completed")
            .into_inner()
            .expect("mutex not poisoned");
        let errors = Arc::try_unwrap(errors)
            .expect("all parallel tasks completed")
            .into_inner()
            .expect("mutex not poisoned");

        // Calculate overall result
        let overall = if !errors.is_empty() {
            CacheResult::Fail
        } else if results.iter().any(|(_, r)| r.result == CacheResult::Fail) {
            CacheResult::Fail
        } else if results.iter().any(|(_, r)| r.result == CacheResult::Warn) {
            CacheResult::Warn
        } else {
            CacheResult::Pass
        };

        Ok(ParallelGateResults {
            overall,
            results,
            errors,
            total_duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Run gates with smart scheduling (cached gates skip, uncached run in parallel)
    pub fn run_gates_smart<F>(
        &self,
        gates: Vec<GateDefinition>,
        runner: F,
    ) -> Result<SmartGateResults>
    where
        F: Fn(&GateDefinition) -> Result<GateRunResult> + Sync + Send,
    {
        let start = std::time::Instant::now();

        // Phase 2: Check which gates can be skipped
        let check_result = self.check_gates(&gates)?;

        // Collect cached results
        let mut cached_results: Vec<(String, GateRunResult)> = check_result
            .cached
            .into_iter()
            .map(|(name, entry)| {
                (
                    name,
                    GateRunResult {
                        result: entry.result,
                        duration_ms: 0, // Cached, no execution time
                        warnings: entry.warnings,
                        from_cache: true,
                    },
                )
            })
            .collect();

        // Phase 3: Run uncached gates in parallel
        let parallel_results = if !check_result.uncached.is_empty() {
            self.run_gates_parallel(check_result.uncached.clone(), &runner)?
        } else {
            ParallelGateResults {
                overall: CacheResult::Pass,
                results: vec![],
                errors: vec![],
                total_duration_ms: 0,
            }
        };

        // Update cache for newly run gates
        for (name, result) in &parallel_results.results {
            if let Some(gate) = check_result.uncached.iter().find(|g| &g.name == name) {
                let _ = self.update_gate(
                    name,
                    &gate.files,
                    result.result,
                    result.duration_ms,
                    result.warnings.clone(),
                );
            }
        }

        // Combine results
        cached_results.extend(parallel_results.results);

        // Calculate overall result
        let overall = if !parallel_results.errors.is_empty() {
            CacheResult::Fail
        } else if cached_results
            .iter()
            .any(|(_, r)| r.result == CacheResult::Fail)
        {
            CacheResult::Fail
        } else if cached_results
            .iter()
            .any(|(_, r)| r.result == CacheResult::Warn)
        {
            CacheResult::Warn
        } else {
            CacheResult::Pass
        };

        Ok(SmartGateResults {
            overall,
            results: cached_results,
            errors: parallel_results.errors,
            gates_cached: if check_result.uncached.is_empty() {
                gates.len()
            } else {
                gates.len() - check_result.uncached.len()
            },
            gates_run: check_result.uncached.len(),
            total_duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    // =========================================================================
    // Private helpers
    // =========================================================================

    /// Hash a list of files for Level 1/2 caching
    fn hash_files(&self, files: &[PathBuf]) -> Result<String> {
        let mut hasher = blake3::Hasher::new();

        for file in files {
            let path = if file.is_absolute() {
                file.clone()
            } else {
                self.project_path.join(file)
            };

            if path.exists() {
                let content = fs::read(&path)?;
                hasher.update(&content);
                // Also hash the path for uniqueness
                hasher.update(file.to_string_lossy().as_bytes());
            }
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Get git tree hash for HEAD
    fn get_tree_hash(&self) -> Result<String> {
        // Use HEAD^{tree} to get the tree hash of the root tree
        // This is more reliable than HEAD:. across different git versions
        let output = Command::new("git")
            .args(["rev-parse", "HEAD^{tree}"])
            .current_dir(&self.project_path)
            .output()
            .context("Failed to get git tree hash")?;

        if !output.status.success() {
            anyhow::bail!(
                "git rev-parse failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get hash of config files
    fn get_config_hash(&self) -> Result<String> {
        let mut hasher = blake3::Hasher::new();

        // Hash tdg-rules.toml if it exists
        let rules_path = self.project_path.join(".pmat/tdg-rules.toml");
        if rules_path.exists() {
            let content = fs::read(&rules_path)?;
            hasher.update(&content);
        }

        // Hash pmat.toml if it exists
        let pmat_path = self.project_path.join("pmat.toml");
        if pmat_path.exists() {
            let content = fs::read(&pmat_path)?;
            hasher.update(&content);
        }

        Ok(hasher.finalize().to_hex().to_string())
    }

    /// Save metrics to file
    fn save_metrics(&self, metrics: &HooksCacheMetrics) -> Result<()> {
        let metrics_path = self.cache_dir.join("metrics.json");
        let content = serde_json::to_string_pretty(metrics)?;
        fs::write(metrics_path, content)?;
        Ok(())
    }

    /// Calculate total cache size
    fn calculate_cache_size(&self) -> Result<u64> {
        let mut size = 0u64;
        if self.cache_dir.exists() {
            for entry in walkdir::WalkDir::new(&self.cache_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    size += entry.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
        Ok(size)
    }
}

impl std::fmt::Display for CacheMissReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheMissReason::NoCacheFile => write!(f, "No cache file exists"),
            CacheMissReason::TreeHashChanged { old, new } => {
                write!(f, "Tree hash changed: {} → {}", &old[..8], &new[..8])
            }
            CacheMissReason::ConfigHashChanged => write!(f, "Config file changed"),
            CacheMissReason::CacheStale { age_hours } => {
                write!(
                    f,
                    "Cache stale ({}h old, max {}h)",
                    age_hours, MAX_CACHE_AGE_HOURS
                )
            }
            CacheMissReason::VersionChanged { old, new } => {
                write!(f, "PMAT version changed: {} → {}", old, new)
            }
            CacheMissReason::CacheCorrupted(msg) => write!(f, "Cache corrupted: {}", msg),
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> Result<TempDir> {
        let temp = TempDir::new()?;

        // Initialize git repo
        let init_out = Command::new("git")
            .args(["init"])
            .current_dir(temp.path())
            .output()?;
        if !init_out.status.success() {
            anyhow::bail!(
                "git init failed: {}",
                String::from_utf8_lossy(&init_out.stderr)
            );
        }

        // Configure git user (required for commit)
        let _ = Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(temp.path())
            .output()?;
        let _ = Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(temp.path())
            .output()?;

        // Create a file and commit
        fs::write(temp.path().join("test.rs"), "fn main() {}")?;
        let add_out = Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()?;
        if !add_out.status.success() {
            anyhow::bail!(
                "git add failed: {}",
                String::from_utf8_lossy(&add_out.stderr)
            );
        }

        let commit_out = Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(temp.path())
            .output()?;
        if !commit_out.status.success() {
            anyhow::bail!(
                "git commit failed: {}",
                String::from_utf8_lossy(&commit_out.stderr)
            );
        }

        Ok(temp)
    }

    #[test]
    fn test_cache_manager_init() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());

        manager.init().unwrap();

        assert!(temp.path().join(".pmat/hooks-cache").exists());
        assert!(temp.path().join(".pmat/hooks-cache/gates").exists());
        assert!(temp.path().join(".pmat/hooks-cache/files").exists());
        assert!(temp.path().join(".pmat/hooks-cache/metrics.json").exists());
    }

    #[test]
    fn test_cache_miss_no_file() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        let result = manager.check().unwrap();

        match result {
            CacheCheckResult::Miss {
                reason: CacheMissReason::NoCacheFile,
            } => {}
            _ => panic!("Expected NoCacheFile miss"),
        }
    }

    #[test]
    fn test_cache_hit_after_update() {
        let temp = create_test_repo().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Update cache
        let gates = HashMap::new();
        manager.update(CacheResult::Pass, gates).unwrap();

        // Check should hit
        let result = manager.check().unwrap();

        match result {
            CacheCheckResult::Hit { result, .. } => {
                assert_eq!(result, CacheResult::Pass);
            }
            _ => panic!("Expected cache hit"),
        }
    }

    #[test]
    fn test_cache_clear() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Create some cache files
        fs::write(temp.path().join(".pmat/hooks-cache/tree-hash.json"), "{}").unwrap();

        manager.clear().unwrap();

        // Cache dir should be recreated empty
        assert!(temp.path().join(".pmat/hooks-cache").exists());
        assert!(!temp
            .path()
            .join(".pmat/hooks-cache/tree-hash.json")
            .exists());
    }

    #[test]
    fn test_metrics_recording() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Record some runs
        manager.record_run(true, 5).unwrap(); // hit
        manager.record_run(true, 7).unwrap(); // hit
        manager.record_run(false, 1000).unwrap(); // miss

        let metrics = manager.get_metrics().unwrap();

        assert_eq!(metrics.total_runs, 3);
        assert_eq!(metrics.cache_hits, 2);
        assert_eq!(metrics.cache_misses, 1);
    }

    #[test]
    fn test_hit_rate_calculation() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        manager.record_run(true, 5).unwrap();
        manager.record_run(true, 5).unwrap();
        manager.record_run(true, 5).unwrap();
        manager.record_run(false, 1000).unwrap();

        let hit_rate = manager.hit_rate().unwrap();

        assert!((hit_rate - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_cache_health_check() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Not enough data
        assert!(manager.is_healthy().unwrap());

        // Add 10 runs with good hit rate
        for _ in 0..8 {
            manager.record_run(true, 5).unwrap();
        }
        for _ in 0..2 {
            manager.record_run(false, 1000).unwrap();
        }

        // 80% hit rate - healthy
        assert!(manager.is_healthy().unwrap());
    }

    // =========================================================================
    // Phase 2: Per-Gate Caching Tests
    // =========================================================================

    #[test]
    fn test_gate_cache_miss_no_file() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        let result = manager.check_gate("complexity", &[]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_gate_cache_hit_after_update() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Create a test file
        let test_file = temp.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let files = vec![test_file.clone()];

        // Update gate cache
        manager
            .update_gate("complexity", &files, CacheResult::Pass, 100, vec![])
            .unwrap();

        // Check should hit
        let result = manager.check_gate("complexity", &files).unwrap();
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.result, CacheResult::Pass);
        assert_eq!(entry.duration_ms, 100);
    }

    #[test]
    fn test_gate_cache_miss_on_file_change() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Create a test file
        let test_file = temp.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let files = vec![test_file.clone()];

        // Update gate cache
        manager
            .update_gate("complexity", &files, CacheResult::Pass, 100, vec![])
            .unwrap();

        // Modify file
        fs::write(&test_file, "fn main() { println!(\"hello\"); }").unwrap();

        // Check should miss (file changed)
        let result = manager.check_gate("complexity", &files).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_check_gates_partial() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Create test files
        let file1 = temp.path().join("file1.rs");
        let file2 = temp.path().join("file2.rs");
        fs::write(&file1, "fn one() {}").unwrap();
        fs::write(&file2, "fn two() {}").unwrap();

        // Cache only complexity gate
        manager
            .update_gate(
                "complexity",
                &[file1.clone()],
                CacheResult::Pass,
                50,
                vec![],
            )
            .unwrap();

        // Check gates - complexity should be cached, satd should not
        let gates = vec![
            GateDefinition::new("complexity", vec![file1]),
            GateDefinition::new("satd", vec![file2]),
        ];

        let result = manager.check_gates(&gates).unwrap();

        assert_eq!(result.cached.len(), 1);
        assert_eq!(result.cached[0].0, "complexity");
        assert_eq!(result.uncached.len(), 1);
        assert_eq!(result.uncached[0].name, "satd");
    }

    // =========================================================================
    // Phase 3: Parallel Execution Tests
    // =========================================================================

    #[test]
    fn test_parallel_gate_execution() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        let gates = vec![
            GateDefinition::new("gate1", vec![]),
            GateDefinition::new("gate2", vec![]),
            GateDefinition::new("gate3", vec![]),
        ];

        // Simple runner that always passes
        let runner = |_gate: &GateDefinition| -> Result<GateRunResult> {
            Ok(GateRunResult {
                result: CacheResult::Pass,
                duration_ms: 10,
                warnings: vec![],
                from_cache: false,
            })
        };

        let results = manager.run_gates_parallel(gates, runner).unwrap();

        assert_eq!(results.overall, CacheResult::Pass);
        assert_eq!(results.results.len(), 3);
        assert!(results.errors.is_empty());
    }

    #[test]
    fn test_parallel_gate_with_failure() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        let gates = vec![
            GateDefinition::new("pass_gate", vec![]),
            GateDefinition::new("fail_gate", vec![]),
        ];

        // Runner that fails for fail_gate
        let runner = |gate: &GateDefinition| -> Result<GateRunResult> {
            Ok(GateRunResult {
                result: if gate.name == "fail_gate" {
                    CacheResult::Fail
                } else {
                    CacheResult::Pass
                },
                duration_ms: 10,
                warnings: vec![],
                from_cache: false,
            })
        };

        let results = manager.run_gates_parallel(gates, runner).unwrap();

        assert_eq!(results.overall, CacheResult::Fail);
    }

    #[test]
    fn test_smart_gate_execution_with_cache() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Create test files
        let file1 = temp.path().join("file1.rs");
        let file2 = temp.path().join("file2.rs");
        fs::write(&file1, "fn one() {}").unwrap();
        fs::write(&file2, "fn two() {}").unwrap();

        // Pre-cache gate1
        manager
            .update_gate("gate1", &[file1.clone()], CacheResult::Pass, 50, vec![])
            .unwrap();

        let gates = vec![
            GateDefinition::new("gate1", vec![file1]),
            GateDefinition::new("gate2", vec![file2]),
        ];

        let runner = |_gate: &GateDefinition| -> Result<GateRunResult> {
            Ok(GateRunResult {
                result: CacheResult::Pass,
                duration_ms: 100,
                warnings: vec![],
                from_cache: false,
            })
        };

        let results = manager.run_gates_smart(gates, runner).unwrap();

        assert_eq!(results.overall, CacheResult::Pass);
        assert_eq!(results.gates_cached, 1); // gate1 from cache
        assert_eq!(results.gates_run, 1); // gate2 had to run
        assert_eq!(results.results.len(), 2);

        // Verify gate1 came from cache
        let gate1_result = results.results.iter().find(|(n, _)| n == "gate1");
        assert!(gate1_result.is_some());
        assert!(gate1_result.unwrap().1.from_cache);
    }

    // =========================================================================
    // Additional Coverage Tests
    // =========================================================================

    #[test]
    fn test_cache_miss_corrupted_json() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Write corrupted JSON
        let cache_path = temp.path().join(".pmat/hooks-cache/tree-hash.json");
        fs::write(&cache_path, "{ invalid json ]]").unwrap();

        let result = manager.check().unwrap();
        match result {
            CacheCheckResult::Miss {
                reason: CacheMissReason::CacheCorrupted(_),
            } => {}
            _ => panic!("Expected CacheCorrupted miss"),
        }
    }

    #[test]
    fn test_cache_miss_tree_hash_changed() {
        let temp = create_test_repo().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Update cache
        manager.update(CacheResult::Pass, HashMap::new()).unwrap();

        // Modify file and commit to change tree hash
        fs::write(
            temp.path().join("test.rs"),
            "fn main() { println!(\"changed\"); }",
        )
        .unwrap();
        let _ = Command::new("git")
            .args(["add", "."])
            .current_dir(temp.path())
            .output()
            .unwrap();
        let _ = Command::new("git")
            .args(["commit", "-m", "change"])
            .current_dir(temp.path())
            .output()
            .unwrap();

        // Check should miss due to tree hash change
        let result = manager.check().unwrap();
        match result {
            CacheCheckResult::Miss {
                reason: CacheMissReason::TreeHashChanged { .. },
            } => {}
            _ => panic!("Expected TreeHashChanged miss, got {:?}", result),
        }
    }

    #[test]
    fn test_cache_miss_stale() {
        let temp = create_test_repo().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Create cache with old timestamp
        let cache = TreeHashCache {
            tree_hash: manager.get_tree_hash().unwrap(),
            result: CacheResult::Pass,
            gates: HashMap::new(),
            timestamp: Utc::now() - chrono::Duration::hours(48), // 48 hours old
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            config_hash: manager.get_config_hash().unwrap(),
        };

        let cache_path = temp.path().join(".pmat/hooks-cache/tree-hash.json");
        fs::write(&cache_path, serde_json::to_string_pretty(&cache).unwrap()).unwrap();

        // Check should miss due to staleness
        let result = manager.check().unwrap();
        match result {
            CacheCheckResult::Miss {
                reason: CacheMissReason::CacheStale { age_hours },
            } => {
                assert!(age_hours >= 48);
            }
            _ => panic!("Expected CacheStale miss, got {:?}", result),
        }
    }

    #[test]
    fn test_cache_miss_version_changed() {
        let temp = create_test_repo().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Create cache with different version
        let cache = TreeHashCache {
            tree_hash: manager.get_tree_hash().unwrap(),
            result: CacheResult::Pass,
            gates: HashMap::new(),
            timestamp: Utc::now(),
            pmat_version: "0.0.1-fake".to_string(), // Different version
            config_hash: manager.get_config_hash().unwrap(),
        };

        let cache_path = temp.path().join(".pmat/hooks-cache/tree-hash.json");
        fs::write(&cache_path, serde_json::to_string_pretty(&cache).unwrap()).unwrap();

        // Check should miss due to version change
        let result = manager.check().unwrap();
        match result {
            CacheCheckResult::Miss {
                reason: CacheMissReason::VersionChanged { old, new },
            } => {
                assert_eq!(old, "0.0.1-fake");
                assert_eq!(new, env!("CARGO_PKG_VERSION"));
            }
            _ => panic!("Expected VersionChanged miss, got {:?}", result),
        }
    }

    #[test]
    fn test_cache_miss_config_changed() {
        let temp = create_test_repo().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Create cache with different config hash
        let cache = TreeHashCache {
            tree_hash: manager.get_tree_hash().unwrap(),
            result: CacheResult::Pass,
            gates: HashMap::new(),
            timestamp: Utc::now(),
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            config_hash: "different_hash".to_string(), // Different config
        };

        let cache_path = temp.path().join(".pmat/hooks-cache/tree-hash.json");
        fs::write(&cache_path, serde_json::to_string_pretty(&cache).unwrap()).unwrap();

        // Check should miss due to config change
        let result = manager.check().unwrap();
        match result {
            CacheCheckResult::Miss {
                reason: CacheMissReason::ConfigHashChanged,
            } => {}
            _ => panic!("Expected ConfigHashChanged miss, got {:?}", result),
        }
    }

    #[test]
    fn test_clear_gate() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Create a test file
        let test_file = temp.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        // Cache a gate
        manager
            .update_gate(
                "complexity",
                &[test_file.clone()],
                CacheResult::Pass,
                100,
                vec![],
            )
            .unwrap();

        // Verify gate is cached
        let gate_path = temp.path().join(".pmat/hooks-cache/gates/complexity.json");
        assert!(gate_path.exists());

        // Clear specific gate
        manager.clear_gate("complexity").unwrap();

        // Verify gate is cleared
        assert!(!gate_path.exists());
    }

    #[test]
    fn test_clear_gate_nonexistent() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Clear nonexistent gate should succeed
        manager.clear_gate("nonexistent").unwrap();
    }

    #[test]
    fn test_cache_miss_reason_display() {
        // Test Display implementations
        let no_cache = CacheMissReason::NoCacheFile;
        assert_eq!(format!("{}", no_cache), "No cache file exists");

        let tree_changed = CacheMissReason::TreeHashChanged {
            old: "abc12345678".to_string(), // Need 8+ chars for truncation
            new: "def45678901".to_string(),
        };
        let tree_str = format!("{}", tree_changed);
        assert!(tree_str.contains("abc12345")); // First 8 chars
        assert!(tree_str.contains("def45678"));

        let config_changed = CacheMissReason::ConfigHashChanged;
        assert_eq!(format!("{}", config_changed), "Config file changed");

        let stale = CacheMissReason::CacheStale { age_hours: 48 };
        let stale_str = format!("{}", stale);
        assert!(stale_str.contains("48"));
        assert!(stale_str.contains("stale"));

        let version = CacheMissReason::VersionChanged {
            old: "1.0.0".to_string(),
            new: "2.0.0".to_string(),
        };
        let version_str = format!("{}", version);
        assert!(version_str.contains("1.0.0"));
        assert!(version_str.contains("2.0.0"));

        let corrupted = CacheMissReason::CacheCorrupted("bad json".to_string());
        assert!(format!("{}", corrupted).contains("bad json"));
    }

    #[test]
    fn test_parallel_gate_with_error() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        let gates = vec![
            GateDefinition::new("good_gate", vec![]),
            GateDefinition::new("error_gate", vec![]),
        ];

        // Runner that errors for error_gate
        let runner = |gate: &GateDefinition| -> Result<GateRunResult> {
            if gate.name == "error_gate" {
                anyhow::bail!("Simulated error")
            } else {
                Ok(GateRunResult {
                    result: CacheResult::Pass,
                    duration_ms: 10,
                    warnings: vec![],
                    from_cache: false,
                })
            }
        };

        let results = manager.run_gates_parallel(gates, runner).unwrap();

        // Should have errors recorded
        assert!(!results.errors.is_empty());
        assert!(results.errors.iter().any(|(name, _)| name == "error_gate"));
    }

    #[test]
    fn test_parallel_gate_with_warnings() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        let gates = vec![GateDefinition::new("warn_gate", vec![])];

        // Runner that returns warnings
        let runner = |_gate: &GateDefinition| -> Result<GateRunResult> {
            Ok(GateRunResult {
                result: CacheResult::Warn,
                duration_ms: 10,
                warnings: vec!["Test warning".to_string()],
                from_cache: false,
            })
        };

        let results = manager.run_gates_parallel(gates, runner).unwrap();

        assert_eq!(results.overall, CacheResult::Warn);
        assert_eq!(results.results.len(), 1);
        assert!(!results.results[0].1.warnings.is_empty());
    }

    #[test]
    fn test_gate_definition_patterns() {
        let gate = GateDefinition {
            name: "complexity".to_string(),
            files: vec![PathBuf::from("test.rs")],
            patterns: vec!["*.rs".to_string(), "*.ts".to_string()],
        };

        assert_eq!(gate.name, "complexity");
        assert_eq!(gate.files.len(), 1);
        assert_eq!(gate.patterns.len(), 2);
    }

    #[test]
    fn test_cache_result_warn() {
        let temp = create_test_repo().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Update cache with Warn result
        manager.update(CacheResult::Warn, HashMap::new()).unwrap();

        // Check should hit with Warn
        let result = manager.check().unwrap();
        match result {
            CacheCheckResult::Hit { result, .. } => {
                assert_eq!(result, CacheResult::Warn);
            }
            _ => panic!("Expected cache hit"),
        }
    }

    #[test]
    fn test_cache_result_fail() {
        let temp = create_test_repo().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Update cache with Fail result
        manager.update(CacheResult::Fail, HashMap::new()).unwrap();

        // Check should hit with Fail
        let result = manager.check().unwrap();
        match result {
            CacheCheckResult::Hit { result, .. } => {
                assert_eq!(result, CacheResult::Fail);
            }
            _ => panic!("Expected cache hit"),
        }
    }

    #[test]
    fn test_low_hit_rate_unhealthy() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Add 10 runs with low hit rate (20%)
        for _ in 0..2 {
            manager.record_run(true, 5).unwrap();
        }
        for _ in 0..8 {
            manager.record_run(false, 1000).unwrap();
        }

        // 20% hit rate - unhealthy (threshold is 60%)
        assert!(!manager.is_healthy().unwrap());
    }

    #[test]
    fn test_empty_gates_parallel() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        let gates: Vec<GateDefinition> = vec![];

        let runner = |_gate: &GateDefinition| -> Result<GateRunResult> {
            Ok(GateRunResult {
                result: CacheResult::Pass,
                duration_ms: 10,
                warnings: vec![],
                from_cache: false,
            })
        };

        let results = manager.run_gates_parallel(gates, runner).unwrap();

        // Empty gates should pass
        assert_eq!(results.overall, CacheResult::Pass);
        assert!(results.results.is_empty());
    }

    #[test]
    fn test_smart_gates_all_cached() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        // Create test file
        let file1 = temp.path().join("file1.rs");
        fs::write(&file1, "fn one() {}").unwrap();

        // Pre-cache gate1
        manager
            .update_gate("gate1", &[file1.clone()], CacheResult::Pass, 50, vec![])
            .unwrap();

        let gates = vec![GateDefinition::new("gate1", vec![file1])];

        let runner = |_gate: &GateDefinition| -> Result<GateRunResult> {
            panic!("Should not be called - gate is cached");
        };

        let results = manager.run_gates_smart(gates, runner).unwrap();

        assert_eq!(results.overall, CacheResult::Pass);
        assert_eq!(results.gates_cached, 1);
        assert_eq!(results.gates_run, 0);
    }

    #[test]
    fn test_gate_with_warnings_stored() {
        let temp = TempDir::new().unwrap();
        let manager = HooksCacheManager::new(temp.path());
        manager.init().unwrap();

        let test_file = temp.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();

        let warnings = vec!["Warning 1".to_string(), "Warning 2".to_string()];
        manager
            .update_gate(
                "complexity",
                &[test_file.clone()],
                CacheResult::Warn,
                100,
                warnings.clone(),
            )
            .unwrap();

        let result = manager.check_gate("complexity", &[test_file]).unwrap();
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.result, CacheResult::Warn);
        assert_eq!(entry.warnings.len(), 2);
    }
}
