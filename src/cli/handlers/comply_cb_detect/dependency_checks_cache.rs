// dependency_checks_cache.rs — included by dependency_checks.rs
// DependencyCache impl, Cargo.lock parsing, transitive dependency counting

impl DependencyCache {
    /// Check if cache is valid (Cargo.lock unchanged)
    fn is_valid(&self, cargo_lock_path: &Path) -> bool {
        debug_assert!(cargo_lock_path.exists(), "cargo_lock_path must exist: {}", cargo_lock_path.display());
        if let Ok(metadata) = fs::metadata(cargo_lock_path) {
            if let Ok(modified) = metadata.modified() {
                let mtime = modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                return mtime == self.cargo_lock_mtime;
            }
        }
        false
    }

    /// Load cache from .pmat/deps-cache.json
    fn load(project_path: &Path) -> Option<Self> {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let cache_path = project_path.join(".pmat/deps-cache.json");
        fs::read_to_string(&cache_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Save cache to .pmat/deps-cache.json
    fn save(&self, project_path: &Path) {
        debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
        let cache_path = project_path.join(".pmat/deps-cache.json");
        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&cache_path, json);
        }
    }
}

/// Parse Cargo.lock once and return both transitive count and duplicates
pub(super) fn parse_cargo_lock(cargo_lock_path: &Path) -> (usize, Vec<DuplicateCrate>) {
    let content = match fs::read_to_string(cargo_lock_path) {
        Ok(c) => c,
        Err(_) => return (0, Vec::new()),
    };

    let mut crate_versions: HashMap<String, Vec<String>> = HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_version: Option<String> = None;
    let mut package_count = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[[package]]" {
            package_count += 1;
            // Save previous package if complete
            if let (Some(name), Some(version)) = (current_name.take(), current_version.take()) {
                crate_versions.entry(name).or_default().push(version);
            }
        } else if let Some(name) = trimmed.strip_prefix("name = \"") {
            current_name = name.strip_suffix('"').map(|s| s.to_string());
        } else if let Some(version) = trimmed.strip_prefix("version = \"") {
            current_version = version.strip_suffix('"').map(|s| s.to_string());
        }
    }

    // Don't forget the last package
    if let (Some(name), Some(version)) = (current_name, current_version) {
        crate_versions.entry(name).or_default().push(version);
    }

    // Filter to only duplicates (>1 version)
    let duplicates: Vec<DuplicateCrate> = crate_versions
        .into_iter()
        .filter(|(_, versions)| versions.len() > 1)
        .map(|(name, mut versions)| {
            versions.sort();
            versions.dedup();
            DuplicateCrate { name, versions }
        })
        .filter(|d| d.versions.len() > 1)
        .collect();

    (package_count, duplicates)
}

/// Count production-only transitive dependencies using `cargo tree -e no-dev`
/// Returns None if cargo tree is unavailable or fails
pub(super) fn count_production_transitive(project_path: &Path) -> Option<usize> {
    let output = std::process::Command::new("cargo")
        .args(["tree", "-e", "no-dev", "--prefix=none"])
        .current_dir(project_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut unique_packages: HashSet<String> = HashSet::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            // cargo tree --prefix=none outputs "crate_name v1.2.3"
            if let Some(name) = trimmed.split_whitespace().next() {
                unique_packages.insert(name.to_string());
            }
        }
    }

    Some(unique_packages.len())
}

/// Get dependency analysis with O(1) caching (issue #148 fix)
pub(super) fn get_cached_dependency_analysis(
    project_path: &Path,
    cargo_lock_path: &Path,
) -> (usize, Option<usize>, Vec<DuplicateCrate>) {
    // Try to use cached results first
    if let Some(cache) = DependencyCache::load(project_path) {
        if cache.is_valid(cargo_lock_path) {
            return (
                cache.transitive_count,
                cache.prod_transitive_count,
                cache.duplicate_crates,
            );
        }
    }

    // Cache miss or invalid - parse Cargo.lock
    let (transitive_count, duplicate_crates) = parse_cargo_lock(cargo_lock_path);

    // Get production-only count via cargo tree
    let prod_transitive_count = count_production_transitive(project_path);

    // Save to cache
    let mtime = fs::metadata(cargo_lock_path)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        })
        .unwrap_or(0);

    let cache = DependencyCache {
        cargo_lock_mtime: mtime,
        transitive_count,
        prod_transitive_count,
        duplicate_crates: duplicate_crates.clone(),
    };
    cache.save(project_path);

    (transitive_count, prod_transitive_count, duplicate_crates)
}
