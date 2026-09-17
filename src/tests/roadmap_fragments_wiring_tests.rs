#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-1363 (#1363) — the WIRING: every `pmat work` reader sees base ⊕ fragments,
//! every writer writes fragments, and none of them opens `roadmap.yaml` for write,
//! in a repository that has opted in to `docs/roadmaps/entries/`.
//!
//! The contract predicate is stated so it holds however many write sites there turn
//! out to be: after the operation, `git diff --name-only` excludes
//! `docs/roadmaps/roadmap.yaml`. At this level that is "the file's bytes did not
//! change", which is the same claim without needing a git repository.
//!
//! OPT-IN is `[ -d docs/roadmaps/entries ]` — the predicate paiml/.github's shared
//! `roadmap-fragment-parity` gate uses — so pmat writes fragments exactly where that
//! gate refuses a pull request that edits the aggregate, and nowhere else.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs`, for the reason its sibling
//! `roadmap_fragments_tests.rs` gives.

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fs2::FileExt;

use crate::models::roadmap::{ItemStatus, RoadmapItem};
use crate::roadmap::aggregate::{run_aggregate, AggregateMode};
use crate::services::roadmap_id_authority::IdAuthority;
use crate::services::roadmap_service::RoadmapService;

const BASE: &str = "roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: null\nroadmap:\n- id: PMAT-001\n  title: first\n  status: planned\n- id: PMAT-005\n  title: fifth\n  status: planned\n";

fn fixture(with_entries: bool) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let docs = dir.path().join("docs/roadmaps");
    std::fs::create_dir_all(&docs).expect("mkdir");
    if with_entries {
        std::fs::create_dir_all(docs.join("entries")).expect("mkdir entries");
    }
    let path = docs.join("roadmap.yaml");
    std::fs::write(&path, BASE).expect("write");
    (dir, path)
}

fn entries(path: &Path) -> PathBuf {
    path.parent().expect("parent").join("entries")
}

fn fragment_names(path: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(entries(path))
        .expect("readdir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn an_item(id: String) -> RoadmapItem {
    // Built from YAML like the neighbouring suites do: RoadmapItem has no Default,
    // and hand-filling it would drift from the real shape.
    let block = format!("- id: {id}\n  title: wired\n  status: planned\n");
    let mut items: Vec<RoadmapItem> =
        serde_yaml_ng::from_str(&block).expect("the block must parse as one item");
    items.pop().expect("exactly one item")
}

fn write_fragment_file(path: &Path, id: &str, title: &str) {
    std::fs::write(
        entries(path).join(format!("{id}.yaml")),
        format!("- id: {id}\n  title: {title}\n  status: planned\n"),
    )
    .expect("write fragment");
}

// ------------------------------------------------------------------ writers

#[test]
fn roadmap_fragments_work_add_writes_a_fragment_and_leaves_the_aggregate_untouched() {
    let (_dir, path) = fixture(true);
    let svc = RoadmapService::new(&path);
    let id = svc.add_item_with_next_id(an_item).expect("add");

    assert_eq!(
        std::fs::read_to_string(&path).expect("read the roadmap"),
        BASE,
        "roadmap.yaml must be byte-identical"
    );
    let body =
        std::fs::read_to_string(entries(&path).join(format!("{id}.yaml"))).expect("fragment");
    assert!(body.starts_with(&format!("- id: {id}\n")), "{body}");
    assert_eq!(id, "PMAT-006", "the allocator counts the base");
}

#[test]
fn roadmap_fragments_work_add_still_appends_when_the_repo_has_not_migrated() {
    // A repository without entries/ keeps the PMAT-679 append behaviour exactly.
    // Changing that silently on a version bump would stop its roadmap updating with
    // nothing to read as an error.
    let (_dir, path) = fixture(false);
    let id = RoadmapService::new(&path)
        .add_item_with_next_id(an_item)
        .expect("add");
    let after = std::fs::read_to_string(&path).expect("read the roadmap");
    assert!(
        after.starts_with(BASE),
        "the append must preserve every prior byte (PMAT-679)"
    );
    assert!(after.contains(&format!("- id: {id}")));
    assert!(
        !entries(&path).exists(),
        "and no entries/ appears from nowhere"
    );
}

#[test]
fn roadmap_fragments_work_add_with_a_caller_supplied_id_also_writes_a_fragment() {
    // `pmat work add --github-issue N` takes add_item_with_id, NOT the allocator path.
    // Patching only the allocator would have made the behaviour depend on the flag.
    let (_dir, path) = fixture(true);
    RoadmapService::new(&path)
        .add_item_with_id("PMAT-1363", an_item)
        .expect("add with id");
    assert_eq!(
        std::fs::read_to_string(&path).expect("read the roadmap"),
        BASE
    );
    assert_eq!(fragment_names(&path), ["PMAT-1363.yaml"]);
}

#[test]
fn roadmap_fragments_work_add_refuses_an_id_that_is_not_filename_safe_prefix_n() {
    let (_dir, path) = fixture(true);
    let svc = RoadmapService::new(&path);
    for bad in [
        "Push completed work to origin/main (5 commits)",
        "PMAT-12 (notes)",
        "ROADMAP-RECONCILE-2026-07-04",
        "PMAT-7-a",
        "../PMAT-7",
    ] {
        let err = svc.add_item_with_id(bad, an_item).expect_err(bad);
        assert!(
            err.to_string().contains("filename-safe PREFIX-N"),
            "{bad}: {err}"
        );
    }
    assert!(
        fragment_names(&path).is_empty(),
        "a refused id writes nothing at all"
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("read the roadmap"),
        BASE
    );

    // Unconditional: a repository that has not opted in today migrates tomorrow.
    let (_plain, plain) = fixture(false);
    assert!(RoadmapService::new(&plain)
        .add_item_with_id("PMAT-7 x", an_item)
        .is_err());
    assert_eq!(
        std::fs::read_to_string(&plain).expect("read the roadmap"),
        BASE
    );
}

#[test]
fn roadmap_fragments_an_id_held_by_a_fragment_is_in_use() {
    let (_dir, path) = fixture(true);
    write_fragment_file(&path, "PMAT-009", "held");
    let svc = RoadmapService::new(&path);
    let err = svc
        .add_item_with_id("PMAT-009", an_item)
        .expect_err("already in use");
    assert!(err.to_string().contains("already in use"), "{err}");
    assert_eq!(
        svc.add_item_with_next_id(an_item).expect("mint"),
        "PMAT-010"
    );
}

#[test]
fn roadmap_fragments_save_writes_only_the_changed_tickets_as_fragments() {
    // `work start` / `work complete` / `work sync` save the WHOLE model. In an
    // opted-in repository that must become one fragment per changed ticket.
    let (_dir, path) = fixture(true);
    let svc = RoadmapService::new(&path);
    let mut roadmap = svc.load().expect("load");
    roadmap.roadmap[1].status = ItemStatus::InProgress;
    svc.save(&roadmap).expect("save");

    assert_eq!(
        std::fs::read_to_string(&path).expect("read the roadmap"),
        BASE,
        "roadmap.yaml untouched"
    );
    assert_eq!(
        fragment_names(&path),
        ["PMAT-005.yaml"],
        "PMAT-001 did not change"
    );
    let reloaded = svc.load().expect("reload");
    assert_eq!(
        reloaded
            .find_item("PMAT-005")
            .expect("the ticket is present")
            .status,
        ItemStatus::InProgress
    );
    assert_eq!(reloaded, roadmap, "what was saved is what is loaded");
}

#[test]
fn roadmap_fragments_upsert_and_edit_write_fragments_too() {
    let (_dir, path) = fixture(true);
    let svc = RoadmapService::new(&path);

    svc.upsert_item(an_item("PMAT-002".into())).expect("upsert");
    svc.upsert_item_checked(an_item("PMAT-003".into()))
        .expect("upsert checked");
    let mut edited = svc
        .find_item("PMAT-001")
        .expect("the roadmap loads")
        .expect("present");
    edited.title = "edited".into();
    svc.replace_item_raw("PMAT-001", &edited).expect("edit");

    assert_eq!(
        std::fs::read_to_string(&path).expect("read the roadmap"),
        BASE,
        "roadmap.yaml untouched"
    );
    assert_eq!(
        fragment_names(&path),
        ["PMAT-001.yaml", "PMAT-002.yaml", "PMAT-003.yaml"]
    );
    let view = svc.load().expect("load");
    let ids: Vec<&str> = view.roadmap.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(
        ids,
        ["PMAT-001", "PMAT-002", "PMAT-003", "PMAT-005"],
        "sorted slots"
    );
    assert_eq!(
        view.find_item("PMAT-001")
            .expect("the ticket is present")
            .title,
        "edited",
        "the fragment supersedes"
    );
}

#[test]
fn roadmap_fragments_remove_deletes_a_fragment_and_refuses_a_base_row() {
    let (_dir, path) = fixture(true);
    let svc = RoadmapService::new(&path);
    svc.add_item_with_id("PMAT-004", an_item).expect("add");

    assert!(svc.remove_item("PMAT-004").expect("remove").is_some());
    assert!(fragment_names(&path).is_empty(), "the fragment is gone");

    let err = svc
        .remove_item("PMAT-001")
        .expect_err("a base row cannot be removed by a fragment");
    assert!(err.to_string().contains("cannot delete one"), "{err}");
    assert_eq!(
        std::fs::read_to_string(&path).expect("read the roadmap"),
        BASE
    );
    assert!(
        fragment_names(&path).is_empty(),
        "the refusal wrote nothing"
    );
}

#[test]
fn roadmap_fragments_a_header_change_is_refused_not_written() {
    let (_dir, path) = fixture(true);
    let svc = RoadmapService::new(&path);
    let mut roadmap = svc.load().expect("load");
    roadmap.github_repo = Some("paiml/elsewhere".into());
    roadmap.roadmap[0].title = "would be a fragment".into();
    let err = svc
        .save(&roadmap)
        .expect_err("the header lives only in the generated file");
    assert!(err.to_string().contains("roadmap header"), "{err}");
    assert!(
        fragment_names(&path).is_empty(),
        "nothing was written before the refusal"
    );
}

#[test]
fn roadmap_fragments_entries_without_a_base_is_refused() {
    let (_dir, path) = fixture(true);
    std::fs::remove_file(&path).expect("rm base");
    let svc = RoadmapService::new(&path);
    assert!(svc.add_item_with_id("PMAT-2", an_item).is_err());
    assert!(svc.upsert_item(an_item("PMAT-2".into())).is_err());
    assert!(!path.exists(), "no base was created");
    assert!(fragment_names(&path).is_empty(), "no fragment was created");
}

// ------------------------------------------------------------------ readers

#[test]
fn roadmap_fragments_work_list_and_status_read_base_plus_fragments() {
    // `pmat work list` and `pmat work status <id>` both load through the service; a
    // ticket must be visible the moment its fragment exists, not after the next
    // post-merge aggregation.
    let (_dir, path) = fixture(true);
    write_fragment_file(&path, "PMAT-003", "only a fragment");
    write_fragment_file(&path, "PMAT-005", "superseded by a fragment");
    let svc = RoadmapService::new(&path);

    let roadmap = svc.load().expect("load");
    let ids: Vec<&str> = roadmap.roadmap.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(ids, ["PMAT-001", "PMAT-003", "PMAT-005"]);
    assert_eq!(
        svc.find_item("PMAT-003")
            .expect("the roadmap loads")
            .expect("the ticket is present")
            .title,
        "only a fragment"
    );
    assert_eq!(
        svc.find_item("PMAT-005")
            .expect("the roadmap loads")
            .expect("the ticket is present")
            .title,
        "superseded by a fragment"
    );

    // And a malformed fragment fails the read loudly rather than vanishing.
    std::fs::write(entries(&path).join("PMAT-007.yaml"), "- id: PMAT-008\n")
        .expect("write the fixture");
    let err = svc
        .load()
        .expect_err("a fragment naming another id is refused");
    assert!(err.to_string().contains("PMAT-007"), "{err}");
}

// --------------------------------------------------------------------- lock

/// How a second `pmat` process holds the repository's roadmap lock.
#[derive(Clone, Copy, Debug)]
enum Holder {
    /// A writer mid-write.
    Exclusive,
    /// A reader mid-read (`pmat work list`). A writer that took only the SHARED lock
    /// would proceed past this holder, so it is the one that tells shared from
    /// exclusive — an exclusive holder blocks both.
    Shared,
}

/// Take the repository's roadmap lock from OUTSIDE the service, the way a second
/// `pmat` process would.
fn hold_the_lock(path: &Path, holder: Holder) -> (std::fs::File, PathBuf) {
    let lock_path = IdAuthority::discover(path).lock_path;
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .expect("open lock");
    match holder {
        Holder::Exclusive => file.lock_exclusive().expect("take exclusive lock"),
        Holder::Shared => FileExt::lock_shared(&file).expect("take shared lock"),
    }
    (file, lock_path)
}

/// Prove `write` cannot land `fragment` while another process holds the lock,
/// exclusively or shared, and lands it once released.
fn assert_blocks_on_the_lock(
    write: impl Fn(PathBuf) -> anyhow::Result<()> + Send + Sync + Clone + 'static,
    fragment: &str,
) {
    for holder in [Holder::Exclusive, Holder::Shared] {
        let (_dir, path) = fixture(true);
        let (held, lock_path) = hold_the_lock(&path, holder);

        let target = path.clone();
        let write = write.clone();
        let writer = std::thread::spawn(move || write(target));
        std::thread::sleep(Duration::from_millis(500));
        assert!(
            !entries(&path).join(fragment).exists(),
            "{fragment} was written while another process held {} ({holder:?}) — fragment writes are not exclusive",
            lock_path.display()
        );
        assert!(
            !writer.is_finished(),
            "the writer returned without waiting for the {holder:?} lock"
        );

        FileExt::unlock(&held).expect("release");
        drop(held);
        writer
            .join()
            .expect("writer thread")
            .expect("the write succeeds once the lock is free");
        assert!(
            entries(&path).join(fragment).exists(),
            "{fragment} after releasing the {holder:?} lock"
        );
    }
}

#[test]
fn roadmap_fragments_an_add_waits_for_the_repository_lock() {
    assert_blocks_on_the_lock(
        |path| {
            RoadmapService::new(path)
                .add_item_with_id("PMAT-007", an_item)
                .map(|_| ())
        },
        "PMAT-007.yaml",
    );
}

#[test]
fn roadmap_fragments_a_save_waits_for_the_repository_lock() {
    assert_blocks_on_the_lock(
        |path| RoadmapService::new(path).upsert_item(an_item("PMAT-008".into())),
        "PMAT-008.yaml",
    );
}

#[test]
fn roadmap_fragments_aggregate_write_waits_for_the_repository_lock() {
    // `pmat roadmap aggregate --write` is the one writer of the aggregate itself.
    // Against a writer it must not read a half-landed fragment set; against a
    // reader it must not truncate roadmap.yaml under a `pmat work list`.
    for holder in [Holder::Exclusive, Holder::Shared] {
        let (_dir, path) = fixture(true);
        write_fragment_file(&path, "PMAT-003", "third");
        let (held, lock_path) = hold_the_lock(&path, holder);

        let target = path.clone();
        let writer = std::thread::spawn(move || run_aggregate(&target, None, AggregateMode::Write));
        std::thread::sleep(Duration::from_millis(500));
        assert_eq!(
            std::fs::read_to_string(&path).expect("read the roadmap"),
            BASE,
            "aggregate --write rewrote the roadmap while another process held {} ({holder:?})",
            lock_path.display()
        );
        assert!(
            !writer.is_finished(),
            "aggregate --write returned without waiting for the {holder:?} lock"
        );

        FileExt::unlock(&held).expect("release");
        drop(held);
        let report = writer.join().expect("writer thread");
        assert_eq!(report.code, 0, "{}", report.stderr);
        assert!(std::fs::read_to_string(&path)
            .expect("read the roadmap")
            .contains("- id: PMAT-003"));
    }
}

#[test]
fn roadmap_fragments_concurrent_fragment_writers_lose_nothing() {
    // Eight writers, one repository: every ticket lands, and the view parses.
    let (_dir, path) = fixture(true);
    let handles: Vec<_> = (10..18)
        .map(|n| {
            let p = path.clone();
            std::thread::spawn(move || {
                let svc = RoadmapService::new(&p);
                svc.add_item_with_id(&format!("PMAT-{n:03}"), an_item)
                    .map(|_| ())
                    .and_then(|()| {
                        let mut item = svc.find_item(&format!("PMAT-{n:03}"))?.expect("just added");
                        item.status = ItemStatus::InProgress;
                        svc.upsert_item(item)
                    })
            })
        })
        .collect();
    for h in handles {
        h.join().expect("thread").expect("write");
    }
    let view = RoadmapService::new(&path).load().expect("load");
    assert_eq!(view.roadmap.len(), 10);
    assert!(view
        .roadmap
        .iter()
        .filter(|i| i.id.as_str() >= "PMAT-010")
        .all(|i| i.status == ItemStatus::InProgress));
    assert_eq!(
        std::fs::read_to_string(&path).expect("read the roadmap"),
        BASE
    );
}

// ---------------------------------------------------------------------- CLI

#[test]
fn roadmap_fragments_aggregate_command_prints_writes_and_checks() {
    let (_dir, path) = fixture(true);
    write_fragment_file(&path, "PMAT-003", "third");

    let printed = run_aggregate(&path, None, AggregateMode::Print);
    assert_eq!(printed.code, 0, "{}", printed.stderr);
    assert!(printed.stdout.contains("- id: PMAT-003"));

    // Not yet aggregated: --check names the first row that differs.
    let stale = run_aggregate(&path, None, AggregateMode::Check);
    assert_eq!(stale.code, 1, "{}", stale.stderr);
    assert!(
        stale.stderr.contains("first differing row: PMAT-003"),
        "{}",
        stale.stderr
    );

    // Three consecutive writes: byte-identical after the first.
    let mut runs = Vec::new();
    for _ in 0..3 {
        let w = run_aggregate(&path, None, AggregateMode::Write);
        assert_eq!(w.code, 0, "{}", w.stderr);
        runs.push(std::fs::read_to_string(&path).expect("read the roadmap"));
    }
    assert!(
        runs[0] == runs[1] && runs[1] == runs[2],
        "three writes differ"
    );
    let leftovers: Vec<String> = std::fs::read_dir(path.parent().expect("parent"))
        .expect("readdir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "--write replaces the aggregate by rename and leaves no staging file: {leftovers:?}"
    );
    assert_eq!(
        runs[0], printed.stdout,
        "--write writes what the default prints"
    );

    let ok = run_aggregate(&path, None, AggregateMode::Check);
    assert_eq!(ok.code, 0, "{}", ok.stderr);

    // A hand edit of a fragment-supplied row in the aggregate is caught, by row.
    // (A hand edit of a base-only row is invisible to aggregation BY CONSTRUCTION —
    // the base is its input — which is why the parity gate judges the pull
    // request's diff: a PR that touches roadmap.yaml at all is refused there.)
    std::fs::write(&path, runs[0].replace("title: third", "title: hand-edited"))
        .expect("write the fixture");
    let edited = run_aggregate(&path, None, AggregateMode::Check);
    assert_eq!(edited.code, 1, "{}", edited.stderr);
    assert!(
        edited.stderr.contains("first differing row: PMAT-003"),
        "{}",
        edited.stderr
    );
}

#[test]
fn roadmap_fragments_aggregate_command_exit_codes_distinguish_violation_from_unreadable() {
    let (dir, path) = fixture(true);
    std::fs::write(entries(&path).join("PMAT-003.yaml"), "- id: PMAT-004\n")
        .expect("write the fixture");
    let malformed = run_aggregate(&path, None, AggregateMode::Check);
    assert_eq!(malformed.code, 1, "{}", malformed.stderr);

    let missing = dir.path().join("nope/roadmap.yaml");
    let unreadable = run_aggregate(&missing, None, AggregateMode::Check);
    assert_eq!(
        unreadable.code, 2,
        "an input that cannot be read is never a pass: {}",
        unreadable.stderr
    );

    // --entries moves the fragment source; --roadmap alone moves both.
    let other = dir.path().join("elsewhere");
    std::fs::create_dir_all(&other).expect("create the directory");
    let explicit = run_aggregate(&path, Some(&other), AggregateMode::Check);
    assert_eq!(explicit.code, 0, "{}", explicit.stderr);
}

#[test]
fn roadmap_fragments_aggregate_is_a_parsed_subcommand() {
    // pmat's clap tree needs more than the 2 MiB default test stack (the idiom in
    // cli/commands/cli_struct.rs).
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(parse_the_aggregate_subcommand)
        .expect("spawn")
        .join()
        .expect("the clap tree must build and parse");
}

fn parse_the_aggregate_subcommand() {
    use crate::cli::commands::{Commands, RoadmapCommands};
    use crate::cli::Cli;
    use clap::Parser;

    let cli = Cli::try_parse_from([
        "pmat",
        "roadmap",
        "aggregate",
        "--check",
        "--roadmap",
        "r.yaml",
    ])
    .expect("parses");
    assert!(
        matches!(
            &cli.command,
            Commands::Roadmap(RoadmapCommands::Aggregate {
                write: false,
                check: true,
                entries: None,
                roadmap,
            }) if roadmap == &PathBuf::from("r.yaml")
        ),
        "parsed as {:?}",
        cli.command
    );
    assert!(
        Cli::try_parse_from(["pmat", "roadmap", "aggregate", "--write", "--check"]).is_err(),
        "--write and --check are exclusive"
    );
}

// ---------------------------------------------------------- id authority

fn git(dir: &Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .expect("git must be runnable");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn roadmap_fragments_an_id_spent_by_a_fragment_on_another_ref_is_spent() {
    // PMAT-680 made the mint read EVERY ref's roadmap. In an opted-in repository a
    // ticket added on another branch lives only in that branch's entries/<id>.yaml
    // until the post-merge aggregation, so reading the ref's roadmap.yaml alone
    // mints its id a second time.
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let root = tmp.path().join("repo");
    let docs = root.join("docs/roadmaps");
    std::fs::create_dir_all(docs.join("entries")).expect("mkdir");
    std::fs::write(docs.join("roadmap.yaml"), BASE).expect("base");
    std::fs::write(docs.join("entries/.gitkeep"), "").expect("keep");
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-q", "-m", "base"]);

    git(&root, &["checkout", "-q", "-b", "other"]);
    std::fs::write(
        docs.join("entries/PMAT-050.yaml"),
        "- id: PMAT-050\n  title: on another branch\n  status: planned\n",
    )
    .expect("fragment");
    git(&root, &["add", "."]);
    git(&root, &["commit", "-q", "-m", "a ticket on another branch"]);
    git(&root, &["checkout", "-q", "main"]);
    assert!(
        !docs.join("entries/PMAT-050.yaml").exists(),
        "not in this checkout"
    );

    let path = docs.join("roadmap.yaml");
    let authority = IdAuthority::discover(&path);
    assert_eq!(authority.max_id_across_refs(), Some(50));

    // A fragment NAME is read by the same rule as a roadmap ROW of that id — one
    // reader, `roadmap_text::id_number` — so a ticket cannot be spent in one form
    // and free in the other. `ABC-70-1` spends 1 as a row, so it spends 1 as a
    // fragment; a second parser here (e.g. `parse_id`, which reads 70) would move
    // the maximum and this line.
    git(&root, &["checkout", "-q", "other"]);
    std::fs::write(
        docs.join("entries/ABC-70-1.yaml"),
        "- id: ABC-70-1\n  title: a legacy multi-dash id\n  status: planned\n",
    )
    .expect("fragment");
    git(&root, &["add", "."]);
    git(&root, &["commit", "-q", "-m", "a multi-dash fragment"]);
    git(&root, &["checkout", "-q", "main"]);
    let as_rows =
        crate::services::roadmap_text::max_id_number("roadmap:\n- id: PMAT-050\n- id: ABC-70-1\n");
    assert_eq!(as_rows, Some(50));
    assert_eq!(
        authority.max_id_across_refs(),
        as_rows,
        "fragment names and roadmap rows must spend the same numbers"
    );
    let minted = RoadmapService::new(&path)
        .add_item_with_next_id(an_item)
        .expect("mint");
    assert_eq!(minted, "PMAT-051", "PMAT-050 is spent on `other`");
}
