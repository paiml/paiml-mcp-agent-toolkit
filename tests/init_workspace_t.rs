//! End-to-end gate for `pmat init` (#1030, #1032).
//!
//! The unit tests in `src/services/workspace_init/tests.rs` assert what the
//! generator *writes*. This file asserts the thing that actually motivated the
//! ticket: that the MCP registration it writes **works** — by reading the
//! emitted JSON, spawning the command it names with the arguments it names,
//! and requiring a valid JSON-RPC `initialize` reply.
//!
//! The template this repository shipped before `pmat init` existed named
//! `cargo run --bin pmat -- serve --transport stdio`. It parsed fine, it read
//! plausibly, and it had never been executed: `--transport stdio` is a clap
//! parse error and `cargo run` cannot start outside a Cargo workspace. A test
//! that only checked "a file was written" would have passed on it, which is
//! why `broken_invocation_is_rejected_by_this_same_probe` exists — it runs the
//! old invocation through the identical probe and requires it to FAIL. Without
//! that control, a probe that accidentally passed on anything would look like
//! proof.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The binary Cargo built for this test — never a `pmat` that happens to be
/// installed, which could be any older version.
const PMAT: &str = env!("CARGO_BIN_EXE_pmat");

fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn run_init(root: &Path, args: &[&str]) -> (bool, String, String) {
    let out = Command::new(PMAT)
        .arg("init")
        .args(args)
        .arg("--path")
        .arg(root)
        .output()
        .expect("spawn pmat init");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Send `initialize` to `binary argv…` over stdio and return the first line of
/// stdout, if any. `None` means the process produced nothing usable — which is
/// exactly what the broken invocation does.
fn probe_mcp(binary: &Path, argv: &[String]) -> Option<serde_json::Value> {
    let mut child = Command::new(binary)
        .args(argv)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let frame = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                   "clientInfo": {"name": "pmat-init-test", "version": "0"}}
    });
    {
        let stdin = child.stdin.as_mut()?;
        writeln!(stdin, "{frame}").ok()?;
        stdin.flush().ok()?;
    }

    let stdout = child.stdout.take()?;
    let mut line = String::new();
    let read = BufReader::new(stdout).read_line(&mut line).ok()?;
    let _ = child.kill();
    let _ = child.wait();
    if read == 0 {
        return None;
    }
    serde_json::from_str(line.trim()).ok()
}

fn emitted_mcp_server(root: &Path, rel: &str) -> (String, Vec<String>) {
    let text = std::fs::read_to_string(root.join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
    let json: serde_json::Value = serde_json::from_str(&text).expect("emitted config must parse");
    let server = &json["mcpServers"]["pmat"];
    let command = server["command"].as_str().expect("command").to_string();
    let args = server["args"]
        .as_array()
        .expect("args")
        .iter()
        .map(|v| v.as_str().expect("arg is a string").to_string())
        .collect();
    (command, args)
}

// ── #1030 claim 1: the command exists ──────────────────────────────────────

#[test]
fn pmat_init_exists_and_documents_its_targets() {
    let out = Command::new(PMAT)
        .args(["init", "--help"])
        .output()
        .expect("spawn");
    assert!(
        out.status.success(),
        "`pmat init --help` failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let help = String::from_utf8_lossy(&out.stdout);
    for expected in [
        "--target",
        "--path",
        "--force",
        "agy",
        "claude",
        "ultracode",
    ] {
        assert!(help.contains(expected), "help omits {expected}:\n{help}");
    }
}

// ── #1030 claims 2 & 4: hook script and root rules file ────────────────────

#[test]
fn init_writes_the_hook_script_executable_and_a_root_rules_file() {
    let dir = tempdir();
    let (ok, stdout, stderr) = run_init(dir.path(), &["--target", "agy"]);
    assert!(ok, "init failed: {stderr}\n{stdout}");

    let hook = dir.path().join(".agents/hooks/pmat-quality-feedback.sh");
    assert!(hook.is_file(), "no pmat-quality-feedback.sh:\n{stdout}");
    assert!(dir.path().join("AGENTS.md").is_file(), "no AGENTS.md");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&hook).expect("stat").permissions().mode();
        assert_eq!(mode & 0o111, 0o111, "hook is not executable");
    }

    // The script must actually run. Both clients treat a hook that fails to
    // launch as an APPROVAL, so a non-executable or non-launching hook is a
    // silent no-op rather than an error anyone would notice.
    let out = Command::new(&hook)
        .arg("antigravity")
        .stdin(Stdio::null())
        .output()
        .expect("the emitted hook must be launchable");
    let decision: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim())
            .expect("antigravity mode must emit parseable JSON on every path");
    assert!(decision["decision"].is_string(), "got {decision}");
}

// ── #1030 claim 3 / #1032 claim 2: the MCP registration actually works ─────

/// The test this whole ticket is about.
///
/// Reads the emitted config, takes the argv verbatim, spawns it, and requires
/// a well-formed JSON-RPC `initialize` result. `command` is asserted to be the
/// installed binary name and then substituted with the binary Cargo just
/// built, so the assertion is about *this* tree rather than about whichever
/// `pmat` happens to be on the runner's PATH.
#[test]
fn emitted_mcp_config_names_a_command_that_actually_speaks_mcp() {
    for (target, rel) in [
        ("agy", ".agents/mcp_config.json"),
        ("claude", ".mcp.json"),
        ("ultracode", ".mcp.json"),
    ] {
        let dir = tempdir();
        let (ok, stdout, stderr) = run_init(dir.path(), &["--target", target]);
        assert!(ok, "{target}: init failed: {stderr}\n{stdout}");

        let (command, args) = emitted_mcp_server(dir.path(), rel);
        assert_eq!(
            command, "pmat",
            "{target}: config must name the installed binary, not a build tool"
        );

        let reply = probe_mcp(Path::new(PMAT), &args).unwrap_or_else(|| {
            panic!(
                "{target}: `{command} {}` produced no JSON-RPC at all",
                args.join(" ")
            )
        });
        assert_eq!(reply["jsonrpc"], "2.0", "{target}: {reply}");
        assert_eq!(reply["id"], 1, "{target}: {reply}");
        assert!(
            reply["result"]["serverInfo"]["name"].is_string(),
            "{target}: initialize returned no serverInfo: {reply}"
        );
        assert!(
            reply["result"]["capabilities"]["tools"].is_object(),
            "{target}: server advertises no tools: {reply}"
        );
    }
}

/// The control. The invocation the old committed template named must FAIL this
/// same probe — otherwise the probe proves nothing about the new one.
#[test]
fn broken_invocation_is_rejected_by_this_same_probe() {
    let old: Vec<String> = ["serve", "--transport", "stdio"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert!(
        probe_mcp(Path::new(PMAT), &old).is_none(),
        "`pmat serve --transport stdio` answered the probe — if that ever starts \
         working, re-check what the emitted config should say"
    );
}

// ── idempotence ────────────────────────────────────────────────────────────

#[test]
fn a_second_run_writes_nothing_and_says_so() {
    let dir = tempdir();
    let (ok, _, err) = run_init(dir.path(), &["--target", "ultracode"]);
    assert!(ok, "first run failed: {err}");

    let before = snapshot(dir.path());

    let (ok, stdout, err) = run_init(dir.path(), &["--target", "ultracode", "--format", "json"]);
    assert!(ok, "second run failed: {err}");
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("json report");
    assert_eq!(json["summary"]["written"], 0, "second run wrote files");
    assert_eq!(json["summary"]["kept"], 0);
    assert!(
        json["summary"]["already_current"].as_u64().unwrap_or(0) > 0,
        "second run must report the files as already current: {json}"
    );

    assert_eq!(before, snapshot(dir.path()), "second run mutated bytes");
}

#[test]
fn a_hand_edited_file_survives_and_the_report_says_why() {
    let dir = tempdir();
    let (ok, _, err) = run_init(dir.path(), &["--target", "agy"]);
    assert!(ok, "first run failed: {err}");

    let mine = "{\n  \"mine\": true\n}\n";
    std::fs::write(dir.path().join(".agents/hooks.json"), mine).expect("edit");

    let (ok, stdout, err) = run_init(dir.path(), &["--target", "agy", "--format", "json"]);
    assert!(ok, "run over an edited workspace failed: {err}");
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".agents/hooks.json")).expect("read"),
        mine,
        "pmat init destroyed a hand-edited file"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("json report");
    assert_eq!(json["summary"]["kept"], 1, "{json}");

    // --force replaces it, and only then.
    let (ok, _, err) = run_init(dir.path(), &["--target", "agy", "--force"]);
    assert!(ok, "forced run failed: {err}");
    assert_ne!(
        std::fs::read_to_string(dir.path().join(".agents/hooks.json")).expect("read"),
        mine,
        "--force did not replace the file"
    );
}

// ── #1032: the generated ultracode workflow is real ESM ────────────────────

/// `node --check` is the authority the Makefile's `release-sweep` target uses
/// on the committed workflow; the generated one is held to the same bar.
///
/// When `node` is absent the test does not quietly pass: it falls back to the
/// structural invariants `qa_mcp_sweep::tests::workflow_*` assert for the
/// committed ground truth, and both paths announce which one ran.
#[test]
fn generated_ultracode_workflow_is_valid_esm() {
    let dir = tempdir();
    let (ok, _, err) = run_init(dir.path(), &["--target", "ultracode"]);
    assert!(ok, "init failed: {err}");
    let wf = dir
        .path()
        .join("contracts/workflows/pmat-quality-sweep.ultracode.mjs");
    assert!(wf.is_file(), "no workflow generated");

    match Command::new("node").arg("--check").arg(&wf).output() {
        Ok(out) => {
            assert!(
                out.status.success(),
                "node --check rejected the generated workflow:\n{}",
                String::from_utf8_lossy(&out.stderr)
            );
            eprintln!("checked with node --check");
        }
        Err(_) => {
            let src = std::fs::read_to_string(&wf).expect("read");
            assert_eq!(src.matches("readFileSync(").count(), 1);
            assert_eq!(src.matches("spawnSubagent(").count(), 1);
            assert!(src.contains("main().catch("));
            assert!(src.contains("work event --type refusal"));
            assert!(src.len() > 500);
            eprintln!("node unavailable — checked structural invariants instead");
        }
    }
}

// ── refusals are visible, not silent ───────────────────────────────────────

#[test]
fn undefined_artifacts_are_refused_in_full_not_omitted() {
    for (target, needle) in [("agy", "plugins.json"), ("ultracode", "ultracode schema")] {
        let dir = tempdir();
        let (ok, stdout, err) = run_init(dir.path(), &["--target", target, "--format", "json"]);
        assert!(ok, "{target}: init failed: {err}");
        let json: serde_json::Value = serde_json::from_str(&stdout).expect("json report");
        let refused = json["refused"].as_array().expect("refused[]");
        assert!(
            refused
                .iter()
                .any(|r| r["artifact"].as_str().is_some_and(|a| a.contains(needle))),
            "{target}: {needle} is neither written nor reported: {json}"
        );
        // ...and no file was created for it.
        assert!(
            !dir.path().join(".agents/plugins.json").exists(),
            "{target}: a refused artifact was written anyway"
        );
    }
}

/// Sorted (path, bytes) for every regular file under `root`.
fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read_dir").flatten() {
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p.strip_prefix(root).expect("strip").to_path_buf();
                out.push((rel, std::fs::read(&p).expect("read")));
            }
        }
    }
    out.sort();
    out
}
