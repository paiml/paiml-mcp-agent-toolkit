# Release plan: pmat 3.43.0

- **Epic:** #1457 (pmat release train), train `3.43.0`. Rows: `docs/roadmaps/epics.yaml` (R43).
- **Last tag:** `v3.42.0`. **Next tag:** `v3.43.0`. **Following tag:** `v3.44.0` (look-ahead, epic #1017, kit `docs/lookahead/3.44.0.yaml`).
- **Goal:** a release that runs end to end on its own. `release.yml` has not completed since v3.39.0, and 3.42.0 shipped through `make publish-from-tag`. 3.43.0 ships with the release workflow, the master gate and the coverage job green. No new features.

## Rows (core)

| Row | Item | done_when | Baseline (measured) | First-green proof |
|---|---|---|---|---|
| C1 | #1461 (PMAT-1461), must-carry | the prerelease job of `release.yml` dispatches the release listeners with exit 0 | HTTP 403, missing `actions: write` (#1461) | the release.yml run URL on a `v3.43.0-rc` dry run |
| C2 | #1358 (PMAT-1358), must-carry | `release.yml` gate / cpu-gates completes under its timeout | cancelled at about 30m39s on every release run since v3.39.0 | the first `release.yml` run with conclusion `success` |
| C3 | #1433 (GH-1433), must-carry | `pmat comply check --checks CB-2113,CB-2115` exits 0 on master | CB-2115 reported 17 roadmap and GitHub orphans | the master `ci / gate` run that is green on traceability |
| C4 | #1202 (PMAT-694) | the coverage job passes 3 consecutive master runs with no flake rerun | flaky: quality_proxy tests compile the host workspace | 3 run URLs |
| C5 | #1127 (GH-1127), absorbed by #1202 | quality_proxy child compilers run under a memory bound | deadline and process group, no memory bound | the test that fails without the bound |

C4 and C5 are core but can roll: if they are still open at the cut, they move to 3.44.0.

## Release gate (v3.43.0)

1. The issues labelled `must-carry` on milestone `3.43.0` are closed (C1–C3). Every other open item rolls to `3.44.0` if the #1017 body cites it, and to `backlog` otherwise.
2. The cut happens on `release/3.43.0`: version bump, CHANGELOG and the release ticket. The ticket is filed at the cut with `pmat work add --epic 1457`.
3. `ci / gate` is green on the release commit.
4. The clean room is green on **exactly** the tagged commit. Nothing is uploaded before that.
5. Tag `v3.43.0` on that commit, then run `make publish-from-tag TAG=v3.43.0` (publishing to crates.io is automatic from there). Merge `release/3.43.0` back to master.
6. Tags are immutable. A defect found after the tag ships as 3.43.1.

## Workers

- **Core:** one worker on milestone `3.43.0`, rows C1–C5.
- **Look-ahead:** one slot on `3.44.0`, working on branch `la-44/wip` with a heartbeat every 2 hours. It proposes tickets and never mints them. During a 3.43.0 release pass it yields CI and opens no PRs. It is the first work shed when the 5-hour budget window reaches 80%.

Split decided by a planted 3-lane quorum on 2026-10-10. Receipt: `docs/audits/quorum-epic-3.43.0-r1.md`.
