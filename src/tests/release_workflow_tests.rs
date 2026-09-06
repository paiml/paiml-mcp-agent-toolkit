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
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} must be readable: {e}", path.display()));
    (path, text)
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
