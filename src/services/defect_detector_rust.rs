/// Where a source path sits *inside its own project* — the one implementation
/// of the "is this support code rather than production code?" rule, shared by
/// [`RustDefectDetector::should_exclude_file`] and the SATD detector's
/// `should_exclude_file` / `is_test_file` / `is_minified_or_vendor_file`.
///
/// #923: the rule used to be written twice, and both copies matched substrings
/// (`"/tests/"`, `"/benches/"`, `"/examples/"`, `"/fuzz/"`, `"/demo/"`,
/// `"_demo"`, `"/vendor/"`, `"/book/"`) against the **absolute** path. So the
/// verdict was a property of where the checkout happened to sit: one
/// byte-identical crate, one md5, only the parent directory name differing —
///
/// ```text
/// <tmp>/normal            -> analyze defects: 1 critical, exit 1 | satd: 1 violation
/// <tmp>/tests/myproject   -> 0 critical,      exit 0             | 0 violations
/// <tmp>/examples/myproject-> 0 critical,      exit 0             | 0 violations
/// ```
///
/// Any CI runner, container image or monorepo whose checkout lives under a
/// segment with one of those names turned **both** gates permanently green.
///
/// The exclusions describe a package's own layout (`tests/`, `benches/`,
/// `examples/`, `fuzz/` are siblings of `src/`), so they are only meaningful
/// *below the project root*. Everything above it is the caller's filesystem and
/// says nothing about the code.
pub(crate) mod source_scope {
    use std::path::{Path, PathBuf};

    /// A checkout boundary. Checked first, because #923 is exactly a bug about
    /// directories ABOVE a checkout.
    const VCS_MARKERS: [&str; 3] = [".git", ".hg", ".svn"];

    /// Package manifests, for source trees that are not a repository — an
    /// unpacked tarball, a vendored copy, a scratch directory.
    const MANIFEST_MARKERS: [&str; 6] = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "go.mod",
        "pom.xml",
        "build.gradle",
    ];

    /// The directory `path`'s layout is measured against: the nearest ancestor
    /// that is a checkout root, else the nearest ancestor holding a package
    /// manifest.
    ///
    /// `None` for a relative path — it is already project-relative by
    /// construction at every call site, and probing the filesystem for one
    /// would make the answer depend on the process working directory, which is
    /// the same class of bug (#919).
    pub(crate) fn project_root_of(path: &Path) -> Option<PathBuf> {
        if !path.is_absolute() {
            return None;
        }

        let mut nearest_manifest: Option<PathBuf> = None;
        let mut cursor = path.parent();

        while let Some(dir) = cursor {
            if VCS_MARKERS.iter().any(|marker| dir.join(marker).exists()) {
                return Some(dir.to_path_buf());
            }
            if nearest_manifest.is_none()
                && MANIFEST_MARKERS
                    .iter()
                    .any(|marker| dir.join(marker).is_file())
            {
                nearest_manifest = Some(dir.to_path_buf());
            }
            cursor = dir.parent();
        }

        nearest_manifest
    }

    /// `path` with its project root stripped, as a `/`-separated string that
    /// always starts with `/`, so a caller can test a leading directory with
    /// the same `"/name/"` shape it uses for an interior one.
    pub(crate) fn project_relative_str(path: &Path) -> String {
        let relative = match project_root_of(path) {
            Some(root) => path.strip_prefix(&root).unwrap_or(path),
            None => path,
        };
        let text = relative.to_string_lossy().replace('\\', "/");
        let text = text.trim_start_matches("./").trim_start_matches('/');
        format!("/{text}")
    }

    /// True when the project-relative string produced by
    /// [`project_relative_str`] has one of `names` as a **directory**
    /// component. The final segment is the file name and is never a directory,
    /// so `examples.rs` is not `examples/`.
    pub(crate) fn has_dir_component(relative_str: &str, names: &[&str]) -> bool {
        let mut components = relative_str.split('/').collect::<Vec<_>>();
        components.pop(); // file name
        components.iter().any(|component| names.contains(component))
    }

    /// The directories a package uses for code that is not shipped: test
    /// harnesses, benchmarks, examples and fuzz targets. Shared so the rule is
    /// read in one place; the SATD detector names its own (wider) set beside
    /// this one in `detection_file_discovery.rs`.
    pub(crate) const NON_PRODUCTION_DIRS: [&str; 4] = ["tests", "benches", "examples", "fuzz"];
}

/// Test code is what the compiler compiles under `cfg(test)` — not what the
/// file happens to be called.
///
/// #927: exclusion recognised test code by filename only (`*_tests.rs`,
/// `*_test.rs`, `test_*`), and this repository's dominant test convention is
/// `include!()`-ed fragments with names that do not match
/// (`supervisor_tests_integration.rs`, `context_graph_coverage_fixtures.rs`,
/// `cli_tests/part2.rs`). 78 of the 80 Critical defects reported on pmat were
/// such fragments; the run exited 1, and the documented escape hatch
/// (`#![allow(clippy::unwrap_used)]`) cannot even be written in an `include!`-ed
/// fragment.
///
/// So resolve the graph instead of pattern-matching the leaf name: a file
/// pulled in by `include!`, `#[path] mod` or plain `mod` from inside a
/// `#[cfg(test)]` item is test code, transitively, whatever it is called.
pub(crate) mod include_scope {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex, OnceLock};

    /// Guard against `include!` cycles and pathological chains.
    const MAX_CHAIN: usize = 8;

    /// Directories that hold no crate source.
    const SKIPPED_DIRS: [&str; 5] = ["target", "node_modules", ".git", "book", "dist"];

    /// Every reference to a file: who pulls it in, and whether that reference
    /// is `cfg(test)`-gated.
    type ReferenceIndex = HashMap<PathBuf, Vec<(PathBuf, bool)>>;

    fn cache() -> &'static Mutex<HashMap<PathBuf, Arc<ReferenceIndex>>> {
        static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<ReferenceIndex>>>> = OnceLock::new();
        CACHE.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn resolved(path: &Path) -> Option<PathBuf> {
        std::fs::canonicalize(path).ok()
    }

    /// The tree the reference graph is built over: the file's own project, so
    /// a `#[path = "../cli_tests.rs"]` in a sibling directory is found as
    /// readily as an `include!` in the same one. Falls back to the file's own
    /// directory when there is no project around it.
    fn index_root(target: &Path) -> Option<PathBuf> {
        super::source_scope::project_root_of(target)
            .or_else(|| target.parent().map(Path::to_path_buf))
    }

    fn index_for(root: &Path) -> Arc<ReferenceIndex> {
        if let Some(hit) = cache().lock().ok().and_then(|c| c.get(root).cloned()) {
            return hit;
        }
        let index = Arc::new(build_index(root));
        if let Ok(mut c) = cache().lock() {
            c.insert(root.to_path_buf(), Arc::clone(&index));
        }
        index
    }

    fn build_index(root: &Path) -> ReferenceIndex {
        let mut index = ReferenceIndex::new();
        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_entry(|e| !is_skipped(e))
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let (Ok(content), Some(includer)) = (std::fs::read_to_string(path), resolved(path))
            else {
                continue;
            };
            let Some(dir) = path.parent() else { continue };
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            for (target, is_test) in references_of(&content, dir, stem) {
                index
                    .entry(target)
                    .or_default()
                    .push((includer.clone(), is_test));
            }
        }
        index
    }

    fn is_skipped(entry: &walkdir::DirEntry) -> bool {
        entry.depth() > 0
            && entry.file_type().is_dir()
            && entry
                .file_name()
                .to_str()
                .is_some_and(|name| SKIPPED_DIRS.contains(&name) || name.starts_with('.'))
    }

    /// Every file `content` pulls in, and whether it does so under `cfg(test)`.
    ///
    /// `dir` is the directory holding the file and `stem` its name without the
    /// extension: a `mod` declared in `foo.rs` resolves against `foo/` in the
    /// 2018 module system, and against `dir/` when the file is a `mod.rs`. Both
    /// are tried — a candidate that does not exist on disk is dropped, and an
    /// extra reference can only make a file look MORE like production code.
    fn references_of(content: &str, dir: &Path, stem: &str) -> Vec<(PathBuf, bool)> {
        let mut references = Vec::new();
        let mut depth: i32 = 0;
        let mut test_regions: Vec<i32> = Vec::new();
        let mut pending_cfg_test = false;
        let mut pending_path: Option<String> = None;

        for line in content.lines() {
            let trimmed = line.trim();
            let is_test_here = !test_regions.is_empty() || pending_cfg_test;

            if let Some(target) = include_target(trimmed) {
                // `include!` is always relative to the including file's own
                // directory.
                if let Some(path) = resolved(&dir.join(target)) {
                    references.push((path, is_test_here));
                }
            } else if let Some(name) = module_declaration(trimmed) {
                for candidate in module_candidates(dir, stem, name, pending_path.take()) {
                    if let Some(path) = resolved(&candidate) {
                        references.push((path, is_test_here));
                    }
                }
            }

            if trimmed.starts_with("#[") {
                if is_cfg_test_attr(trimmed) {
                    pending_cfg_test = true;
                }
                if let Some(explicit) = attribute_path(trimmed) {
                    pending_path = Some(explicit);
                }
            }

            let opens = line.matches('{').count() as i32;
            let closes = line.matches('}').count() as i32;
            if opens > 0 {
                if pending_cfg_test {
                    test_regions.push(depth);
                    pending_cfg_test = false;
                }
                depth += opens;
            }
            if closes > 0 {
                depth -= closes;
                while test_regions.last().is_some_and(|entry| depth <= *entry) {
                    test_regions.pop();
                }
            }
            // An attribute applies to the next item only.
            if !trimmed.is_empty() && !trimmed.starts_with("#[") && !trimmed.starts_with("//") {
                pending_cfg_test = false;
                pending_path = None;
            }
        }

        references
    }

    fn module_candidates(
        dir: &Path,
        stem: &str,
        name: &str,
        explicit: Option<String>,
    ) -> Vec<PathBuf> {
        let bases = [dir.to_path_buf(), dir.join(stem)];
        match explicit {
            Some(explicit) => bases.iter().map(|base| base.join(&explicit)).collect(),
            None => bases
                .iter()
                .flat_map(|base| {
                    [
                        base.join(format!("{name}.rs")),
                        base.join(name).join("mod.rs"),
                    ]
                })
                .collect(),
        }
    }

    fn is_cfg_test_attr(trimmed: &str) -> bool {
        trimmed.starts_with("#[cfg(")
            && !trimmed.contains("not(test")
            && (trimmed.contains("(test)") || trimmed.contains("(test,"))
    }

    fn attribute_path(trimmed: &str) -> Option<String> {
        if trimmed.starts_with("#[path") {
            quoted(trimmed)
        } else {
            None
        }
    }

    fn include_target(trimmed: &str) -> Option<String> {
        if trimmed.contains("include!(") {
            quoted(trimmed)
        } else {
            None
        }
    }

    /// `mod name;` / `pub mod name;` — a declaration, not an inline module.
    fn module_declaration(trimmed: &str) -> Option<&str> {
        let rest = trimmed
            .strip_prefix("pub(crate) mod ")
            .or_else(|| trimmed.strip_prefix("pub(super) mod "))
            .or_else(|| trimmed.strip_prefix("pub mod "))
            .or_else(|| trimmed.strip_prefix("mod "))?;
        let name = rest.strip_suffix(';')?.trim();
        (!name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')).then_some(name)
    }

    fn quoted(line: &str) -> Option<String> {
        let start = line.find('"')? + 1;
        let end = line[start..].find('"')? + start;
        Some(line[start..end].to_string())
    }

    /// Is `file` compiled only under `cfg(test)`?
    ///
    /// A relative path is resolved the same way the caller resolved it to READ
    /// the file (`analyze defects` walks `.` and hands out `./src/…`), so the
    /// answer follows the bytes that were scanned. A path that resolves to
    /// nothing is not test code — "I could not look" never becomes "excluded".
    pub(crate) fn is_test_only(file: &Path) -> bool {
        match resolved(file) {
            Some(target) => reached_from_test(&target, 0),
            None => false,
        }
    }

    /// The filename half of the exclusion rule, in one place so the graph walk
    /// below applies the SAME test to an includer that `should_exclude_file`
    /// applies to the file itself.
    pub(crate) fn has_test_file_name(file_name: &str) -> bool {
        file_name.ends_with("_tests.rs")
            || file_name.ends_with("_test.rs")
            || file_name.starts_with("test_")
    }

    fn is_test_code(path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(has_test_file_name)
    }

    /// "Only under cfg(test)": every reference to the file must be gated, come
    /// from a file that is itself test code, or come from a file that is only
    /// compiled under `cfg(test)`. A fragment that is ALSO pulled into
    /// production code is production code, and a file nothing references at all
    /// is production code — an empty answer is never an exclusion.
    fn reached_from_test(target: &Path, chain: usize) -> bool {
        if chain >= MAX_CHAIN {
            return false;
        }
        let references = references_to(target);
        !references.is_empty()
            && references.iter().all(|(includer, is_test)| {
                *is_test
                    || is_test_code(includer)
                    || (includer != target && reached_from_test(includer, chain + 1))
            })
    }

    /// Files that pull `target` in, with each reference's `cfg(test)` status.
    pub(crate) fn references_to(target: &Path) -> Vec<(PathBuf, bool)> {
        let Some(target) = resolved(target) else {
            return Vec::new();
        };
        let Some(root) = index_root(&target) else {
            return Vec::new();
        };
        index_for(&root)
            .get(&target)
            .cloned()
            .unwrap_or_else(Vec::new)
    }
}

impl RustDefectDetector {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            unwrap_regex: Regex::new(r"\.unwrap\(\)").expect("internal error"),
        }
    }

    /// Check if a file should be excluded from defect detection.
    ///
    /// The directory test runs against the path RELATIVE TO ITS OWN PROJECT
    /// ROOT (see [`source_scope`]). It used to run against the absolute path,
    /// which made the verdict a property of where the checkout sat: a crate
    /// under `<tmp>/tests/myproject` reported 0 critical defects and exit 0
    /// while the byte-identical crate under `<tmp>/normal` reported 1 and
    /// exit 1 (#923).
    pub(crate) fn should_exclude_file(&self, file_path: &Path) -> bool {
        let path_str = source_scope::project_relative_str(file_path);
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Exclude a package's own tests/, benches/, examples/ and fuzz/ trees.
        // (Examples and fuzz targets legitimately use .unwrap() for brevity.)
        if source_scope::has_dir_component(&path_str, &source_scope::NON_PRODUCTION_DIRS) {
            return true;
        }

        // Exclude test file patterns
        if include_scope::has_test_file_name(file_name) {
            return true;
        }

        // #927: and the ones the naming convention misses — a fragment is test
        // code when the item that pulls it in is `#[cfg(test)]`, whatever the
        // fragment is called.
        include_scope::is_test_only(file_path)
    }

    /// The documented per-file escape hatch is an INNER attribute
    /// (`#![allow(clippy::unwrap_used)]`), which an `include!`-ed fragment
    /// cannot carry — `check_codegen_pub_fn_coverage.rs` says so in its own
    /// header: *"Included from check_codegen.rs — do NOT add `use` imports or
    /// `#!` attributes here."* So the fragment's findings were unsuppressable
    /// (#927). Honour the attribute where it CAN be written: on the includer.
    fn includer_allows_unwrap(file_path: &Path) -> bool {
        if !file_path.is_absolute() {
            return false;
        }
        include_scope::references_to(file_path)
            .iter()
            .filter_map(|(includer, _)| std::fs::read_to_string(includer).ok())
            .any(|content| file_allows_unwrap(&content))
    }

    /// True if a single trimmed line is a test-function attribute (`#[test]`,
    /// `#[tokio::test]`, …). The item it precedes is test code even when it is
    /// not wrapped in a `#[cfg(test)]` module.
    fn is_test_attr_line(trimmed: &str) -> bool {
        trimmed.starts_with("#[test]")
            || trimmed.starts_with("#[tokio::test")
            || trimmed.starts_with("#[async_test")
            || trimmed.starts_with("#[async_std::test")
    }

    /// Detect all defects in Rust source code
    /// Returns vector of detected defect patterns with instances
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn detect(&self, content: &str, file_path: &Path) -> Vec<DefectPattern> {
        let mut defects = Vec::new();

        // Exclude test files entirely
        if self.should_exclude_file(file_path) {
            return defects;
        }

        // NOTE: there used to be a whole-file bail here —
        // `if self.has_test_markers(content) { return defects; }` — where
        // has_test_markers matched the bare substrings "#[cfg(test)]" / "#[test]"
        // ANYWHERE in the file. Rust's idiomatic co-located unit-test module
        // therefore blanked out every line of production code above it: a crate
        // whose src/lib.rs was one `.unwrap()`-bearing function reported 1 defect,
        // and reported 0 the moment an unrelated `#[cfg(test)] mod tests` was
        // appended to the same file. Scanning this repo it left 44 defects in 5
        // files against ~19,500 `.unwrap()` calls in ~1,300 files.
        //
        // Exclusion is now per-range, not per-file: detect_unwraps already skips
        // the body of any `#[cfg(...)]` item by brace depth (which covers
        // `#[cfg(test)] mod tests`), and additionally skips `#[test]`-attributed
        // items, so test code is dropped while the production code around it is
        // still reported.

        // Detect .unwrap() calls
        let unwrap_instances = self.detect_unwraps(content, file_path);
        if !unwrap_instances.is_empty() {
            defects.push(DefectPattern {
                id: "RUST-UNWRAP-001".to_string(),
                name: ".unwrap() calls".to_string(),
                severity: Severity::Critical,
                fix_recommendation:
                    "Use .expect() with descriptive messages or proper error handling with ?"
                        .to_string(),
                bad_example: "let x = result.unwrap();".to_string(),
                good_example: "let x = result.expect(\"Bot feature file must be valid\");"
                    .to_string(),
                evidence_description: "Cloudflare outage 2025-11-18 (3+ hour network outage)"
                    .to_string(),
                // The slug used to say 2025-01-18 while the description said
                // 2025-11-18: a citation for an outage that never happened, and
                // the URL 404ed. The post about the November 18 outage — the one
                // an `.unwrap()` on a bot-features file caused — is this one.
                evidence_url: Some(
                    "https://blog.cloudflare.com/18-november-2025-outage/".to_string(),
                ),
                instances: unwrap_instances,
            });
        }

        defects
    }

    fn detect_unwraps(&self, content: &str, file_path: &Path) -> Vec<DefectInstance> {
        let mut instances = Vec::new();

        // Honor a file-level (inner) suppression attribute. A developer who has
        // written `#![allow(clippy::unwrap_used)]` (or the broader
        // `#![allow(clippy::restriction)]`, the lint group that contains
        // `unwrap_used`) has explicitly opted these unwraps out of the lint that
        // owns this policy. Auto-failing such a file would be a false positive,
        // so mirror clippy's suppression semantics and skip the whole file.
        if file_allows_unwrap(content) || Self::includer_allows_unwrap(file_path) {
            return instances;
        }

        // Track #[cfg(...)] blocks via brace depth so we can skip .unwrap()
        // inside conditional compilation code (issue #279). The same
        // brace-depth machinery is reused to honor item-level (outer)
        // `#[allow(clippy::unwrap_used)]` attributes.
        let mut brace_depth: i32 = 0;
        let mut suppress_entry_depth: Option<i32> = None; // depth when #[cfg]/#[allow] was seen
        let mut pending_suppress = false;
        let mut in_block_comment = false;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Track block comments (simplified — no nesting)
            if in_block_comment {
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }
            if trimmed.starts_with("/*") {
                in_block_comment = !trimmed.contains("*/");
                continue;
            }

            // Skip doc comments and line comments
            if trimmed.starts_with("///")
                || trimmed.starts_with("//!")
                || trimmed.starts_with("//")
            {
                continue;
            }

            // Detect #[cfg(...)] attributes — marks the next braced item as cfg-gated.
            // Also honor item-level (outer) #[allow(clippy::unwrap_used)] / clippy::restriction.
            // `#[test]`-attributed items are skipped the same way, so a test
            // function that is not inside a `#[cfg(test)]` module is still
            // excluded without discarding the rest of the file.
            if trimmed.starts_with("#[cfg(")
                || trimmed.starts_with("#[cfg_attr(")
                || attr_line_allows_unwrap(trimmed)
                || Self::is_test_attr_line(trimmed)
            {
                pending_suppress = true;
            }

            // Track brace depth and suppressed-block boundaries
            for ch in line.chars() {
                if ch == '{' {
                    if pending_suppress && suppress_entry_depth.is_none() {
                        suppress_entry_depth = Some(brace_depth);
                        pending_suppress = false;
                    }
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if let Some(entry) = suppress_entry_depth {
                        if brace_depth <= entry {
                            suppress_entry_depth = None;
                        }
                    }
                }
            }

            // Skip .unwrap() detection inside #[cfg] blocks (conditional
            // compilation, e.g. GPU init / platform-specific code) or inside
            // an item explicitly annotated with #[allow(clippy::unwrap_used)].
            if suppress_entry_depth.is_some() {
                continue;
            }

            // Strip string literal contents to avoid false positives on
            // documentation strings like: "Detects .unwrap() panics"
            let code_only = strip_string_literals(line);

            for mat in self.unwrap_regex.find_iter(&code_only) {
                instances.push(DefectInstance {
                    file: file_path.to_string_lossy().to_string(),
                    line: line_num + 1,
                    column: mat.start() + 1,
                    code_snippet: line.trim().to_string(),
                });
            }
        }

        instances
    }

    /// Count unwrap() calls (used by rust-project-score)
    /// Skips comments and string literal contents to avoid false positives.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn count_unwraps(&self, content: &str) -> usize {
        content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with("//") && !trimmed.starts_with("/*") && !trimmed.is_empty()
            })
            .map(|line| {
                let code = strip_string_literals(line);
                self.unwrap_regex.find_iter(&code).count()
            })
            .sum()
    }
}

impl Default for RustDefectDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// True if `attrs` (a clippy lint list, e.g. `clippy::unwrap_used, clippy::all`)
/// suppresses the `unwrap_used` lint. Honors both the specific lint and the
/// `clippy::restriction` group that contains it. Note: `clippy::all` does NOT
/// include `unwrap_used` (it lives in the `restriction` group), so it is
/// intentionally not treated as a suppression here.
fn allow_list_suppresses_unwrap(attrs: &str) -> bool {
    attrs.contains("clippy::unwrap_used") || attrs.contains("clippy::restriction")
}

/// True if the file carries a module/crate-level (inner) attribute
/// `#![allow(clippy::unwrap_used)]` / `#![allow(clippy::restriction)]`,
/// which suppresses the unwrap lint for the entire file.
fn file_allows_unwrap(content: &str) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        // Stop scanning once we reach real code: inner attributes must appear
        // before the first item. Bail out on the first non-attribute,
        // non-comment, non-blank line for a cheap early exit.
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            continue;
        }
        if trimmed.starts_with("#![allow(") {
            if allow_list_suppresses_unwrap(trimmed) {
                return true;
            }
            continue;
        }
        if trimmed.starts_with("#!") || trimmed.starts_with("#[") {
            // Other inner/outer attributes (e.g. #![feature], #![deny]) — keep scanning.
            continue;
        }
        // First line of real code reached without finding the allow — done.
        break;
    }
    false
}

/// True if a single trimmed line is an item-level (outer) attribute
/// `#[allow(clippy::unwrap_used)]` / `#[allow(clippy::restriction)]` that
/// suppresses the unwrap lint for the item it precedes.
fn attr_line_allows_unwrap(trimmed: &str) -> bool {
    trimmed.starts_with("#[allow(") && allow_list_suppresses_unwrap(trimmed)
}

/// Strip contents of string literals to prevent false-positive defect detection.
/// Replaces `"..."` contents with spaces (preserving column offsets).
fn strip_string_literals(line: &str) -> String {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut out = Vec::with_capacity(len);
    let mut i = 0;

    while i < len {
        if bytes[i] == b'"' {
            out.push(b'"');
            i += 1;
            while i < len && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < len {
                    out.push(b' ');
                    out.push(b' ');
                    i += 2;
                } else {
                    out.push(b' ');
                    i += 1;
                }
            }
            if i < len {
                out.push(b'"');
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }

    String::from_utf8(out).unwrap_or_else(|_| line.to_string())
}

/// Regression tests for the whole-file test-marker bail.
///
/// `detect` used to return an empty vec for any file containing the substring
/// "#[cfg(test)]" or "#[test]", so a co-located unit-test module — the Rust
/// idiom — hid every defect in the production code above it.
#[cfg(test)]
mod colocated_test_module_regression_tests {
    use super::*;

    const PRODUCTION_THEN_TEST_MODULE: &str = "pub fn dangerous(v: &[i32]) -> i32 {\n    \
         *v.first().unwrap()\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        \
         assert_eq!(1, 1);\n    }\n}\n";

    #[test]
    fn production_unwrap_survives_a_colocated_test_module() {
        let detector = RustDefectDetector::new();
        let defects = detector.detect(PRODUCTION_THEN_TEST_MODULE, Path::new("src/lib.rs"));

        assert_eq!(
            defects.len(),
            1,
            "the production .unwrap() was discarded because the file also contains a test module"
        );
        assert_eq!(defects[0].instances.len(), 1);
        assert_eq!(defects[0].instances[0].line, 2);
    }

    #[test]
    fn unwrap_inside_the_colocated_test_module_is_still_excluded() {
        let detector = RustDefectDetector::new();
        let code = "pub fn safe(v: &[i32]) -> i32 {\n    v.len() as i32\n}\n\n#[cfg(test)]\n\
                    mod tests {\n    #[test]\n    fn t() {\n        \
                    let _ = Some(1).unwrap();\n    }\n}\n";
        let defects = detector.detect(code, Path::new("src/lib.rs"));

        assert!(
            defects.is_empty(),
            "test-module .unwrap() must stay excluded: {defects:?}"
        );
    }

    /// The citation used to point at `/2025-01-18-outage` while claiming the
    /// outage of 2025-11-18: two different dates in one piece of evidence, and
    /// the URL 404ed. The date in the slug must match the date in the prose.
    #[test]
    fn unwrap_evidence_url_matches_its_own_description() {
        let detector = RustDefectDetector::new();
        let defects = detector.detect(PRODUCTION_THEN_TEST_MODULE, Path::new("src/lib.rs"));
        let unwrap = defects
            .iter()
            .find(|d| d.id == "RUST-UNWRAP-001")
            .expect("RUST-UNWRAP-001 must be reported");
        let url = unwrap
            .evidence_url
            .as_deref()
            .expect("RUST-UNWRAP-001 cites a URL");

        assert!(
            unwrap.evidence_description.contains("2025-11-18"),
            "description names the November 2025 outage: {}",
            unwrap.evidence_description
        );
        assert!(
            url.contains("18-november-2025"),
            "the cited URL must be the November 2025 outage post, not {url}"
        );
    }

    /// #923: the exclusion rule matched `"/tests/"`, `"/benches/"`,
    /// `"/examples/"` and `"/fuzz/"` against the ABSOLUTE path, so a crate that
    /// merely SAT under a directory with one of those names — any CI runner,
    /// container image or monorepo checkout — excluded every one of its files
    /// and reported 0 critical defects with exit 0.
    ///
    /// Byte-identical crates; only the name of a directory ABOVE the crate
    /// differs. The verdict must not.
    #[test]
    fn a_crate_is_graded_the_same_wherever_the_checkout_sits() {
        const SRC: &str = "pub fn dangerous(v: &[i32]) -> i32 {\n    *v.first().unwrap()\n}\n";

        let tmp = tempfile::TempDir::new().expect("tempdir");
        let detector = RustDefectDetector::new();
        let mut verdicts = Vec::new();

        for parent in [
            "normal",
            "tests/myproject",
            "benches/myproject",
            "examples/myproject",
            "fuzz/myproject",
        ] {
            let crate_root = tmp.path().join(parent);
            std::fs::create_dir_all(crate_root.join("src")).expect("src dir");
            std::fs::write(
                crate_root.join("Cargo.toml"),
                "[package]\nname = \"myproject\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            )
            .expect("manifest");
            let lib = crate_root.join("src/lib.rs");
            std::fs::write(&lib, SRC).expect("lib.rs");

            let critical: usize = detector
                .detect(SRC, &lib)
                .iter()
                .filter(|d| d.severity == Severity::Critical)
                .map(|d| d.instances.len())
                .sum();
            verdicts.push((parent, critical));
        }

        assert!(
            verdicts.iter().all(|(_, count)| *count == 1),
            "one crate, one md5 — the parent directory's name changed the verdict: {verdicts:?}"
        );
    }

    /// The guard rail for the fix above: a package's OWN `tests/` tree is still
    /// test code. The rule moved from the absolute path to the project-relative
    /// one; it did not go away.
    #[test]
    fn a_packages_own_support_directories_are_still_excluded() {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        let crate_root = tmp.path().join("myproject");
        std::fs::create_dir_all(&crate_root).expect("crate dir");
        std::fs::write(
            crate_root.join("Cargo.toml"),
            "[package]\nname = \"myproject\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("manifest");

        let detector = RustDefectDetector::new();
        for support in ["tests", "benches", "examples", "fuzz"] {
            let dir = crate_root.join(support);
            std::fs::create_dir_all(&dir).expect("support dir");
            let file = dir.join("it.rs");
            std::fs::write(&file, "fn f() { let _ = Some(1).unwrap(); }\n").expect("file");
            assert!(
                detector.should_exclude_file(&file),
                "{support}/ inside the package is support code: {}",
                file.display()
            );
        }

        let src = crate_root.join("src");
        std::fs::create_dir_all(&src).expect("src dir");
        let lib = src.join("lib.rs");
        std::fs::write(&lib, "fn f() { let _ = Some(1).unwrap(); }\n").expect("file");
        assert!(
            !detector.should_exclude_file(&lib),
            "src/lib.rs is production code"
        );
    }

    #[test]
    fn bare_test_function_is_excluded_without_hiding_the_file() {
        let detector = RustDefectDetector::new();
        let code = "pub fn dangerous(v: &[i32]) -> i32 {\n    *v.first().unwrap()\n}\n\n\
                    #[test]\nfn stray() {\n    let _ = Some(2).unwrap();\n}\n";
        let defects = detector.detect(code, Path::new("src/lib.rs"));

        assert_eq!(defects.len(), 1);
        assert_eq!(
            defects[0].instances.len(),
            1,
            "only the production .unwrap() should be reported: {:?}",
            defects[0].instances
        );
        assert_eq!(defects[0].instances[0].line, 2);
    }
}

/// #927 — exclusion recognised test code by FILENAME, and this repository's
/// dominant test convention is `include!()`-ed fragments whose names do not
/// match the pattern. 78 of the 80 Critical defects `analyze defects` reported
/// on pmat were such fragments, the run exited 1, and the documented
/// `#![allow(clippy::unwrap_used)]` escape hatch cannot be written inside an
/// `include!`-ed fragment at all.
#[cfg(test)]
mod include_graph_regression_tests {
    use super::*;

    const MANIFEST: &str = "[package]\nname = \"frag\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

    /// A test helper's `.unwrap()`, in a file called nothing like a test.
    const FRAGMENT: &str = "fn helper() -> i32 {\n    let v: Option<i32> = Some(1);\n    \
         v.unwrap()\n}\n\n#[test]\nfn t_add() {\n    assert_eq!(add(1, 2), helper() + 2);\n}\n";

    fn crate_with(files: &[(&str, &str)]) -> tempfile::TempDir {
        let tmp = tempfile::TempDir::new().expect("tempdir");
        std::fs::write(tmp.path().join("Cargo.toml"), MANIFEST).expect("manifest");
        for (name, content) in files {
            let path = tmp.path().join(name);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("dir");
            std::fs::write(&path, content).expect("file");
        }
        tmp
    }

    fn critical_instances(root: &Path, file: &str) -> usize {
        let path = root.join(file);
        let content = std::fs::read_to_string(&path).expect("fragment");
        RustDefectDetector::new()
            .detect(&content, &path)
            .iter()
            .filter(|d| d.severity == Severity::Critical)
            .map(|d| d.instances.len())
            .sum()
    }

    #[test]
    fn a_fragment_included_from_a_cfg_test_module_is_test_code() {
        let tmp = crate_with(&[
            (
                "src/lib.rs",
                "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\n#[cfg(test)]\nmod tests {\n    \
                 use super::*;\n    include!(\"lib_helpers_part1.rs\");\n}\n",
            ),
            ("src/lib_helpers_part1.rs", FRAGMENT),
        ]);

        assert_eq!(
            critical_instances(tmp.path(), "src/lib_helpers_part1.rs"),
            0,
            "a fragment included from `#[cfg(test)] mod tests` is test code"
        );
    }

    #[test]
    fn a_fragment_reached_through_a_chain_of_includes_is_test_code() {
        let tmp = crate_with(&[
            (
                "src/executor.rs",
                "pub fn run() {}\n\n#[cfg(test)]\n#[path = \"executor_tests.rs\"]\nmod tests;\n",
            ),
            (
                "src/executor_tests.rs",
                "include!(\"parts/state_action.rs\");\n",
            ),
            ("src/parts/state_action.rs", FRAGMENT),
        ]);

        assert_eq!(
            critical_instances(tmp.path(), "src/parts/state_action.rs"),
            0,
            "the include chain ends in `#[cfg(test)] #[path] mod`, so this is test code"
        );
    }

    /// The guard rail, and the direction #752 broke in: a fragment pulled into
    /// PRODUCTION code is production code, whatever it is called.
    #[test]
    fn a_fragment_included_from_production_code_is_still_reported() {
        let tmp = crate_with(&[
            (
                "src/lib.rs",
                "pub fn add(a: i32, b: i32) -> i32 { a + b }\ninclude!(\"lib_helpers_part1.rs\");\n",
            ),
            (
                "src/lib_helpers_part1.rs",
                "pub fn helper() -> i32 {\n    let v: Option<i32> = Some(1);\n    v.unwrap()\n}\n",
            ),
        ]);

        assert_eq!(
            critical_instances(tmp.path(), "src/lib_helpers_part1.rs"),
            1,
            "production code was excluded because it lives in a fragment"
        );
    }

    /// A file no one includes is production code — the resolver must not treat
    /// "I found nothing" as "it is test code".
    #[test]
    fn an_unreferenced_file_is_not_test_code() {
        let tmp = crate_with(&[(
            "src/lib.rs",
            "pub fn f(v: &[i32]) -> i32 {\n    *v.first().unwrap()\n}\n",
        )]);

        assert_eq!(critical_instances(tmp.path(), "src/lib.rs"), 1);
    }

    /// The unsuppressable-finding half of #927: an `include!`-ed fragment
    /// cannot carry `#![allow(clippy::unwrap_used)]`, so the attribute is
    /// honoured where it CAN be written — on the includer.
    #[test]
    fn the_includers_allow_attribute_suppresses_a_fragment() {
        let tmp = crate_with(&[
            (
                "src/lib.rs",
                "#![allow(clippy::unwrap_used)]\npub fn add(a: i32, b: i32) -> i32 { a + b }\n\
                 include!(\"lib_helpers_part1.rs\");\n",
            ),
            (
                "src/lib_helpers_part1.rs",
                "pub fn helper() -> i32 {\n    let v: Option<i32> = Some(1);\n    v.unwrap()\n}\n",
            ),
        ]);

        assert_eq!(
            critical_instances(tmp.path(), "src/lib_helpers_part1.rs"),
            0,
            "the includer's #![allow(clippy::unwrap_used)] must reach the fragment"
        );
    }

    /// `#[cfg(not(test))]` is production configuration, not test code.
    #[test]
    fn cfg_not_test_is_not_test_code() {
        let tmp = crate_with(&[
            (
                "src/lib.rs",
                "#[cfg(not(test))]\nmod real;\npub fn f() {}\n",
            ),
            (
                "src/real.rs",
                "pub fn helper() -> i32 {\n    let v: Option<i32> = Some(1);\n    v.unwrap()\n}\n",
            ),
        ]);

        assert_eq!(critical_instances(tmp.path(), "src/real.rs"), 1);
    }
}
