# Implementation receipt — PMAT-660 (AD-06: the worker receipt carries the gate verdict)

Spec: `docs/specifications/agentic-delivery-pmat.md` §5.3 / §9.6. Epic #1153. Branch `PMAT-660-receipt-gate`.
Bundle side: paiml/paiml-implement#7 (merged 0fd5589), installed here (`verify.sh` 25 PASS rows).

## What changed
- Bundle: worker rule 4b (`gate_cmd` once, `gate {cmd, ok, stages_measured, not_measured}` in the receipt; missing ⇒ `partial=true`); `scripts/receipt-lint.sh` with `--rerun` diff and `--self-test`; SKILL.md Phase 2 step 3 re-runs the gate; `verify.sh` check `receipt-gate`.
- pmat: spec §9.6 note, this receipt, the PMAT-657 worker's real receipt as `docs/audits/worker-receipt-PMAT-657.json`, the PMAT-660 roadmap row.

## Verification (orchestrator runs)

| check | result |
|---|---|
| RED — before this change no lint existed: a receipt without `gate` was accepted as complete (every worker receipt through PMAT-657's first two) | by construction; `receipt-lint.sh` did not exist in the installed bundle |
| GREEN — `receipt-lint.sh --self-test` | four ✓: no-gate → partial; with gate → complete; agreeing rerun → pass; disagreeing rerun → finding (exit 1) |
| the PMAT-657 worker's real receipt | `receipt complete: gate.ok=False stages_measured=5 not_measured=[]`, exit 0 |
| rerun of `pmat verify` on the worker's exact tree (ebf1580e2, clean scratch worktree) vs its claim | **FINDING** (`receipt-lint.sh --rerun`, exit 1): `ok` agrees (false/false) but `stages_measured` receipt=5 rerun=4 and `not_measured` receipt=[] rerun=[complexity]. Cause, verified: the worker ran the gate on its dirty tree before committing, where verify's complexity stage measures the changed files; the orchestrator's rerun on the committed tree found nothing changed against HEAD and withdrew the stage. Same tree, two measurement sets — the claim was honest and still not comparable. Rule for the next bundle revision: the worker runs `gate_cmd` on the tree it commits, and the orchestrator measures complexity directly (`analyze complexity --files <changed>`) as CRUX-04 already requires |
| bundle `verify.sh` | 25 PASS, `receipt-gate` among them |

Verdict: **DONE** once the PR merges green (docs + one JSON; no Rust). The first real use of the lint produced a finding on the first receipt it examined.
