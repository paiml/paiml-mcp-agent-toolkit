# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.43.0] - 2026-10-16

### Fixed

- **cargo deny: RUSTSEC-2024-0449 (unsafe-libyaml, unmaintained) no longer reds the
  supply-chain gate (#1489).** The advisory lists `patched = []`, so no upgrade clears it.
  `deny.toml` carries a dated `ignore` entry with its reason and removal condition
  (#1492, moving YAML to a maintained crate).


## [3.42.0] - 2026-09-28

### Added

- **ONT-11 (ONT-001 R-21): pmat maps pv verdicts, lists its rules, and holds arming
  monotone (#1422, #1454).**
  - `pmat comply check`'s CB-1201 now reads `pv lint`'s exit code. Exit 2 (`Unknown`) is a
    `Fail` carrying pv's `decline:` line verbatim, exit 3 is a `Fail` carrying its `error:`
    line, and pv missing while `contracts/` exists is a `Fail` with `decline: pv not found`.
    All three used to be a `Warn` under the default `pv_lint_is_error = false`, and a `Warn`
    exits 0.
  - `pmat comply check --list --format json` prints every rule as `{id, group, name}` (174
    rules), taken from the same registry `--checks` validates against.
  - New rule CB-2118 `contracts-armed-gates-monotone`: an entry of `armed_gates[]` or
    `armed_shapes[]` in `contracts/lint-baseline.json` that was armed at the merge-base and
    is gone at HEAD is a `Fail` naming each one. No `contracts/` is a `Skip` with
    `decline: no contracts/`, and a repo that never armed passes, so the rule never forces
    arming.
  - pmat takes no ontology types: no `aprender_contracts::` under `src/` (F-31).
  - No required check runs these rules yet. In the ledger they are NEUTERED behind the
    `continue-on-error` "Ladder gate" step. Blocking an `Unknown` in `ci / gate` is
    PVL EV-15's job.
- **`pmat work add` refuses an untriaged ticket and links it under its epic (FLOW-03,
  #1440, #1446).**
- **`pmat roadmap sync` is the one writer of `roadmap.yaml`, and `sync --check` is the
  parity gate (#1370, #1443).**
- **`make publish-from-tag TAG=vX.Y.Z [DRY_RUN=1]`: the crates.io publish path that
  `release.yml` names, which had never existed (PMAT-678, #1274, #1456).** It refuses
  before running cargo when the tag is missing, badly shaped, not on origin/master, or
  disagrees with `Cargo.toml`. It publishes with `--locked` from a pristine detached
  worktree of the tag, under `env -u CARGO_REGISTRY_TOKEN`. `.claude/agent-memory/` is now
  gitignored, and `make release-verify` installs with `--locked`.

### Fixed

- **`make validate-book` fails when the pmat-book checkout is absent (#1441, #1442).**
- **Release assets: linux-gnu assets have a glibc 2.35 floor (#1432, #1417, #1438), and
  cargo-zigbuild 0.23.4 fixes aarch64-gnu linking (#1448, #1449).**

Lifecycle PRs #1444 #1447 #1450 #1452 #1455 carry no user-visible change.

## [3.41.1] - 2026-09-18

### Fixed

- **A read-only `pmat analyze dead-code` could rewrite the analysed project's
  `Cargo.lock` (#1403, #1406).** The analyzer runs `cargo check` inside the directory it is
  analysing, with that project's lockfile writable, and simply assumed cargo would not need
  to write. Cargo rewrites a lockfile whenever the resolution it computes differs from the
  bytes on disk — so an ambient `[patch.crates-io]` in `$CARGO_HOME/config.toml` was enough
  to have `[[patch.unused]]` appended to a lockfile that pmat had promised only to read.
  An ambient patch is one trigger among several: a stale lockfile, a different lockfile
  `version`, a `[replace]`, or no lockfile at all all produce the same rewrite. Reproduced
  byte-identically on cargo 1.98.0 in about two seconds with a throwaway `CARGO_HOME`
  holding one `[patch.crates-io]` entry, so this is **not** a toolchain difference — any
  cargo does it.

  The fix is `LockfileGuard`: the workspace root's lockfile is snapshotted before the cargo
  child spawns and restored on every exit path — success, cargo failure, deadline kill, and
  `Drop`. `--locked` and `--frozen` stay **out**, and a test now pins their absence: adding
  `--locked` was tried on 2026-08-25 and reverted the same day because it silently disabled
  the compiler scan (80 dead functions became 0). Contract
  `contracts/dead-code-lockfile-isolation-v1.yaml`, 10 obligations and 11 falsification
  tests.

  Latent since `2bdc6b90c` (2026-08-25), when that `--locked` revert left the analyzer with
  no mechanism enforcing the read-only property it claims. What was new was only the
  observer: paiml/.github#72 raised the release clean room's timeout, so its unit-test stage
  reached a verdict on a pmat tag for the first time since v3.39.0.

### Note on 3.41.0

**3.41.0 was cut but never published, and this release supersedes it.** PR #1400 merged as
`ecd97c6bc` and the annotated tag `v3.41.0` was pushed on it; the release clean room
(`release.yml` run **35286438196**, job `gate / cpu-gates`) then failed in its unit-test
stage — 21774 passed, 2 failed — on the two tests that assert the defect above, so every
publish and release step behind that gate was skipped. There is no `v3.41.0` release object
and no `3.41.0` on crates.io (`/api/v1/crates/pmat/3.41.0` -> HTTP 404). The `[3.41.0]`
section below is left exactly as it was written, and its tag is left pointing where it
points, as the record of a cut that failed its own gate; everything that section describes
ships here in 3.41.1.

## [3.41.0] - 2026-09-17

### Added

- **Roadmap fragments: `pmat work add` writes one file per ticket, so `roadmap.yaml`
  leaves the merge path (#1364).** In a repository that has opted in — the predicate is
  `[ -d docs/roadmaps/entries ]`, the same one paiml/.github's `roadmap-fragment-parity`
  gate uses — a ticket is `docs/roadmaps/entries/<id>.yaml` and `roadmap.yaml` is a
  generated aggregate. `RoadmapService::save`, `upsert_item`, `remove_item` and
  `replace_item_raw` all write fragments under that predicate. The problem being solved is
  Amdahl serial fraction 1: N pull requests contend on one file however disjoint their
  code is. Measured in paiml/aprender on 2026-09-15, 146 PRs over 10 days against 88
  merged, and of the last 25 `merge_group` runs **7 succeeded and 15 were cancelled**.
  New command `pmat roadmap aggregate` renders the aggregate and `--check` verifies it
  without writing. On aprender's 899-row roadmap three runs were byte-identical to each
  other, to the committed file and to the Python aggregator it replaces; 22/22
  differential scenarios byte-equal. **pmat's own repository has NOT opted in** — there is
  no `docs/roadmaps/entries/` on master — so nothing here changes for pmat itself.
- **`pmat work estimate record` and `pmat work estimate check` (#1382).**
  `docs/audits/impl-estimates.jsonl` was written by hand and gated by nothing: all 16
  correctly-keyed rows for this repository carried `unit: null`, so the reader
  (`estimate.sh`) excluded every one of them and could produce no estimate at all. The
  ledger now has exactly one writer, which refuses an unpoolable row at write time and
  appends under `O_APPEND` in a single `write_all`, plus a `--lib` test that fails on any
  committed row without `unit`. 18 legacy rows were backfilled from their own text or
  receipts.
- **`make gate` (#1368).** pmat had no declared gate, so paiml-implement's discovery fell
  back to `cargo test --workspace`. `make gate` runs `scripts/gate.sh`: **one table**
  mapping every required context (`gate`, `ci / gate`, `feature-gate`,
  `docs build (docs.rs environment)`, `pmat score`, `provable ladder`) to the legs that
  can run locally — 48 rows at this release, 32 of which run here. The 16 that cannot are
  **printed by name on every exit** with a categorised reason rather than silently
  skipped, and a required context with no row is refused with exit 2.
- **`pmat work edit --notes` (#1391).** The only other writer, `-d`, replaces a row's
  acceptance criteria, so there was no way to add a cross-reference to a ticket without
  destroying its criteria.

### Fixed

- **A crate that does not compile could be reported as cleanly measured (#1305, #1388).**
  The dead-code analyzer inherited `CARGO_TARGET_DIR`. Cargo fingerprints a workspace
  member by its path *relative to the workspace root*, so two fixture crates with the same
  package name sharing one target dir share a fingerprint; if the broken crate's source
  was older than the other's fingerprint, cargo called it fresh, replayed the other
  crate's empty diagnostics and exited 0 — the analyzer read that as
  `violations=[] not_applicable=None`, a clean full measurement of a crate that cannot
  compile. `ci / test` and `ci / coverage` mount the same per-PR target dir, which is how
  the two jobs answered for each other. The analyzer now builds into a target directory
  only its own workspace root uses. This was tracked for months as an intermittent test
  flake; it was not a flake.
- **`pmat work migrate` rewrote `roadmap.yaml` with no lock (#1389).** It used a bare
  `std::fs::write` after an unlocked read, while every `RoadmapService` writer holds
  `<git common dir>/pmat/roadmap-id.lock`; in a fragment-mode repository it also rewrote
  the generated aggregate and migrated no fragment. `RoadmapWriteLock`
  (`src/services/roadmap_write_lock.rs`) now exists only while the exclusive flock is
  held, and its methods are the only raw writes to a roadmap path. A `--lib` taint
  analysis over every file the crate compiles outside `cfg(test)` (3037 files, a full
  `syn` walk of `mod` and `include!` at any depth) follows path literals to their
  serialisation sites and refuses any that is not an allowed `(function, sink)` pair.
- **CB-200 is back under its baseline and is measured where a merge is decided
  (#1266, #1394).** Definitions below grade A had drifted from the banked 1688 to 1741.
  Measured with the same binary against cold indexes and clean clones, the baseline tree
  reproduces 1688 exactly — the identical multiset of (file, definition), so this was code
  drift and not grader drift. This release measures **1680**: 61 definitions brought to
  grade A, 0 newly below A. The baseline was not raised. The measurement now runs in a
  required CI job, which is what had been missing: it was invisible to CI because CI
  checkouts carry no index.

