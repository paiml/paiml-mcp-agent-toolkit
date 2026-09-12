# IMPL-PMAT-1329 — `cargo test` stopped writing to the repository, and the test that did it asserted nothing

## Identity

| field | value |
|---|---|
| ticket | PMAT-1329 (`kind:code`), found while un-quarantining the dispatcher tests (PMAT-1321) |
| branch | `PMAT-1329-tests-dont-write`, from master `39abc6aaa` |
| `gate_cmd` | `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings` |
| model gate | `model=opus class=opus decision=admit basis=file` |

## The ticket's criteria, restated (lanes are briefed with `pmat work status`, which prints a title — #1333)

1. `git status --porcelain` is empty after `cargo test --lib`, **proven by a check that runs the suite and asserts the tree is clean rather than by reading the tests**;
2. no test writes anywhere under `docs/`;
3. the duplicated markers already committed to `docs/execution/roadmap.md` are cleaned up once, in the same PR.

Criterion 1 is the one that makes this durable, and the first cut of this PR did not satisfy it: the fix was proven by hand. Three lanes refused it for exactly that. `scripts/tests-dont-write-control.sh` now satisfies it.

## The defect

`cargo test --lib -- command_dispatcher` mutated a **tracked** file: 28 lines of `docs/execution/roadmap.md`, each an existing heading with another `✅ COMPLETED` appended. Every run appended another, so the file grew monotonically and silently.

`test_roadmap_init_routing` called `execute_roadmap_command(RoadmapCommands::Init{..})`, which wrote to a hardcoded path. Its whole assertion was:

```rust
assert!(result.is_ok() || result.is_err());
```

true for every possible value. **It asserted nothing while performing a destructive write.**

Dormant until PMAT-1321 un-quarantined these tests. It also collides with `pmat analyze unrun-tests --write-ledger`, which refuses a dirty tree by design — so the refusal appeared with no visible cause.

## What lands

- **One seam.** `roadmap::default_roadmap_path()` reads `PMAT_ROADMAP_PATH`, falling back to today's value. Both hardcode sites call it — and there were **two independent ones**, because `execute_roadmap_command` builds its own struct literal instead of calling `Default`.
- **The test points at a `TempDir`**, through an RAII guard that restores the previous value, `#[serial_test::serial(env_vars)]` so it cannot race sibling env-var tests, and never touching the process's cwd (that races everything — PMAT-726).
- **The assertion means something**: the command succeeds *and* the written file contains the sprint title it was given.
- **The committed damage is cleaned**: four headings' repeated markers collapsed to one each.
- **`scripts/tests-dont-write-control.sh`**, 3 arms, wired into the `traceability` job.

## Verification (RED and GREEN both re-run by the orchestrator)

| what | result |
|---|---|
| suite, with the fix | `docs/` dirty lines = **0** |
| suite, production seam reverted, test kept | `docs/` dirty lines = **1** |
| `tests-dont-write-control.sh`, with the fix | **3/3 arms**, exit 0 |
| the same control, seam reverted | **FAILED**, exit 1 |
| `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` | clean |
| ratchets | `panic!(` 785, SATD 324 — unmoved |
| CB-2113 / CB-2115 | ✓ / ✓ against live GitHub, 106 ↔ 106 |

The control's **arm 2 is the one that matters**: it plants a write under `docs/` and requires the checker to see it. Without that arm the script would assert "git found nothing", which is also what a checker that stopped looking says.

## Why this PR completes PMAT-1321, a different ticket

Three lanes flagged it as out of scope. It is cross-ticket, and CB-2113 requires it to be:

> every non-merge commit the PR adds names a real, **NON-TERMINAL** roadmap item

A commit marking PMAT-1321 completed while naming `Pmat-Ticket: PMAT-1321` names a terminal item, and CB-2113 fails it — **a ticket cannot complete itself**. PMAT-1321 merged as `39abc6aaa` and GitHub closed #1321 with it, so leaving the item non-terminal makes master **ORPHAN-ROADMAP under CB-2115**: red, with no commit to blame, until a later PR corrects it. The completion has to ride in the next open ticket's PR, which is this one.

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 601

routes:
  ph1  class=impl            route=agy-goal w=1.00 basis=absent effort=1[U]  note=paiml-impl-worker; the seam, the test and the cleanup
  ph2  class=review          route=agy-quorum w=1.00 basis=absent effort=1[U] note=3 FAIL on a missing acceptance check and an unjustified cross-ticket edit; both addressed
  ph3  class=orchestration   route=self  w=100.00  basis=absent              note=RED/GREEN on the seam, the control and its own falsifier, receipt

verification:
  cmd=suite-then-git-status(with-fix)  claimed_exit=0  rerun_exit=0(0-dirty)  log_path=/tmp/qr1329.log
  cmd=suite-then-git-status(seam-reverted)  claimed_exit=-  rerun_exit=1(1-dirty)  log_path=/tmp/qr1329.log
  cmd=tests-dont-write-control(3-arms)  claimed_exit=-  rerun_exit=0  log_path=/tmp/qr1329.log
  cmd=tests-dont-write-control(seam-reverted)  claimed_exit=-  rerun_exit=1(FAILED)  log_path=/tmp/qr1329.log
  cmd=cargo-fmt+clippy  claimed_exit=0  rerun_exit=0  log_path=/tmp/qr1329.log
  cmd=comply-check--checks-CB-2113,CB-2115  claimed_exit=-  rerun_exit=0(106-bijection)  log_path=/tmp/qr1329.log
