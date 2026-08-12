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
            // Git object ids
            (r(r"\b[0-9a-f]{7,40}\b"), "<SHA>"),
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
    out.trim_end().to_string()
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
        // Deterministic output: no colour unless a flag asks, no
        // parallelism-dependent ordering, no user config bleeding in.
        .env("NO_COLOR", "1")
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
    std::fs::write(
        root.join("Makefile"),
        "\
.PHONY: build test lint

build:\n\tcargo build\n
test:\n\tcargo test\n
lint:\n\tcargo clippy -- -D warnings\n",
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
    for i in 0..15 {
        let name = format!("satd_{i:02}");
        let body = format!(
            "// TODO: replace the placeholder below with a real implementation\n// FIXME: this ignores the error path entirely\n// HACK: works only because callers pre-validate\n// XXX: revisit before release\n\n/// Returns a hardcoded answer.\npub fn lookup_{i:02}(_key: &str) -> u32 {{\n    // For now, we know the answer is always this.\n    {}\n}}\n",
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
        let shared = format!(
            "/// Normalises a record.\npub fn normalise(record: &str) -> String {{\n    let trimmed = record.trim();\n    let lowered = trimmed.to_lowercase();\n    let collapsed = lowered.split_whitespace().collect::<Vec<_>>().join(\" \");\n    let stripped = collapsed.replace(['\\'', '\"'], \"\");\n    if stripped.is_empty() {{\n        return String::from(\"<blank>\");\n    }}\n    format!(\"{{}}:{{}}\", stripped.len(), stripped)\n}}\n"
        );
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
        for k in 0..14 {
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
            "struct UnusedRecord{i:02} {{\n    _id: u64,\n    _label: String,\n    _tags: Vec<String>,\n}}\n\n/// The only reachable item here.\npub fn entry_{i:02}() -> &'static str {{\n    \"ok\"\n}}\n"
        ));
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

    // A test file, so test-aware analyses have something to find.
    std::fs::create_dir_all(root.join("tests")).expect("mkdir tests");
    std::fs::write(
        root.join("tests/basic.rs"),
        "#[test]\nfn entry_is_ok() {\n    assert_eq!(corpus::dead_00::entry_00(), \"ok\");\n}\n",
    )
    .expect("write test");
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
        if t.starts_with("[default:") {
            if flag.values.is_none() {
                flag.takes_free_value = true;
            }
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
