#![cfg_attr(coverage_nightly, coverage(off))]

use super::parsing::{build_coverage_map, try_load_coverage_json, try_load_lcov_info};
use super::types::CoverageCache;
use std::collections::HashMap;
use std::path::Path;

// ── Cache Loading ───────────────────────────────────────────────────────────

/// Try to load coverage from the cache file.
///
/// Invalidation strategy (profdata-mtime-primary):
/// 1. If profdata dir found and mtime matches cached mtime -> VALID (regardless of git hash)
/// 2. If profdata dir found but mtime differs -> INVALID (profdata was regenerated)
/// 3. If no profdata dir found -> fall back to git hash comparison
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn load_coverage_from_cache(
    cache_path: &Path,
    head_hash: &str,
    project_root: &Path,
) -> Option<HashMap<String, HashMap<usize, u64>>> {
    let cache_json = std::fs::read_to_string(cache_path).ok()?;
    let cache: CoverageCache = serde_json::from_str(&cache_json).ok()?;

    // Primary: profdata mtime comparison (handles custom target dirs, symlinks)
    if let Some(cached_mtime) = cache.coverage_mtime {
        if let Some((current_mtime, _)) =
            get_profdata_mtime_and_dir(project_root, cache.profdata_dir.as_deref())
        {
            if current_mtime <= cached_mtime {
                return Some(cache.files); // profdata unchanged -> cache valid
            }
            return None; // profdata was regenerated
        }
    }

    // Fallback: git hash (only when profdata mtime unavailable)
    if cache.git_hash != head_hash {
        return None;
    }

    Some(cache.files)
}

/// Get the mtime (seconds since epoch) of a specific directory.
#[cfg_attr(coverage_nightly, coverage(off))]
fn dir_mtime(dir: &Path) -> Option<u64> {
    std::fs::metadata(dir)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

/// Extract target-dir from a cargo config TOML file (simple line-based parsing).
#[cfg_attr(coverage_nightly, coverage(off))]
fn target_dir_from_cargo_config(
    config_path: &Path,
    project_root: &Path,
) -> Vec<std::path::PathBuf> {
    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("target-dir") {
                return None;
            }
            let val = trimmed.split('=').nth(1)?;
            let dir = val.trim().trim_matches('"').trim_matches('\'');
            let target_path = if std::path::Path::new(dir).is_absolute() {
                std::path::PathBuf::from(dir)
            } else {
                project_root.join(dir)
            };
            Some(target_path.join("llvm-cov-target"))
        })
        .collect()
}

/// Scan /mnt/*/targets/{project_name}/llvm-cov-target for NVMe/RAID overrides.
///
/// Shell functions that wrap `cargo` may set CARGO_TARGET_DIR to NVMe paths,
/// but Command::new("cargo") bypasses shell functions. This heuristic finds
/// those directories directly.
#[cfg_attr(coverage_nightly, coverage(off))]
fn mnt_target_candidates(project_root: &Path) -> Vec<std::path::PathBuf> {
    let canonical = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf());
    let project_name = match canonical.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return vec![],
    };
    let entries = match std::fs::read_dir("/mnt") {
        Ok(e) => e,
        Err(_) => return vec![],
    };
    entries
        .flatten()
        .map(|e| {
            e.path()
                .join("targets")
                .join(project_name)
                .join("llvm-cov-target")
        })
        .collect()
}

/// Collect fast profdata candidate directories (no subprocess).
///
/// Resolution order:
/// 1. `stored_path` from previous cache
/// 2. `CARGO_TARGET_DIR` env var
/// 3. `.cargo/config.toml` target-dir (project-local, then global)
/// 4. `/mnt/*/targets/{project_name}/` (NVMe/RAID overrides)
/// 5. `project_root/target` (default, follows symlinks)
#[cfg_attr(coverage_nightly, coverage(off))]
fn collect_fast_candidates(
    project_root: &Path,
    stored_path: Option<&str>,
) -> Vec<std::path::PathBuf> {
    let mut candidates: Vec<std::path::PathBuf> = Vec::with_capacity(8);

    // Fast path: check previously stored directory first
    if let Some(p) = stored_path {
        candidates.push(std::path::PathBuf::from(p));
    }

    // 1. CARGO_TARGET_DIR env var
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        candidates.push(std::path::PathBuf::from(&target_dir).join("llvm-cov-target"));
    }

    // 2. .cargo/config.toml (project-local, then global)
    candidates.extend(target_dir_from_cargo_config(
        &project_root.join(".cargo/config.toml"),
        project_root,
    ));
    let cargo_home = std::env::var("CARGO_HOME").unwrap_or_else(|_| {
        std::env::var("HOME")
            .map(|h| format!("{h}/.cargo"))
            .unwrap_or_default()
    });
    if !cargo_home.is_empty() {
        let global_config = std::path::PathBuf::from(&cargo_home).join("config.toml");
        candidates.extend(target_dir_from_cargo_config(&global_config, project_root));
    }

    // 3. NVMe/RAID mount scan
    candidates.extend(mnt_target_candidates(project_root));

    // 4. Default target dir (follows symlinks via canonicalize)
    let default_target = project_root.join("target");
    if let Ok(canonical) = default_target.canonicalize() {
        candidates.push(canonical.join("llvm-cov-target"));
    }
    candidates.push(default_target.join("llvm-cov-target"));

    candidates
}

/// Fast profdata discovery: same as `get_profdata_mtime_and_dir()` but skips
/// `cargo metadata` subprocess (step 6). Returns immediately if no fast
/// candidate found. Used for pre-checks where hanging on a subprocess is
/// unacceptable (e.g., `--coverage-gaps` on repos without coverage data).
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn get_profdata_mtime_fast(
    project_root: &Path,
    stored_path: Option<&str>,
) -> Option<(u64, String)> {
    let candidates = collect_fast_candidates(project_root, stored_path);

    // For stored_path, check it specially (it's the raw path, not necessarily llvm-cov-target)
    if let Some(p) = stored_path {
        if let Some(mtime) = dir_mtime(std::path::Path::new(p)) {
            return Some((mtime, p.to_string()));
        }
    }

    // Check fast candidates only -- NO cargo metadata subprocess
    for dir in &candidates {
        if let Some(mtime) = dir_mtime(dir) {
            return Some((mtime, dir.to_string_lossy().to_string()));
        }
    }

    None
}

/// Try `cargo metadata` to discover target_directory (slow -- subprocess spawn).
#[cfg_attr(coverage_nightly, coverage(off))]
fn cargo_metadata_target_dir(project_root: &Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    for toolchain_arg in &["+nightly", "+stable"] {
        let output = match std::process::Command::new("cargo")
            .args([toolchain_arg, "metadata", "--no-deps", "--format-version=1"])
            .current_dir(project_root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
        {
            Ok(o) if o.status.success() => o,
            _ => continue,
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(dir) = extract_target_directory(&stdout) {
            result.push(std::path::PathBuf::from(dir).join("llvm-cov-target"));
        }
    }
    result
}

/// Extract "target_directory" value from cargo metadata JSON output.
fn extract_target_directory(json: &str) -> Option<&str> {
    let idx = json.find("\"target_directory\":\"")?;
    let rest = json.get(idx + 20..)?;
    let end = rest.find('"')?;
    rest.get(..end)
}

/// Resolve the cargo target directory, then find llvm-cov-target underneath.
///
/// Resolution order:
/// 1. `stored_path` from previous cache (fastest -- skip all resolution)
/// 2. `CARGO_TARGET_DIR` env var
/// 3. `.cargo/config.toml` -> `[build] target-dir` (project-local, then global)
/// 4. `/mnt/*/targets/{project_name}/` (NVMe/RAID overrides from shell functions)
/// 5. `project_root/target` (default, follows symlinks)
/// 6. `cargo metadata` target_directory (slow last resort)
///
/// Returns `(mtime, profdata_dir_path)` if found.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn get_profdata_mtime_and_dir(
    project_root: &Path,
    stored_path: Option<&str>,
) -> Option<(u64, String)> {
    // Fast path: check previously stored directory first
    if let Some(p) = stored_path {
        if let Some(mtime) = dir_mtime(std::path::Path::new(p)) {
            return Some((mtime, p.to_string()));
        }
    }

    let candidates = collect_fast_candidates(project_root, None);

    // Check fast candidates first, fall back to cargo metadata
    for dir in &candidates {
        if let Some(mtime) = dir_mtime(dir) {
            return Some((mtime, dir.to_string_lossy().to_string()));
        }
    }

    // 5. cargo metadata (heavyweight -- spawns subprocess)
    for dir in cargo_metadata_target_dir(project_root) {
        if let Some(mtime) = dir_mtime(&dir) {
            return Some((mtime, dir.to_string_lossy().to_string()));
        }
    }

    None
}

// ── Subprocess + Cache Write ────────────────────────────────────────────────

/// Run `cargo llvm-cov report --json` and parse the output into a coverage map.
///
/// On success, also writes the result to the cache file for future reuse.
/// Uses a 30-second timeout to prevent hanging on broken profdata.
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn run_cargo_llvm_cov_and_cache(
    project_root: &Path,
    cache_path: &Path,
    head_hash: &str,
) -> Result<HashMap<String, HashMap<usize, u64>>, String> {
    // Try lcov.info fallback first (no subprocess needed)
    if let Some(cov) = try_load_lcov_info(project_root) {
        write_coverage_cache(cache_path, head_hash, project_root, &cov);
        return Ok(cov);
    }

    // Try coverage.json files (from `cargo llvm-cov report --json --output-path`) (#158)
    if let Some(cov) = try_load_coverage_json(project_root) {
        write_coverage_cache(cache_path, head_hash, project_root, &cov);
        return Ok(cov);
    }

    // Fast pre-check: verify profdata directory exists before spawning subprocess.
    // Uses get_profdata_mtime_fast() which skips `cargo metadata` subprocess --
    // prevents 30s hangs on repos without coverage data (#212).
    if get_profdata_mtime_fast(project_root, None).is_none() {
        return Err("No coverage data available.\n\n\
            To generate it, run:\n  \
            cargo llvm-cov test --lib --no-report\n\n\
            Then re-run with --coverage-gaps.\n\
            Or pass --coverage-file <path> to use existing coverage JSON."
            .to_string());
    }

    eprintln!("Generating coverage report...");
    let output = run_llvm_cov_subprocess(project_root)?;
    let json = String::from_utf8_lossy(&output.stdout);
    let file_coverage = build_coverage_map(&json, project_root)?;

    write_coverage_cache(cache_path, head_hash, project_root, &file_coverage);
    Ok(file_coverage)
}

/// Write coverage data to the cache file.
pub(super) fn write_coverage_cache(
    cache_path: &Path,
    head_hash: &str,
    project_root: &Path,
    files: &HashMap<String, HashMap<usize, u64>>,
) {
    let (mtime, dir) = get_profdata_mtime_and_dir(project_root, None)
        .map(|(m, d)| (Some(m), Some(d)))
        .unwrap_or((None, None));
    let cache = CoverageCache {
        git_hash: head_hash.to_string(),
        coverage_mtime: mtime,
        profdata_dir: dir,
        files: files.clone(),
    };
    if let Ok(cache_json) = serde_json::to_string(&cache) {
        let _ = std::fs::create_dir_all(project_root.join(".pmat"));
        let _ = std::fs::write(cache_path, cache_json);
    }
}

/// Write a negative coverage cache -- records that no coverage data is available.
///
/// Uses `get_profdata_mtime_fast()` (no subprocess) so this never blocks.
/// Invalidated when git hash changes or profdata mtime changes (user runs
/// `cargo llvm-cov test`). Avoids 30s subprocess timeout on every invocation (#212).
#[cfg_attr(coverage_nightly, coverage(off))]
pub(super) fn write_negative_coverage_cache(
    cache_path: &Path,
    head_hash: &str,
    project_root: &Path,
) {
    let (mtime, dir) = get_profdata_mtime_fast(project_root, None)
        .map(|(m, d)| (Some(m), Some(d)))
        .unwrap_or((None, None));
    let cache = CoverageCache {
        git_hash: head_hash.to_string(),
        coverage_mtime: mtime,
        profdata_dir: dir,
        files: HashMap::new(),
    };
    if let Ok(cache_json) = serde_json::to_string(&cache) {
        let _ = std::fs::create_dir_all(project_root.join(".pmat"));
        let _ = std::fs::write(cache_path, cache_json);
    }
}

/// Spawn `cargo llvm-cov report --json` with timeout and pipe-safe I/O.
///
/// Tries `cargo +nightly` first (matching the toolchain used for instrumented builds),
/// falls back to default toolchain if nightly is unavailable.
fn run_llvm_cov_subprocess(project_root: &Path) -> Result<std::process::Output, String> {
    use std::process::{Command, Stdio};

    // Try nightly first (profdata is usually generated by nightly toolchain)
    let mut child = Command::new("cargo")
        .args(["+nightly", "llvm-cov", "report", "--json"])
        .current_dir(project_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .or_else(|_| {
            // Fallback: default toolchain
            Command::new("cargo")
                .args(["llvm-cov", "report", "--json"])
                .current_dir(project_root)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
        })
        .map_err(|e| format!("cargo llvm-cov report --json failed to spawn: {e}"))?;

    let stdout_handle = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture stdout from cargo llvm-cov".to_string())?;
    let stderr_handle = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to capture stderr from cargo llvm-cov".to_string())?;

    let (stdout, stderr) = spawn_reader_threads(stdout_handle, stderr_handle);

    wait_with_timeout(&mut child, std::time::Duration::from_secs(30))?;

    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait on cargo llvm-cov: {e}"))?;
    let stdout = stdout
        .join()
        .map_err(|_| "stdout reader thread panicked".to_string())?;
    let stderr = stderr
        .join()
        .map_err(|_| "stderr reader thread panicked".to_string())?;
    let output = std::process::Output {
        status,
        stdout,
        stderr,
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "No coverage data available.\n\nTo generate it, run:\n  \
            cargo llvm-cov test --lib --no-report\n\nThen re-run with --coverage or --coverage-gaps.\n\
            Or pass --coverage-file <path> to use existing coverage JSON.\n\n\
            cargo llvm-cov report --json stderr: {}",
            stderr.lines().take(3).collect::<Vec<_>>().join("\n")
        ));
    }

    Ok(output)
}

/// Spawn reader threads for stdout and stderr handles.
fn spawn_reader_threads(
    mut stdout_handle: std::process::ChildStdout,
    mut stderr_handle: std::process::ChildStderr,
) -> (
    std::thread::JoinHandle<Vec<u8>>,
    std::thread::JoinHandle<Vec<u8>>,
) {
    let stdout_thread = std::thread::spawn(move || -> Vec<u8> {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stdout_handle.read_to_end(&mut buf);
        buf
    });
    let stderr_thread = std::thread::spawn(move || -> Vec<u8> {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stderr_handle.read_to_end(&mut buf);
        buf
    });
    (stdout_thread, stderr_thread)
}

/// Poll child process with timeout, killing if exceeded.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: std::time::Duration,
) -> Result<(), String> {
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("No coverage data available.\n\n\
                    cargo llvm-cov report --json timed out after 30s.\n\
                    This usually means corrupted profdata. Try:\n  \
                    cargo llvm-cov clean\n  \
                    cargo llvm-cov test --lib --no-report\n\n\
                    Then re-run with --coverage or --coverage-gaps.\n\
                    Or pass --coverage-file <path> to use existing coverage JSON."
                    .to_string());
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
            Err(e) => return Err(format!("Failed to wait on cargo llvm-cov: {e}")),
        }
    }
}
