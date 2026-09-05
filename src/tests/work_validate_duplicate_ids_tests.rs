//! PMAT-674 — `pmat work validate` must refuse a roadmap that declares the same
//! id twice, and must locate a YAML parse error by `file:line`.
//!
//! On master `914fe6246` `docs/roadmaps/roadmap.yaml` carried `- id: PMAT-654`
//! on two byte-identical rows (4001 and 4035) and `work validate` exited 0:
//! serde deserialises a sequence, so two rows sharing an id are two perfectly
//! good rows. Nothing in the strict parse can see the collision, which is why
//! the check below reads the RAW text.
//!
//! Registered from `src/cli/handlers/work_handlers/mod.rs` with `#[path]`.
//! `src/tests/lib.rs` is an orphan target (`docs/status/orphan-files-ledger.md`)
//! and CI runs `cargo test --lib`, so a file registered only there never runs —
//! its silence would read as a pass.

use crate::cli::handlers::work_handlers::handle_work_validate;
use std::path::PathBuf;
use tempfile::TempDir;

/// A roadmap whose id `PMAT-654` appears on two separate rows.
const DUPLICATE_FIXTURE: &str = "roadmap_version: '1.0'
github_enabled: false
github_repo: null
roadmap:
- id: PMAT-654
  title: The first row
  status: planned
  acceptance_criteria:
  - it parses
- id: PMAT-700
  title: An unrelated row
  status: planned
  acceptance_criteria:
  - it parses
- id: PMAT-654
  title: The second row, same id
  status: planned
  acceptance_criteria:
  - it parses
";

/// A roadmap that no lenient reading can accept: `bogus` is not a status.
const UNPARSEABLE_FIXTURE: &str = "roadmap_version: '1.0'
github_enabled: false
github_repo: null
roadmap:
- id: PMAT-001
  title: A row with an impossible status
  status: bogus
";

/// A roadmap with distinct ids everywhere, subtasks included.
const VALID_FIXTURE: &str = "roadmap_version: '1.0'
github_enabled: false
github_repo: null
roadmap:
- id: PMAT-001
  title: The first row
  status: planned
  acceptance_criteria:
  - it parses
  subtasks:
  - id: PMAT-001-a
    title: A subtask
    status: planned
- id: PMAT-002
  title: The second row
  status: completed
  acceptance_criteria:
  - it parses
";

/// Write `yaml` as `docs/roadmaps/roadmap.yaml` of a fresh project directory.
fn project_with_roadmap(yaml: &str) -> TempDir {
    let dir = tempfile::tempdir().expect("a temp dir must be creatable");
    let roadmaps = dir.path().join("docs/roadmaps");
    std::fs::create_dir_all(&roadmaps).expect("docs/roadmaps must be creatable");
    std::fs::write(roadmaps.join("roadmap.yaml"), yaml).expect("roadmap.yaml must be writable");
    dir
}

/// The 1-based line number of the `nth` (0-based) line reading exactly `needle`.
///
/// The expected line numbers are computed from the fixture text rather than
/// written down, so editing a fixture cannot silently make an assertion vacuous.
fn line_of_nth_occurrence(text: &str, needle: &str, nth: usize) -> usize {
    let hits: Vec<usize> = text
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim() == needle)
        .map(|(index, _)| index + 1)
        .collect();
    assert!(
        hits.len() > nth,
        "fixture must contain at least {} lines {needle:?}, found {hits:?}",
        nth + 1
    );
    hits[nth]
}

/// V1: two rows sharing an id are an error that names both line numbers.
#[tokio::test]
async fn work_validate_duplicate_ids_are_refused_and_both_lines_reported() {
    let project = project_with_roadmap(DUPLICATE_FIXTURE);
    let first = line_of_nth_occurrence(DUPLICATE_FIXTURE, "- id: PMAT-654", 0);
    let second = line_of_nth_occurrence(DUPLICATE_FIXTURE, "- id: PMAT-654", 1);

    let result = handle_work_validate(Some(project.path().to_path_buf()), false, false).await;

    let error = result.err().map_or_else(String::new, |e| format!("{e:#}"));
    assert!(
        !error.is_empty(),
        "a roadmap declaring PMAT-654 on lines {first} and {second} must not validate"
    );
    assert!(
        error.contains("PMAT-654"),
        "the error must name the duplicated id: {error}"
    );
    assert!(
        error.contains(&format!("roadmap.yaml:{first}")),
        "the error must locate the first occurrence (line {first}): {error}"
    );
    assert!(
        error.contains(&format!("roadmap.yaml:{second}")),
        "the error must locate the second occurrence (line {second}): {error}"
    );
}

/// V2: an unparseable roadmap fails with `<path>:<line>` in the error itself,
/// not only in the context block printed to stdout.
#[tokio::test]
async fn work_validate_duplicate_unparseable_roadmap_is_located_by_line() {
    let project = project_with_roadmap(UNPARSEABLE_FIXTURE);

    let result = handle_work_validate(Some(project.path().to_path_buf()), false, false).await;

    let error = result.err().map_or_else(String::new, |e| format!("{e:#}"));
    assert!(!error.is_empty(), "`status: bogus` must not validate");
    let located = error.split("roadmap.yaml:").nth(1).unwrap_or("");
    assert!(
        located.starts_with(|c: char| c.is_ascii_digit()),
        "the error must carry `roadmap.yaml:<line>`: {error}"
    );
}

/// V3: distinct ids, subtask ids included, still validate.
#[tokio::test]
async fn work_validate_duplicate_distinct_ids_still_validate() {
    let project = project_with_roadmap(VALID_FIXTURE);

    let result = handle_work_validate(Some(project.path().to_path_buf()), false, false).await;

    assert!(
        result.is_ok(),
        "a roadmap with distinct ids must validate: {:?}",
        result.err().map(|e| format!("{e:#}"))
    );
}

/// V4: this repository's own roadmap is clean, so the new check must not
/// manufacture a failure on it.
#[tokio::test]
async fn work_validate_duplicate_this_repositorys_roadmap_validates() {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let result = handle_work_validate(Some(project), false, false).await;

    assert!(
        result.is_ok(),
        "docs/roadmaps/roadmap.yaml must validate: {:?}",
        result.err().map(|e| format!("{e:#}"))
    );
}

/// The `ci.yml` of this repository, read from the source tree.
fn ci_workflow() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/ci.yml");
    let read = std::fs::read_to_string(&path);
    assert!(
        read.is_ok(),
        "{} must be readable: {:?}",
        path.display(),
        read.as_ref().err()
    );
    read.unwrap_or_default()
}

/// True for a line that opens a job (two spaces of indent, then `key:`).
fn opens_a_job(line: &str) -> bool {
    line.starts_with("  ")
        && !line.starts_with("   ")
        && !line.trim_start().starts_with('#')
        && line.trim_end().ends_with(':')
}

/// The block of `yaml` belonging to the job keyed `  <name>:`.
fn job_block(yaml: &str, name: &str) -> String {
    let key = format!("  {name}:");
    let lines: Vec<&str> = yaml.lines().collect();
    let start = lines.iter().position(|line| *line == key);
    assert!(start.is_some(), "ci.yml must declare a job `{key}`");
    let start = start.unwrap_or_default();
    let end = lines[start + 1..]
        .iter()
        .position(|line| opens_a_job(line))
        .map_or(lines.len(), |offset| start + 1 + offset);
    lines[start..end].join("\n")
}

/// V6: CI must actually run `work validate`, and the aggregate gate must be
/// able to fail on it. A job nothing depends on cannot block a merge.
#[test]
fn work_validate_duplicate_ci_runs_validate_as_a_gated_job() {
    let ci = ci_workflow();

    let roadmap_job = job_block(&ci, "roadmap-validate");
    assert!(
        roadmap_job.contains("work validate"),
        "the roadmap-validate job must run `work validate`: {roadmap_job}"
    );

    let gate = job_block(&ci, "gate");
    let needs = gate
        .lines()
        .find(|line| line.trim_start().starts_with("needs:"))
        .unwrap_or("");
    assert!(
        needs.contains("roadmap-validate"),
        "the gate job must need roadmap-validate: {needs:?}"
    );
    assert!(
        ci.contains("roadmap-validate:${{ needs.roadmap-validate.result }}"),
        "the gate must read the roadmap-validate result"
    );
}

/// The rendered long help of `pmat work validate`.
///
/// Building pmat's clap tree needs more than the 2 MiB default test stack, and
/// the acceptance command runs with `RUST_MIN_STACK` unset — hence the thread.
fn long_help_of_work_validate() -> String {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            use clap::CommandFactory;
            let mut cli = crate::cli::Cli::command();
            let rendered = cli.find_subcommand_mut("work").and_then(|work| {
                work.find_subcommand_mut("validate")
                    .map(|validate| validate.render_long_help().to_string())
            });
            let _ = sender.send(rendered);
        })
        .expect("the help thread must spawn")
        .join()
        .expect("the clap tree must build");
    receiver
        .recv()
        .ok()
        .flatten()
        .expect("`pmat work validate` must exist in the clap tree")
}

/// V7: the exit codes are part of the contract, so `--help` must state them.
#[test]
fn work_validate_duplicate_help_documents_the_exit_codes() {
    let help = long_help_of_work_validate();

    assert!(
        help.contains("Exit codes"),
        "`work validate --help` must document its exit codes: {help}"
    );
    assert!(
        help.contains("0 —"),
        "`work validate --help` must document exit 0: {help}"
    );
    assert!(
        help.contains("1 —"),
        "`work validate --help` must document exit 1: {help}"
    );
}
