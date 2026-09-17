#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-1385 (#1385) — `pmat work migrate` writes under the repository lock, keeps
//! every byte it does not normalise, and in a repository with
//! `docs/roadmaps/entries/` never opens the generated `roadmap.yaml` for write.
//!
//! Before: `handle_work_migrate` read `roadmap.yaml` with no lock and `write_migration`
//! wrote it back with a bare `std::fs::write` — the lock test below saw the rewrite
//! land while another process held the exclusive lock — and in fragment mode it
//! rewrote the aggregate and left every fragment's legacy status where it was.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs`, beside the PMAT-1363 suites.

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fs2::FileExt;

use crate::cli::handlers::work_handlers::handle_work_migrate;
use crate::services::roadmap_id_authority::IdAuthority;
use crate::services::roadmap_service::RoadmapService;

const HEADER: &str = "roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: null\n";

/// A project directory holding `docs/roadmaps/roadmap.yaml` with `base`, and
/// `entries/` when `with_entries`. Returns (guard, project path, roadmap path).
fn project(base: &str, with_entries: bool) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let roadmaps = dir.path().join("docs/roadmaps");
    std::fs::create_dir_all(&roadmaps).expect("mkdir");
    if with_entries {
        std::fs::create_dir_all(roadmaps.join("entries")).expect("mkdir entries");
    }
    let roadmap = roadmaps.join("roadmap.yaml");
    std::fs::write(&roadmap, base).expect("write base");
    let project = dir.path().to_path_buf();
    (dir, project, roadmap)
}

fn migrate(project: &Path, dry_run: bool, backup: bool) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("runtime")
        .block_on(handle_work_migrate(
            Some(project.to_path_buf()),
            dry_run,
            backup,
            false,
        ))
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .map_err(|e| format!("read {}: {e}", path.display()))
        .expect("a fixture file is readable")
}

#[test]
fn work_migrate_waits_for_the_repository_lock() {
    for with_entries in [false, true] {
        let base = format!("{HEADER}roadmap:\n- id: PMAT-001\n  title: first\n  status: done\n");
        let (_dir, project, roadmap) = project(&base, with_entries);
        let written = if with_entries {
            roadmap
                .parent()
                .expect("the roadmap has a directory")
                .join("entries/PMAT-001.yaml")
        } else {
            roadmap.clone()
        };

        let lock_path = IdAuthority::discover(&roadmap).lock_path;
        let held = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .expect("open lock");
        held.lock_exclusive().expect("take the exclusive lock");

        let target = project.clone();
        let writer = std::thread::spawn(move || migrate(&target, false, true));
        std::thread::sleep(Duration::from_millis(500));
        let landed = written.exists() && read(&written).contains("status: completed");
        assert!(
            !landed,
            "pmat work migrate wrote {} while another process held {} (entries/: {with_entries})",
            written.display(),
            lock_path.display()
        );
        assert!(
            !writer.is_finished(),
            "migrate returned without waiting for the lock (entries/: {with_entries})"
        );

        FileExt::unlock(&held).expect("release");
        drop(held);
        writer
            .join()
            .expect("writer thread")
            .expect("migrate succeeds once the lock is free");
        assert!(
            read(&written).contains("status: completed"),
            "{} after the lock was released",
            written.display()
        );
    }
}

#[test]
fn work_migrate_keeps_every_byte_it_does_not_normalise() {
    let base = format!(
        "# hand-written roadmap: this comment must survive\n{HEADER}roadmap:\n\
         - id: PMAT-001\n  title: first\n  status: done\n  custom_key: kept # trailing\n\
         - id: PMAT-002\n  title: second\n  status: WIP\n  notes: |\n    a block scalar\n    of two lines\n"
    );
    let (_dir, project, roadmap) = project(&base, false);

    migrate(&project, false, true).expect("migrate");

    let expected = base
        .replace("status: done", "status: completed")
        .replace("status: WIP", "status: inprogress");
    assert_eq!(read(&roadmap), expected, "only the status spellings change");
    assert_eq!(
        read(&roadmap.with_extension("yaml.bak")),
        base,
        "the backup is the roadmap as it was read under the lock"
    );
}

#[test]
fn work_migrate_in_fragment_mode_never_opens_the_aggregate() {
    let base = format!(
        "{HEADER}roadmap:\n- id: PMAT-001\n  title: first\n  status: done\n\
         - id: PMAT-005\n  title: fifth\n  status: planned\n- id: PMAT-007\n  title: seventh\n  status: planned\n"
    );
    let (_dir, project, roadmap) = project(&base, true);
    let entries = roadmap
        .parent()
        .expect("the roadmap has a directory")
        .join("entries");
    // A fragment superseding PMAT-005 with a legacy spelling of its own.
    std::fs::write(
        entries.join("PMAT-005.yaml"),
        "- id: PMAT-005\n  title: fifth, edited\n  status: WIP\n",
    )
    .expect("write fragment");

    migrate(&project, true, true).expect("dry run");
    assert_eq!(read(&roadmap), base, "a dry run writes nothing");
    assert!(
        !entries.join("PMAT-001.yaml").exists(),
        "a dry run writes no fragment"
    );

    migrate(&project, false, true).expect("migrate");

    assert_eq!(
        read(&roadmap),
        base,
        "roadmap.yaml is generated here: byte-identical"
    );
    assert!(
        !roadmap.with_extension("yaml.bak").exists(),
        "no backup of a file that was not written"
    );
    assert_eq!(
        read(&entries.join("PMAT-005.yaml")),
        "- id: PMAT-005\n  title: fifth, edited\n  status: inprogress\n",
        "the fragment's own legacy status is migrated in place"
    );
    assert_eq!(
        read(&entries.join("PMAT-001.yaml")),
        "- id: PMAT-001\n  title: first\n  status: completed\n",
        "a base row with a legacy status gets a superseding fragment"
    );
    assert!(
        !entries.join("PMAT-007.yaml").exists(),
        "a base row with nothing to migrate gets no fragment"
    );

    let view = RoadmapService::new(&roadmap)
        .load()
        .expect("the view parses");
    let status = |id: &str| format!("{:?}", view.find_item(id).expect(id).status);
    assert_eq!(status("PMAT-001"), "Completed");
    assert_eq!(status("PMAT-005"), "InProgress");
}

#[test]
fn work_migrate_in_fragment_mode_refuses_before_writing_anything() {
    // `bad id` needs a superseding fragment it cannot have: its id is not a filename,
    // and PMAT-001, earlier in the file, must not land a fragment on its way there.
    let base = format!(
        "{HEADER}roadmap:\n- id: PMAT-001\n  title: first\n  status: done\n\
         - id: bad id\n  title: second\n  status: done\n"
    );
    let (_dir, project, roadmap) = project(&base, true);
    let entries = roadmap
        .parent()
        .expect("the roadmap has a directory")
        .join("entries");

    let err = migrate(&project, false, true).expect_err("an unwritable row is refused");

    assert!(err.to_string().contains("nothing was written"), "{err}");
    assert_eq!(read(&roadmap), base);
    assert_eq!(
        std::fs::read_dir(&entries).expect("entries").count(),
        0,
        "PMAT-001's fragment must not land when PMAT-002's is refused"
    );
}
