//! The one hygienic way to spawn the pmat binary from a test.
//!
//! # The defect this closes
//!
//! A test that builds `Command::new(env!("CARGO_BIN_EXE_pmat"))` by hand hands
//! the child **the developer's entire environment**. Several of those variables
//! change what the binary DOES, so the assertions then compare against a
//! different program than the one the author had in mind.
//!
//! This is not hypothetical. Measured against `target/debug/pmat` at the commit
//! that added this file:
//!
//! ```text
//! $ env -u MCP_VERSION  pmat --version   -> "pmat 3.32.0 …", exit 0
//! $ MCP_VERSION=1.0.0   pmat --version   -> 0 bytes of stdout, exit 0
//! ```
//!
//! `src/bin/pmat.rs:41` reads `MCP_VERSION` and, when it is set, starts the
//! stdio MCP server and ignores argv entirely ("Explicit MCP opt-in via env var
//! always wins", for Claude Desktop). Claude Desktop exports that variable, so a
//! developer running the suite under it gets a different binary. `tests/e2e_cli_t.rs`
//! — the CLI *release transport gate* — was exposed to exactly this.
//!
//! # How to use it
//!
//! Each test target declares it by path, because `tests/e2e_cli_t.rs`,
//! `tests/e2e_mcp_stdio_t.rs`, `tests/e2e_http_serve_t.rs` and
//! `tests/init_workspace_t.rs` are SEPARATE binaries and cannot share a module
//! declared under `tests/modules/`:
//!
//! ```text
//! #[path = "support/pmat_cmd.rs"]
//! mod pmat_cmd;
//! use pmat_cmd::pmat;
//!
//! let out = pmat().arg("--version").output().expect("spawn pmat");
//! ```
//!
//! From inside `tests/modules/…` the path is relative to that file, e.g.
//! `#[path = "../support/pmat_cmd.rs"]`.
//!
//! The guard that enforces this lives in `src/services/test_env_hygiene.rs` and
//! runs under `cargo test --lib`, which is what merge CI actually executes.

// Every target that declares this module uses a different subset of it; without
// this a target that only calls `pmat()` warns about the rest.
#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;

/// Environment the child must NOT inherit, and the `src/` read site that makes
/// each one load-bearing.
///
/// Every entry names a reader in this repository. A variable with no cited
/// reader does not belong here: scrubbing it would be superstition, and a list
/// of superstitions is one nobody can audit. (`PMAT_MODE` was a candidate and
/// was dropped for exactly that reason — the only mentions in `src/` are in
/// `src/tests/clap_env_var_tests.rs`, so nothing in production reads it.)
pub(crate) const SCRUBBED: &[(&str, &str)] = &[
    // ── Changes which program runs at all ────────────────────────────────────
    (
        "MCP_VERSION",
        "src/bin/pmat.rs:41 — when set, `detect_execution_mode` returns Mcp and \
         the binary starts the stdio MCP server, ignoring argv. Measured: \
         `MCP_VERSION=1.0.0 pmat --version` emits 0 bytes and exits 0.",
    ),
    (
        "PMAT_MCP_HTTP_TOKEN",
        "src/mcp_pmcp/http_server.rs:70 (via TOKEN_ENV, declared :39) — with a \
         token present `pmat serve` BINDS A SOCKET instead of refusing, so \
         `Command::output()` blocks until the test harness times out.",
    ),
    // ── Changes the bytes a test asserts on ──────────────────────────────────
    (
        "NO_COLOR",
        "src/cli/progress.rs:185 and :247, plus the color crates' own \
         auto-detection. Measured on piped stdout, which is the case every \
         `output()` call is in: `pmat --color always analyze complexity \
         --format summary` differs with and without it (ANSI SGR present vs \
         absent), so a `contains`/equality assertion flips.",
    ),
    (
        "PMAT_QUIET",
        "src/cli/progress.rs:87 (QUIET_ENV) read by `quiet_mode_enabled`. \
         `cli::run` overwrites it from `--quiet` early, so the exposure is the \
         window before `apply_ux_settings` — narrow, not closed.",
    ),
    (
        "LINT_HOTSPOT_DEBUG",
        "src/cli/handlers/lint_hotspot_handlers/clippy_file_analysis.rs:344 and \
         :355 — emits extra diagnostic lines into the output under assertion.",
    ),
    // ── Moves what gets analysed ─────────────────────────────────────────────
    (
        "PMAT_WORKSPACE",
        "src/services/configuration_impl.rs:230 and \
         src/mcp_integration/server_types.rs:33 — silently relocates the \
         analysis root, so a test that writes a fixture into a tempdir measures \
         a different tree.",
    ),
    (
        "PMAT_COVERAGE_FILE",
        "src/cli/commands/commands_enum/definition.rs:214 is `#[arg(env = \
         \"PMAT_COVERAGE_FILE\")]`, so the variable supplies the flag's DEFAULT; \
         read at src/services/agent_context/query/coverage/loader.rs:32.",
    ),
    (
        "PMAT_CONTRACTS_PATH",
        "src/cli/handlers/work_contract_binding.rs:78 — repoints contract \
         resolution away from `contracts/`.",
    ),
    (
        "PMAT_METRICS_DIR",
        "src/cli/handlers/predict_quality_handlers.rs:25 — swaps the metric \
         store the prediction is computed from.",
    ),
    (
        "PMAT_VECTOR_DB_PATH",
        "src/services/configuration_impl.rs:248 and \
         src/mcp_integration/server_types.rs:41 — swaps the vector DB, so \
         semantic results come from someone else's index.",
    ),
    (
        "PMAT_SEMANTIC_ENABLED",
        "src/mcp_integration/server_types.rs:30 — turns the semantic path on or \
         off underneath the assertion.",
    ),
    // ── Suppresses or reshapes the measurement itself ────────────────────────
    (
        "PMAT_DEAD_CODE_SKIP",
        "src/services/cargo_dead_code_analyzer/analysis.rs:260 — when set the \
         analyzer SKIPS, so a test asserting on findings reads a legitimate-\
         looking zero.",
    ),
    (
        "PMAT_COMPLY_JOBS",
        "src/cli/handlers/comply_handlers/check_handlers/check.rs:421 — changes \
         parallelism, and with it timing-sensitive and ordering-sensitive output.",
    ),
    (
        "PMAT_MUTATION_DIFF",
        "src/services/mutation_gate.rs:654 — restricts the mutation set to a diff.",
    ),
    (
        "PMAT_MUTANTS_OUT",
        "src/services/mutation_gate.rs:356 — redirects the mutant report to a \
         path the test does not own.",
    ),
    // ── Determinism seeds: inheriting one makes a run irreproducible ─────────
    (
        "PMAT_EMBEDDING_SEED",
        "src/services/ml_seed.rs:69 — changes embedding output.",
    ),
    (
        "PMAT_CLUSTERING_SEED",
        "src/services/ml_seed.rs:75 — changes cluster assignment.",
    ),
    (
        "PMAT_MUTATION_SEED",
        "src/services/ml_seed.rs:81 — changes mutant selection order.",
    ),
    // ── Cache behaviour ──────────────────────────────────────────────────────
    (
        "PAIML_CACHE_MAX_MB",
        "src/services/cache/config_methods.rs:11 — cache capacity, hence hit rate.",
    ),
    (
        "PAIML_CACHE_TTL_AST",
        "src/services/cache/config_methods.rs:17 — AST cache TTL; a stale-vs-fresh \
         difference is exactly what a re-analysis test is asserting about.",
    ),
    (
        "PAIML_CACHE_ENABLE_WATCH",
        "src/services/cache/config_methods.rs:23 — starts a filesystem watcher \
         in the child.",
    ),
    (
        "PAIML_CACHE_GIT_BRANCH_AWARE",
        "src/services/cache/config_methods.rs:27 — changes the cache key.",
    ),
    // ── clap `env =` defaults for `pmat work agent …` ────────────────────────
    (
        "PMAT_AGENT_MODEL",
        "src/cli/commands/work_commands_work.rs:464 — `#[arg(env)]`, supplies the \
         flag default.",
    ),
    (
        "PMAT_AGENT_EFFORT",
        "src/cli/commands/work_commands_work.rs:468 — `#[arg(env)]`.",
    ),
    (
        "PMAT_AGENT_HARNESS",
        "src/cli/commands/work_commands_work.rs:472 — `#[arg(env)]`.",
    ),
    (
        "PMAT_AGENT_WORKFLOW_ID",
        "src/cli/commands/work_commands_work.rs:476 — `#[arg(env)]`.",
    ),
    (
        "PMAT_AGENT_PARENT",
        "src/cli/commands/work_commands_work.rs:480 — `#[arg(env)]`.",
    ),
    (
        "PMAT_AGENT_ID",
        "src/cli/commands/work_commands_work.rs:544, :583, :634, :654 — `#[arg(env)]`.",
    ),
    // ── Credentials: a test must never depend on the developer having one ────
    (
        "GITHUB_TOKEN",
        "src/services/git_clone_url_parsing.rs:216 — with a token the child \
         authenticates and can reach private remotes, so the test passes on the \
         author's machine and fails everywhere else.",
    ),
    (
        "GH_TOKEN",
        "src/services/github_integration.rs:124 — same, second name for it.",
    ),
];

/// Environment the child KEEPS, and why removing it would be wrong.
///
/// This list exists because "scrub everything" is the tempting over-correction
/// and it does not work: the child is a cargo-built Rust binary that shells out
/// to `git` and `cargo`, and starving it produces failures that look like
/// product defects.
pub(crate) const NOT_SCRUBBED: &[(&str, &str)] = &[
    (
        "PATH",
        "the child shells out to `git` (churn, blame) and `cargo` (dead-code, \
         clippy-backed analyses). Without PATH those spawns fail with ENOENT and \
         the analysis reports an empty result that reads like a real zero.",
    ),
    (
        "HOME",
        "git refuses to read `~/.gitconfig` without it, and several analyses \
         resolve caches under `$HOME`. src/ reads HOME in 10 places.",
    ),
    (
        "CARGO_HOME",
        "cargo subprocesses resolve the registry through it; removing it makes \
         the child re-resolve against a default path it has no right to write.",
    ),
    (
        "RUSTUP_HOME",
        "rustup's shims need it to pick a toolchain; without it a `cargo` spawn \
         can fail or silently select a different toolchain than the test built \
         against.",
    ),
    (
        "CI",
        "MEASURED, not assumed: both readers (src/cli/progress.rs:180 and \
         src/demo/runner_repository.rs:175) are dominated by an `is_terminal()` \
         check, and every `Command::output()` pipes stdout. Diffing `pmat \
         analyze complexity --format summary` with `CI=true` against `-u CI` \
         gave byte-identical output. Scrubbing it would make the child disagree \
         with the environment it actually ships into, for no observable gain.",
    ),
    (
        "CARGO_TARGET_DIR",
        "this repo redirects target-dir; a child that re-resolves it can rebuild \
         into, or read a stale artifact from, a different tree.",
    ),
];

/// `RUST_LOG` is FORCED rather than removed.
///
/// It is a clap `env =` binding (`src/cli/commands/cli_struct.rs:88`) and is
/// read again at `src/cli/mod.rs:166`, so an inherited `RUST_LOG=debug` buries
/// the assertion under log noise. Removing it entirely is not enough: the
/// default filter still emits WARN, so pinning it to `error` is what actually
/// makes stderr readable in a failure message.
pub(crate) const FORCED_RUST_LOG: &str = "error";

/// The binary under test.
///
/// `PMAT_BIN` retargets every helper-routed test at an installed artifact — the
/// release gate points it at `$(which pmat)` so the same source proves something
/// about the thing users get, not only about the working tree. This mirrors
/// `tests/modules/quality_harness/mod.rs`, which established the convention.
pub(crate) fn pmat_bin() -> PathBuf {
    match std::env::var_os("PMAT_BIN") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(env!("CARGO_BIN_EXE_pmat")),
    }
}

/// Remove every variable in [`SCRUBBED`] and pin `RUST_LOG`.
///
/// Call this LAST if you also set environment of your own. Order is load-bearing:
/// applying the scrub first lets a caller's overlay put back the exact variable
/// being removed, and the resulting "fix" compiles, runs, and changes nothing.
/// The flag-efficacy harness made that mistake; `tests/modules/serve_fail_loud.rs`
/// documents it at the site.
pub(crate) fn scrub(cmd: &mut Command) -> &mut Command {
    for (var, _why) in SCRUBBED {
        cmd.env_remove(var);
    }
    cmd.env("RUST_LOG", FORCED_RUST_LOG);
    cmd
}

/// A `std::process::Command` for the pmat binary with the ambient environment
/// already scrubbed.
pub(crate) fn pmat() -> Command {
    let mut cmd = Command::new(pmat_bin());
    scrub(&mut cmd);
    cmd
}

/// [`pmat`], plus environment the CALLER wants the child to start from.
///
/// The overlay is applied FIRST and the scrub SECOND, so no caller can defeat
/// the harness's invariants by accident. If you genuinely need a scrubbed
/// variable set — the MCP tests need `MCP_VERSION` — build the command with
/// [`pmat`] and set it explicitly afterwards, where the reader can see the
/// exception.
pub(crate) fn pmat_with_env(extra: &[(&str, &str)]) -> Command {
    let mut cmd = Command::new(pmat_bin());
    for (k, v) in extra {
        cmd.env(k, v);
    }
    scrub(&mut cmd);
    cmd
}

/// The same command as an `assert_cmd::Command`, for the ~290 call sites in
/// this repo written against `Command::cargo_bin("pmat")`.
///
/// `cargo_bin` also resolves a path, but it does nothing about the environment,
/// which is the half that makes those assertions compare against the wrong
/// program.
pub(crate) fn pmat_assert() -> assert_cmd::Command {
    assert_cmd::Command::from_std(pmat())
}
