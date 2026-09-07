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

## Gate
`bashrs lint scripts/board-check.sh scripts/board-check-selftest.sh`: 0 errors. `pmat verify --skip clippy,tests --format json`: ok=None measured=2 not_measured=['complexity']. `pv validate` + `pv lint contracts/work/PMAT-693.yaml`: PASS.

Orchestrator turns: 6 (footprint, worker dispatch, one resume, takeover, contract/receipt)

verdict: GREEN

IMPL-PMAT-693-RECEIPT-END
