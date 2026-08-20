//! #1020: `pmat quality-gate --project-path X` must return the same verdict no
//! matter which directory the process was launched from.
//!
//! It did not. `check_complexity` read its thresholds from the global
//! `configuration()` singleton, which is built once from
//! `std::env::current_dir().join("pmat.toml")`. The identical command against
//! the identical fixture answered:
//!
//! | invoked from        | complexity_violations |
//! |---------------------|-----------------------|
//! | this repo           | 1                     |
//! | `/tmp`              | 2                     |
//! | the fixture dir     | 2                     |
//!
//! — because this repo's `pmat.toml` raises `max_cognitive_complexity` to 100
//! while the built-in default is 25. Deterministic within each directory, so
//! not flakiness: a config-resolution bug. CI and a laptop disagree with nothing
//! in the output to explain why.
//!
//! These tests drive `env!("CARGO_BIN_EXE_pmat")`, the artifact cargo just
//! built, so they cannot pass against a stale binary elsewhere on disk.

use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// A function with cyclomatic ≈ 37 and cognitive ≈ 72: above the built-in
/// ceilings (30 / 25), below this repo's `pmat.toml` cognitive ceiling (100).
/// That gap is exactly what made the verdict depend on the caller's shell.
fn write_fixture(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"cwd-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    let mut body = String::from("pub fn tangled(v: &[i32]) -> i32 {\n    let mut acc = 0;\n");
    for i in 0..12 {
        body.push_str(&format!(
            "    if v.len() > {i} {{\n        for x in v {{\n            \
             if *x > {i} {{ acc += 1; }} else {{ acc -= 1; }}\n        }}\n    }}\n"
        ));
    }
    body.push_str("    acc\n}\n");
    std::fs::write(dir.join("src").join("lib.rs"), body).unwrap();
}

/// Run the gate against `project` with the process CWD set to `cwd`.
/// Returns (exit code, raw stdout bytes).
fn run_gate_from(cwd: &Path, project: &Path) -> (Option<i32>, Vec<u8>) {
    let out = Command::new(env!("CARGO_BIN_EXE_pmat"))
        .args([
            "quality-gate",
            "--project-path",
            project.to_str().unwrap(),
            "--checks",
            "complexity",
            "--format",
            "json",
        ])
        .current_dir(cwd)
        .output()
        .expect("failed to spawn pmat");
    (out.status.code(), out.stdout)
}

/// The three directories the original measurement used: this repo (which has a
/// `pmat.toml`), `/tmp` (which has none), and the fixture itself.
fn probe_dirs(fixture: &Path) -> Vec<(&'static str, PathBuf)> {
    vec![
        ("repo root", PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
        ("/tmp", std::env::temp_dir()),
        ("fixture dir", fixture.to_path_buf()),
    ]
}

fn complexity_violations(stdout: &[u8]) -> i64 {
    let parsed: serde_json::Value =
        serde_json::from_slice(stdout).expect("gate must emit parseable JSON on stdout");
    parsed["results"]["complexity_violations"]
        .as_i64()
        .expect("results.complexity_violations must be present")
}

#[test]
fn test_gate_verdict_is_byte_identical_across_working_directories() {
    let tmp = TempDir::new().unwrap();
    let fixture = tmp.path();
    write_fixture(fixture);

    let mut runs: Vec<(&str, Option<i32>, Vec<u8>)> = Vec::new();
    for (label, cwd) in probe_dirs(fixture) {
        let (code, stdout) = run_gate_from(&cwd, fixture);
        runs.push((label, code, stdout));
    }

    let (base_label, base_code, base_stdout) = &runs[0];
    for (label, code, stdout) in &runs[1..] {
        assert_eq!(
            code, base_code,
            "exit status differs: {base_label} gave {base_code:?}, {label} gave {code:?}"
        );
        assert_eq!(
            String::from_utf8_lossy(stdout),
            String::from_utf8_lossy(base_stdout),
            "gate JSON differs between working directories ({base_label} vs {label}). \
             The verdict must be a function of the analysed tree and the CLI flags, \
             not of the caller's shell."
        );
    }

    // Not vacuous: the fixture really does breach the defaults, so all three
    // runs agreeing on "nothing found" (or on an error) would not satisfy this.
    assert_eq!(
        complexity_violations(base_stdout),
        2,
        "fixture has no pmat.toml, so the built-in ceilings (cyclomatic 30, \
         cognitive 25) apply and `tangled` breaches both"
    );
}

#[test]
fn test_the_analysed_projects_own_config_is_what_moves_the_verdict() {
    // The converse of the test above: pinning the verdict to the project path is
    // only correct if the PROJECT's config is the thing that changes it. Same
    // fixture, same three directories — but now the fixture carries a pmat.toml
    // that lifts both ceilings above what `tangled` scores.
    let tmp = TempDir::new().unwrap();
    let fixture = tmp.path();
    write_fixture(fixture);
    std::fs::write(
        fixture.join("pmat.toml"),
        "[quality]\nmax_complexity = 100\nmax_cognitive_complexity = 100\n",
    )
    .unwrap();

    let mut stdouts: Vec<(&str, Vec<u8>)> = Vec::new();
    for (label, cwd) in probe_dirs(fixture) {
        let (_, stdout) = run_gate_from(&cwd, fixture);
        stdouts.push((label, stdout));
    }

    for (label, stdout) in &stdouts {
        assert_eq!(
            complexity_violations(stdout),
            0,
            "run from {label}: the fixture's own pmat.toml raises both ceilings \
             to 100, so `tangled` (cyclomatic 37, cognitive 72) is clean"
        );
    }
    let (base_label, base_stdout) = &stdouts[0];
    for (label, stdout) in &stdouts[1..] {
        assert_eq!(
            String::from_utf8_lossy(stdout),
            String::from_utf8_lossy(base_stdout),
            "gate JSON differs between working directories ({base_label} vs {label})"
        );
    }
}
