//! PMAT-675 — the release path a `v*` tag takes, pinned as data.
//!
//! release.yml.disabled (#382) meant a tag ran no clean-room gate and created
//! no release; 3.38.0 was cut by hand. The restored `.github/workflows/release.yml`
//! runs the fleet gate and a package verification on `[self-hosted, clean-room]`
//! and only then creates a PRERELEASE. No job may publish the crate, carry the
//! registry token, or continue on error. `docker-publish.yml` — red on every
//! tag since v3.32.0 — no longer runs on tags and refuses by name when its
//! secrets are absent.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs` with `#[path]` so it is
//! compiled under CI's `cargo test --lib` (src/tests/lib.rs is an orphan).

use std::path::PathBuf;

fn workflow(name: &str) -> (PathBuf, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".github/workflows")
        .join(name);
    let text = std::fs::read_to_string(&path);
    assert!(
        text.is_ok(),
        "{} must be readable: {:?}",
        path.display(),
        text.as_ref().err()
    );
    (path, text.unwrap_or_default())
}

fn yaml(text: &str) -> serde_yaml_ng::Value {
    serde_yaml_ng::from_str(text).expect("workflow parses as YAML")
}

/// Every `continue-on-error` key anywhere in a YAML document.
fn continue_on_error_keys(value: &serde_yaml_ng::Value, path: &str, out: &mut Vec<String>) {
    match value {
        serde_yaml_ng::Value::Mapping(map) => {
            for (k, v) in map {
                let key = k.as_str().unwrap_or("?").to_string();
                if key == "continue-on-error" {
                    out.push(format!("{path}/{key}"));
                }
                continue_on_error_keys(v, &format!("{path}/{key}"), out);
            }
        }
        serde_yaml_ng::Value::Sequence(items) => {
            for (i, item) in items.iter().enumerate() {
                continue_on_error_keys(item, &format!("{path}[{i}]"), out);
            }
        }
        _ => {}
    }
}

/// `needs:` as a list — GitHub accepts a scalar (`needs: build`) or a sequence.
fn job_needs(doc: &serde_yaml_ng::Value, job: &str) -> Vec<String> {
    match &doc["jobs"][job]["needs"] {
        serde_yaml_ng::Value::String(one) => vec![one.clone()],
        serde_yaml_ng::Value::Sequence(many) => many
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}

#[test]
fn release_workflow_gates_then_verifies_then_prereleases_and_never_publishes() {
    let (_, text) = workflow("release.yml");
    let doc = yaml(&text);

    let jobs: Vec<&str> = doc["jobs"]
        .as_mapping()
        .expect("jobs")
        .keys()
        .filter_map(|k| k.as_str())
        .collect();
    assert_eq!(
        jobs,
        vec!["create-release", "gate", "verify", "prerelease"],
        "exactly these four jobs, in order"
    );
    for deleted in ["publish", "build-binaries", "publish-release"] {
        assert!(!jobs.contains(&deleted), "{deleted} was deleted on purpose (binary-release.yml owns binaries; the crate is published from a worktree of the tag)");
    }

    assert_eq!(job_needs(&doc, "gate"), vec!["create-release"]);
    assert_eq!(job_needs(&doc, "verify"), vec!["create-release", "gate"]);
    assert_eq!(
        job_needs(&doc, "prerelease"),
        vec!["create-release", "gate", "verify"],
        "the prerelease exists only after gate AND verify"
    );

    assert!(
        doc["jobs"]["prerelease"].get("if").is_none(),
        "the prerelease job carries no `if:` — `needs` alone decides, an `if: always()` would prerelease on a red gate"
    );
    assert_eq!(
        doc["jobs"]["gate"]["with"]["pr_sha"].as_str().unwrap_or_default(),
        "${{ needs.create-release.outputs.sha }}",
        "the gate validates the tag's own commit, not github.sha (the dispatch branch on workflow_dispatch)"
    );
    let dispatches = doc["jobs"]["prerelease"]["steps"]
        .as_sequence()
        .map(|s| {
            s.iter()
                .filter_map(|st| st["run"].as_str())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    assert!(
        dispatches.contains("gh workflow run binary-release.yml") && dispatches.contains("gh workflow run post-release.yml"),
        "a GITHUB_TOKEN-created release fires no `release: published` event, so the listeners are dispatched explicitly: {dispatches}"
    );
    let gate_uses = doc["jobs"]["gate"]["uses"].as_str().unwrap_or_default();
    assert!(
        gate_uses.starts_with("paiml/.github/.github/workflows/unified-gate.yml@"),
        "the gate is the fleet clean-room gate: {gate_uses}"
    );
    let verify_on = doc["jobs"]["verify"]["runs-on"]
        .as_sequence()
        .map(|s| s.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();
    assert_eq!(
        verify_on,
        vec!["self-hosted", "clean-room"],
        "verify runs in the clean room"
    );

    let verify_run = doc["jobs"]["verify"]["steps"]
        .as_sequence()
        .and_then(|s| s.iter().find_map(|st| st["run"].as_str()))
        .unwrap_or_default();
    assert!(
        verify_run.contains("cargo package") && verify_run.contains("--locked") && !verify_run.contains("--verify"),
        "verify packages with --locked; `--verify` is not a cargo flag (Usage: cargo package --no-verify) and would fail the job: {verify_run}"
    );
    let mut coe = Vec::new();
    continue_on_error_keys(&doc, "", &mut coe);
    assert!(coe.is_empty(), "no step may continue on error: {coe:?}");
    for forbidden in [
        "cargo publish",
        "CARGO_REGISTRY_TOKEN",
        "crates-io-auth-action",
        "id-token",
    ] {
        assert!(
            !text.contains(forbidden),
            "release.yml must not carry `{forbidden}`"
        );
    }
    let prerelease_run = doc["jobs"]["prerelease"]["steps"]
        .as_sequence()
        .and_then(|s| s.iter().find_map(|st| st["run"].as_str()))
        .unwrap_or_default();
    assert!(
        prerelease_run.contains("gh release create")
            && prerelease_run.contains("--prerelease")
            && prerelease_run.contains("--verify-tag"),
        "the last job creates a verified prerelease: {prerelease_run}"
    );

    let on = &doc["on"];
    assert!(
        on["push"]["tags"].as_sequence().is_some(),
        "a v* tag triggers it"
    );
    assert!(
        on["workflow_dispatch"]["inputs"]["probe_fail_verify"].is_mapping(),
        "the falsifier probe input exists"
    );

    let disabled =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/release.yml.disabled");
    assert!(
        !disabled.exists(),
        "release.yml.disabled must be gone: one release path, not two"
    );
}

#[test]
fn docker_publish_is_dispatch_only_and_refuses_without_secrets() {
    let (_, text) = workflow("docker-publish.yml");
    let doc = yaml(&text);
    let on = doc["on"].as_mapping().expect("on");
    assert!(
        on.get("push").is_none(),
        "no tag trigger: it failed on every tag since v3.32.0"
    );
    assert!(
        on.get("workflow_dispatch").is_some(),
        "manual, credentialed runs only"
    );
    let steps: Vec<&str> = doc["jobs"]["build"]["steps"]
        .as_sequence()
        .expect("steps")
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    let preflight = steps
        .iter()
        .position(|s| s.starts_with("Preflight"))
        .expect("a preflight step");
    let login = steps
        .iter()
        .position(|s| s.starts_with("Log in"))
        .expect("a login step");
    assert!(
        preflight < login,
        "the preflight runs before any login attempt"
    );
    assert!(
        text.contains("DOCKER_USERNAME") && text.contains("exit 1"),
        "the preflight names the secrets and exits 1"
    );
    let tags = doc["jobs"]["build"]["steps"]
        .as_sequence()
        .expect("steps")
        .iter()
        .find_map(|s| s["with"]["tags"].as_str())
        .unwrap_or_default();
    assert!(
        !tags.contains("2.10.0"),
        "no hard-coded image version (#1122): {tags}"
    );
    assert!(
        tags.contains("steps.ver.outputs.version"),
        "the image is tagged with the crate version: {tags}"
    );
}

/// The "Upload assets to release" step of binary-release.yml, run for real
/// under bash against a fake `gh` that records every call. `on_release` is the
/// asset list the fake release starts with; the return value is the fake's log.
#[cfg(unix)]
fn run_upload_step(on_release: &[&str]) -> (bool, String) {
    use std::os::unix::fs::PermissionsExt;
    let (_, text) = workflow("binary-release.yml");
    let doc = yaml(&text);
    let script = doc["jobs"]["build"]["steps"]
        .as_sequence()
        .expect("steps")
        .iter()
        .find(|s| s["name"].as_str() == Some("Upload assets to release"))
        .and_then(|s| s["run"].as_str())
        .expect("the upload step has a run script")
        .replace("${{ steps.tag.outputs.tag }}", "v9.9.9")
        .replace(
            "${{ steps.package.outputs.archive }}",
            "pmat-v9.9.9-t.tar.gz",
        )
        .replace(
            "${{ steps.package.outputs.sha }}",
            "pmat-v9.9.9-t.tar.gz.sha256",
        );
    assert!(
        !script.contains("${{"),
        "an expression the test does not bind: {script}"
    );

    let dir = tempfile::tempdir().expect("tempdir");
    let work = dir.path().join("work");
    let bin = dir.path().join("bin");
    std::fs::create_dir_all(&work).expect("work");
    std::fs::create_dir_all(&bin).expect("bin");
    // The rebuild in the workspace differs from the published archive on purpose.
    std::fs::write(work.join("pmat-v9.9.9-t.tar.gz"), b"REBUILT").expect("archive");
    std::fs::write(
        work.join("pmat-v9.9.9-t.tar.gz.sha256"),
        b"sha-of-rebuild\n",
    )
    .expect("sha");
    std::fs::write(dir.path().join("assets"), on_release.join("\n")).expect("assets");
    let fake = r#"#!/usr/bin/env bash
echo "$*" >> "$FAKE_GH_DIR/log"
case "$1 $2" in
  "release view") cat "$FAKE_GH_DIR/assets"; echo ;;
  "release download")
    dir=.; while [ "$#" -gt 0 ]; do [ "$1" = "--dir" ] && dir=$2; shift; done
    printf PUBLISHED > "$dir/pmat-v9.9.9-t.tar.gz" ;;
  "release upload")
    for f in "${@:4}"; do [ "${f%.sha256}" != "$f" ] && echo "uploaded-sha: $(cat "$f")" >> "$FAKE_GH_DIR/log"; done ;;
esac
"#;
    let gh = bin.join("gh");
    std::fs::write(&gh, fake).expect("fake gh");
    std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let status = std::process::Command::new("bash")
        .args(["-e", "-o", "pipefail", "-c", &script])
        .current_dir(&work)
        .env("PATH", path)
        .env("FAKE_GH_DIR", dir.path())
        .output()
        .expect("bash runs");
    let log = std::fs::read_to_string(dir.path().join("log")).unwrap_or_default();
    (status.status.success(), log)
}

#[cfg(unix)]
#[test]
fn binary_release_upload_never_replaces_a_published_asset() {
    const ARCHIVE: &str = "pmat-v9.9.9-t.tar.gz";
    const SHA: &str = "pmat-v9.9.9-t.tar.gz.sha256";
    let (_, text) = workflow("binary-release.yml");
    assert!(
        !text.contains("--clobber"),
        "no step may replace a release asset: hosts pin it by sha256"
    );

    // Nothing on the release yet: both files go up, in one call.
    let (ok, log) = run_upload_step(&["pmat-v9.9.9-other.tar.gz"]);
    assert!(ok, "{log}");
    assert!(
        log.contains(&format!("release upload v9.9.9 {ARCHIVE} {SHA}")),
        "{log}"
    );

    // Both already published: a backfill dispatch uploads nothing.
    let (ok, log) = run_upload_step(&[ARCHIVE, SHA]);
    assert!(ok, "{log}");
    assert!(
        !log.contains("release upload"),
        "nothing is replaced: {log}"
    );

    // Archive published, checksum missing: only the checksum goes up, and it is
    // the checksum of the PUBLISHED bytes, not of this run's rebuild.
    let (ok, log) = run_upload_step(&[ARCHIVE]);
    assert!(ok, "{log}");
    assert!(
        log.contains(&format!("release upload v9.9.9 {SHA}")),
        "{log}"
    );
    assert!(
        !log.lines()
            .any(|l| l.starts_with("release upload") && l.split(' ').any(|w| w == ARCHIVE)),
        "the archive is not replaced: {log}"
    );
    // sha256("PUBLISHED")
    assert!(
        log.contains(
            "uploaded-sha: 9466f57050053c595fb5b7a8ad1767dbecca63033f5a69db97268f640f6cd648"
        ),
        "the checksum describes the published archive: {log}"
    );
}
