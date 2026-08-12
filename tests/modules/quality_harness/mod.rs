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
        .stdin(std::process::Stdio::null());

    scrub_cargo_env(&mut cmd);

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

    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"corpus\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
    )
    .expect("write Cargo.toml");

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
",
    )
    .expect("write ci.yml");
    std::fs::create_dir_all(root.join("docs")).expect("mkdir docs");
    std::fs::write(
        root.join("docs/architecture.md"),
        "# Architecture\n\nOne crate, many generated modules.\n",
    )
    .expect("write docs");
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
    for i in 0..15 {
        let name = format!("faults_{i:02}");
        let body = format!(
            "/// Parses without handling failure.\npub fn parse_{i:02}(raw: &str) -> i64 {{\n    let first = raw.split(',').next().unwrap();\n    let value: i64 = first.trim().parse().expect(\"caller guarantees a number\");\n    if value < 0 {{\n        panic!(\"negative value: {{value}}\");\n    }}\n    if value > 1_000_000 {{\n        unreachable!(\"validated upstream\");\n    }}\n    value\n}}\n"
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
    for i in 0..6 {
        let name = format!("superlinear_{i:02}");
        let body = format!(
            "/// Quadratic scan.\npub fn pairwise_{i:02}(items: &[i64]) -> i64 {{\n    let mut acc = 0i64;\n    for a in items {{\n        for b in items {{\n            acc += a * b;\n        }}\n    }}\n    acc\n}}\n\n/// Cubic scan.\npub fn triple_{i:02}(items: &[i64]) -> i64 {{\n    let mut acc = 0i64;\n    for a in items {{\n        for b in items {{\n            for c in items {{\n                acc += a + b + c;\n            }}\n        }}\n    }}\n    acc\n}}\n"
        );
        write_module(root, &name, &body, &mut modules);
    }

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

    let lib = modules
        .iter()
        .map(|m| format!("pub mod {m};"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(
        root.join("src/lib.rs"),
        format!("//! Defect-rich fixture.\n\n{lib}\n"),
    )
    .expect("write lib.rs");

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
