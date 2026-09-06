# impl receipt — PMAT-675 (release path restored: clean-room gate → verify → prerelease)

| field | value |
|---|---|
| ticket | PMAT-675 · kind=code · PR #1203 (merged 1f83a1c99) |
| route | direct (workflow edits are orchestrator-only) |
| commits | c8fda0bad (workflows + 2 lib tests), 5b6dec9ee (quorum fixes: tag sha to the gate, explicit listener dispatch, no if: on prerelease), 6868246e1 (panic→assert, ratchet), 0f7a7be43 (master merge) |
| acceptance | `cargo test --lib -- release_workflow_tests` 2 passed; RED observed on a planted `continue-on-error` (`/jobs/verify/continue-on-error`) |
| mutation (discrimination) | the planted continue-on-error (local); live probe on `v3.38.0` (run 34034967876): `create-release` ✓, `gate / lint-gate` ✗ at the fleet Banned path scan (pmat's own tree, PMAT-686/687), verify and prerelease skipped — a red gate creates no release, as designed; the failure named the gate, not verify, because the tree could not reach verify |
| quorum | 3/3 lanes (agy `--mode plan`): needs-changes ×3 → all fixed in 5b6dec9ee; conversations 2b7153a4…, e5d77397…, 3eaa9248… |
| pv contract | `contracts/work/PMAT-675.yaml` (kind: pattern) valid, lint PASS |

## Gaps / findings
- The fleet lint-gate fails on pmat's own tree (banned-path literals in the hardcoded-path analyzer's fixtures) until PMAT-687 lands or a per-repo EXCLUDE is added to `paiml/.github` unified-gate.yml; the 3.39.0 prerelease is created by hand.
- `docker-publish.yml` demoted, not deleted (docs/DOCKER.md documents the image, #1122) — HRQ.

verdict: DONE pending the required checks on PR #1203 (merged 1f83a1c99).

IMPL-PMAT-675-RECEIPT-END
