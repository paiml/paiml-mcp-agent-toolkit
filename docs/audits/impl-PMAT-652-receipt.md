# impl receipt — PMAT-652 (AD-02: post-publish dog-food of the crate crates.io serves)

| field | value |
|---|---|
| ticket | PMAT-652 (agentic-delivery-pmat.md §3.6 / §9.2) |
| branch | PMAT-652-dogfood-published (off master 627fb6a85, after the 3.36.0 release and CRUX-03) |
| phase gate | `pmat verify --format json` on the branch: **exit 0** after the PMAT-654 fix (first run: tests stage red on four `models::git_context` tests that assume `.git` is a directory — the branch runs in a linked worktree; fixed in the same PR, see Jidoka) |
| DoD gate | the PR's required checks |
| quorum | never (`--quorum never` per the invocation) |
| subagents | 0 |

## Deliverables
`scripts/dogfood-published.sh`, `make dogfood-published VERSION=`, `contracts/dogfood-published-v1.yaml` (`pv validate` valid, `pv lint` PASS), spec §9.2 implementation note, script output `docs/audits/release-3.36.0-dogfood-published.md`.

## Verification (orchestrator runs)

| check | result |
|---|---|
| control: `bash scripts/dogfood-published.sh 9.9.9` | **exit 1** — `FAIL: dogfood-published: pmat 9.9.9 is not on crates.io`; no receipt written |
| control: `bash scripts/dogfood-published.sh v3` | **exit 1** — `'v3' is not a version` (refused before any network call) |
| usage: no argument | exit 2 |
| real run: `bash scripts/dogfood-published.sh 3.36.0` | **exit 0** — `GO: pmat 3.36.0 — 13 checks, 0 failure(s)`; registry size 8931399, created 2026-09-03T19:23:58Z; installed `--version` = `pmat 3.36.0`, no commit line (CRUX-21); receipt `docs/audits/release-3.36.0-dogfood-published.md` |
| `bashrs lint scripts/dogfood-published.sh` | 0 errors |

## Jidoka

- PMAT-654 (linked): the four `git_context` tests fail in any linked worktree because their repo-root walk requires `.git/HEAD`; the helper now accepts a `.git` file. RED in the worktree (4 failed, exit 101) → GREEN (7 passed); five whys in `.pmat/jidoka.jsonl`.
- Ticket-id collision: `pmat work add` allocated PMAT-653 twice (once on the docs branch #1173, once here); this branch's ticket was renumbered PMAT-654 before committing (PMAT-635 class).

## Named mutation
The hand-run predecessor (`/mnt/nvme-raid0/agent-wt/pmat-release/publish-3.36.0.sh dogfood`) aborted under `set -e` at `commit=$(grep …)` because the crates.io build has no `commit:` line — the receipt was never written although every gate had passed. The script carries `|| true` on that grep with the CRUX-21 reason beside it; deleting the `|| true` reproduces the abort on any crates.io build.

## Decisions taken as the conservative option ([A])
- [A] Pin the artifact by registry `crate_size` + `created_at`, not a commit line (CRUX-21).
- [A] The registry leg runs first, so a fabricated version costs one HTTP call, not a `cargo install`.
- [A] The script writes `docs/audits/release-<v>-dogfood-published.md`; the hand-written release-chain receipt `release-<v>-dogfood.md` (in #1173) stays and links to it — two artifacts, no add/add collision between the PRs.

## Verdict
Real run GREEN, controls RED; DONE when the PR merges.
