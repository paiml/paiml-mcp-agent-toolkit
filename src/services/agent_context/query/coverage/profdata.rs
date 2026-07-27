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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    //! PMAT-643: cover pure fs/compute helpers.
    //! Skip subprocess paths (`cargo llvm-cov report` / `cargo metadata`).
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(path, content).expect("write");
    }

    // --- dir_mtime ---

    #[test]
    fn test_dir_mtime_returns_none_for_missing_dir() {
        assert!(dir_mtime(Path::new("/tmp/absolutely-does-not-exist-aaa")).is_none());
    }

    #[test]
    fn test_dir_mtime_returns_some_for_existing_dir() {
        let tmp = TempDir::new().unwrap();
        let mtime = dir_mtime(tmp.path());
        assert!(
            mtime.is_some(),
            "expected Some mtime for {}",
            tmp.path().display()
        );
        assert!(mtime.unwrap() > 0);
    }

    // --- target_dir_from_cargo_config ---

    #[test]
    fn test_target_dir_from_cargo_config_missing_file_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let result = target_dir_from_cargo_config(&tmp.path().join("nonexistent.toml"), tmp.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_target_dir_from_cargo_config_no_target_dir_key_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let cfg = tmp.path().join("config.toml");
        write(&cfg, "[build]\nrustflags = []\n");
        let result = target_dir_from_cargo_config(&cfg, tmp.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_target_dir_from_cargo_config_relative_resolves_against_project_root() {
        let tmp = TempDir::new().unwrap();
        let cfg = tmp.path().join("config.toml");
        write(&cfg, "target-dir = \"custom-target\"\n");
        let result = target_dir_from_cargo_config(&cfg, tmp.path());
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            tmp.path().join("custom-target").join("llvm-cov-target")
        );
    }

    #[test]
    fn test_target_dir_from_cargo_config_absolute_preserved() {
        let tmp = TempDir::new().unwrap();
        let cfg = tmp.path().join("config.toml");
        write(&cfg, "target-dir = \"/mnt/custom\"\n");
        let result = target_dir_from_cargo_config(&cfg, tmp.path());
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            std::path::PathBuf::from("/mnt/custom/llvm-cov-target")
        );
    }

    #[test]
    fn test_target_dir_from_cargo_config_single_quotes_accepted() {
        let tmp = TempDir::new().unwrap();
        let cfg = tmp.path().join("config.toml");
        write(&cfg, "target-dir = 'with-single-quotes'\n");
        let result = target_dir_from_cargo_config(&cfg, tmp.path());
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            tmp.path()
                .join("with-single-quotes")
                .join("llvm-cov-target")
        );
    }

    // --- mnt_target_candidates ---

    #[test]
    fn test_mnt_target_candidates_returns_vec_shaped_like_project_name() {
        let tmp = TempDir::new().unwrap();
        // mnt_target_candidates canonicalizes the project path to get its file name,
        // then scans /mnt. We can't assert the count (host-dependent), but every
        // returned path should end with "{project_name}/llvm-cov-target".
        let project_name = tmp
            .path()
            .canonicalize()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let out = mnt_target_candidates(tmp.path());
        for p in out {
            assert!(
                p.to_string_lossy().contains(&project_name),
                "path missing project name: {}",
                p.display()
            );
            assert!(p.ends_with("llvm-cov-target"));
        }
    }

    // --- collect_fast_candidates ---

    #[test]
    fn test_collect_fast_candidates_uses_stored_path_first() {
        let tmp = TempDir::new().unwrap();
        let stored = "/tmp/my-stored-path";
        let out = collect_fast_candidates(tmp.path(), Some(stored));
        assert_eq!(out[0], std::path::PathBuf::from(stored));
    }

    #[test]
    fn test_collect_fast_candidates_includes_default_target_dir() {
        let tmp = TempDir::new().unwrap();
        let out = collect_fast_candidates(tmp.path(), None);
        // Default `project_root/target/llvm-cov-target` must be present.
        let default = tmp.path().join("target").join("llvm-cov-target");
        assert!(
            out.contains(&default),
            "default target dir missing; got: {out:?}"
        );
    }

    /// Restores `CARGO_TARGET_DIR` to its original value on drop, so a failing
    /// assertion cannot leak the override into other tests.
    struct CargoTargetDirGuard(Option<String>);

    impl CargoTargetDirGuard {
        fn set(value: &str) -> Self {
            let guard = Self(std::env::var("CARGO_TARGET_DIR").ok());
            std::env::set_var("CARGO_TARGET_DIR", value);
            guard
        }
    }

    impl Drop for CargoTargetDirGuard {
        fn drop(&mut self) {
            match self.0.take() {
                Some(v) => std::env::set_var("CARGO_TARGET_DIR", v),
                None => std::env::remove_var("CARGO_TARGET_DIR"),
            }
        }
    }

    #[test]
    #[serial_test::serial(cargo_target_dir_env)]
    fn test_collect_fast_candidates_honors_cargo_target_dir_env_var() {
        let tmp = TempDir::new().unwrap();
        let marker = "/tmp/test-cargo-target-dir-marker";
        let _guard = CargoTargetDirGuard::set(marker);

        let out = collect_fast_candidates(tmp.path(), None);

        let expected = std::path::PathBuf::from(marker).join("llvm-cov-target");
        assert!(
            out.contains(&expected),
            "expected marker path to be in candidates, got: {out:?}"
        );
    }

    #[test]
    fn test_collect_fast_candidates_picks_up_project_local_cargo_config() {
        let tmp = TempDir::new().unwrap();
        let cfg = tmp.path().join(".cargo").join("config.toml");
        write(&cfg, "target-dir = \"custom\"\n");
        let out = collect_fast_candidates(tmp.path(), None);
        let expected = tmp.path().join("custom").join("llvm-cov-target");
        assert!(
            out.contains(&expected),
            "expected project-local target-dir to be picked up; got: {out:?}"
        );
    }

    // --- extract_target_directory ---

    #[test]
    fn test_extract_target_directory_parses_basic_json() {
        let json = r#"{"target_directory":"/tmp/some/target","other":"x"}"#;
        assert_eq!(extract_target_directory(json), Some("/tmp/some/target"));
    }

    #[test]
    fn test_extract_target_directory_missing_key_returns_none() {
        let json = r#"{"other":"x"}"#;
        assert_eq!(extract_target_directory(json), None);
    }

    #[test]
    fn test_extract_target_directory_unterminated_value_returns_none() {
        // No closing quote after `target_directory`'s value → rest.find('"') returns None.
        let json = r#"{"target_directory":"still-open"#;
        assert_eq!(extract_target_directory(json), None);
    }

    #[test]
    fn test_extract_target_directory_handles_embedded_json_fragment() {
        let json = r#"{"packages":[],"target_directory":"/x/y","version":1}"#;
        assert_eq!(extract_target_directory(json), Some("/x/y"));
    }

    // --- load_coverage_from_cache ---

    #[test]
    fn test_load_cache_missing_file_returns_none() {
        let tmp = TempDir::new().unwrap();
        assert!(
            load_coverage_from_cache(&tmp.path().join("missing.json"), "abc1234", tmp.path(),)
                .is_none()
        );
    }

    #[test]
    fn test_load_cache_corrupt_json_returns_none() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("cache.json");
        write(&p, "{not valid json");
        assert!(load_coverage_from_cache(&p, "abc1234", tmp.path()).is_none());
    }

    #[test]
    fn test_load_cache_hash_mismatch_returns_none_when_no_mtime() {
        // When cache has no coverage_mtime, fallback is git hash comparison.
        let tmp = TempDir::new().unwrap();
        let cache = CoverageCache {
            git_hash: "oldhash".to_string(),
            coverage_mtime: None,
            profdata_dir: None,
            files: HashMap::new(),
        };
        let p = tmp.path().join("cache.json");
        write(&p, &serde_json::to_string(&cache).unwrap());
        assert!(load_coverage_from_cache(&p, "newhash", tmp.path()).is_none());
    }

    #[test]
    fn test_load_cache_hash_match_returns_files_when_no_mtime() {
        let tmp = TempDir::new().unwrap();
        let mut files = HashMap::new();
        let mut hits = HashMap::new();
        hits.insert(1usize, 2u64);
        files.insert("src/foo.rs".to_string(), hits);
        let cache = CoverageCache {
            git_hash: "abc".to_string(),
            coverage_mtime: None,
            profdata_dir: None,
            files: files.clone(),
        };
        let p = tmp.path().join("cache.json");
        write(&p, &serde_json::to_string(&cache).unwrap());
        let loaded = load_coverage_from_cache(&p, "abc", tmp.path()).expect("some");
        assert_eq!(loaded, files);
    }

    // --- write_coverage_cache ---

    #[test]
    fn test_write_coverage_cache_writes_json_and_creates_pmat_dir() {
        let tmp = TempDir::new().unwrap();
        let cache_path = tmp.path().join(".pmat").join("coverage-cache.json");
        let mut files = HashMap::new();
        let mut hits = HashMap::new();
        hits.insert(5usize, 1u64);
        files.insert("src/lib.rs".to_string(), hits.clone());
        write_coverage_cache(&cache_path, "sha-xyz", tmp.path(), &files);
        assert!(cache_path.exists(), "cache file missing");
        let content = std::fs::read_to_string(&cache_path).unwrap();
        let cache: CoverageCache = serde_json::from_str(&content).unwrap();
        assert_eq!(cache.git_hash, "sha-xyz");
        assert_eq!(cache.files.get("src/lib.rs"), Some(&hits));
    }

    // --- write_negative_coverage_cache ---

    #[test]
    fn test_write_negative_coverage_cache_writes_empty_files_map() {
        let tmp = TempDir::new().unwrap();
        let cache_path = tmp.path().join(".pmat").join("neg.json");
        write_negative_coverage_cache(&cache_path, "sha-neg", tmp.path());
        assert!(cache_path.exists());
        let cache: CoverageCache =
            serde_json::from_str(&std::fs::read_to_string(&cache_path).unwrap()).unwrap();
        assert_eq!(cache.git_hash, "sha-neg");
        assert!(cache.files.is_empty());
    }

    // --- get_profdata_mtime_fast ---

    #[test]
    fn test_get_profdata_mtime_fast_returns_stored_path_when_extant() {
        let tmp = TempDir::new().unwrap();
        // stored_path points to an existing directory — function returns (mtime, path).
        let stored = tmp.path().to_string_lossy().into_owned();
        let out = get_profdata_mtime_fast(tmp.path(), Some(&stored));
        assert!(out.is_some(), "expected Some for extant stored_path");
        let (_, path) = out.unwrap();
        assert_eq!(path, stored);
    }

    /// `CARGO_TARGET_DIR` is checked before the machine-dependent candidates
    /// (`/mnt/*/targets/*`, global cargo config), so pointing it at a directory
    /// that really exists makes the result deterministic on any host.
    #[test]
    #[serial_test::serial(cargo_target_dir_env)]
    fn test_get_profdata_mtime_fast_returns_cargo_target_dir_hit() {
        let project = TempDir::new().unwrap();
        let target = TempDir::new().unwrap();
        let profdata_dir = target.path().join("llvm-cov-target");
        std::fs::create_dir_all(&profdata_dir).unwrap();
        let _guard = CargoTargetDirGuard::set(&target.path().to_string_lossy());

        let (_mtime, dir) =
            get_profdata_mtime_fast(project.path(), None).expect("existing target dir is found");

        assert_eq!(std::path::Path::new(&dir), profdata_dir);
    }

    /// A candidate that does not exist on disk must never be returned. This is
    /// asserted instead of "the result is None" because `collect_fast_candidates`
    /// also probes `/mnt/*/targets/*` and the global cargo config, so a bare
    /// `is_none()` would be host-dependent — which is why this test previously
    /// discarded its result and asserted nothing at all.
    #[test]
    #[serial_test::serial(cargo_target_dir_env)]
    fn test_get_profdata_mtime_fast_never_returns_missing_candidate() {
        let project = TempDir::new().unwrap();
        let empty_target = TempDir::new().unwrap();
        let _guard = CargoTargetDirGuard::set(&empty_target.path().to_string_lossy());

        let out = get_profdata_mtime_fast(project.path(), None);

        let never = empty_target.path().join("llvm-cov-target");
        assert!(
            !never.exists(),
            "precondition: the candidate must not exist on disk"
        );
        if let Some((_, dir)) = out {
            assert_ne!(
                std::path::Path::new(&dir),
                never,
                "returned a candidate directory that does not exist"
            );
        }
    }

    // --- full load_coverage_from_cache with mtime (most involved path) ---

    #[test]
    fn test_load_cache_with_matching_mtime_bypasses_git_hash() {
        let tmp = TempDir::new().unwrap();
        // Create a fake profdata dir
        let profdata = tmp.path().join("target").join("llvm-cov-target");
        std::fs::create_dir_all(&profdata).unwrap();
        let current_mtime = dir_mtime(&profdata).expect("mtime");
        // Build a cache whose stored mtime >= current mtime → cache valid.
        let cache = CoverageCache {
            git_hash: "wrong-hash".to_string(),
            coverage_mtime: Some(current_mtime),
            profdata_dir: Some(profdata.to_string_lossy().into_owned()),
            files: {
                let mut m = HashMap::new();
                m.insert("x.rs".to_string(), HashMap::new());
                m
            },
        };
        let cache_path = tmp.path().join("cache.json");
        write(&cache_path, &serde_json::to_string(&cache).unwrap());

        let loaded = load_coverage_from_cache(&cache_path, "any-hash", tmp.path());
        assert!(
            loaded.is_some(),
            "cached mtime should validate regardless of git hash"
        );
        assert!(loaded.unwrap().contains_key("x.rs"));
    }

    #[test]
    fn test_load_cache_stale_mtime_returns_none() {
        let tmp = TempDir::new().unwrap();
        let profdata = tmp.path().join("target").join("llvm-cov-target");
        std::fs::create_dir_all(&profdata).unwrap();
        // Sleep 1s so current mtime is strictly later than a fresh-captured cached mtime.
        // Actually the test goes the other way: we claim cached_mtime=0 (old) and
        // current is now() >> 0, so current > cached → INVALID.
        let cache = CoverageCache {
            git_hash: "h".to_string(),
            coverage_mtime: Some(0), // epoch
            profdata_dir: Some(profdata.to_string_lossy().into_owned()),
            files: HashMap::new(),
        };
        let cache_path = tmp.path().join("cache.json");
        write(&cache_path, &serde_json::to_string(&cache).unwrap());
        let loaded = load_coverage_from_cache(&cache_path, "h", tmp.path());
        assert!(loaded.is_none(), "stale mtime must invalidate cache");
    }
}
