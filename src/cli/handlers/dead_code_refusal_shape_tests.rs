//! What `analyze dead-code` hands a MACHINE when it declines to measure.
//!
//! Issue #1050 P7. Pointed at a cargo workspace root — a `Cargo.toml` that is
//! `[workspace]` with no `[package]` — pmat is right to refuse, and the
//! sentence it writes to stderr is excellent: it names the virtual manifest,
//! every member crate, and ends *"This is not a clean result."*
//!
//! `--format json` wrote **zero bytes** to stdout for that refusal:
//!
//! ```text
//! $ pmat analyze dead-code --path <workspace-root> --format json > out.json; echo $?
//! 1
//! $ wc -c out.json
//! 0
//! ```
//!
//! Zero bytes is what a consumer also gets from a crash, from a process killed
//! before it wrote, and from a tool that produced nothing at all: `json.load()`
//! raises rather than reading a field. `tdg` answers the same class with
//! `{"not_measured": true, "score": {"total": null, …}}` at exit 5, and one
//! tool must not answer the same question two ways.
//!
//! These tests read the EMITTERS, which is what a `--format json` consumer
//! actually receives; the end-to-end refusal itself is already pinned by
//! `workspace_root_tests`.

use super::{dead_code_refusal_json, dead_code_refusal_sarif};
use std::path::Path;

/// The prose the workspace-root refusal actually carries, abridged. Used
/// verbatim so the test pins that the REASON travels, not a placeholder.
const REASON: &str = "/ws holds Rust sources but is inside no cargo PACKAGE: the nearest \
                      Cargo.toml above it, /ws/Cargo.toml, is a WORKSPACE manifest. \
                      This is not a clean result.";

fn refusal() -> serde_json::Value {
    let body = dead_code_refusal_json(Path::new("/ws"), &anyhow::anyhow!(REASON));
    // The whole defect in one line: this used to be zero bytes, so parsing is
    // the assertion.
    serde_json::from_str(&body).expect("a refusal must be a parsable document")
}

/// The field a consumer branches on, spelled as `tdg` spells it.
#[test]
fn the_refusal_declares_itself_not_measured() {
    let value = refusal();
    assert_eq!(value["not_measured"], true);
    assert_eq!(value["path"], "/ws");
}

/// A reason a machine can read is the difference between "retry with a bigger
/// --timeout" and "point --path at a member crate". The sentence stderr carries
/// is the sentence the document carries.
#[test]
fn the_refusal_carries_the_reason_verbatim() {
    let value = refusal();
    let reason = value["reason"].as_str().expect("reason is a string");
    assert_eq!(reason, REASON);
    assert!(reason.contains("WORKSPACE manifest"), "{reason}");
}

/// The half that matters most. `dead_functions: 0` and `files: []` are what a
/// CLEAN crate reports; a refusal rendered with zeros would be worse than the
/// empty file it replaced, because it would parse and be believed.
#[test]
fn nothing_in_the_refusal_can_be_read_as_a_measurement() {
    let value = refusal();

    for key in [
        "total_files_analyzed",
        "files_with_dead_code",
        "total_dead_lines",
        "dead_percentage",
        "dead_functions",
        "dead_classes",
        "dead_modules",
        "unreachable_blocks",
    ] {
        assert!(
            value["summary"][key].is_null(),
            "summary.{key} is {} — a refusal must not publish a number",
            value["summary"][key]
        );
    }

    for key in ["files", "files_analyzed", "files_discovered", "total_files"] {
        assert!(
            value[key].is_null(),
            "{key} is {} — an empty list is a measured claim that nothing was found",
            value[key]
        );
    }
}

/// The key set is the successful document's key set, so a consumer written
/// against a real report finds every field it looks for and finds it null,
/// rather than raising a `KeyError` on the refusal path.
#[test]
fn the_refusal_answers_the_same_keys_a_report_answers() {
    let value = refusal();
    let object = value.as_object().expect("object");
    for key in ["summary", "files", "files_analyzed", "files_discovered"] {
        assert!(object.contains_key(key), "missing {key} from {value}");
    }
}

/// SARIF is a machine surface too, and an empty `results` array is a SARIF file
/// that says the scan passed. The refusal says the run did not succeed.
#[test]
fn the_sarif_refusal_is_not_a_clean_scan() {
    let body = dead_code_refusal_sarif(Path::new("/ws"), &anyhow::anyhow!(REASON));
    let value: serde_json::Value = serde_json::from_str(&body).expect("parsable SARIF");

    let invocation = &value["runs"][0]["invocations"][0];
    assert_eq!(invocation["executionSuccessful"], false);
    assert_eq!(
        invocation["toolExecutionNotifications"][0]["message"]["text"],
        REASON
    );
    assert!(
        value["runs"][0]["results"].is_null(),
        "an empty results array reads as a scan that found nothing"
    );
}

/// The exit code is a property of the FAILURE, not of the rendering — the whole
/// point of `crate::cli_exit`. `--format summary` and `--format json` must not
/// disagree about whether a measurement happened.
#[tokio::test]
async fn every_format_exits_analysis_error() {
    use crate::cli::DeadCodeOutputFormat;

    for format in [
        DeadCodeOutputFormat::Json,
        DeadCodeOutputFormat::Sarif,
        DeadCodeOutputFormat::Summary,
        DeadCodeOutputFormat::Markdown,
    ] {
        // Written to a file rather than stdout so the assertion is about the
        // code, and so a `cargo test` run is not sprayed with refusal bodies.
        let out = tempfile::TempDir::new().expect("tempdir");
        let error = super::refuse_dead_code_measurement(
            Path::new("/ws"),
            &format,
            Some(out.path().join("report")),
            anyhow::anyhow!(REASON),
        )
        .await
        .expect_err("a refusal must not report success");

        assert_eq!(
            crate::cli_exit::code_for(&error),
            crate::cli_exit::ExitCode::AnalysisError,
            "{format:?} refused with the wrong exit code"
        );
        // …and the message a human reads is unchanged by any of this.
        assert_eq!(format!("{error:#}"), REASON);
    }
}

/// The counter-test bounding the correction: a refusal body is written for the
/// machine formats and NOT for the text ones, whose prose already says all of
/// it on stderr. Emitting JSON under `--format summary` would be a second
/// surface contradicting the first.
#[tokio::test]
async fn only_the_machine_formats_get_a_body() {
    use crate::cli::DeadCodeOutputFormat;

    for (format, expect_body) in [
        (DeadCodeOutputFormat::Json, true),
        (DeadCodeOutputFormat::Sarif, true),
        (DeadCodeOutputFormat::Summary, false),
        (DeadCodeOutputFormat::Markdown, false),
    ] {
        let out = tempfile::TempDir::new().expect("tempdir");
        let path = out.path().join("report");
        let _ = super::refuse_dead_code_measurement(
            Path::new("/ws"),
            &format,
            Some(path.clone()),
            anyhow::anyhow!(REASON),
        )
        .await;

        assert_eq!(
            path.exists(),
            expect_body,
            "{format:?} wrote the wrong thing: exists={}",
            path.exists()
        );
    }
}
