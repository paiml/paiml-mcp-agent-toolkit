# impl receipt — PMAT-693 [train/BC]: board-check, the cleared-board instrument

**Ticket:** PMAT-693 (kind:code) · **Branch:** `PMAT-693-board-check` · **Orchestrator:** Fable (paiml-implement). Worker: paiml-impl-worker (sonnet) wrote the self-test, the RED commit and the first draft of `board-check.sh`, then hit its 40-turn cap twice on bashrs findings (one resume used); the orchestrator finished the lint fix (three jq `$name[` expansions rewritten as `($name | .[])`, a bashrs SC1087 false positive on jq syntax), the contract and this receipt. `gate_cmd_fallback=true` (discovery); `pmat verify --skip clippy,tests` used (no Rust changed).

## What
`scripts/board-check.sh` + `make board-check` (read-only; `gh` for PRs and issues, a raw-text scan of `docs/roadmaps/roadmap.yaml` for tickets, the lexically newest `docs/audits/dispositions-*.json` as the ledger). Gap rules: an open PR without a `pr` ledger row; an open issue without exactly one milestone (a `complete` row on an open issue is a gap — complete means closed); a non-completed ticket absent from the ledger or with a non-enactable disposition. Flags: `--ledger`, `--repo`, `--roadmap`, `--offline`. Summary line `board-check: <n> gap(s)`, exit 1 iff n > 0. `scripts/board-check-selftest.sh` + `make board-check-selftest` is the acceptance arm.

## RED → GREEN
- RED 122571090 ( 2 files changed, 191 insertions(+)): `make board-check-selftest` failed — `scripts/board-check.sh` did not exist.
- GREEN (this commit): `board-check-selftest: PASSED` — fixture 1 exits 1 naming exactly the planted id (`board-check: 1 gap(s)`), fixture 2 exits 0 (`board-check: 0 gap(s)`).
- Discrimination: the self-test's first fixture has three tickets and the gap list names one; a script that flagged every planned ticket would name two.

## Live board
`bash scripts/board-check.sh` on this repository before Phase E of the 3.40.0 train: `board-check: 45 gap(s)` (issues without a milestone, the train's freshly minted tickets not yet in a ledger, and `#1029` "ledger says complete but the issue is still open"). The offline leg alone: 13 ticket gaps. Phase E enacts them and re-runs this command; the release receipt records the after count.

## Quorum (agy `--mode plan`, width 3, review-only) on 09ec0880e

Lanes 86c6118c-1828-4cf8-a71e-82b36244cfb7, 50aa693e-be2e-43d2-b4a3-097d47755871, fb64e5a1-aef4-4373-bc02-b47d7c64ac12 — 3/3 needs-changes. Every finding re-verified by the orchestrator and fixed in the commit that carries this section:

| # | finding | lanes | fix |
|---|---|---|---|
| 1 | `--offline` printed a clean summary and exited 0 having run one leg of three | 3/3 | a clean partial run exits **3**; 1 with gaps; documented in `--help` |
| 2 | a roadmap that parses to zero `- id:` rows was silently zero gaps | 3/3 | exit **2** "parsed to zero tickets — refusing to report a board it could not read" |
| 3 | the evidence-closure arm still emitted a gap, worded "no evidence-closure" exactly when there was one | lane 3 (verified) | the rule is now stated as it is meant: an OPEN issue whose row carries evidence is a gap in its own words ("close it") — evidence is how a `complete` row gets closed, never an exemption |
| 4 | the self-test drove only the ticket leg (`--offline`), so the issue leg was never exercised, and there was no injection seam | delegate | `--prs-json` / `--issues-json` seams; fixture 2 now runs all three legs (HRQ reject covered, milestoned issue covered); fixture 3 drives the issue leg (unmilestoned → gap, open-but-complete → gap, HRQ reject → not named, exactly 2 gaps); fixture 4 zero-ticket roadmap → exit 2; fixture 5 clean `--offline` → exit 3 |
| 5 | `gh pr list` took gh's default page (30) and `gh issue list --limit 200` could truncate silently | delegate | both legs ask for 200 and refuse a page that is exactly full (exit 2) |
| 6 | `sort` not `sort -V` for the newest ledger (`3.9.0` would beat `3.39.0`) | delegate | `sort -V` |

Agreed sound across all lanes: the RED commit is real (the script absent), fixture 1 discriminates (exit code, exact planted id, count, absence of the other ids), the HRQ-reject exemption needs the ledger row AND the label, the enactable set is a positive allowlist, the jq `($name | .[])` rewrites are identical to `$name[]` on jq 1.6.

After the fixes: `make board-check-selftest` → PASSED (5 fixtures); `bashrs lint` 0 errors; live board against `dispositions-3.40.0.json` → `board-check: 7 gap(s)` exit 1 — the seven tickets this train's Phase 2 / RC / IP complete (PMAT-687, 691, 693, 694, 695, 700, 701), which is E1's exit predicate.

## Gate
`bashrs lint scripts/board-check.sh scripts/board-check-selftest.sh`: 0 errors. `pmat verify --skip clippy,tests --format json`: ok=None measured=2 not_measured=['complexity']. `pv validate` + `pv lint contracts/work/PMAT-693.yaml`: PASS.

Orchestrator turns: 9 (footprint, worker dispatch, one resume, takeover, contract/receipt, quorum fixes)

verdict: GREEN

IMPL-PMAT-693-RECEIPT-END
