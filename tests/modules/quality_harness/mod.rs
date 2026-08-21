//! Artifact-level falsification harnesses.
//!
//! These do not check that pmat produces the *right* answer — for a
//! code-quality tool nobody knows the right answer, which is exactly why 51
//! fabricated values shipped in 3.29.0 behind a green CI. They check the two
//! properties that hold for any honest measurement, regardless of what it
//! measures:
//!
//! * [`flag_efficacy`] — a flag that parses must change something observable.
//!   49 flags in 3.29.0 parsed and changed nothing.
//! * [`differential_corpus`] — a metric must differ between an empty project
//!   and a large one. A number identical for both measures nothing.
//!
//! Both run against a *binary*, not the library, and honour `PMAT_BIN` so the
//! same gate can be pointed at a `cargo install`ed artifact before release.
//! Green `cargo test --lib` coexisted with all 243 defects; the working-tree
//! build is not the thing users run.

pub(crate) mod differential_corpus;
pub(crate) mod flag_efficacy;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// The binary under test.
///
/// Defaults to this workspace's build. `PMAT_BIN=$(which pmat)` retargets every
/// harness at the installed artifact without touching the test source — the
/// release gate and the dev loop run identical code.
pub(crate) fn pmat_bin() -> PathBuf {
    match std::env::var_os("PMAT_BIN") {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(env!("CARGO_BIN_EXE_pmat")),
    }
}

/// Everything a caller can observe from one invocation.
///
/// Deliberately includes the exit code: a flag whose only effect is to flip
/// exit status (`--fail-on-violation`) is effective, and a check that reports
/// findings on stdout while exiting 0 is a defect. Comparing stdout alone
/// would have missed nine exit-code defects in the 3.29.0 sweep.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Observable {
    pub(crate) code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) timed_out: bool,
}

impl Observable {
    /// The comparison key: normalised, so run-to-run jitter is not mistaken
    /// for a flag taking effect.
    pub(crate) fn key(&self) -> String {
        format!(
            "exit={:?}\n--stdout--\n{}\n--stderr--\n{}",
            self.code,
            normalize(&self.stdout),
            normalize(&self.stderr)
        )
    }

    pub(crate) fn succeeded(&self) -> bool {
        self.code == Some(0)
    }
}

/// Erase run-to-run variation that is not a behaviour change.
///
/// Without this every flag looks effective, because durations and temp paths
/// differ on every invocation and the harness would be comparing noise.
pub(crate) fn normalize(s: &str) -> String {
    use std::sync::OnceLock;
    static RULES: OnceLock<Vec<(regex::Regex, &'static str)>> = OnceLock::new();
    let rules = RULES.get_or_init(|| {
        let r = |p: &str| regex::Regex::new(p).expect("harness regex must compile");
        vec![
            // Durations: "1.23s", "45ms", "900µs", "12 ns"
            (r(r"\d+(?:\.\d+)?\s*(?:ns|µs|us|ms|s)\b"), "<DUR>"),
            // ISO-8601 timestamps
            (
                r(r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?"),
                "<TS>",
            ),
            // Temp dirs, which carry a fresh random component per corpus build
            (r(r"/tmp/[A-Za-z0-9._\-]+"), "<TMP>"),
            (r(r"\.tmp[A-Za-z0-9]+"), "<TMP>"),
            // Progress spinners / carriage-return redraws
            (r(r"[\r\u{8}]+"), "\n"),
            // Memory and throughput readouts
            (
                r(r"\d+(?:\.\d+)?\s*(?:B|KB|MB|GB|KiB|MiB|GiB)/?s?\b"),
                "<SIZE>",
            ),
        ]
    });
    let mut out = s.to_string();
    for (re, rep) in rules {
        out = re.replace_all(&out, *rep).into_owned();
    }
    normalize_object_ids(&out)
}

/// Replace git object ids, and *only* git object ids.
///
/// The plain rule `\b[0-9a-f]{7,40}\b` also matches the fractional digits of a
/// decimal number: `"pagerank": 0.008631379960390049` normalised to
/// `"pagerank": 0.<SHA>`, which erased the only observable difference
/// `analyze graph-metrics --convergence-threshold` produces. The flag *did*
/// change the numbers and the sweep still booked it a no-op — a normalisation
/// rule that eats the measurement is the harness lying to itself.
///
/// Requiring at least one `a`-`f` character keeps every realistic object id
/// (all-decimal short ids are ~4% of 7-char prefixes and are, in any case,
/// stable within one corpus, so both sides of a comparison carry the same
/// text) while no decimal fraction can ever match.
fn normalize_object_ids(s: &str) -> String {
    use std::sync::OnceLock;
    static SHA: OnceLock<regex::Regex> = OnceLock::new();
    let re = SHA.get_or_init(|| {
        regex::Regex::new(r"\b[0-9a-f]{7,40}\b").expect("harness regex must compile")
    });
    re.replace_all(s, |c: &regex::Captures<'_>| {
        let m = &c[0];
        if m.bytes().any(|b| b.is_ascii_alphabetic()) {
            "<SHA>".to_string()
        } else {
            m.to_string()
        }
    })
    .trim_end()
    .to_string()
}

/// Strip the ambient `cargo test` environment from a child process.
///
/// Several pmat commands shell out to `cargo` — `analyze dead-code` runs
/// `cargo check --message-format=json` to collect diagnostics. When the
/// harness itself runs under `cargo test`, that nested cargo inherits the
/// parent's jobserver (`CARGO_MAKEFLAGS`), target directory and manifest
/// path. It then fails silently, emits no diagnostics, and the analyser
/// truthfully reports **zero dead code**.
///
/// The effect is severe and completely invisible: the differential gate
/// reported `analyze dead-code` as wholly inert across all three corpora,
/// while running the identical command from a shell on the identical fixture
/// found 195 dead lines in 15 files. The harness was manufacturing the defect
/// it claimed to detect.
///
/// `CARGO_HOME` and `RUSTUP_HOME` are kept — without them the nested cargo
/// cannot locate a toolchain at all.
fn scrub_cargo_env(cmd: &mut Command) {
    for (key, _) in std::env::vars() {
        let poison = (key.starts_with("CARGO") && key != "CARGO_HOME")
            || key.starts_with("__CARGO")
            || key.starts_with("RUSTC")
            || key.starts_with("CARGO_PKG_")
            || matches!(
                key.as_str(),
                "RUSTFLAGS"
                    | "RUSTDOCFLAGS"
                    | "RUSTDOC"
                    | "RUSTUP_TOOLCHAIN"
                    | "OUT_DIR"
                    | "LD_LIBRARY_PATH"
                    | "DYLD_LIBRARY_PATH"
                    | "DYLD_FALLBACK_LIBRARY_PATH"
            );
        if poison {
            cmd.env_remove(&key);
        }
    }
}

/// Run the binary in `cwd`, killing it past `timeout`.
///
/// A hung command must be reported as `timed_out`, never silently treated as
/// empty output — "no output" is the exact shape of the pass-by-default bug
/// these harnesses exist to catch.
pub(crate) fn run(args: &[&str], cwd: &Path, timeout: Duration) -> Observable {
    run_with_env(args, cwd, timeout, &[])
}

/// As [`run`], with an environment overlay a single flag declares it needs to be
/// observable at all.
///
/// Per-flag, never global — see the RUST_LOG note below.
pub(crate) fn run_with_env(
    args: &[&str],
    cwd: &Path,
    timeout: Duration,
    extra_env: &[(&str, &str)],
) -> Observable {
    let mut cmd = Command::new(pmat_bin());
    cmd.args(args)
        .current_dir(cwd)
        // Deterministic output without suppressing anything a flag controls.
        //
        // `NO_COLOR=1` was set here originally and made every `--color` flag
        // look like a no-op: ~40 false positives in one sweep, because
        // `--color always` and `--color auto` both correctly produce no colour
        // when NO_COLOR is in force. Determinism comes from stdout being a
        // pipe rather than a tty, which is already true here.
        .env("PMAT_NO_UPDATE_CHECK", "1")
        .env("RAYON_NUM_THREADS", "2")
        .env_remove("PMAT_CONFIG")
        // RUST_LOG is scrubbed for the same reason NO_COLOR is not SET: the
        // sweep's verdict must be a property of the binary, not of the shell it
        // was launched from.
        //
        // `--quiet` is honoured ABOVE clap dispatch (cli/mod.rs
        // `effective_trace_filter` drops the RUST_LOG fallback, then
        // `log_level_directive` forces "error"), so it suppresses framework
        // chatter for EVERY command. With RUST_LOG unset there is no chatter to
        // suppress and `--quiet` reads as a no-op on ~43 commands; with
        // RUST_LOG=info exported it reads as effective on all of them. The
        // verdict flipped on the developer's environment — the same class as the
        // NO_COLOR bug above, third instance in this file.
        //
        // Removed rather than set: setting it globally would make `--verbose`
        // measure 129B against 129B and read as a no-op on every command. A
        // flag that needs an environment to be observable declares it per-flag
        // in PROBE_ENV, never globally.
        .env_remove("RUST_LOG")
        .stdin(std::process::Stdio::null());

    scrub_cargo_env(&mut cmd);

    // LAST, deliberately. The chain above calls `.env_remove("RUST_LOG")`, so
    // applying the overlay before it would delete the very variable a flag
    // declared it needs — the fix would compile, run, and change nothing.
    for (k, v) in extra_env {
        cmd.env(k, v);
    }

    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Observable {
                code: None,
                stdout: String::new(),
                stderr: format!("spawn failed: {e}"),
                timed_out: false,
            }
        }
    };

    // Drain both pipes on threads; a command that fills a 64KiB pipe buffer
    // while we poll for exit would otherwise deadlock forever.
    let mut so = child.stdout.take().expect("stdout piped");
    let mut se = child.stderr.take().expect("stderr piped");
    let t_out = std::thread::spawn(move || {
        let mut b = Vec::new();
        let _ = std::io::Read::read_to_end(&mut so, &mut b);
        b
    });
    let t_err = std::thread::spawn(move || {
        let mut b = Vec::new();
        let _ = std::io::Read::read_to_end(&mut se, &mut b);
        b
    });

    let deadline = std::time::Instant::now() + timeout;
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Some(st),
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break None;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(_) => break None,
        }
    };

    let stdout = String::from_utf8_lossy(&t_out.join().unwrap_or_default()).into_owned();
    let stderr = String::from_utf8_lossy(&t_err.join().unwrap_or_default()).into_owned();

    Observable {
        code: status.and_then(|s| s.code()),
        stdout,
        stderr,
        timed_out,
    }
}

pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(45);

// ---------------------------------------------------------------------------
// Corpora
// ---------------------------------------------------------------------------

/// Three project sizes spanning the range every metric must distinguish.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CorpusSize {
    /// A valid Rust project containing no code worth measuring.
    Empty,
    /// One clean function.
    Tiny,
    /// ~110 files carrying every defect class pmat claims to detect.
    Large,
}

impl CorpusSize {
    pub(crate) fn name(self) -> &'static str {
        match self {
            CorpusSize::Empty => "empty",
            CorpusSize::Tiny => "tiny",
            CorpusSize::Large => "large",
        }
    }
}

/// Build a git-backed Rust project of the requested size.
///
/// Git matters: several 3.29.0 defects only appeared outside a repository
/// (the critical-defect waiver inverted, churn silently zero), so a corpus
/// without history cannot exercise them.
pub(crate) fn build_corpus(size: CorpusSize) -> tempfile::TempDir {
    let dir = tempfile::Builder::new()
        .prefix(&format!("pmat-corpus-{}-", size.name()))
        .tempdir()
        .expect("create corpus tempdir");
    let root = dir.path();
    std::fs::create_dir_all(root.join("src")).expect("mkdir src");

    // The large corpus declares feature flags and a benchmark target, which
    // `repo-score`'s parent — the Rust Project Score embedded in `pmat score` —
    // reads straight out of the manifest: `rps_categories."Dependency Health"`
    // scores feature flags in tiers (dependency_scorer_scoring_methods.rs:235-241)
    // and `rps_categories."Performance & Benchmarking"` gives 5 of its 10 points
    // for a `[[bench]]` section and 2 more for `harness = false`
    // (performance_scorer_scoring.rs:44-46). Both categories were identical for
    // an empty crate and a 200-file one because no corpus carried a manifest
    // with anything in it.
    let manifest = match size {
        CorpusSize::Large => "\
[package]
name = \"corpus\"
version = \"0.1.0\"
edition = \"2021\"

[dependencies]

[features]
default = [\"std\"]
std = []
simd = []
tracing = []

[[bench]]
name = \"throughput\"
harness = false
",
        _ => "[package]\nname = \"corpus\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
    };
    std::fs::write(root.join("Cargo.toml"), manifest).expect("write Cargo.toml");
    if size == CorpusSize::Large {
        // A declared `[[bench]]` whose file is missing makes cargo refuse the
        // manifest outright, which would take every cargo-backed command down.
        std::fs::create_dir_all(root.join("benches")).expect("mkdir benches");
        std::fs::write(
            root.join("benches/throughput.rs"),
            "//! Custom-harness benchmark (harness = false), so it is a plain main().\n\nfn main() {\n    let start = std::time::Instant::now();\n    let mut acc = 0u64;\n    for i in 0..10_000u64 {\n        acc = acc.wrapping_add(i);\n    }\n    println!(\"throughput: {acc} in {:?}\", start.elapsed());\n}\n",
        )
        .expect("write benches/throughput.rs");
    }

    match size {
        CorpusSize::Empty => {
            std::fs::write(root.join("src/lib.rs"), "//! Nothing to measure.\n")
                .expect("write lib.rs");
        }
        CorpusSize::Tiny => {
            std::fs::write(
                root.join("src/lib.rs"),
                "//! One clean function.\n\n/// Adds two numbers.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
            )
            .expect("write lib.rs");
        }
        CorpusSize::Large => write_large_corpus(root),
    }

    write_repo_hygiene(root, size);
    git_init(root, size);
    if size == CorpusSize::Large {
        write_precommit_hook(root);
        write_critical_risk_file(root);
    }
    dir
}

/// Graduate the *repository* alongside the code.
///
/// Without this the three corpora differ only in source, so anything scoring
/// repo hygiene — `repo-score`, `comply`, `project-diag` — reports the same
/// number for all three and the invariant fires on every one of its leaves.
/// The first sweep produced 30 such false positives. A metric that only reads
/// README/CI/licence files is still a measurement; it just needs an axis to
/// measure along.
fn write_repo_hygiene(root: &Path, size: CorpusSize) {
    // The three coverage branches, one per corpus. `check_coverage` reads a
    // report someone else produced and has three outcomes — a report above the
    // 80% floor, a report below it, and no report at all — and with no corpus
    // carrying a report only the third was ever taken, so `coverage_violations`
    // was the same number for a project with no tests and one with full
    // coverage. Empty gets the passing report because it is the only corpus
    // that can honestly hold one (one file, one line, covered), and because
    // `results.passed` needs a project that passes: an empty crate is it.
    // Tiny stays reportless, so "nobody measured" is still exercised.
    if size == CorpusSize::Empty {
        write_coverage_report(root, CoverageReport::Aggregate("{\"coverage\": 100.0}\n"));
    }

    if size == CorpusSize::Empty {
        // Deliberately bare: no README, no licence, no CI, no gitignore.
        return;
    }

    std::fs::write(
        root.join("README.md"),
        "# corpus\n\nGenerated fixture for the pmat falsification harnesses.\n",
    )
    .expect("write README");
    std::fs::write(root.join(".gitignore"), "/target\n").expect("write gitignore");

    if size != CorpusSize::Large {
        return;
    }

    std::fs::write(
        root.join("LICENSE"),
        "MIT License\n\nPermission is hereby granted, free of charge, to any person obtaining a copy...\n",
    )
    .expect("write LICENSE");
    std::fs::write(
        root.join("CHANGELOG.md"),
        "# Changelog\n\n## [0.1.0]\n\n### Added\n- Initial fixture.\n",
    )
    .expect("write CHANGELOG");
    std::fs::write(
        root.join("CONTRIBUTING.md"),
        "# Contributing\n\nOpen a pull request.\n",
    )
    .expect("write CONTRIBUTING");
    std::fs::write(
        root.join("rustfmt.toml"),
        "edition = \"2021\"\nmax_width = 100\n",
    )
    .expect("write rustfmt.toml");
    // `coverage` is not decoration: `analyze coverage-improve` shells out to
    // `make coverage` and parses an llvm-cov `TOTAL` line out of it. Without
    // the target it died with "No rule to make target 'coverage'" before any
    // of its own flags were read, and all of them were booked as no-ops
    // against a baseline that had already failed in an external tool.
    std::fs::write(
        root.join("Makefile"),
        "\
.PHONY: build test lint coverage

build:\n\tcargo build\n
test:\n\tcargo test\n
lint:\n\tcargo clippy -- -D warnings\n
coverage:\n\t@echo \"TOTAL                        1200    240    80.00%    300    60    80.00%\"\n",
    )
    .expect("write Makefile");
    std::fs::create_dir_all(root.join(".github/workflows")).expect("mkdir workflows");
    std::fs::write(
        root.join(".github/workflows/ci.yml"),
        "\
name: ci
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo test --all
      - run: cargo clippy -- -D warnings
      - run: cargo fmt --check
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      # 3 of the 10 Performance & Benchmarking points are for a workflow that
      # runs benchmarks in CI (performance_scorer_scoring.rs:10-21).
      - run: cargo bench --bench throughput
",
    )
    .expect("write ci.yml");
    std::fs::create_dir_all(root.join("docs")).expect("mkdir docs");
    std::fs::write(
        root.join("docs/architecture.md"),
        "# Architecture\n\nOne crate, many generated modules.\n",
    )
    .expect("write docs");

    write_hygiene_debt(root);
}

/// The *repository-level* defects, as distinct from the source-level ones.
///
/// Everything here exists because a `repo-score` or `quality-gate` counter was
/// constant across all three corpora while being genuinely computed: the corpus
/// simply contained none of the inputs those checks look for. Each block names
/// the leaf it unsticks.
fn write_hygiene_debt(root: &Path) {
    // repo-score C1 (cruft) and C2 (team-specific files): the hygiene category
    // scored a full 15.0/15.0 on every corpus because there was no cruft and no
    // editor directory anywhere in the tree.
    for (name, body) in [
        (".DS_Store", "\u{0}\u{0}cruft\n"),
        ("backup.bak", "old copy of the Makefile\n"),
        ("old.orig", "<<<<<<< HEAD merge leftover\n"),
        ("scratch.tmp", "scratch\n"),
    ] {
        std::fs::write(root.join(name), body).expect("write cruft file");
    }
    std::fs::create_dir_all(root.join(".idea")).expect("mkdir .idea");
    std::fs::write(
        root.join(".idea/workspace.xml"),
        "<?xml version=\"1.0\"?>\n<project version=\"4\" />\n",
    )
    .expect("write .idea/workspace.xml");
    std::fs::create_dir_all(root.join(".vscode")).expect("mkdir .vscode");
    std::fs::write(
        root.join(".vscode/settings.json"),
        "{ \"rust-analyzer.checkOnSave.command\": \"clippy\" }\n",
    )
    .expect("write .vscode/settings.json");

    // repo-score C3: a blob over 1MB that is still in the tree. Written before
    // `git_init` on purpose — the check pipes `git rev-list --objects HEAD`, so
    // an uncommitted file is invisible to it.
    let mut blob = Vec::with_capacity(2_100_000);
    while blob.len() < 2_100_000 {
        blob.extend_from_slice(b"corpus-large-object-payload\n");
    }
    std::fs::write(root.join("bigblob.dat"), &blob).expect("write bigblob.dat");

    // repo-score F1/F2: both key off `.pmat-gates.toml`, which no corpus had,
    // so `pmat_compliance` sat at a constant 2.5/5.0 (F1 zero for the missing
    // file, F2 full marks under its can't-violate-what-doesn't-exist branch).
    std::fs::write(
        root.join(".pmat-gates.toml"),
        "[complexity]\nmax = 10\n\n[satd]\nmax = 0\n",
    )
    .expect("write .pmat-gates.toml");

    // `comply`: every corpus was an unpinned project, so `versions_behind` was
    // 0, `breaking_changes[]` empty and `is_compliant` true — all three
    // truthfully, and all three constant. A pin two major versions back is the
    // input the whole comply surface is built to react to.
    std::fs::create_dir_all(root.join(".pmat")).expect("mkdir .pmat");
    std::fs::write(
        root.join(".pmat/project.toml"),
        "[pmat]\nversion = \"2.0.0\"\n",
    )
    .expect("write .pmat/project.toml");

    // `quality-gate`'s coverage check does not measure coverage: it reads
    // `.pmat/coverage-cache.json`, else `.pmat-metrics/coverage.json`. See
    // `write_coverage_report` for why each corpus carries a different one.
    //
    // The hit map — not a pre-computed percentage — is deliberate: the check
    // has to do the 1-of-4 arithmetic itself, so the resulting 25% proves the
    // computation rather than echoing a number the fixture chose.
    write_coverage_report(
        root,
        CoverageReport::HitMap("{\"files\":{\"src/lib.rs\":{\"1\":1,\"2\":0,\"3\":0,\"4\":0}}}\n"),
    );

    // A THIRD coverage location, because `pmat score` reads a different file
    // from `quality-gate`: `.pmat-metrics/coverage.result` with a `coverage_pct`
    // key (score_handler.rs:532-545). Without it score's coverage dimension is
    // "not measured" on every corpus, so `dimensions_measured` and
    // `not_measured[]` are the same 4 for a project with coverage and one
    // without. 25.0 matches the hit map above rather than contradicting it.
    std::fs::create_dir_all(root.join(".pmat-metrics")).expect("mkdir .pmat-metrics");
    std::fs::write(
        root.join(".pmat-metrics/coverage.result"),
        "{\"coverage_pct\": 25.0}\n",
    )
    .expect("write coverage.result");

    // An lcov artifact where a coverage run would leave one, for the commands
    // that read a report rather than the `.pmat` caches:
    // `discover_line_coverage` looks in `target/coverage/lcov.info` first
    // (services/agent_context/query/coverage/parsing.rs:184-190).
    //
    // Written the way cargo-llvm-cov writes it — absolute `SF:` paths — which
    // matters: `analyze incremental-coverage` still reports every changed file
    // NotMeasured with this artifact present, because the parser normalises
    // `SF:` to `src/x.rs` while the changed-file list is keyed `./src/x.rs`.
    // Handing it a `./`-prefixed artifact would make the leaf move and hide
    // that; the corpus carries the realistic input instead.
    let mut lcov = String::new();
    for (name, covered, total) in [
        ("src/chain_00.rs", 9, 10),
        ("src/chain_01.rs", 2, 10),
        ("src/complex_00.rs", 9, 10),
        ("src/complex_01.rs", 1, 10),
        ("src/lib.rs", 40, 200),
    ] {
        lcov.push_str(&format!("SF:{}\n", root.join(name).display()));
        for line in 1..=total {
            lcov.push_str(&format!("DA:{line},{}\n", u8::from(line <= covered)));
        }
        lcov.push_str(&format!("LF:{total}\nLH:{covered}\nend_of_record\n"));
    }
    std::fs::create_dir_all(root.join("target/coverage")).expect("mkdir target/coverage");
    std::fs::write(root.join("target/coverage/lcov.info"), lcov).expect("write lcov.info");

    // GPU code, at the repository ROOT — `has_gpu_simd_code` decides whether the
    // category applies at all by `std::fs::read_dir(project_path)` over the top
    // level (gpu_simd_scorer.rs:103-110), with no recursion, so the same file in
    // `cuda/` leaves the category N/A. With no GPU code anywhere the category
    // reported 0.0 for every corpus; this kernel takes it to 50.0, and it earns
    // less than full marks honestly: `divergent` calls __syncthreads() inside a
    // divergent branch, which is the barrier-safety defect the analyser hunts.
    std::fs::write(
        root.join("kernel.cu"),
        "\
__global__ void reduce(const float* in, float* out, int n) {
    extern __shared__ float buf[];
    int tid = threadIdx.x;
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    buf[tid] = (i < n) ? in[i] : 0.0f;
    __syncthreads();
    for (int s = blockDim.x / 2; s > 0; s >>= 1) {
        if (tid < s) {
            buf[tid] += buf[tid + s];
        }
        __syncthreads();
    }
    if (tid == 0) out[blockIdx.x] = buf[0];
}

__global__ void divergent(const float* in, float* out, int n) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i % 2 == 0) {
        __syncthreads();
        out[i] = in[i] * 2.0f;
    }
}
",
    )
    .expect("write kernel.cu");

    // `quality-gate`'s security check walks only the project's top level
    // (`fs::read_dir`, no recursion), so with every source file under `src/` it
    // never opened one and reported zero honestly. A root-level file is the
    // only input that reaches it — worth knowing in itself, since no
    // conventional Rust layout puts sources there.
    std::fs::write(
        root.join("leak.rs"),
        "// Not part of the crate: no `mod leak;` anywhere, so rustc never sees it.\npub fn connect() -> &'static str {\n    let password = \"hunter2\";\n    let api_key = \"AKIAIOSFODNN7EXAMPLE\";\n    let _ = (password, api_key);\n    \"connected\"\n}\n",
    )
    .expect("write leak.rs");
}

/// The two shapes of coverage report pmat knows how to read.
///
/// Both are consumed by `read_coverage_from_cache`
/// (src/cli/analysis_utilities/quality_checks_part2_coverage_sections.rs), in
/// this priority order. Using both across the corpora keeps the fallback path
/// covered as well as the primary one.
enum CoverageReport {
    /// `.pmat/coverage-cache.json` — per-file hit maps, percentage computed.
    HitMap(&'static str),
    /// `.pmat-metrics/coverage.json` — a pre-aggregated percentage.
    Aggregate(&'static str),
}

fn write_coverage_report(root: &Path, report: CoverageReport) {
    match report {
        CoverageReport::HitMap(body) => {
            std::fs::create_dir_all(root.join(".pmat")).expect("mkdir .pmat");
            std::fs::write(root.join(".pmat/coverage-cache.json"), body)
                .expect("write coverage-cache.json");
        }
        CoverageReport::Aggregate(body) => {
            std::fs::create_dir_all(root.join(".pmat-metrics")).expect("mkdir .pmat-metrics");
            std::fs::write(root.join(".pmat-metrics/coverage.json"), body)
                .expect("write coverage.json");
        }
    }
}

/// A pre-commit hook, so `repo-score`'s B1/B2 have one to grade.
///
/// Written after `git_init`, and harmless: every corpus commit is made with
/// `-c core.hooksPath=<corpus>/.corpus-nohooks` and `--no-verify`, so this file
/// is scored but never executed. Without it `precommit_hooks` scored 0.0/20.0
/// on all three corpora — the honest reading of a repository that has no hook.
fn write_precommit_hook(root: &Path) {
    let hook = root.join(".git/hooks/pre-commit");
    std::fs::write(
        &hook,
        "#!/bin/sh\nset -eu\ncargo clippy -- -D warnings\ncargo fmt --check\n",
    )
    .expect("write pre-commit hook");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
            .expect("chmod pre-commit hook");
    }
}

/// Generate a project that trips every detector, at graded intensity.
///
/// Each family is emitted at several magnitudes so a metric can be checked for
/// *ordering*, not merely for being non-constant.
fn write_large_corpus(root: &Path) {
    let mut modules = Vec::new();

    // Complexity: nested branching that scales with the file index.
    for i in 0..40 {
        let name = format!("complex_{i:02}");
        let mut body = String::new();
        body.push_str(&format!(
            "/// Branch-heavy function #{i}.\npub fn evaluate_{i:02}(input: i64, mode: u8) -> i64 {{\n    let mut acc = 0i64;\n"
        ));
        for depth in 0..(2 + i % 8) {
            body.push_str(&format!(
                "    if input > {depth} && mode != {depth} {{\n        for step in 0..input {{\n            match step % 4 {{\n                0 => acc += step,\n                1 if step > 10 => acc -= step,\n                2 => acc ^= step,\n                _ => acc = acc.wrapping_add({depth}),\n            }}\n        }}\n    }}\n"
            ));
        }
        body.push_str("    acc\n}\n");
        write_module(root, &name, &body, &mut modules);
    }

    // Self-admitted technical debt, in every marker dialect.
    //
    // Half of each file's debt is deliberately *not* a canonical marker.
    // `analyze satd --strict` narrows the pattern set to the canonical markers,
    // so a fixture carrying only TODO/FIXME/HACK/XXX gives strict mode nothing
    // to narrow: strict and non-strict returned the identical 63 violations and
    // the sweep booked the mode as a no-op.
    for i in 0..15 {
        let name = format!("satd_{i:02}");
        let body = format!(
            "// TODO: replace the placeholder below with a real implementation\n// FIXME: this ignores the error path entirely\n// HACK: works only because callers pre-validate\n// XXX: revisit before release\n// this is a temporary workaround we should optimize later\n// technical debt lives here and nobody has paid it down\n// code smell in this module, kept only to avoid a rewrite\n\n/// Returns a hardcoded answer.\npub fn lookup_{i:02}(_key: &str) -> u32 {{\n    // For now, we know the answer is always this.\n    {}\n}}\n",
            i * 7
        );
        write_module(root, &name, &body, &mut modules);
    }

    // Fault patterns: unwrap / expect / panic / unreachable.
    //
    // `faults_00` carries *three* `unwrap()` calls where the rest carry one.
    // TDG's critical-defect penalty is `69.9 * 0.6^(n-1)`, so a single critical
    // defect is pinned to the 69.9/D ceiling and can never reach F: with one
    // unwrap per file the corpus had no F-grade input at all, `f_grade_count`
    // was truthfully zero on every corpus, and the F-grade gate truthfully
    // passed — indistinguishable, to a differential check, from a counter
    // nobody increments. Three defects put one file at ~25/F.
    for i in 0..15 {
        let name = format!("faults_{i:02}");
        let extra = if i == 0 {
            "    let second = raw.split(',').nth(1).unwrap();\n    let third = second.split(':').next().unwrap();\n    let _ = third.len();\n"
        } else {
            ""
        };
        let body = format!(
            "/// Parses without handling failure.\npub fn parse_{i:02}(raw: &str) -> i64 {{\n    let first = raw.split(',').next().unwrap();\n{extra}    let value: i64 = first.trim().parse().expect(\"caller guarantees a number\");\n    if value < 0 {{\n        panic!(\"negative value: {{value}}\");\n    }}\n    if value > 1_000_000 {{\n        unreachable!(\"validated upstream\");\n    }}\n    value\n}}\n"
        );
        write_module(root, &name, &body, &mut modules);
    }

    // Duplication: ten identical pairs, for clone detection.
    for i in 0..10 {
        let shared = "/// Normalises a record.\npub fn normalise(record: &str) -> String {\n    let trimmed = record.trim();\n    let lowered = trimmed.to_lowercase();\n    let collapsed = lowered.split_whitespace().collect::<Vec<_>>().join(\" \");\n    let stripped = collapsed.replace(['\\'', '\"'], \"\");\n    if stripped.is_empty() {\n        return String::from(\"<blank>\");\n    }\n    format!(\"{}:{}\", stripped.len(), stripped)\n}\n"
            .to_string();
        write_module(root, &format!("dup_a_{i:02}"), &shared, &mut modules);
        write_module(root, &format!("dup_b_{i:02}"), &shared, &mut modules);
    }

    // Dead code: private items nothing references.
    //
    // Each dead function is deliberately longer than `analyze dead-code`'s
    // `--min-dead-lines` default of 10. A six-line dead function is filtered
    // out by that threshold and the corpus would report zero dead code — a
    // fixture artifact indistinguishable from a broken detector.
    for i in 0..15 {
        let name = format!("dead_{i:02}");
        let mut body =
            format!("fn never_called_{i:02}(x: usize) -> usize {{\n    let mut total = 0usize;\n");
        // `i * 0` trips clippy's deny-by-default `erasing_op`, which made the
        // whole corpus fail `cargo clippy` — so every pmat command that shells
        // out to clippy returned Err before reading any of its own flags, and
        // the sweep blamed the flags. Fixtures must be dirty in the ways the
        // tool measures and clean in the ways its toolchain refuses to run.
        for k in 1..15 {
            body.push_str(&format!(
                "    for i in 0..x {{\n        total = total.wrapping_add(i * {k});\n    }}\n"
            ));
        }
        body.push_str("    total\n}\n\n");
        body.push_str(&format!(
            "fn also_never_called_{i:02}(label: &str) -> String {{\n    let mut out = String::new();\n"
        ));
        for k in 0..12 {
            body.push_str(&format!(
                "    out.push_str(&format!(\"{k}:{{label}};\"));\n"
            ));
        }
        body.push_str("    out\n}\n\n");
        body.push_str(&format!(
            "struct UnusedRecord{i:02} {{\n    _id: u64,\n    _label: String,\n    _tags: Vec<String>,\n}}\n\n"
        ));
        // Two more dead items per file, which is what carries every dead_* file
        // over the 20% mark `quality-gate`'s per-file warning uses. That warning
        // inspects `files_with_dead_code.iter().take(5)`
        // (quality_checks_part1_dead_code.rs:85) — an *unordered* first five —
        // so a single very-dead file is not enough: whichever five the report
        // happens to list must contain one over the line, which means all of
        // them have to be.
        for suffix in ["a", "b"] {
            body.push_str(&format!(
                "fn spare_{i:02}_{suffix}(seed: usize) -> usize {{\n    let mut acc = seed;\n    acc = acc.wrapping_mul(3);\n    acc = acc.wrapping_add(11);\n    acc = acc.rotate_left(2);\n    acc = acc.wrapping_sub(5);\n    acc\n}}\n\n"
            ));
        }
        // Unused and unreachable are different findings, and the corpus used to
        // supply only the first: every dead item here was an item nothing
        // *references*, so `--include-unreachable` had nothing to include and
        // the sweep booked a working flag as a no-op. rustc emits exactly one
        // `unreachable_code` diagnostic per function no matter how many
        // statements trail the `return`, so three patched modules mean a stable
        // "Unreachable blocks: 3" — and because the flag is the only thing that
        // lets such a finding into the report, the default run is unchanged.
        // Kept to three of fifteen so the family still exercises the plain
        // never-referenced path that the default report measures.
        let trailing = if i < 3 {
            "    return \"ok\";\n    let ignored = 1 + 1;\n    let _ = ignored;\n    \"unreachable\"\n"
        } else {
            "    \"ok\"\n"
        };
        body.push_str(&format!(
            "/// The only reachable item here.\npub fn entry_{i:02}() -> &'static str {{\n{trailing}}}\n"
        ));
        write_module(root, &name, &body, &mut modules);
    }

    // Superlinear complexity, for algorithmic-complexity detectors.
    //
    // The `complex_*` family above has *sequential* loops inside separate
    // branches, which is genuinely O(n) — `analyze big-o` classifying it that
    // way is correct, and treating its empty O(n^2) bucket as a defect was a
    // fixture gap. A metric can only be checked across a range the corpus
    // actually spans.
    //
    // Fourteen files, not six: `analyze big-o` ranks with a default
    // `--top-files 10`, so `summary.high_complexity_truncated` can only become
    // true when the high-complexity findings span more than ten files. At six
    // it was false for every corpus and read as a dead flag echo rather than
    // the truncation disclosure it is.
    for i in 0..14 {
        let name = format!("superlinear_{i:02}");
        let body = format!(
            "/// Quadratic scan.\npub fn pairwise_{i:02}(items: &[i64]) -> i64 {{\n    let mut acc = 0i64;\n    for a in items {{\n        for b in items {{\n            acc += a * b;\n        }}\n    }}\n    acc\n}}\n\n/// Cubic scan.\npub fn triple_{i:02}(items: &[i64]) -> i64 {{\n    let mut acc = 0i64;\n    for a in items {{\n        for b in items {{\n            for c in items {{\n                acc += a + b + c;\n            }}\n        }}\n    }}\n    acc\n}}\n"
        );
        write_module(root, &name, &body, &mut modules);
    }

    // A dimension that GATES, so `pmat score`'s gated_by[] is exercised.
    //
    // `score`'s pv_lint returns 0.0 for a project with no `contracts/` that
    // declares a `pub fn` named after an ML kernel — a deliberate probe in
    // score_handler_compute.rs. A zero is the one input that reaches the
    // gating path added for paiml/aprender #2463, where one zeroed dimension
    // used to collapse the geometric mean and render `0.0 / F` for a tree whose
    // File Health was 100. Without a gating input in any corpus, `gated_by[].len`
    // was 0 everywhere and the differential gate correctly called it a constant:
    // the verdict this release was built around was never being measured.
    write_module(
        root,
        "matmul",
        "/// Trips `score`'s ML-kernel probe so one dimension gates.\npub fn matmul(a: &[f32], b: &[f32]) -> f32 {\n    a.len() as f32 + b.len() as f32\n}\n",
        &mut modules,
    );

    // One genuinely awful file, so the grade-distribution tail is populated.
    //
    // Without it the worst file in the corpus grades C+, `f_grade_count` is
    // truthfully zero, and the F-grade gate truthfully passes — both of which
    // read as defects to a differential check that never supplied an F-grade
    // input to distinguish them from.
    let mut awful = String::from(
        "// TODO: rewrite this entire module\n// FIXME: known to be wrong for negative input\n// HACK: retained only for backwards compatibility\n\n/// Pathological.\npub fn tangle(a: i64, b: i64, c: i64, d: i64, mode: u8) -> i64 {\n    let mut acc = 0i64;\n",
    );
    for i in 0..30 {
        awful.push_str(&format!(
            "    if a > {i} {{\n        if b < {i} {{\n            for x in 0..a {{\n                for y in 0..b {{\n                    match (x + y) % 3 {{\n                        0 if c > {i} => acc += x * y,\n                        1 if d < {i} => acc -= x,\n                        _ => acc ^= y,\n                    }}\n                }}\n            }}\n        }} else if mode == {i} {{\n            acc = acc.wrapping_mul(2);\n        }}\n    }}\n"
        ));
    }
    awful.push_str("    acc\n}\n");
    write_module(root, "awful", &awful, &mut modules);

    // A real dependency chain, for anything that walks edges.
    //
    // Every other family here is mutually independent, so the corpus graph was
    // a pure star: `analyze dag --max-depth 1` and `--max-depth 50` rendered
    // byte-identical output because there was no second hop to cut, and
    // betweenness centrality was truthfully 0.0 for every node. A depth limit
    // can only be falsified on a graph that has depth.
    const CHAIN_LEN: usize = 6;
    for i in 0..CHAIN_LEN {
        let name = format!("chain_{i:02}");
        let body = if i + 1 < CHAIN_LEN {
            format!(
                "use crate::chain_{next:02};\n\n/// Hop {i} of the dependency chain.\npub fn step_{i:02}(n: i64) -> i64 {{\n    chain_{next:02}::step_{next:02}(n) + 1\n}}\n",
                next = i + 1
            )
        } else {
            format!("/// Last hop of the dependency chain.\npub fn step_{i:02}(n: i64) -> i64 {{\n    n\n}}\n")
        };
        write_module(root, &name, &body, &mut modules);
    }

    // A long function, for length-based checks.
    let mut long = String::from(
        "/// Deliberately long.\npub fn long_body() -> i64 {\n    let mut acc = 0i64;\n",
    );
    for i in 0..300 {
        long.push_str(&format!("    acc += {i};\n"));
    }
    long.push_str("    acc\n}\n");
    write_module(root, "long_body", &long, &mut modules);

    // A file over 1000 lines, because that is the only axis `score`'s
    // `sub_scores.file_health` has: it is `(1 - files_over_1000 / total_rs) *
    // 100`, so with the corpus's biggest file pinned just under 1000 lines
    // (see `write_critical_risk_file`) it returned exactly 100.0 for an empty
    // project and a 121-file one alike. Split into ~100-line functions so the
    // file is long without also being the corpus's worst-graded one.
    let mut huge = String::from("//! Over the 1000-line file-health threshold.\n");
    for f in 0..12 {
        huge.push_str(&format!(
            "\n/// Ballast block {f}.\npub fn ballast_{f:02}() -> i64 {{\n    let mut acc = 0i64;\n"
        ));
        for i in 0..100 {
            huge.push_str(&format!("    acc += {i};\n"));
        }
        huge.push_str("    acc\n}\n");
    }
    write_module(root, "huge", &huge, &mut modules);

    // Sorting and binary search, the two shapes `analyze big-o`'s pattern
    // matcher looks for by name. Without them `pattern_matches[]` was empty on
    // every corpus — an empty list is not evidence that the matcher runs.
    // These also populate the O(log n) / O(n log n) buckets the corpus
    // previously could not reach.
    let searching = "\
/// Sorts, then searches — the two patterns the matcher names.
pub fn lookup(mut values: Vec<i64>, needle: i64) -> Option<usize> {
    values.sort();
    values.binary_search(&needle).ok()
}

/// Exponential recursion: two self-calls per level.
pub fn fib(n: u64) -> u64 {
    if n < 2 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

/// A halving loop — the textbook O(log n) shape.
pub fn locate(values: &[i64], needle: i64) -> usize {
    let mut lo = 0usize;
    let mut hi = values.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if values[mid] < needle {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}
";
    write_module(root, "searching", searching, &mut modules);

    // Findings the `--fail-on-*` flags need in order to be observable AT ALL.
    //
    // `analyze hardcoded-paths --fail-on-any` and `--fail-on-shipped`,
    // `analyze vacuous-tests --fail-on-any` and `analyze unrun-tests
    // --fail-on-any` were all reported as no-ops by the flag-efficacy sweep.
    // They are wired correctly — each reaches a `std::process::exit` — and the
    // corpus simply gave them nothing to fail on: 0 machine-specific paths,
    // 0 vacuous tests, and 0 LIB tests (the corpus's only test lives in
    // tests/basic.rs, an integration target, which is why unrun-tests read
    // "0 of 0 lib tests").
    //
    // A flag that cannot fail because the fixture is clean is indistinguishable
    // from one that cannot fail at all, which is the whole reason this harness
    // exists. One module supplies all three findings:
    let fail_on_fixtures = "\
//! Deliberate findings, so the --fail-on-* flags have something to act on.

/// A machine-specific path in SHIPPED code — `/home/<user>/` with the trailing
/// slash is what `analyze hardcoded-paths` looks for, and being outside a test
/// module is what makes it Site::Shipped rather than Site::Test. That
/// distinction is the difference between --fail-on-any and --fail-on-shipped.
pub fn cache_dir() -> &'static str {
    \"/home/alice/.cache/corpus\"
}

#[cfg(test)]
mod tests {
    /// Vacuous by construction: it executes a line and checks nothing, so it
    /// catches a panic and not a wrong answer. `analyze vacuous-tests` calls
    /// this NoFailureMode.
    #[test]
    fn smoke() {
        let _ = super::cache_dir();
    }

    /// A LIB test behind a feature the corpus's only CI leg does not enable —
    /// its ci.yml runs `cargo test --all`, i.e. default features, and `simd` is
    /// not in `default`. So no leg compiles this, which is precisely what
    /// `analyze unrun-tests` reports.
    #[cfg(feature = \"simd\")]
    #[test]
    fn simd_only() {
        assert_eq!(super::cache_dir().len(), 27);
    }
}
";
    write_module(root, "fail_on_fixtures", fail_on_fixtures, &mut modules);

    // A file that is almost entirely dead, so `quality-gate`'s dead-code check
    // has something to fire on. The `dead_*` family above leaves the corpus at
    // ~3% dead overall with a worst file of ~17.5%, and the check's thresholds
    // are >15% project-wide and >20% per file — so the detector saw 195 dead
    // lines and the gate still reported zero violations, honestly.
    let mut dead_heavy = String::new();
    for i in 0..20 {
        dead_heavy.push_str(&format!(
            "fn buried_{i:02}(seed: usize) -> usize {{\n    let mut total = seed;\n"
        ));
        for k in 1..12 {
            dead_heavy.push_str(&format!(
                "    total = total.wrapping_add(seed.wrapping_mul({k}));\n"
            ));
        }
        dead_heavy.push_str("    total\n}\n\n");
    }
    dead_heavy.push_str("/// The one reachable item in an otherwise dead module.\npub fn surface() -> usize {\n    1\n}\n");
    write_module(root, "dead_heavy", &dead_heavy, &mut modules);

    // Low-provability functions, for `quality-gate`'s provability check.
    //
    // That check scores four properties per function and fires under 0.70, and
    // it is an *average over the first 50 functions* walkdir happens to yield —
    // so a handful of low scorers cannot move it: the corpus averaged 0.92 and
    // reported zero violations honestly. Each function here scores 0.20: a raw
    // pointer costs both nullability and aliasing, `.expect()` with no `?`
    // erases the bounds evidence, and `println!` costs purity. 40 files x 4
    // functions at 0.20 keeps the sampled mean under the floor whichever files
    // the walk reaches first — an earlier version at 0.50 measured 0.72 and the
    // check stayed silent, which is exactly the near-miss a fixture must not
    // sit on.
    //
    // The pointer is null-checked and never dereferenced: `clippy::
    // not_unsafe_ptr_arg_deref` is deny-by-default, and a corpus that fails
    // clippy takes every clippy-backed command down with it.
    //
    // `.expect()` rather than `.unwrap()` is load-bearing. Provability treats
    // them alike (`has_unwrap` matches both), but TDG's critical-defect
    // detector counts only `.unwrap()` — with unwraps here every one of these
    // files would auto-fail to F, the corpus average would fall under 80, and
    // `grade_capped` (which needs a good average *and* an F file) could never
    // become true. One family must not eat another's axis.
    for i in 0..40 {
        let name = format!("unproven_{i:02}");
        let mut body =
            String::from("//! FFI-shaped accessors: raw handles in, panics and stdout out.\n\n");
        for f in 0..4 {
            body.push_str(&format!(
                "/// Unverifiable on all four properties the checker scores.\npub fn emit_{i:02}_{f}(handle: *const u8, raw: &str) -> usize {{\n    if handle.is_null() {{\n        println!(\"null handle {i:02}/{f}\");\n        return 0;\n    }}\n    let head = raw.split(',').next().expect(\"caller guarantees a field\");\n    println!(\"emit {i:02}/{f}: {{head}}\");\n    let width: usize = head.trim().parse().expect(\"caller guarantees a number\");\n    width\n}}\n\n"
            ));
        }
        write_module(root, &name, &body, &mut modules);
    }

    // A hundred-odd unwraps in ONE file, for the Rust Project Score's Known
    // Defects category: it is `20 - 5 * (production_unwraps / 100)`
    // (known_defects_scorer_scoring.rs:13-17), so a corpus with 17 unwraps
    // scores full marks exactly like an empty one. Concentrating them in a
    // single module is deliberate — TDG grades per file, so 110 unwraps here
    // cost one F rather than the thirty-odd that spreading them would, and
    // `grade_capped` still needs the average above 80.
    let mut unwrap_heavy = String::from(
        "/// Parses settings, panicking on every unexpected shape.\npub fn parse_settings(raw: &str) -> Vec<String> {\n    let mut out = Vec::new();\n",
    );
    for i in 0..110 {
        unwrap_heavy.push_str(&format!(
            "    out.push(raw.split('=').nth({}).unwrap().trim().to_string());\n",
            i % 4
        ));
    }
    unwrap_heavy.push_str("    out\n}\n");
    write_module(root, "unwrap_heavy", &unwrap_heavy, &mut modules);

    // Deep nesting and unsafe: the two things the Rust Project Score's Code
    // Quality category actually measures in the mode `pmat score` runs.
    // `score_complexity_simple` counts lines indented past 40 columns and
    // `score_unsafe` counts unsafe blocks against documented ones
    // (code_quality_scoring_heuristics.rs:4-14, 64-76) — with neither in the
    // corpus the category was 100% for an empty crate and for one carrying 59
    // complexity violations alike. Unsafe also moves Formal Verification, whose
    // fast-mode Miri score keys off the unsafe count.
    let mut nested = String::from("/// Nested past the point any linter forgives.\npub fn deeply(a: i64, b: i64) -> i64 {\n    let mut acc = 0i64;\n");
    for level in 1..=12 {
        nested.push_str(&" ".repeat(level * 4));
        nested.push_str(&format!("if a > {level} {{\n"));
    }
    for k in 0..30 {
        nested.push_str(&" ".repeat(13 * 4));
        nested.push_str(&format!("acc += b + {k};\n"));
    }
    for level in (1..=12).rev() {
        nested.push_str(&" ".repeat(level * 4));
        nested.push_str("}\n");
    }
    nested.push_str("    acc\n}\n");
    write_module(root, "nested", &nested, &mut modules);

    let ffi = "\
//! Raw-pointer helpers, one documented and one not.

/// Reads the first byte behind a caller-supplied pointer.
///
/// # Safety
/// `ptr` must be valid for reads of one byte.
pub unsafe fn first_byte(ptr: *const u8) -> u8 {
    *ptr
}

/// Copies the common prefix of two slices.
pub fn copy_prefix(src: &[u8], dst: &mut [u8]) {
    let n = src.len().min(dst.len());
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), n);
    }
}
";
    write_module(root, "ffi", ffi, &mut modules);

    // Sources in languages this build has no TDG analyser for, so
    // `ungraded_files[]` — the disclosure that some of the tree was not part of
    // the score — has entries to carry. With an all-Rust corpus it was empty
    // everywhere, which looks identical to a disclosure that is never emitted.
    std::fs::write(
        root.join("src/deploy.sh"),
        "#!/bin/sh\nset -eu\n# TODO: this deploy script is not covered by any gate\ncargo build --release\n",
    )
    .expect("write deploy.sh");
    std::fs::write(
        root.join("src/mod.zig"),
        "pub fn add(a: i32, b: i32) i32 {\n    return a + b;\n}\n",
    )
    .expect("write mod.zig");

    // A file that is not valid Rust, so `files_not_analyzed` counts something.
    //
    // Deliberately *not* declared in lib.rs: rustc never sees it, so `cargo
    // check` still succeeds and every clippy-backed command keeps working,
    // while the AST walkers that read the directory rather than the module
    // tree still have to fail on it.
    std::fs::write(
        root.join("src/broken_syntax.rs"),
        "pub fn broken( { this is not rust ,,, ]\nfn ) ( unbalanced {{{\n",
    )
    .expect("write broken_syntax.rs");

    // Duplication *inside lib.rs itself*. `file_statistics` is keyed by path,
    // and `./src/lib.rs` is the one file every corpus has, so it is the only
    // duplication row the three can be compared on — and it was 0 everywhere
    // because the corpus put its clones in dup_a_*/dup_b_* instead.
    let dup_body = "\
    /// Normalises a record.
    pub fn normalise(record: &str) -> String {
        let trimmed = record.trim();
        let lowered = trimmed.to_lowercase();
        let collapsed = lowered.split_whitespace().collect::<Vec<_>>().join(\" \");
        let stripped = collapsed.replace(['\\'', '\"'], \"\");
        if stripped.is_empty() {
            return String::from(\"<blank>\");
        }
        format!(\"{}:{}\", stripped.len(), stripped)
    }
";
    let lib = modules
        .iter()
        .map(|m| format!("pub mod {m};"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(
        root.join("src/lib.rs"),
        format!(
            "//! Defect-rich fixture.\n\n{lib}\n\npub mod inline_dup_a {{\n{dup_body}}}\n\npub mod inline_dup_b {{\n{dup_body}}}\n"
        ),
    )
    .expect("write lib.rs");

    // Two files that exist only to be DECLINED, each carrying debt so that
    // declining them is observable.
    //
    // `analyze satd` reports `files_not_read` broken down by reason, and two of
    // those buckets read 0 for every corpus — not because the counters are
    // broken (both fire: an `examples/` file books `out_of_scope`, a `.min.`
    // file books `minified_or_vendor`, verified one file at a time) but because
    // no corpus contained anything for them to count. A bucket that is 0 on
    // every input is indistinguishable from a bucket nothing can ever reach,
    // which is the very thing this harness exists to catch.
    //
    // Neither is declared in `lib.rs`: `bundled.min.rs` is not a valid module
    // name, and `examples/demo.rs` is cargo's own target layout. Both are still
    // walked by the analysers, which is the point.
    std::fs::write(
        root.join("src/bundled.min.rs"),
        "// TODO: vendored bundle, not ours to fix
// FIXME: regenerate from upstream
pub fn bundled_entry() -> u32 {
    7
}
",
    )
    .expect("write bundled.min.rs");
    std::fs::create_dir_all(root.join("examples")).expect("mkdir examples");
    std::fs::write(
        root.join("examples/demo.rs"),
        "//! Example target: excluded from production scope, so it books
//! `out_of_scope` rather than being read.

// TODO: the example still uses the old API
// HACK: hardcoded so the demo always succeeds

fn main() {
    println!(\"demo\");
}
",
    )
    .expect("write examples/demo.rs");

    // A test file, so test-aware analyses have something to find — carrying
    // debt of its own, because a flag whose job is to *include test files*
    // ("--include-tests") can only be observed when the test files contain
    // something to include. With a clean test file it changed nothing, which
    // reads exactly like a flag that is never read.
    std::fs::create_dir_all(root.join("tests")).expect("mkdir tests");
    std::fs::write(
        root.join("tests/basic.rs"),
        "// TODO: this test only covers the happy path\n// FIXME: assert on the error branch too\n\n#[test]\nfn entry_is_ok() {\n    assert_eq!(corpus::dead_00::entry_00(), \"ok\");\n}\n",
    )
    .expect("write test");

    write_wasm_fixtures(root);
    write_assemblyscript_fixtures(root);
    write_model_fixtures(root);
}

/// WebAssembly inputs, so `analyze web-assembly` has something to analyse.
///
/// Without these the command reports "Found 0 WebAssembly files" and every one
/// of its eight flags compares equal for the only reason a fixture can produce:
/// there was nothing on disk for any of them to act on. A no-op verdict drawn
/// from an empty room says nothing about the flag.
///
/// `mod.wasm` is a hand-assembled module (type / import / function / memory /
/// export / code sections) so the binary reader has real sections to report;
/// `broken.wasm` carries deliberately wrong magic, which is the input the
/// format validator exists to reject.
fn write_wasm_fixtures(root: &Path) {
    std::fs::write(
        root.join("mod.wat"),
        "(module\n  (func $t (result i32)\n    (local $i i32)\n    i32.const 42))\n",
    )
    .expect("write mod.wat");
    std::fs::write(
        root.join("two.wat"),
        "(module (func $a) (func $b) (memory 1) (export \"a\" (func $a)))\n",
    )
    .expect("write two.wat");

    // \0asm + version 1, then: type(1), import(2), function(3), memory(5),
    // export(7), code(10) — one function returning i32 const 42.
    #[rustfmt::skip]
    const MOD_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
        0x02, 0x09, 0x01, 0x03, b'e', b'n', b'v', 0x01, b'f', 0x00, 0x00,
        0x03, 0x02, 0x01, 0x00,
        0x05, 0x03, 0x01, 0x00, 0x02,
        0x07, 0x05, 0x01, 0x01, b'e', 0x00, 0x01,
        0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x2a, 0x0b,
    ];
    std::fs::write(root.join("mod.wasm"), MOD_WASM).expect("write mod.wasm");

    // A SECOND reportable binary, because a ranking flag needs a ranking to cut.
    // Of the four wasm files above, the two `.wat` are parsed but deliberately
    // not reported and `broken.wasm` fails validation, so the report had exactly
    // one row and `--top-files 1` compared equal to no limit at all — a fixture
    // artifact the sweep read as an inert flag. Deliberately unlike `mod.wasm`
    // (two functions, no import, one memory page against one/import/two pages)
    // so the two rows are distinguishable in every field the report prints.
    #[rustfmt::skip]
    const SMALL_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
        0x03, 0x03, 0x02, 0x00, 0x00,
        0x05, 0x03, 0x01, 0x00, 0x01,
        0x07, 0x05, 0x01, 0x01, b'a', 0x00, 0x00,
        0x0a, 0x07, 0x02, 0x02, 0x00, 0x0b, 0x02, 0x00, 0x0b,
    ];
    std::fs::write(root.join("small.wasm"), SMALL_WASM).expect("write small.wasm");

    let mut broken = b"NOTWASM!".to_vec();
    broken.resize(40, 0);
    std::fs::write(root.join("broken.wasm"), &broken).expect("write broken.wasm");

    // Ten more valid modules, because the report's `files_truncated` disclosure
    // can only become true past `--top-files`, which defaults to 10. With two
    // reportable binaries it was false for every corpus — a truncation flag
    // that never truncates is indistinguishable from one nobody reads.
    std::fs::create_dir_all(root.join("wasm")).expect("mkdir wasm");
    for i in 0..10 {
        std::fs::write(root.join(format!("wasm/mod_{i:02}.wasm")), SMALL_WASM)
            .expect("write extra wasm module");
    }
}

/// AssemblyScript sources, so `analyze assembly-script` finds three files.
///
/// `assembly/mem.ts` uses `memory.grow`, which is the construct the
/// memory-analysis flag claims to report on — the fixture must contain the
/// thing a flag is documented to find, or its silence is the fixture's.
fn write_assemblyscript_fixtures(root: &Path) {
    let add = "export function add(a: i32, b: i32): i32 {\n  let s: i32 = 0;\n  for (let i: i32 = 0; i < a; i++) { s += b; }\n  return s;\n}\n";
    std::fs::create_dir_all(root.join("assembly")).expect("mkdir assembly");
    std::fs::write(root.join("assembly/index.ts"), add).expect("write index.ts");
    std::fs::write(
        root.join("assembly/mem.ts"),
        "@inline\nexport function g(): f64 { memory.grow(1); return 1.0; }\n",
    )
    .expect("write mem.ts");
    std::fs::write(root.join("extra.as"), add).expect("write extra.as");

    // Past `--top-files`' default of 10, for the same reason the wasm family
    // gets ten more modules: `files_truncated` has no other axis.
    for i in 0..9 {
        std::fs::write(
            root.join(format!("assembly/mod_{i:02}.ts")),
            format!("export function scale_{i:02}(a: i32): i32 {{\n  let s: i32 = 0;\n  for (let i: i32 = 0; i < a; i++) {{ s += {i}; }}\n  return s;\n}}\n"),
        )
        .expect("write extra assemblyscript module");
    }
}

/// Model files, so `analyze models` has an inventory to report.
///
/// On a corpus with none, the handler returns at "No model files found" before
/// it ever reaches `--check`, so the flag was unreachable rather than inert.
/// The set is chosen to trip three distinct validations: `garbage.gguf`
/// declares GGUF in its extension and does not parse (unreadable header), the
/// directory has model files and no model card, and it holds a GGUF with no
/// tokenizer beside it.
fn write_model_fixtures(root: &Path) {
    let models = root.join("models");
    std::fs::create_dir_all(&models).expect("mkdir models");

    // GGUF: magic, u32 version, u64 tensor count, u64 metadata count.
    let mut gguf = b"GGUF".to_vec();
    gguf.extend_from_slice(&3u32.to_le_bytes());
    gguf.extend_from_slice(&7u64.to_le_bytes());
    gguf.extend_from_slice(&2u64.to_le_bytes());
    gguf.resize(88, 0);
    std::fs::write(models.join("tiny.gguf"), &gguf).expect("write tiny.gguf");

    // APR: magic, u32 metadata length, then that many bytes of JSON.
    let apr_meta = br#"{"tensors":[{"name":"w1"},{"name":"w2"}]}"#;
    let mut apr = b"APR2".to_vec();
    apr.extend_from_slice(&(apr_meta.len() as u32).to_le_bytes());
    apr.extend_from_slice(apr_meta);
    apr.resize(65, 0);
    apr.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
    std::fs::write(models.join("tiny.apr"), &apr).expect("write tiny.apr");

    // safetensors: u64 header length, then that many bytes of JSON header.
    let st_header = br#"{"w":{"dtype":"F32","shape":[2,2],"data_offsets":[0,16]}}"#;
    let mut st = (st_header.len() as u64).to_le_bytes().to_vec();
    st.extend_from_slice(st_header);
    st.resize(st.len() + 16, 0);
    std::fs::write(models.join("tiny.safetensors"), &st).expect("write tiny.safetensors");

    let mut garbage = b"NOTAGGUFHEADER!!".to_vec();
    garbage.resize(32, 0);
    std::fs::write(models.join("garbage.gguf"), &garbage).expect("write garbage.gguf");
}

/// One file the defect predictor must rank Critical, left uncommitted.
///
/// `analyze comprehensive` only emits its "Focus on N high-risk files"
/// recommendation when defect-prediction produces a High/Critical file, and
/// that recommendation is the sole output `--confidence-threshold` and
/// `--min-lines` filter. Without such a file both flags filtered an empty set
/// at every value and the sweep called them decoration.
///
/// It is written *after* `git_init` on purpose: an uncommitted file has no
/// churn history, which keeps the prediction's confidence below the top of the
/// range so a confidence threshold has a boundary to move across. Its length is
/// held just under 1000 lines for the same reason — `--min-lines` needs a value
/// that admits it and one that does not.
fn write_critical_risk_file(root: &Path) {
    const IMPORTS: &[&str] = &[
        "std::collections::HashMap",
        "std::collections::HashSet",
        "std::collections::BTreeMap",
        "std::collections::BTreeSet",
        "std::collections::VecDeque",
        "std::collections::BinaryHeap",
        "std::cell::RefCell",
        "std::cmp::Ordering",
        "std::env",
        "std::ffi::OsString",
        "std::fmt::Debug",
        "std::fs::File",
        "std::io::Read",
        "std::io::Write",
        "std::num::NonZeroUsize",
        "std::ops::Range",
        "std::path::Path",
        "std::path::PathBuf",
        "std::process::Command",
        "std::rc::Rc",
        "std::sync::Arc",
        "std::sync::Mutex",
        "std::sync::RwLock",
        "std::time::Duration",
        "std::time::Instant",
    ];
    let mut body =
        String::from("//! Critical-risk fixture: long, branch-heavy and import-heavy.\n");
    for i in IMPORTS {
        body.push_str(&format!("#[allow(unused_imports)]\nuse {i};\n"));
    }
    body.push_str(
        "\n// TODO: this module is the deliberate worst case\n// FIXME: nothing here is validated\n\n/// Pathological decision tree.\npub fn triage(a: i64, b: i64, c: i64, d: i64, mode: u8) -> i64 {\n    let mut acc = 0i64;\n",
    );
    for i in 0..100 {
        body.push_str(&format!(
            "    if a > {i} && mode != {i} {{\n        if b < {i} {{\n            acc += a * {i};\n        }} else if c > {i} {{\n            acc -= b;\n        }} else {{\n            acc ^= d;\n        }}\n    }}\n"
        ));
    }
    body.push_str("    acc\n}\n");
    // Pad to just under the 1000-line mark the --min-lines probe straddles.
    let mut n = body.lines().count();
    body.push_str("\n/// Padding so the file sits just under 1000 lines.\npub fn ballast() -> i64 {\n    let mut acc = 0i64;\n");
    n += 4;
    while n < 985 {
        body.push_str(&format!("    acc += {n};\n"));
        n += 1;
    }
    body.push_str("    acc\n}\n");
    std::fs::write(root.join("src/critical.rs"), body).expect("write critical.rs");
}

fn write_module(root: &Path, name: &str, body: &str, modules: &mut Vec<String>) {
    std::fs::write(root.join(format!("src/{name}.rs")), body).expect("write module");
    modules.push(name.to_string());
}

/// Commit the corpus with pinned identity and dates, and with the user's git
/// hooks disarmed.
///
/// Two traps, both of which silently produced a commit-less repository on the
/// first run of this harness:
///
/// * `init.templateDir` copies the user's hooks into every `git init`, so the
///   corpus inherited pmat's own pre-commit gate — which ran, failed, and
///   aborted the commit.
/// * Without `--no-verify` any surviving hook can do the same.
///
/// A commit-less corpus reports `total_commits: 0`, which the differential
/// gate then attributes to `analyze churn` as a defect. The fixture must not
/// manufacture the findings it is used to judge.
///
/// Commit dates are *relative* ("10 days ago"), not absolute. An earlier
/// version pinned them to a fixed calendar date for reproducibility, which put
/// every commit outside `analyze churn`'s 30-day default window: churn
/// correctly reported `total_commits: 0`, and the gate booked that as a defect
/// in pmat. Relative dates keep the corpus inside every default window while
/// staying identical across the three corpora built in one run.
///
/// The large corpus gets a second, more recent commit touching a subset of
/// files, so history-based metrics have a distribution rather than a point.
fn git_init(root: &Path, size: CorpusSize) {
    // Hook lookup is redirected here rather than to the user's global path.
    let nohooks = root.join(".corpus-nohooks");
    std::fs::create_dir_all(&nohooks).expect("mkdir nohooks");
    let hooks_arg = format!("core.hooksPath={}", nohooks.display());

    let git = |args: &[&str]| -> bool {
        let mut full = vec!["-c", hooks_arg.as_str()];
        full.extend_from_slice(args);
        Command::new("git")
            .args(&full)
            .current_dir(root)
            .env("GIT_AUTHOR_NAME", "Corpus Fixture")
            .env("GIT_AUTHOR_EMAIL", "corpus@example.invalid")
            .env("GIT_COMMITTER_NAME", "Corpus Fixture")
            .env("GIT_COMMITTER_EMAIL", "corpus@example.invalid")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    let git_at = |when: &str, args: &[&str]| -> bool {
        let mut full = vec!["-c", hooks_arg.as_str()];
        full.extend_from_slice(args);
        Command::new("git")
            .args(&full)
            .current_dir(root)
            .env("GIT_AUTHOR_NAME", "Corpus Fixture")
            .env("GIT_AUTHOR_EMAIL", "corpus@example.invalid")
            .env("GIT_COMMITTER_NAME", "Corpus Fixture")
            .env("GIT_COMMITTER_EMAIL", "corpus@example.invalid")
            .env("GIT_AUTHOR_DATE", when)
            .env("GIT_COMMITTER_DATE", when)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };

    // `--template=` with an empty value stops the user's hook templates from
    // being copied in at all.
    git(&["init", "--quiet", "--template=", "--initial-branch", "main"]);
    // ...and it also stops git creating `.git/hooks`, which several commands
    // treat as the definition of "is this a git repository": `comply enforce`
    // aborted with "Error: Not a git repository (no .git/hooks directory)" on
    // every invocation, so its flags were compared against a command that
    // never started. The empty template is right; the missing directory is a
    // side effect to undo.
    std::fs::create_dir_all(root.join(".git/hooks")).expect("mkdir .git/hooks");
    // git's date environment variables reject approxidate ("10 days ago" is a
    // `fatal: invalid date format`); the raw `@<epoch> <tz>` form is accepted.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after 1970")
        .as_secs();
    let days_ago = |d: u64| format!("@{} +0000", now.saturating_sub(d * 86_400));

    git(&["add", "-A"]);
    git_at(
        &days_ago(10),
        &["commit", "--quiet", "--no-verify", "-m", "corpus: initial"],
    );

    if size == CorpusSize::Large {
        // The second commit lands on a branch, leaving `main` behind it.
        //
        // Committing both revisions onto `main` left `main` == `HEAD`, so
        // `analyze incremental-coverage` — which defaults to `--base-branch
        // main` — analysed 0 changed files and every flag under it was
        // compared against an empty result set. A limit cannot truncate a list
        // that does not exist.
        git(&["checkout", "--quiet", "-b", "feature"]);
        // A second, more recent commit touching a subset of files, so churn
        // has a distribution rather than a single flat point.
        for i in 0..8 {
            let p = root.join(format!("src/complex_{i:02}.rs"));
            if let Ok(mut body) = std::fs::read_to_string(&p) {
                body.push_str("\n// revised\n");
                let _ = std::fs::write(&p, body);
            }
        }
        git(&["add", "-A"]);
        git_at(
            &days_ago(2),
            &[
                "commit",
                "--quiet",
                "--no-verify",
                "-m",
                "corpus: revise hot files",
            ],
        );

        // Five more commits, each touching the *same pair* of files.
        //
        // Two leaves needed this and neither is about volume. `analyze
        // bottleneck` reports a file only once its churn ratio clears a
        // threshold and reports a coupling only for files that change
        // *together*, so a history of two commits gave it an empty
        // `bottlenecks[]` and `couplings[]` on every corpus. `analyze churn`'s
        // `summary.stable_files[]` is the complement: with every file touched
        // in the same fraction of history nothing is quiet enough to be
        // stable, so that list was empty too. One hot pair against ~180
        // untouched files produces both.
        //
        // Ten, not five, because `stable_files` has an arithmetic floor:
        // `churn_score = 0.6 * commits/max_commits + 0.4 * changes/max_changes`
        // and the list takes only files under 0.10 (git_analysis.rs:404-410).
        // A file touched once needs `max_commits > 6` before 0.6/max_commits
        // even fits under the threshold, so a shorter history cannot produce a
        // stable file no matter how quiet the file is.
        for c in 0..10 {
            for name in ["chain_00", "chain_01"] {
                let p = root.join(format!("src/{name}.rs"));
                if let Ok(mut body) = std::fs::read_to_string(&p) {
                    body.push_str(&format!("// churn revision {c}\n"));
                    let _ = std::fs::write(&p, body);
                }
            }
            git(&["add", "-A"]);
            git_at(
                &days_ago(2),
                &[
                    "commit",
                    "--quiet",
                    "--no-verify",
                    "-m",
                    &format!("corpus: churn the hot pair ({c})"),
                ],
            );
        }
    }
}

/// A one-line description of what a corpus actually contains.
///
/// Printed in every report. Without it a finding cannot be tied to the fixture
/// that produced it: one sweep reported `analyze dead-code` as wholly inert
/// because a concurrent rebuild left it running against a corpus whose dead
/// functions were still too short to clear `--min-dead-lines`, and there was
/// no way to tell that from the report alone.
pub(crate) fn corpus_fingerprint(root: &Path) -> String {
    let src_files = std::fs::read_dir(root.join("src"))
        .map(|r| r.count())
        .unwrap_or(0);
    let total_bytes: u64 = std::fs::read_dir(root.join("src"))
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| e.metadata().ok().map(|m| m.len()))
                .sum()
        })
        .unwrap_or(0);
    format!(
        "{src_files} src files, {total_bytes} bytes, {} commits",
        commit_count(root)
    )
}

/// How many commits the corpus actually has.
pub(crate) fn commit_count(root: &Path) -> usize {
    Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(root)
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Help-tree walking
// ---------------------------------------------------------------------------

/// One option as the shipped binary describes it.
#[derive(Debug, Clone)]
pub(crate) struct HelpFlag {
    /// The long form, including leading dashes.
    pub(crate) long: String,
    /// `None` for a boolean switch; otherwise the enumerated legal values, if
    /// clap printed them.
    pub(crate) values: Option<Vec<String>>,
    /// True when the option takes a value clap did not enumerate.
    pub(crate) takes_free_value: bool,
}

/// Parse `--help` output into subcommands and flags.
///
/// Reading the *binary's own help* rather than the clap types is deliberate:
/// it is what the artifact actually exposes, it needs no crate-internal
/// access, and it works unchanged against an installed 3.29.0 for A/B runs.
pub(crate) fn parse_help(help: &str) -> (Vec<String>, Vec<HelpFlag>) {
    let mut subcommands = Vec::new();
    let mut flags = Vec::new();
    let mut section = "";
    let mut opts = OptionsReader::default();

    for line in help.lines() {
        let trimmed = line.trim_end();
        let lower = trimmed.trim().to_lowercase();
        if lower.ends_with("commands:") {
            opts.flush(&mut flags);
            section = "commands";
            continue;
        }
        if lower.ends_with("options:") || lower.ends_with("arguments:") {
            opts.flush(&mut flags);
            section = if lower.ends_with("options:") {
                "options"
            } else {
                "arguments"
            };
            continue;
        }
        if trimmed.trim().is_empty() {
            continue;
        }
        // Section bodies are indented; anything flush-left starts a new block.
        if !line.starts_with(' ') {
            opts.flush(&mut flags);
            section = "";
            continue;
        }

        match section {
            "commands" => {
                if let Some(name) = trimmed.split_whitespace().next() {
                    if name != "help" && !name.starts_with('-') {
                        subcommands.push(name.to_string());
                    }
                }
            }
            "options" => opts.feed(line, &mut flags),
            _ => {}
        }
    }
    opts.flush(&mut flags);
    (subcommands, flags)
}

/// Stateful reader for the `Options:` section.
///
/// clap emits two layouts and the harness must read both. Compact (`-h`):
///
/// ```text
///   --format <FORMAT>  Output format [possible values: summary, json]
/// ```
///
/// Expanded (`--help`), which is what this harness requests:
///
/// ```text
///       --format <FORMAT>
///           Output format
///
///           Possible values:
///           - summary: Summary statistics only
///           - json:    Machine-readable
/// ```
///
/// Reading only the flag's own line — as the first version of this parser did
/// — finds zero enumerated values in the expanded layout, so every `--format`
/// silently became "needs a value the harness cannot synthesise" and the sweep
/// reported nothing while exiting 0.
#[derive(Default)]
struct OptionsReader {
    pending: Option<HelpFlag>,
    collecting_values: bool,
}

impl OptionsReader {
    /// A flag declaration sits at shallow indent and names a long option;
    /// `- json: ...` value bullets sit deeper and never do.
    fn is_flag_line(line: &str) -> bool {
        let indent = line.len() - line.trim_start().len();
        let t = line.trim_start();
        indent <= 8 && t.starts_with('-') && t.contains("--")
    }

    fn feed(&mut self, line: &str, out: &mut Vec<HelpFlag>) {
        if Self::is_flag_line(line) {
            self.flush(out);
            self.collecting_values = false;
            self.pending = Self::start_flag(line.trim());
            // The compact layout carries everything on this one line.
            if let Some(f) = self.pending.as_mut() {
                if let Some(v) = Self::compact_values(line) {
                    f.values = Some(v);
                    f.takes_free_value = false;
                }
            }
            return;
        }

        let Some(flag) = self.pending.as_mut() else {
            return;
        };
        let t = line.trim();

        if let Some(v) = Self::compact_values(line) {
            flag.values = Some(v);
            flag.takes_free_value = false;
            return;
        }
        if t.eq_ignore_ascii_case("possible values:") {
            self.collecting_values = true;
            flag.values = Some(Vec::new());
            flag.takes_free_value = false;
            return;
        }
        if self.collecting_values {
            if let Some(rest) = t.strip_prefix("- ") {
                let name = rest.split(':').next().unwrap_or(rest).trim();
                if !name.is_empty() {
                    flag.values
                        .get_or_insert_with(Vec::new)
                        .push(name.to_string());
                }
            } else if !t.is_empty() {
                self.collecting_values = false;
            }
            return;
        }
        // `[default: .]` only appears on options that take a value.
        if t.starts_with("[default:") && flag.values.is_none() {
            flag.takes_free_value = true;
        }
    }

    fn flush(&mut self, out: &mut Vec<HelpFlag>) {
        if let Some(mut f) = self.pending.take() {
            if f.values.as_ref().is_some_and(|v| v.is_empty()) {
                // "Possible values:" announced but none parsed — do not
                // silently downgrade to a boolean switch and mis-test it.
                f.values = None;
                f.takes_free_value = true;
            }
            out.push(f);
        }
    }

    fn start_flag(t: &str) -> Option<HelpFlag> {
        let long_start = t.find("--")?;
        let rest = &t[long_start..];
        let long: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if long.len() <= 2 || matches!(long.as_str(), "--help" | "--version") {
            return None;
        }
        let after = rest[long.len()..].trim_start();
        Some(HelpFlag {
            long,
            values: None,
            takes_free_value: after.starts_with('<') || after.starts_with('='),
        })
    }

    fn compact_values(line: &str) -> Option<Vec<String>> {
        let i = line.find("[possible values:")?;
        let tail = &line[i + "[possible values:".len()..];
        let end = tail.find(']').unwrap_or(tail.len());
        let v: Vec<String> = tail[..end]
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();
        (!v.is_empty()).then_some(v)
    }
}

/// Ask the binary for help on a command path.
pub(crate) fn help_for(path: &[&str], cwd: &Path) -> Option<String> {
    let mut args: Vec<&str> = path.to_vec();
    args.push("--help");
    let out = run(&args, cwd, Duration::from_secs(20));
    if out.timed_out {
        return None;
    }
    let text = format!("{}{}", out.stdout, out.stderr);
    if text.contains("Usage:") {
        Some(text)
    } else {
        None
    }
}

/// Why a check could not be performed. Recorded and printed, never silently
/// dropped — a skipped check that looks like a pass is the original sin these
/// harnesses exist to prevent.
pub(crate) type SkipReasons = BTreeMap<String, String>;

#[test]
#[ignore = "dev helper: dumps the three corpora to $PMAT_CORPUS_DUMP for hand-probing"]
fn dump_corpora() {
    // No env var ⇒ nothing to dump. Panicking here would break any run of the
    // whole ignored set, which is how the release gate invokes these sweeps.
    let Ok(dir) = std::env::var("PMAT_CORPUS_DUMP") else {
        println!("PMAT_CORPUS_DUMP unset; nothing dumped");
        return;
    };
    let out = std::path::PathBuf::from(dir);
    for size in [CorpusSize::Empty, CorpusSize::Tiny, CorpusSize::Large] {
        let d = build_corpus(size);
        let dest = out.join(size.name());
        let _ = std::fs::remove_dir_all(&dest);
        let st = Command::new("cp")
            .args(["-a".as_ref(), d.path().as_os_str(), dest.as_os_str()])
            .status()
            .expect("cp");
        assert!(st.success());
        println!("{} -> {}", size.name(), dest.display());
    }
}
