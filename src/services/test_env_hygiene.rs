//! A test that spawns `pmat` must not inherit the environment that changes it.
//!
//! # The defect this closes
//!
//! `Command::new(env!("CARGO_BIN_EXE_pmat"))` hands the child **every variable
//! the developer happens to have set**, and several of them change what the
//! binary DOES — not how it looks. The assertions then compare against a
//! different program than the author had in mind.
//!
//! Measured against `target/debug/pmat` at the commit that added this module:
//!
//! ```text
//! $ env -u MCP_VERSION  pmat --version   -> "pmat 3.32.0 …", exit 0
//! $ MCP_VERSION=1.0.0   pmat --version   -> 0 bytes of stdout, exit 0
//! ```
//!
//! `src/bin/pmat.rs:41` reads `MCP_VERSION` and starts the stdio MCP server,
//! ignoring argv entirely. Claude Desktop exports it. `pmat_serve_websocket_fails_loudly`
//! really did fail with `left: Some(0), right: Some(2)` in the full run and pass
//! alone, because two sibling tests set the variable process-wide; both setters
//! have since been deleted, but the ambient environment remains a second source
//! that deleting test code cannot reach.
//!
//! # Why a `--lib` test and not a lint, and not a test in `tests/`
//!
//! No general linter knows that `CARGO_BIN_EXE_pmat` names *this* project's
//! binary or which variables it reads. And merge CI runs **`cargo test --lib`**:
//! `tests/all.rs` is not built at the gate that decides a merge, so a guard
//! living beside the code it guards would never run where it matters. Same
//! reasoning as `src/services/contract_integrity.rs`, which this is modelled on.
//!
//! # What was landed, and what was not
//!
//! A release is in progress, so this landed as the **guard plus the helper it
//! points at** — `tests/support/pmat_cmd.rs` — with the 312 pre-existing call
//! sites recorded in the `LEDGER` below rather than rewritten. One target,
//! `tests/e2e_cli_t.rs`, is routed through the helper, both to prove the helper
//! compiles and works and because it is the CLI **release transport gate**: it
//! asserts `--version` output, which is precisely what an inherited
//! `MCP_VERSION` blanks.
//!
//! `LEDGER` is a migration ledger, not a permanent exemption. It ratchets in
//! one direction: a file may lose sites for free and gains one only by editing a
//! number, in a diff a reviewer sees.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    /// The one file allowed to construct the command, because it *is* the
    /// hygienic constructor every other site is supposed to call.
    const HELPER: &str = "tests/support/pmat_cmd.rs";

    /// Files that still construct the pmat command directly, with the exact
    /// number of sites each one has today.
    ///
    /// **This is a migration ledger to be emptied, not an allow-list to be
    /// grown.** Two kinds of entry live here and the reason string says which:
    ///
    /// * `MIGRATION:` — nothing is wrong with the test, it simply has not been
    ///   routed through `tests/support/pmat_cmd.rs` yet. Route it and delete the
    ///   row. 312 sites across 32 files were here on day one; rewriting them all
    ///   during a release would have been a 312-site blind diff over the CLI,
    ///   MCP and HTTP gates, which is a worse trade than recording them.
    /// * anything else — a site that legitimately needs raw control, with the
    ///   reason it does. Adding one of these is a claim someone has to defend.
    ///
    /// Counts are exact on purpose. A range would let a file quietly accumulate
    /// new unhygienic spawns, which is the thing being prevented.
    const LEDGER: &[(&str, usize, &str)] = &[
        // ── Legitimate raw control ───────────────────────────────────────────
        (
            "tests/e2e_cli_t.rs",
            1,
            "the surviving site is `Path::new(env!(\"CARGO_BIN_EXE_pmat\")).is_file()` \
             in `the_binary_under_test_exists` — an assertion ABOUT the artifact's \
             path, not a spawn. It must read cargo's variable directly: routing it \
             through `pmat_bin()` would let `PMAT_BIN` satisfy the check and the \
             test would stop proving that cargo built what the rest of the file \
             runs. Every actual spawn in this target now goes through the helper.",
        ),
        (
            "tests/modules/quality_harness/mod.rs",
            1,
            "`pmat_bin()` here is the ORIGINAL PMAT_BIN-retargeting resolver that \
             `tests/support/pmat_cmd.rs` copied. The falsification harnesses are \
             pointed at an installed artifact before release, and they compare \
             observables across deliberately varied environments, so a fixed scrub \
             list applied underneath them would be part of what they measure.",
        ),
        (
            "tests/bin/pmat_tests.rs",
            13,
            "ORPHAN: no `[[test]]` target names it and `autotests = false`, so \
             nothing compiles this file. Recorded so the count cannot drift while \
             it is invisible; the right fix is to wire it up or delete it, and \
             either way this row goes with it.",
        ),
        // ── MIGRATION: route these through tests/support/pmat_cmd.rs ─────────
        ("tests/e2e_http_serve_t.rs", 2, "MIGRATION"),
        ("tests/e2e_mcp_stdio_t.rs", 1, "MIGRATION"),
        ("tests/init_workspace_t.rs", 1, "MIGRATION"),
        ("tests/modules/analyze_exit_status.rs", 5, "MIGRATION"),
        (
            "tests/modules/cli_comprehensive_integration.rs",
            29,
            "MIGRATION",
        ),
        ("tests/modules/cli_context_tests.rs", 4, "MIGRATION"),
        ("tests/modules/cli_docs_enforcement.rs", 12, "MIGRATION"),
        ("tests/modules/cli_functional_harness.rs", 5, "MIGRATION"),
        ("tests/modules/cli_semantic_integration.rs", 15, "MIGRATION"),
        ("tests/modules/cli_similarity_tests.rs", 8, "MIGRATION"),
        ("tests/modules/cli_smoke_test.rs", 1, "MIGRATION"),
        (
            "tests/modules/command_discoverability_test.rs",
            1,
            "MIGRATION",
        ),
        (
            "tests/modules/complexity_threshold_filtering.rs",
            4,
            "MIGRATION",
        ),
        (
            "tests/modules/comprehensive_assert_cmd_coverage.rs",
            134,
            "MIGRATION: the single biggest holder, all `Command::cargo_bin(\"pmat\")`. \
             Mechanical to route once the release is out.",
        ),
        ("tests/modules/dead_code_timeout_test.rs", 4, "MIGRATION"),
        (
            "tests/modules/deep_context_cli_integration.rs",
            5,
            "MIGRATION",
        ),
        (
            "tests/modules/demo_e2e_integration.rs",
            3,
            "MIGRATION: two of the three are `std::env::var(\"CARGO_BIN_EXE_pmat\")` \
             — a RUNTIME read of a variable cargo only sets at COMPILE time, so \
             they always take the fallback branch. Worth fixing on its own terms.",
        ),
        ("tests/modules/demo_integration.rs", 4, "MIGRATION"),
        ("tests/modules/enhanced_dag_integration.rs", 6, "MIGRATION"),
        (
            "tests/modules/include_pattern_integration.rs",
            4,
            "MIGRATION",
        ),
        ("tests/modules/mcp_stdio_no_truncation.rs", 1, "MIGRATION"),
        ("tests/modules/mcp_tool_composition.rs", 6, "MIGRATION"),
        ("tests/modules/prompt_integration_tests.rs", 20, "MIGRATION"),
        (
            "tests/modules/quality_gate_cwd_independence_test.rs",
            1,
            "MIGRATION",
        ),
        ("tests/modules/quality_gate_exit_status.rs", 1, "MIGRATION"),
        ("tests/modules/quality_gate_integration.rs", 6, "MIGRATION"),
        (
            "tests/modules/refactor_auto_property_integration.rs",
            1,
            "MIGRATION",
        ),
        (
            "tests/modules/repo_score_cli_integration_tests.rs",
            10,
            "MIGRATION",
        ),
        (
            "tests/modules/serve_fail_loud.rs",
            2,
            "MIGRATION: already scrubs `MCP_VERSION` and `PMAT_MCP_HTTP_TOKEN` by \
             hand, with the incident written up at the site. Routing it is a \
             simplification, not a fix.",
        ),
    ];

    /// Every `(file, line)` in `tests/` that names the pmat binary directly.
    ///
    /// Comment lines are skipped: this repo documents the pattern in prose in
    /// several module headers, and `//! env!("CARGO_BIN_EXE_pmat")` is an
    /// explanation, not a spawn.
    ///
    /// The match is `"CARGO_BIN_EXE_pmat"` **with its quotes** — that is what
    /// `env!`, `var()` and `var_os()` all look like — plus
    /// `cargo_bin("pmat")` for the assert_cmd form. Quoting matters: without it
    /// the bare word inside an assertion *message* — `"CARGO_BIN_EXE_pmat must
    /// point at the freshly built binary"` — counts as a call site, and a checker
    /// that miscounts its own subject cannot be a ratchet.
    fn direct_sites() -> Vec<(String, usize)> {
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push(p);
                }
            }
        }
        let root = repo_root();
        let mut files = Vec::new();
        walk(&root.join("tests"), &mut files);
        files.sort();

        let mut found = Vec::new();
        for file in files {
            let rel = file
                .strip_prefix(&root)
                .unwrap_or(&file)
                .display()
                .to_string();
            if rel == HELPER {
                continue; // the hygienic constructor itself
            }
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            for (i, raw) in text.lines().enumerate() {
                let line = raw.trim_start();
                if line.starts_with("//") {
                    continue; // prose, including several module headers
                }
                if line.contains("\"CARGO_BIN_EXE_pmat\"") || line.contains("cargo_bin(\"pmat\")") {
                    found.push((rel.clone(), i + 1));
                }
            }
        }
        found
    }

    /// The `tests/` tree really exists and really was walked.
    ///
    /// Without this the two checks below could both pass on a walk that returned
    /// nothing — a rotted path, a renamed directory — and "we could not measure
    /// it" would read as "it did not regress".
    #[test]
    fn the_walk_finds_something_to_check() {
        let sites = direct_sites();
        assert!(
            sites.len() > 200,
            "only {} direct pmat-binary construction(s) found under tests/ — this \
             repo had 312 across 32 files when the guard was written, so a number \
             this small means the source walk is broken. A checker that measured \
             nothing must not report success.",
            sites.len()
        );
        assert!(
            repo_root().join(HELPER).is_file(),
            "{HELPER} is missing. The guard below tells people to route their \
             spawns through it; a guard with nothing to point at is hostile."
        );
    }

    /// No test spawns the pmat binary outside the ledger.
    ///
    /// RED: delete any row from [`LEDGER`], or add one unhygienic
    /// `Command::new(env!("CARGO_BIN_EXE_pmat"))` anywhere under `tests/`, and
    /// this fails naming the exact `file:line`.
    #[test]
    fn every_pmat_spawn_is_hygienic_or_ledgered() {
        let mut by_file: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (file, line) in direct_sites() {
            by_file.entry(file).or_default().push(line);
        }

        let ledger: BTreeMap<&str, (usize, &str)> =
            LEDGER.iter().map(|(f, n, why)| (*f, (*n, *why))).collect();

        let mut problems: Vec<String> = Vec::new();

        for (file, lines) in &by_file {
            match ledger.get(file.as_str()) {
                None => problems.push(format!(
                    "  {file}: {} unledgered site(s) at line(s) {} — construct the \
                     command with `pmat_cmd::pmat()` (or `pmat_assert()` for the \
                     assert_cmd form) from {HELPER}, which scrubs the environment \
                     that changes what the binary does. If this site genuinely \
                     needs raw control, add it to LEDGER with the reason.",
                    lines.len(),
                    lines
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                Some((expected, _why)) if lines.len() > *expected => problems.push(format!(
                    "  {file}: {} site(s), ledger says {expected}. A NEW direct \
                     construction was added at one of line(s) {}. Route it through \
                     {HELPER}; do not raise the ledger number to make this pass.",
                    lines.len(),
                    lines
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                Some((expected, _why)) if lines.len() < *expected => problems.push(format!(
                    "  {file}: {} site(s), ledger says {expected}. Progress — lower \
                     the ledger entry to {} so the ratchet holds at the new number.",
                    lines.len(),
                    lines.len()
                )),
                Some(_) => {}
            }
        }

        for (file, (expected, _why)) in &ledger {
            if !by_file.contains_key(*file) {
                problems.push(format!(
                    "  {file}: ledger claims {expected} site(s) and the walk found \
                     none. Either the file was routed or deleted — remove the row. \
                     A ledger that outlives its subject is a list of stale excuses."
                ));
            }
        }

        assert!(
            problems.is_empty(),
            "tests that spawn the pmat binary must not inherit the ambient \
             environment — several variables change what the binary DOES, so the \
             assertions compare against a different program (src/bin/pmat.rs:41 \
             reads MCP_VERSION and ignores argv entirely):\n{}\n\n\
             LEDGER in src/services/test_env_hygiene.rs is a MIGRATION LEDGER to \
             be emptied, not a permanent exemption.",
            problems.join("\n")
        );
    }

    /// The helper's scrub list is auditable: every entry cites a real reader.
    ///
    /// A scrub list nobody can check becomes a list of superstitions, and then
    /// somebody scrubs `PATH` and spends a day on an analysis that silently
    /// reports zero because `git` was not found.
    #[test]
    fn the_helper_documents_what_it_scrubs_and_what_it_keeps() {
        // `.expect` rather than an `unwrap_or_else` that panics: it still
        // renders the io::Error via Debug, and the ratchet counting that macro
        // under src/ has no headroom.
        let text = std::fs::read_to_string(repo_root().join(HELPER))
            .expect("the spawn helper must be readable");

        // The four the child NEEDS. Scrubbing any of them turns a missing
        // subprocess into a legitimate-looking empty measurement.
        for var in ["PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME"] {
            assert!(
                text.contains(&format!("\"{var}\",")),
                "{HELPER} must name {var} in NOT_SCRUBBED, with the reason the \
                 child needs it"
            );
        }
        for var in ["PATH", "HOME", "CARGO_HOME", "RUSTUP_HOME"] {
            assert!(
                !text.contains(&format!("env_remove(\"{var}\")")),
                "{HELPER} must never remove {var} — the child shells out to git \
                 and cargo, and starving it produces failures that look like \
                 product defects"
            );
        }

        // The variable that started this: it must be scrubbed, and the reason
        // must cite the reader rather than assert a vibe.
        assert!(
            text.contains("\"MCP_VERSION\","),
            "{HELPER} must scrub MCP_VERSION"
        );
        assert!(
            text.contains("src/bin/pmat.rs:41"),
            "{HELPER} must cite the src/ read site for MCP_VERSION; a scrub list \
             without citations cannot be audited"
        );
    }
}
