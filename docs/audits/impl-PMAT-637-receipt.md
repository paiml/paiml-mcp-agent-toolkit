# Implementation receipt — PMAT-637 (CRUX-01, #1146)

## Identity

| field | value |
|---|---|
| ticket | PMAT-637 |
| spec | docs/specifications/pmat-architecture-crux-audit.md §8.1 (CRUX-01) |
| branch | PMAT-637-verify-verdict-strict-satd (off master 1188c3a81, after #1158) |
| discover.json sha256 (16) | cc89162672484577 (gate_cmd = make gate-artifact, fallback=false) |
| phase gate | pmat verify; spec §8.1 script (`scripts/verify-verdict-audit.sh`) |
| DoD gate | make gate-artifact |

## Plan and routing

| phase | acceptance command | mode | trigger |
|---|---|---|---|
| P1 tri-state composite verdict; `not_measured[]` derived from declined stages | `cargo test --lib composite_verdict_withdraws_rather_than_asserts_over_a_declined_stage` + legs 1, 1-RED, 1-EMPTY, 1-SKIP | direct | - |
| P2 strict SATD accepts every standard separator and the capitalised marker | `cargo test --lib strict_accepts_every_standard_separator_and_the_capitalised_marker strict_is_a_subset_of_default` + legs 2, A3-M, A3-D, A3-D2 | direct | - |
| P3 docs + contract + DoD | `make gate-artifact`; CLAUDE.md command/dead-path checkers | direct | - |

Quorum: never. Routing direct: `|M|=2` (`src/cli/verify.rs`, `src/services/satd_detector/classifier.rs`) but the change was fully specified by the hardened item and dry-run on a copy before the ticket opened; no subagent dispatched.

## Dispatch ledger

| phase | mode | agent | turns | maxTurns hit | resumed |
|---|---|---|---|---|---|
| P1–P3 | direct | orchestrator | see estimates | - | - |

## Verification — claimed vs re-run (all rows are the orchestrator's own)

| check | before (master 1188c3a81) | after |
|---|---|---|
| spec §8.1 script, fail-fast | `FAIL: leg 1: verify still asserts over a stage it declined` | PENDING |
| spec §8.1 script, all legs | 7 RED (1, 1-EMPTY, 2, A3-M, A3-D, A3-D2, REPO) / 5 controls GREEN — matches the spec's transcript exactly | PENDING |
| unit tests (composite table, strict cases, strict ⊆ default, detection strict) | n/a | PENDING |
| `cargo fmt --all -- --check` | — | PENDING |
| `cargo clippy --lib --bins` | — | PENDING |
| unwrap ratchet | 20343 | PENDING |
| CLAUDE.md command + dead-path checkers | — | clean on the patched copy (dry run) |
| `make gate-artifact` | — | PENDING |

## pv contract

`contracts/verify-verdict-v1.yaml` (new): `verdict_tri_state`, `verdict_end_to_end`, `strict_sees_what_the_gate_blocks`, falsifiers naming the two `--lib` tests and the audit script.

## Invalidated doc claims updated in this PR

- CLAUDE.md:178 — `ok:true ⇒ safe to commit` now also states `ok:null ⇒ no verdict`.
- docs/agent-instructions/autonomous-verify-loop.md — the tri-state reading and `not_measured[]`.
- docs/specifications/pmat-verify-autonomous-preflight.md — exit-code/verdict table, `stages_measured` in the JSON example, the strict SATD stage's real rule.
- `--strict` help string and verify's satd detail line (were "only TODO/FIXME/HACK/BUG").

## Jidoka log

- PMAT-638: `pmat work complete` hangs on its quality gates; `--skip-quality` is forbidden; store moved with `work edit -s`.
- #1160 went DIRTY after #1158 merged (both regenerated `docs/status/unrun-tests-ledger.md`); resolved by merging master and regenerating the ledger on the merged tree — a real commit, not a rerun.

## Decisions taken conservatively [A]

- [A] `ok: null` keeps exit 0 (the item's table); a declined stage withdraws the verdict, it does not start failing the build. The alternative (exit 1 on any decline) would fail every pre-commit run on a doc-only change.
- [A] The capitalised-marker rule is exactly `Bug:`-shaped (`Xxx`), not case-insensitive, so `todo:` stays out of strict and `strict ⊆ default` holds.

## Estimates

| K̂ | basis | K | actual |
|---|---|---|---|
| 3 | first-run[U] | 120 | recorded in .pmat/estimates.jsonl at close |

## Verdict

PENDING
