// Cache operations for CargoDeadCodeAnalyzer
// Included from cargo_dead_code_analyzer.rs - shares parent module scope

impl CargoDeadCodeAnalyzer {
    /// Get the cache file path
    ///
    /// The key has to cover every input that changes the ANSWER, not just the
    /// tree hash and the pmat version: two runs that differ only in
    /// `--include-tests` (or the traversal depth) analyse different file sets,
    /// and they used to share one entry, so the second run replayed the first
    /// and the flag looked like a no-op even after the walk started honouring
    /// it.
    fn cache_path(&self) -> PathBuf {
        let included: String = [
            (self.exclude_tests, 't'),
            (self.exclude_examples, 'e'),
            (self.exclude_benches, 'b'),
        ]
        .iter()
        .filter(|(excluded, _)| !excluded)
        .map(|(_, tag)| *tag)
        .collect();
        let scope = if included.is_empty() {
            "default".to_string()
        } else {
            included
        };
        self.project_path.join(".pmat").join(format!(
            "dead-code-cache-{scope}-d{}.json",
            self.max_depth
        ))
    }

    /// The git tree hash of the WORKING tree, for cache invalidation.
    ///
    /// This was `git rev-parse HEAD:` — the tree of the last commit, which is
    /// byte-identical before and after any uncommitted edit, so a warm cache
    /// replayed "0 dead functions" over a dead function appended a second ago
    /// and, once the edit was reverted, "1 dead function" on line 9 of an
    /// 8-line file. The precedent is #748 in the hooks cache (`git write-tree`
    /// of the index); a pre-commit hook gates the INDEX, but this analyzer
    /// reads the checkout, so the index it hashes is a scratch one filled by
    /// `git add -A` — the tree a commit of everything would contain — and the
    /// user's own index is never touched. Falls back to `HEAD^{tree}` when no
    /// scratch index can be built (bare repo, unmerged state); `None` outside
    /// git, in which case nothing is cached.
    pub(crate) fn get_tree_hash(&self) -> Option<String> {
        working_tree_hash(&self.project_path)
    }

    /// Try to load cached result if valid
    fn try_load_cache(&self) -> Option<AccurateDeadCodeReport> {
        if !self.use_cache || self.force_refresh {
            return None;
        }

        let cache_path = self.cache_path();
        let cache_content = std::fs::read_to_string(&cache_path).ok()?;
        let cached: CachedDeadCodeResult = serde_json::from_str(&cache_content).ok()?;

        // Validate cache
        let current_tree_hash = self.get_tree_hash()?;
        let current_version = env!("CARGO_PKG_VERSION");

        // The SHAPE of the cached report is part of the key. Without it, a
        // cache written by an earlier build of the same version was accepted
        // whole, and a field added since (`unreachable_items`) came back empty.
        if cached.report_schema == DEAD_CODE_CACHE_SCHEMA
            && cached.tree_hash == current_tree_hash
            && cached.pmat_version == current_version
        {
            tracing::debug!("Dead code cache hit (tree_hash: {})", current_tree_hash);
            let mut report = cached.report;
            // A replay says so: the lint ran when the entry was written, not
            // now. A reduced scan stays reduced (its reason still names why).
            if let Some(scan) = report.compiler_scan.as_mut() {
                if scan.reason == crate::models::dead_code::COMPILER_SCAN_REASON_OK {
                    *scan = crate::models::dead_code::CompilerScanReport::cached(cached.timestamp);
                }
            }
            report.cache = Some(crate::models::dead_code::DeadCodeCacheReport {
                hit: true,
                tree_hash: cached.tree_hash,
                written_at: Some(cached.timestamp),
                pmat_version: cached.pmat_version,
            });
            Some(report)
        } else {
            tracing::debug!(
                "Dead code cache miss (tree: {} vs {}, version: {} vs {})",
                cached.tree_hash,
                current_tree_hash,
                cached.pmat_version,
                current_version
            );
            None
        }
    }
    /// Save result to cache; returns what was written so the report can say
    /// it (`cache.written_at`), or `None` when nothing was.
    fn save_cache(
        &self,
        report: &AccurateDeadCodeReport,
    ) -> Option<(String, chrono::DateTime<chrono::Utc>)> {
        if !self.use_cache {
            return None;
        }
        let tree_hash = self.get_tree_hash()?;
        let timestamp = chrono::Utc::now();
        let mut stored = report.clone();
        // A stored entry is never a hit; the reader marks it when it serves it.
        stored.cache = None;
        let cached = CachedDeadCodeResult {
            report_schema: DEAD_CODE_CACHE_SCHEMA,
            tree_hash: tree_hash.clone(),
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp,
            report: stored,
        };

        // Ensure .pmat exists AND ignores itself.
        //
        // Issue #1050 P8. This wrote `dead-code-cache-<scope>-d<n>.json` into
        // the analysed project and left it as `?? .pmat/` in that project's git
        // status. Projects were told to ignore `.pmat/dead-code-cache.json`;
        // the key above grew a suffix and that rule stopped matching, in
        // silence. The rule now ships with the directory.
        let _ = crate::utils::pmat_cache_dir::ensure_cache_dir(&self.project_path);

        // Write cache file
        let content = serde_json::to_string_pretty(&cached).ok()?;
        std::fs::write(self.cache_path(), content).ok()?;
        tracing::debug!("Dead code cache saved");
        Some((tree_hash, timestamp))
    }
}

/// The working tree's git tree hash: a scratch index filled from the checkout
/// (`git add -A`, tracked and untracked-but-not-ignored alike), then
/// `write-tree`. The scratch index lives under `.git/` and is removed after
/// use; the user's index is never read or written. Falls back to
/// `HEAD^{tree}` if the scratch index cannot be built, `None` outside git.
pub(crate) fn working_tree_hash(project_path: &Path) -> Option<String> {
    let git = |args: &[&str], index: Option<&Path>| -> Option<String> {
        let mut cmd = Command::new("git");
        cmd.current_dir(project_path).args(args);
        if let Some(index) = index {
            cmd.env("GIT_INDEX_FILE", index);
        }
        let out = cmd.output().ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    // Outside a git checkout there is nothing to key on.
    let scratch_rel = git(
        &["rev-parse", "--git-path", "pmat-dead-code-scratch-index"],
        None,
    )?;
    let scratch = {
        let p = Path::new(&scratch_rel);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            project_path.join(p)
        }
    };
    let _ = std::fs::remove_file(&scratch);
    let hash = git(&["add", "-A", "--", "."], Some(&scratch))
        .and_then(|_| git(&["write-tree"], Some(&scratch)));
    let _ = std::fs::remove_file(&scratch);
    hash.or_else(|| git(&["rev-parse", "HEAD^{tree}"], None))
}

#[cfg(test)]
mod cache_key_tests {
    use super::*;

    /// Toggling a flag that changes WHICH files are analysed must not hit the
    /// entry written by the other configuration — with a shared key, the second
    /// `analyze dead-code` run replayed the first and `--include-tests` looked
    /// like a no-op even on a correct walk.
    #[test]
    fn test_cache_key_separates_include_tests_from_the_default() {
        let root = std::path::Path::new("/p");
        let default_path = CargoDeadCodeAnalyzer::new(root).cache_path();
        let with_tests = CargoDeadCodeAnalyzer::new(root).include_tests().cache_path();

        // `include_examples()` no longer separates keys, because examples and
        // benches are in scope by default — it re-asserts the default rather
        // than widening the walk, so there is no second file set to key apart.
        assert_ne!(default_path, with_tests);
        assert!(default_path.starts_with("/p/.pmat"), "{default_path:?}");
    }

    /// Depth changes the walk, so it changes the answer too.
    #[test]
    fn test_cache_key_separates_traversal_depths() {
        let root = std::path::Path::new("/p");
        assert_ne!(
            CargoDeadCodeAnalyzer::new(root).with_max_depth(2).cache_path(),
            CargoDeadCodeAnalyzer::new(root).with_max_depth(8).cache_path()
        );
    }
}

#[cfg(test)]
mod working_tree_key_tests {
    use super::*;

    /// A throwaway checkout with one committed `src/lib.rs`.
    fn git_crate() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run = |args: &[&str]| {
            let out = Command::new("git")
                .current_dir(tmp.path())
                .args(args)
                .output()
                .expect("git");
            assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        run(&["config", "core.hooksPath", "/dev/null"]);
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"fx\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
        )
        .expect("manifest");
        std::fs::create_dir_all(tmp.path().join("src")).expect("src");
        std::fs::write(tmp.path().join("src/lib.rs"), "pub fn a() {}\n").expect("lib");
        run(&["add", "."]);
        run(&["commit", "-q", "-m", "one"]);
        tmp
    }

    fn staged_changes(dir: &Path) -> String {
        let out = Command::new("git")
            .current_dir(dir)
            .args(["diff", "--cached", "--stat"])
            .output()
            .expect("git");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// CRUX-04 states A→C→E: an UNCOMMITTED, UNSTAGED edit is a different key,
    /// and reverting it restores the old key. `git rev-parse HEAD:` gave the
    /// same key for all three.
    #[test]
    fn an_unstaged_edit_changes_the_key_and_a_revert_restores_it() {
        let tmp = git_crate();
        // Through the analyzer's own key function, so a key that reverted to
        // the committed tree fails HERE and not only in the replay test.
        let key = || CargoDeadCodeAnalyzer::new(tmp.path()).get_tree_hash().expect("hash");
        let clean = key();
        std::fs::write(tmp.path().join("src/lib.rs"), "pub fn a() {}\nfn dead() {}\n").expect("edit");
        let edited = key();
        assert_ne!(clean, edited, "an unstaged edit must change the cache key");
        assert_eq!(staged_changes(tmp.path()), "", "the user's index must not be touched");
        std::fs::write(tmp.path().join("src/lib.rs"), "pub fn a() {}\n").expect("revert");
        assert_eq!(key(), clean, "the same tree hashes the same");
        assert!(
            !tmp.path().join(".git/pmat-dead-code-scratch-index").exists(),
            "the scratch index is removed after use"
        );
    }

    /// Outside a git checkout there is no key, so nothing is cached — never a
    /// constant key that would serve one project's answers to another.
    #[test]
    fn outside_git_there_is_no_key() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // A temp dir may sit under a git checkout on some hosts; guard the
        // assumption before asserting on it.
        let inside_git = Command::new("git")
            .current_dir(tmp.path())
            .args(["rev-parse", "--git-dir"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if inside_git {
            return;
        }
        assert!(working_tree_hash(tmp.path()).is_none());
    }

    fn stored_entry(dir: &Path, schema: u32, scan_reason: &str) -> chrono::DateTime<chrono::Utc> {
        let analyzer = CargoDeadCodeAnalyzer::new(dir);
        let tree_hash = analyzer.get_tree_hash().expect("hash");
        let report = AccurateDeadCodeReport {
            compiler_scan: Some(crate::models::dead_code::CompilerScanReport {
                verdict: crate::models::dead_code::COMPILER_SCAN_FULL.to_string(),
                reason: scan_reason.to_string(),
                detail: "stored".to_string(),
            }),
            ..Default::default()
        };
        let timestamp = chrono::Utc::now();
        let cached = CachedDeadCodeResult {
            report_schema: schema,
            tree_hash,
            pmat_version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp,
            report,
        };
        let _ = crate::utils::pmat_cache_dir::ensure_cache_dir(dir);
        std::fs::write(analyzer.cache_path(), serde_json::to_string(&cached).expect("json")).expect("write");
        timestamp
    }

    /// CRUX-04 G: an entry written under the previous schema is keyed on a
    /// commit tree and must be a MISS after upgrade, or every developer's
    /// existing cache keeps serving pre-fix answers.
    #[test]
    fn an_old_schema_entry_is_a_miss() {
        let tmp = git_crate();
        stored_entry(tmp.path(), DEAD_CODE_CACHE_SCHEMA - 1, crate::models::dead_code::COMPILER_SCAN_REASON_OK);
        assert!(CargoDeadCodeAnalyzer::new(tmp.path()).try_load_cache().is_none());
    }

    /// CRUX-04 B/D: a replay says so — `cache.hit`, the entry's own timestamp,
    /// and the compiler-scan reason in the past tense — where it used to be
    /// byte-identical to a fresh compiler pass.
    #[test]
    fn a_replay_is_marked_as_a_hit_with_a_cached_verdict() {
        let tmp = git_crate();
        let written = stored_entry(tmp.path(), DEAD_CODE_CACHE_SCHEMA, crate::models::dead_code::COMPILER_SCAN_REASON_OK);
        let served = CargoDeadCodeAnalyzer::new(tmp.path())
            .try_load_cache()
            .expect("a current-schema entry keyed on this tree is served");
        let cache = served.cache.expect("cache object");
        assert!(cache.hit);
        assert_eq!(cache.written_at, Some(written));
        assert_eq!(
            served.compiler_scan.expect("scan").reason,
            crate::models::dead_code::COMPILER_SCAN_REASON_CACHED
        );
        // And an uncommitted edit after the entry was written is a MISS.
        std::fs::write(tmp.path().join("src/lib.rs"), "pub fn a() {}\nfn dead() {}\n").expect("edit");
        assert!(CargoDeadCodeAnalyzer::new(tmp.path()).try_load_cache().is_none());
    }

    /// A reduced scan replayed stays reduced: its reason still names why the
    /// compiler layer did not run, and only the cache object says it is a replay.
    #[test]
    fn a_replayed_reduced_scan_keeps_its_reason() {
        let tmp = git_crate();
        stored_entry(tmp.path(), DEAD_CODE_CACHE_SCHEMA, crate::models::dead_code::COMPILER_SCAN_REASON_LOCKFILE);
        let served = CargoDeadCodeAnalyzer::new(tmp.path()).try_load_cache().expect("served");
        assert!(served.cache.expect("cache").hit);
        assert_eq!(served.compiler_scan.expect("scan").reason, crate::models::dead_code::COMPILER_SCAN_REASON_LOCKFILE);
    }
}
