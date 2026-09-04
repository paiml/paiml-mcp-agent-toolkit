# impl receipt — PMAT-671: release 3.37.0

## Identity

| field | value |
|---|---|
| ticket | PMAT-671 |
| release PR | #1189 (`release/3.37.0`), merged as 9beccfbd1 |
| tag / release | `v3.37.0` on 9beccfbd1; https://github.com/paiml/paiml-mcp-agent-toolkit/releases/tag/v3.37.0 |
| crates.io | pmat 3.37.0 (`cargo publish --locked` from the tag checkout) |
| carried | #1188 → #1186 (PMAT-670), #1187 → #1185 (PMAT-669), on top of CRUX-07 (#1183) and AD-01…AD-10 |
| routing | direct; quorum on the release diff (AD-04): 3 runs — see below |

## Why one PR

The runner fleet was saturated (the fix PRs' legs sat queued for hours), so the release branch merged both reviewed fix branches and went through one CI cycle instead of three. GitHub marked #1187 and #1188 merged when their heads reached master through #1189.

## Gates (re-run by the orchestrator)

| gate | result |
|---|---|
| `pmat verify --format json` on 5cfa31439 | format/satd/clippy/tests ok; complexity withdrawn on a clean tree, re-measured at 30/25 on the changed files: no violation, 3/2 control exits 1 |
| ledgers | `analyze unrun-tests --check-ledger` 0, `analyze reachability --check-ledger` 0 |
| `make validate-book` with the 3.37.0 release binary first on PATH | PASS with controls (Ch05, 07, 13, 14) |
| `scripts/work-ladder-claim-audit.sh` against the release binary | GREEN; `--self-test` OK (RED against a no-op pmat) |
| quorum run 1 (5cfa31439) | lane 2 FAIL: audit script lacked `--self-test`, used python3 → fixed (2edf6c5a4) |
| quorum run 2 (2edf6c5a4) | lane 3 FAIL: a claimed compile error in `require_equation` (refuted by the build; expression simplified anyway) + L0 is a ladder level, docs said L1..L5 (fixed); lane 2 FAIL: no dog-food receipt in the cut (by design: AD-02 runs on the published crate; noted on the row and in the PR) → ac2b6a874 |
| quorum run 3 (ac2b6a874) | 3/3 PASS — `docs/audits/quorum-PMAT-671.json` |
| PR checks | 40/40 green on e5734f499 (verdict commit), auto-merged |
| `scripts/release-check.sh` after publish | `3.37.0 is tagged, released and on crates.io` |
| `scripts/dogfood-published.sh 3.37.0` (AD-02) | GO — 13 checks, 0 failures; `docs/audits/release-3.37.0-dogfood-published.md` |

## Jidoka log

- `git fetch origin master --tags` exits 1 on a `nightly` tag clobber; the publish chain now fetches tags separately and reports.
- A leftover `.cargo/config.toml` from a failed verify redirected a worktree's target dir; the mutation chain asserts binary freshness (`-nt` the mutated file).
- The PMAT-669 roadmap row had been dropped by a cascade on its branch; restored in the release cut from the branch's first commit.

## Verdict

DONE.
