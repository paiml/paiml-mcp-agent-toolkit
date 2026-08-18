// Core analysis methods for CargoDeadCodeAnalyzer
// Included from cargo_dead_code_analyzer.rs - shares parent module scope

impl CargoDeadCodeAnalyzer {
    /// Perform accurate dead code analysis using cargo with O(1) caching
    ///
    /// Uses a four-layer detection strategy:
    /// 1. SUPPRESSION_SCAN: Detect  attributes (explicit admissions)
    /// 2. COMPILER_LINT: Run cargo check with -W dead_code
    /// 3. REFERENCE_GRAPH: (future) Build call graph for unreachable code
    /// 4. HEURISTICS: (future) Pattern-based detection
    ///
    /// # Timeout
    ///
    /// The budget (`with_timeout`, default 90s) is enforced by *killing the
    /// cargo child process*, not by a timer around the future. It used to be a
    /// `tokio::time::timeout` wrapped around this whole block while the work
    /// inside was a blocking `std::process::Command::output()`: a blocking call
    /// never yields, so the timer could not fire and neither this 90s bound nor
    /// the CLI's `--timeout` ever elapsed. `--timeout 1` ran to completion in
    /// 20.2s and exited 0.
    ///
    /// The deadline is checked at each stage boundary and, crucially, *inside*
    /// `run_cargo_check`, which owns the child and can kill it. One timer, at
    /// the only place that can actually stop the work.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn analyze(&self) -> Result<AccurateDeadCodeReport> {
        // Try cache first for O(1) performance
        if let Some(cached) = self.try_load_cache() {
            return Ok(cached);
        }

        let deadline = std::time::Instant::now() + self.timeout;

        // Layer 1: Scan for suppression attributes (fast, catches explicit admissions)
        let mut all_dead_items = self.scan_for_suppression_attributes()?;
        self.check_deadline(deadline)?;

        // Layer 2: Run cargo check for compiler-detected dead code
        let cargo_output = self.run_cargo_check(deadline).await?;
        let compiler_dead_items = self.parse_cargo_warnings(&cargo_output)?;
        all_dead_items.extend(compiler_dead_items);

        let files_with_dead_code = self.group_by_file(all_dead_items);
        let report = self.calculate_metrics(files_with_dead_code).await?;
        self.check_deadline(deadline)?;

        // Save to cache for next time
        self.save_cache(&report);

        Ok(report)
    }

    /// Stop here if the analysis budget is already spent.
    fn check_deadline(&self, deadline: std::time::Instant) -> Result<()> {
        if std::time::Instant::now() >= deadline {
            return Err(self.timeout_error());
        }
        Ok(())
    }

    /// The one message this analyzer reports a spent budget with, so the CLI's
    /// `--timeout` and the library default read identically.
    fn timeout_error(&self) -> anyhow::Error {
        anyhow::anyhow!(
            "Dead code analysis timed out after {} seconds",
            self.timeout.as_secs()
        )
    }

    /// Layer 1: Scan for  attributes
    ///
    /// These attributes are explicit admissions that code is unused.
    /// Detecting them is fast (~10ms for large projects) and catches
    /// code that developers knowingly left as dead.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn scan_for_suppression_attributes(&self) -> Result<Vec<(PathBuf, DeadItem)>> {
        use regex::Regex;
        use std::fs;

        let mut suppressed_items = Vec::new();

        // Patterns for dead_code suppression
        // Matches: #[allow(dead_code)], #[allow(unused)], #![allow(dead_code)]
        let suppression_re =
            Regex::new(r#"#!?\[allow\((dead_code|unused)\)\]"#).expect("Invalid regex");

        // Pattern to extract the item name on the following line
        let item_re = Regex::new(
            r#"^\s*(?:pub\s+)?(?:async\s+)?(?:const\s+)?(?:static\s+)?(?:unsafe\s+)?(fn|struct|enum|type|trait|mod|const|static)\s+(\w+)"#
        ).expect("Invalid regex");

        // Walk through all Rust files. Use ignore::WalkBuilder (not raw walkdir)
        // so the walk respects .gitignore and skips hidden directories — matching
        // the complexity/function-index analyzers. The raw walkdir only skipped
        // `target/`, so it descended into hidden trees like `.claude/worktrees/`
        // (git-worktree copies), inflating the analyzed file count ~60x and
        // surfacing worktree duplicates as dead code.
        for entry in ignore::WalkBuilder::new(&self.project_path)
            .max_depth(Some(self.max_depth))
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .build()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();

            // Belt-and-suspenders: also skip target/ explicitly (covered by
            // git_ignore inside a repo, but harmless otherwise).
            if path.starts_with(self.project_path.join("target")) {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            // Honour the include_tests/examples/benches flags. This layer used
            // to walk EVERY .rs file regardless — only `run_cargo_check`
            // consulted them — so `analyze dead-code` reported `tests/it.rs` by
            // default on a crate never analysed before, and `--include-tests`
            // could not add anything because nothing had been excluded.
            if self.is_excluded_source(path) {
                continue;
            }

            // Read file content
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let lines: Vec<&str> = content.lines().collect();

            // Scan for suppression attributes
            self.scan_file_for_suppressions(
                path,
                &lines,
                &suppression_re,
                &item_re,
                &mut suppressed_items,
            );
        }

        tracing::debug!(
            "Layer 1 (suppression scan): found {} items with ",
            suppressed_items.len()
        );

        Ok(suppressed_items)
    }

    /// Is this source file outside the scope the analyzer was configured with?
    ///
    /// Mirrors what `--include-tests` promises: with the flag absent, the test
    /// tree contributes nothing to the report. `examples/` and `benches/` are
    /// NOT excluded by default — this comment used to claim an
    /// `--include-examples` / `--include-benches` pair that does not exist on
    /// any surface, so those arms could only ever fire, and the two trees fell
    /// out of the report entirely.
    fn is_excluded_source(&self, path: &Path) -> bool {
        let relative = path.strip_prefix(&self.project_path).unwrap_or(path);
        let under = |dir: &str| {
            relative
                .components()
                .any(|c| c.as_os_str().to_string_lossy() == dir)
        };

        if self.exclude_tests && (under("tests") || is_test_file_name(relative)) {
            return true;
        }
        if self.exclude_examples && under("examples") {
            return true;
        }
        if self.exclude_benches && (under("benches") || under("benchmarks")) {
            return true;
        }
        false
    }

    /// Scan a single file's lines for suppression attributes
    fn scan_file_for_suppressions(
        &self,
        path: &Path,
        lines: &[&str],
        suppression_re: &regex::Regex,
        item_re: &regex::Regex,
        suppressed_items: &mut Vec<(PathBuf, DeadItem)>,
    ) {
        for (i, line) in lines.iter().enumerate() {
            if suppression_re.is_match(line) {
                // Try to find the item on the next non-attribute line
                let mut item_line = i + 1;
                while item_line < lines.len() {
                    let next_line = lines[item_line];
                    // Skip additional attributes and empty lines
                    if next_line.trim().starts_with("#[")
                        || next_line.trim().starts_with("#![")
                        || next_line.trim().is_empty()
                    {
                        item_line += 1;
                        continue;
                    }

                    // Try to extract the item
                    if let Some(caps) = item_re.captures(next_line) {
                        let kind_str = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown");
                        let name = caps.get(2).map(|m| m.as_str()).unwrap_or("unknown");

                        let relative_path = path
                            .strip_prefix(&self.project_path)
                            .unwrap_or(path)
                            .to_path_buf();

                        suppressed_items.push((
                            relative_path,
                            DeadItem {
                                // The item's REAL kind, which the regex above
                                // has just captured. This used to be a
                                // `DeadCodeKind::Suppressed` that erased it, and
                                // "suppressed" is a category no counter knows:
                                // `dead_functions`/`dead_classes`/`dead_modules`
                                // are counted by kind, so six dead functions
                                // reported `dead_functions: 0` and every one of
                                // them was typed `item_type: "variable"` in a
                                // record whose own reason said `fn`. Provenance
                                // (compiler lint vs. explicit admission) lives
                                // in the message below, where it does not
                                // displace the kind.
                                kind: suppressed_item_kind(kind_str),
                                name: name.to_string(),
                                line: item_line + 1, // 1-indexed
                                column: 1,
                                message: format!(
                                    "{kind_str} `{name}` carries an allow(dead_code) \
                                     suppression (explicit dead code admission)"
                                ),
                            },
                        ));
                    }
                    break;
                }
            }
        }
    }

    /// Run cargo check and capture JSON output, killing the child at `deadline`.
    async fn run_cargo_check(&self, deadline: std::time::Instant) -> Result<String> {
        let Some(cmd) = self.build_cargo_check_command() else {
            return Ok(r#"{"reason":"build-finished","success":true}"#.to_string());
        };
        self.wait_for_cargo_check(cmd, deadline).await
    }

    /// Build the `cargo check` invocation, or `None` when the scan is skipped.
    fn build_cargo_check_command(&self) -> Option<Command> {
        // PMAT_DEAD_CODE_SKIP=1 can be used to skip in specific test scenarios
        // Removed CI bypass per CB-128 spec - dead code detection must work everywhere
        if std::env::var("PMAT_DEAD_CODE_SKIP").is_ok() {
            return None;
        }

        // THE CRATE, not the directory the caller pointed at. rustc cannot
        // type-check half a crate, so a subdirectory request compiles the whole
        // crate and the findings are restricted to the subdirectory afterwards
        // (`scoped_report_path`). This used to be `self.project_path`, which
        // left cargo to walk up for the manifest by itself while every
        // target-shape decision below was still made from the subdirectory —
        // and that is how `--lib` came to be dropped for a library crate,
        // leaving `cargo check --bins` with no target to build at all.
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.cargo_root)
            .arg("check")
            .arg("--message-format=json");

        // Don't modify RUSTFLAGS — changing flags forces full recompilation
        // of all deps (including cc, which fails with extra warnings).
        // Dead code detection uses heuristic source scanning instead.
        // The compiler-level dead_code lint is only useful for catching
        // items the heuristic misses, but the cost (full recompile) is too high.

        // Wave 39 release-prep: detect target shape before passing flags.
        // Bin-only crates (no src/lib.rs and no `[lib]` section) fail
        // `cargo check --lib` with "no library targets found". Match the
        // cargo metadata: --lib only when a lib exists; otherwise --bins.
        // Asked of the CRATE. Asked of the requested path it answered "no
        // library" for every subdirectory of every crate on earth, because a
        // subdirectory holds neither a Cargo.toml nor a src/lib.rs.
        let has_lib = project_has_library(&self.cargo_root);

        // The cargo target set MUST be the same scope `is_excluded_source`
        // walks, or the report's numerator and denominator describe different
        // file sets. `examples/` and `benches/` are in scope by default (see
        // `CargoDeadCodeAnalyzer::new`) and were entering the totals walk, but
        // this builder only ever asked cargo for `--lib`/`--bins`: rustc never
        // compiled them, so their unused items could not be reported and a
        // crate whose only dead code lived in `examples/` read `0 dead lines`
        // over `3 files analyzed`. `--include-tests` had the mirror-image bug —
        // it dropped `--bins` and never added `--tests`, so turning it on
        // narrowed the compile instead of widening it.
        if has_lib {
            cmd.arg("--lib");
        }
        cmd.arg("--bins");
        if !self.exclude_tests {
            cmd.arg("--tests");
        }
        // Named targets, not the blanket `--examples`/`--benches`: a library
        // carries IMPLICIT test and bench targets, so `--benches` compiles the
        // lib a second time with `cfg(test)` on and drags every `#[cfg(test)]`
        // item into the report even when `--include-tests` is off. Selecting
        // each real target by name keeps the compile scope equal to the walk
        // scope in both directions.
        if !self.exclude_examples {
            for name in named_targets(&self.cargo_root, "example") {
                cmd.arg("--example").arg(name);
            }
        }
        if !self.exclude_benches {
            for name in named_targets(&self.cargo_root, "bench") {
                cmd.arg("--bench").arg(name);
            }
        }

        Some(cmd)
    }

    /// Run `cargo check` to completion, or kill it when `deadline` passes.
    ///
    /// This is the whole of `--timeout`. It was `cmd.output()` — a BLOCKING
    /// call sitting inside an `async` block — so the two `tokio::time::timeout`
    /// wrappers around it (the analyzer's hardcoded 90s and the CLI's
    /// `--timeout`) were both timers on a future that never yields. Neither
    /// could fire, and neither could have stopped `cargo` if it had: dropping a
    /// `std::process::Child` does not kill the process.
    ///
    /// So the child is spawned rather than waited on, the deadline is polled
    /// between `try_wait`s at a real `.await` point, and the child is KILLED
    /// when the budget is spent. `cargo` in turn kills its own children on
    /// SIGKILL of the job only for the rustc invocations it is waiting on; a
    /// long-running build script may outlive it briefly, but the command
    /// returns at the deadline instead of running to completion.
    ///
    /// stdout and stderr are drained by threads because `cargo check
    /// --message-format=json` writes far more than a pipe buffer holds, and a
    /// full pipe would block the child forever while we poll it.
    async fn wait_for_cargo_check(
        &self,
        mut cmd: Command,
        deadline: std::time::Instant,
    ) -> Result<String> {
        use std::io::Read;
        use std::process::Stdio;

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to run cargo check")?;

        let mut stdout_pipe = child.stdout.take().expect("stdout was piped");
        let mut stderr_pipe = child.stderr.take().expect("stderr was piped");
        let stdout_reader = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stdout_pipe.read_to_end(&mut buf);
            buf
        });
        let stderr_reader = std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = stderr_pipe.read_to_end(&mut buf);
            buf
        });

        let status = loop {
            match child.try_wait().context("Failed to run cargo check")? {
                Some(status) => break status,
                None => {
                    if std::time::Instant::now() >= deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        // Let the drain threads finish now that the pipes are
                        // closed, so no thread is left holding the child's fds.
                        let _ = stdout_reader.join();
                        let _ = stderr_reader.join();
                        return Err(self.timeout_error());
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                }
            }
        };

        let stdout = stdout_reader.join().unwrap_or_default();
        let stderr = stderr_reader.join().unwrap_or_default();

        if !status.success() {
            return Err(anyhow::anyhow!(
                "Cargo check failed: {}",
                String::from_utf8_lossy(&stderr)
            ));
        }

        // Cargo outputs JSON messages to stdout
        Ok(String::from_utf8_lossy(&stdout).to_string())
    }
}

/// The `DeadCodeKind` behind a suppression attribute, from the keyword the item
/// is declared with.
///
/// The keywords are exactly the alternation the item regex captures, so the
/// `Other` arm is unreachable from that call site and is there to keep an
/// unknown keyword identifiable instead of collapsing it into a wrong category.
fn suppressed_item_kind(kind_str: &str) -> DeadCodeKind {
    match kind_str {
        "fn" => DeadCodeKind::Function,
        "struct" => DeadCodeKind::Struct,
        "enum" => DeadCodeKind::Enum,
        "type" => DeadCodeKind::TypeAlias,
        "trait" => DeadCodeKind::Trait,
        "mod" => DeadCodeKind::Module,
        "const" => DeadCodeKind::Constant,
        "static" => DeadCodeKind::Static,
        other => DeadCodeKind::Other(other.to_string()),
    }
}

/// Is this file name a test module by convention (`tests.rs`, `foo_test.rs`,
/// `foo_tests.rs`, `test_foo.rs`)?
///
/// The repo's own `src/services/cargo_dead_code_analyzer/tests.rs` headed the
/// dead-code list with `--include-tests` absent, which is what the flag exists
/// to prevent.
fn is_test_file_name(path: &std::path::Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
        return false;
    };
    stem == "tests"
        || stem == "test"
        || stem.ends_with("_test")
        || stem.ends_with("_tests")
        || stem.starts_with("test_")
}

/// Return `true` when the project at `project_path` has a library target.
///
/// A library is present when *either* the conventional `src/lib.rs` file
/// exists *or* `Cargo.toml` declares an explicit `[lib]` table. The check is
/// cheap (one stat + one read) and runs once per analysis.
///
/// `pub(crate)` because the multi-language engine — the one that answers when
/// there is no cargo — asks the same question to decide whether a crate's `pub`
/// items are its public API. Two copies of "is this a library" would be two
/// answers, and the two engines disagreeing about what a crate IS is how a
/// library's whole public API came to be reported dead on one of them.
//
// Wave 39 release-prep: contract added — output is bool determined by a
// stat + a substring check on Cargo.toml. Deterministic.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn project_has_library(project_path: &std::path::Path) -> bool {
    if project_path.join("src/lib.rs").exists() {
        return true;
    }
    let cargo_toml = project_path.join("Cargo.toml");
    if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
        // Look for an explicit `[lib]` section (with optional whitespace).
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[lib]") || trimmed.starts_with("[lib.") {
                return true;
            }
        }
    }
    false
}

/// Cargo targets of `kind` ("example" / "bench") belonging to the package at
/// `project_path`, excluding any that need features this check does not enable.
///
/// Named targets rather than the blanket `--examples`/`--benches` flags,
/// because a library carries IMPLICIT test and bench targets: `--benches`
/// compiles the lib a second time with `cfg(test)` on and drags every
/// `#[cfg(test)]` item into the report even when `--include-tests` is off.
///
/// Targets with `required-features` are skipped: naming one whose features are
/// off is a hard cargo error ("target `x` requires the features: `demo`"),
/// which would abort the whole analysis. Returned sorted so the argument list —
/// and therefore the cargo invocation — is deterministic.
fn named_targets(project_path: &std::path::Path, kind: &str) -> Vec<String> {
    let Ok(output) = Command::new("cargo")
        .current_dir(project_path)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    let Ok(meta) = serde_json::from_slice::<Value>(&output.stdout) else {
        return Vec::new();
    };
    let manifest = project_path.join("Cargo.toml");
    let mut names: Vec<String> = Vec::new();
    for package in meta["packages"].as_array().into_iter().flatten() {
        // Workspace roots list every member; only the package being analysed
        // can satisfy a bare `--example NAME` / `--bench NAME`.
        if package["manifest_path"].as_str().map(std::path::Path::new) != Some(manifest.as_path()) {
            continue;
        }
        for target in package["targets"].as_array().into_iter().flatten() {
            let is_kind = target["kind"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|k| k.as_str() == Some(kind));
            if !is_kind {
                continue;
            }
            let needs_features = target["required-features"]
                .as_array()
                .is_some_and(|f| !f.is_empty());
            if needs_features {
                continue;
            }
            if let Some(name) = target["name"].as_str() {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    names
}

#[cfg(test)]
mod project_has_library_tests {
    use super::project_has_library;

    #[test]
    fn test_detects_src_lib_rs() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("src/lib.rs"), "pub fn x() {}").unwrap();
        assert!(project_has_library(tmp.path()));
    }

    #[test]
    fn test_detects_explicit_lib_section() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n[lib]\nname = \"x\"\npath = \"src/x.rs\"\n",
        )
        .unwrap();
        assert!(project_has_library(tmp.path()));
    }

    #[test]
    fn test_bin_only_returns_false() {
        // PIN: bin-only crate with no src/lib.rs and no [lib] section → false.
        // This is the bug-#4 case from release-prep bug hunt.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"bin-only\"\nversion = \"0.1.0\"\n[[bin]]\nname = \"bin-only\"\npath = \"src/main.rs\"\n",
        )
        .unwrap();
        std::fs::create_dir(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("src/main.rs"), "fn main() {}").unwrap();
        assert!(!project_has_library(tmp.path()));
    }

    #[test]
    fn test_no_cargo_toml_returns_false() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!project_has_library(tmp.path()));
    }

    #[test]
    fn test_lib_section_with_dotted_target_detected() {
        // PIN: `[lib.something]` (dotted form) is also recognized as a lib section.
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n[lib.foo]\n",
        )
        .unwrap();
        assert!(project_has_library(tmp.path()));
    }
}

/// `--include-tests` used to be inert: Layer 1 walked every `.rs` file in the
/// project whatever the flag said, so a brand-new crate reported `tests/it.rs`
/// with the flag absent and the flag could add nothing.
#[cfg(test)]
mod include_tests_flag_tests {
    use super::*;

    fn fixture() -> tempfile::TempDir {
        // Non-hidden prefix: the walk skips hidden directories, and tempfile's
        // default prefix is `.tmp`.
        let tmp = tempfile::Builder::new()
            .prefix("dcx")
            .tempdir()
            .expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("src")).expect("mkdir src");
        std::fs::create_dir_all(root.join("tests")).expect("mkdir tests");
        std::fs::create_dir_all(root.join("examples")).expect("mkdir examples");
        std::fs::create_dir_all(root.join("benches")).expect("mkdir benches");
        let body = |name: &str| format!("#[allow(dead_code)]\nfn {name}() {{}}\n");
        std::fs::write(root.join("src/lib.rs"), body("in_lib")).expect("write lib");
        std::fs::write(root.join("src/helpers_tests.rs"), body("in_src_tests")).expect("write");
        std::fs::write(root.join("tests/it.rs"), body("in_it")).expect("write it");
        std::fs::write(root.join("examples/demo.rs"), body("in_demo")).expect("write demo");
        std::fs::write(root.join("benches/bench.rs"), body("in_bench")).expect("write bench");
        tmp
    }

    fn names(items: &[(PathBuf, DeadItem)]) -> Vec<String> {
        let mut v: Vec<String> = items.iter().map(|(_, i)| i.name.clone()).collect();
        v.sort();
        v
    }

    #[test]
    fn test_suppression_scan_excludes_tests_by_default() {
        let tmp = fixture();
        let analyzer = CargoDeadCodeAnalyzer::new(tmp.path());
        let found = names(&analyzer.scan_for_suppression_attributes().expect("scan"));
        // `--include-tests` gates the test tree and nothing else. This used to
        // demand `["in_lib"]`, which pinned the defect: examples and benches
        // have no flag of their own, so excluding them by default removed them
        // from every possible invocation.
        assert_eq!(
            found,
            vec![
                "in_bench".to_string(),
                "in_demo".to_string(),
                "in_lib".to_string()
            ],
            "only the test tree is out of scope without --include-tests"
        );
    }

    #[test]
    fn test_include_tests_adds_the_test_trees() {
        let tmp = fixture();
        let analyzer = CargoDeadCodeAnalyzer::new(tmp.path()).include_tests();
        let found = names(&analyzer.scan_for_suppression_attributes().expect("scan"));
        assert!(found.contains(&"in_it".to_string()), "{found:?}");
        assert!(found.contains(&"in_src_tests".to_string()), "{found:?}");
        assert!(found.contains(&"in_lib".to_string()), "{found:?}");
        // Examples and benches are in scope with or without the flag; the flag
        // adds the test tree on top of them rather than swapping one for another.
        assert!(found.contains(&"in_demo".to_string()), "{found:?}");
        assert!(found.contains(&"in_bench".to_string()), "{found:?}");
    }

    /// 3.30.0 reported `src/lib.rs` alone on a crate whose `examples/`,
    /// `benches/` and `tests/` trees held identical suppressed items, and no
    /// flag put the first two back — not `--include-tests`, not `-u`, and not
    /// `--include 'examples/**'`, which came back EMPTY because the glob ran
    /// over a set the tree had already been cut from. Constructing the
    /// analyzer with no builder at all is the case that regressed.
    #[test]
    fn test_examples_and_benches_are_in_scope_without_any_flag() {
        let tmp = fixture();
        let analyzer = CargoDeadCodeAnalyzer::new(tmp.path());
        let found = names(&analyzer.scan_for_suppression_attributes().expect("scan"));
        assert!(found.contains(&"in_demo".to_string()), "{found:?}");
        assert!(found.contains(&"in_bench".to_string()), "{found:?}");
        // The test tree is still gated on --include-tests.
        assert!(!found.contains(&"in_it".to_string()), "{found:?}");
    }

    #[test]
    fn test_is_test_file_name_conventions() {
        for name in ["tests.rs", "foo_tests.rs", "foo_test.rs", "test_foo.rs"] {
            assert!(is_test_file_name(Path::new(name)), "{name}");
        }
        for name in ["lib.rs", "attestation.rs", "contest.rs"] {
            assert!(!is_test_file_name(Path::new(name)), "{name}");
        }
    }
}
