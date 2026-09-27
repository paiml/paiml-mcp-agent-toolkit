# Implementation receipt — PMAT-1458 (#1458): release pmat 3.42.0

The release commit follows the 3.41.0 precedent (`ad214bd98`, PMAT-1399). That commit touched
`Cargo.toml`, `Cargo.lock`, `CHANGELOG.md` and one roadmap row. This one adds one `exclude` entry, for the reason given below.

| Change | Where |
|---|---|
| `version = "3.41.1"` → `"3.42.0"` | `Cargo.toml` |
| the `pmat` package entry → `3.42.0` (`cargo metadata --locked` passes) | `Cargo.lock` |
| `## [3.42.0] - 2026-09-28`, under `[Unreleased]` | `CHANGELOG.md` |
| `exclude` gains `"/.github/"` (package size, below) | `Cargo.toml` |
| PMAT-1458 row (filed with this tree's own `pmat work add --epic 1457 --priority P1 --kind code`), `inprogress` | `docs/roadmaps/roadmap.yaml` |
| `pmat work sync --direction github-to-yaml`: PMAT-678 → completed (#1274 closed by #1456), GH-1457 row for the new epic | `docs/roadmaps/roadmap.yaml` |

**Why minor.** ONT-11 (#1454) adds a rule (CB-2118) and a flag (`comply check --list`). FLOW-03
(#1446) makes `work add` refuse a ticket without an epic, priority and kind.

**Why an epic.** FLOW-03 files every ticket under an open issue labelled `epic`. The only open
epics were #1017–#1019, audit findings unrelated to releases. #1457, "Epic: pmat release train",
holds release tickets, and #1458 is linked under it (`gh api …/issues/1457/sub_issues` → `1458`).

**CHANGELOG coverage.** Every PR merged to master since v3.41.1 that changes behaviour has a line:

- #1454: ONT-11
- #1446: FLOW-03
- #1443: roadmap sync as the one writer
- #1442: validate-book
- #1438 and #1449: release assets
- #1456: `make publish-from-tag`

Lifecycle-only PRs are listed by number.

**Package size (`feature-matrix.yml` job `package size`).** The first CI run on this PR was RED:
`packaged: 9.0 MiB compressed`, at the job's 9.0 MiB budget. Master at `3306e0052` measured 8.9, and
this release's CHANGELOG entry tipped it over. Measured locally with `cargo package --no-verify`:
the crate was 9,385,367 B (8.951 MiB, which cargo prints as 9.0). `Cargo.toml`'s `exclude` now also lists
`"/.github/"`, which removes 28 workflow files (73 KB compressed) and gives 9,307,924 B (8.877 MiB,
printed as 8.9). Nothing reads `.github/` at build time: `build.rs` names no such path, and no
`include_str!`/`include_bytes!` in `src/` reaches it. The budget itself is unchanged. The margin is now about 76 KB,
so it will run out again.

**CB-2115 on this tree:** `126 open item(s) and 126 open issue(s) are in bijection`.

## After merge (not in this diff)

1. Tag `v3.42.0` on the merge commit on master.
2. The clean-room gate (`release.yml`) goes green on the tag's sha, and the runner is recorded.
3. Dogfood is GO.
4. `make publish-from-tag TAG=v3.42.0` publishes. The receipt records the crates.io version and the
   commit sha.
