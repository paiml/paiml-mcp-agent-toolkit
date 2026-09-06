# Implementation receipt — PMAT-662 (AD-08: swappable executor, width bounded by a budget)

Spec: `docs/specifications/agentic-delivery-pmat.md` §6 / §9.8. Epic #1153. Branch `PMAT-662-executor-width`.
Bundle side: paiml/paiml-implement#8 (`skills/quorum-review/quorum-review.sh`, `agents/paiml-agy-delegate.md`, `verify.sh`). Routing: direct (bundle shell + docs).

## Verification (orchestrator runs)

| check | result |
|---|---|
| RED (before) | `--executor` did not parse; width was a constant 1..10, so a width-20 brief was refused and an unknown executor could not be expressed |
| GREEN — `quorum-review.sh --self-test` | five ✓: unknown executor refused (exit 2); width 21 refused; width 20 accepted (dry run); `PAIML_MAX_WIDTH=5` refuses 6; executor `claude` accepted |
| bundle `verify.sh` | `quorum-self-test` PASS among 26 rows |
| bashrs on the script | 0 errors |
| the ≤1-Claude-subagent rule | unchanged: `claude` as an executor is a refusal that tells the orchestrator to run the lanes itself, sequentially |

## What this does not do
`kimi` is wired by CLI convention (`-p`, `--output-format json`, `--json-schema`) and refused when the binary is absent; it has not been run against a real kimi installation here — the first real run is the control that closes that gap.

Verdict: **DONE** once the PR merges green (docs only on the pmat side).
