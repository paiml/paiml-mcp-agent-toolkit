#![cfg_attr(coverage_nightly, coverage(off))]
//! Per-gate caching (Phase 2) and parallel gate execution (Phase 3).
//!
//! Provides partial cache checking, parallel execution via rayon,
//! and smart scheduling that combines cached and fresh gate results.

use anyhow::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::types::{
    CacheResult, GateCacheEntry, GateCheckResult, GateDefinition, GateRunResult,
    ParallelGateResults, SmartGateResults,
};
use super::HooksCacheManager;

impl HooksCacheManager {
    // =========================================================================
    // Phase 2: Per-Gate Caching
    // =========================================================================

    /// Check if a specific gate can be skipped (Level 1 cache)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn check_gate(&self, gate_name: &str, files: &[PathBuf]) -> Result<Option<GateCacheEntry>> {
        let gate_path = self
            .cache_dir
            .join("gates")
            .join(format!("{}.json", gate_name));

        if !gate_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&gate_path)?;
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
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
        std::fs::write(gate_path, content)?;

        Ok(())
    }

    /// Check which gates need to run (partial cache check)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        let overall = Self::calculate_overall_result(&results, &errors);

        Ok(ParallelGateResults {
            overall,
            results,
            errors,
            total_duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Run gates with smart scheduling (cached gates skip, uncached run in parallel)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
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
        let overall = Self::calculate_overall_result(&cached_results, &parallel_results.errors);

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

    /// Calculate overall result from gate results and errors
    fn calculate_overall_result(
        results: &[(String, GateRunResult)],
        errors: &[(String, String)],
    ) -> CacheResult {
        if !errors.is_empty() {
            CacheResult::Fail
        } else if results.iter().any(|(_, r)| r.result == CacheResult::Fail) {
            CacheResult::Fail
        } else if results.iter().any(|(_, r)| r.result == CacheResult::Warn) {
            CacheResult::Warn
        } else {
            CacheResult::Pass
        }
    }
}
