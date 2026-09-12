# IMPL-PMAT-1324 — the fork bomb: a recursion guard, and a timeout that had to survive four quorum rounds

## Identity

| field | value |
|---|---|
| ticket | PMAT-1324 (`kind:code`, priority **critical**), filed by the operator as #1324 |
| branch | `PMAT-1324-worker`, from master `d76e533e4` |
| `gate_cmd` | `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings` |
| model gate | `model=opus class=opus decision=admit basis=file` |

## What the ticket asks for

Quoted from #1324, because four quorum rounds judged this diff's scope against the issue's **title alone** — `pmat work status` prints Title/Status/Priority and nothing else, so the criteria never reached a lane:

> `pmat comply check` executes the measurement commands from `.pmat-ratchet.toml` … with **no recursion guard, no depth limit, and no timeout**.

and the roadmap item's acceptance criteria:

1. a measurement command that invokes `pmat comply` is refused or bounded, and the refusal names the offending entry;
2. **every measurement command runs under a timeout and a depth limit**, both set in one place;
3. the guard is proven on the reproduction from the issue rather than asserted from the code.

**The timeout is not scope creep.** A depth limit alone does not bound a measurement that never returns — and as round 2 proved, the un-bounded case is not hypothetical.

## What it cost the operator

pmat 3.40.0 on a 48-core machine: **1-minute load average 3,026, 9,740 processes.**

## What lands

**A depth guard.** `PMAT_RATCHET_DEPTH` (one constant), max depth 1 (one constant). A measurement started by `comply ratchet`/`check` runs at depth 0 and is allowed; one started *by another measurement* is refused **before anything is spawned**, naming the depth, the limit, the variable and the offending command. Absent or non-numeric reads as 0.

**A bounded wait, on every path.** `Command::output()` blocks forever. Measurements now spawn into their own process group, poll `try_wait()` to a deadline, and drain stdout/stderr through bounded channels.

## What four quorum rounds found, and none of it was in the first cut

| round | finding | outcome |
|---|---|---|
| 1 | the diff carried an unrelated roadmap entry (PMAT-1326) | **right** — split to its own PR |
| 1 | `try_wait()`'s `Err` arm returned without killing or reaping — the child kept running | **fixed** |
| 1 | `child.kill()` kills only `bash`; a grandchild holding the pipe makes `join()` hang **forever** — and a fork bomb is nothing but grandchildren | **fixed**: process-group kill |
| 2 | the timeout never applies on the **success** path: if the command backgrounds anything, `bash` exits, `try_wait()` returns `Ok(Some)`, and the drain blocks with no timeout in play | **fixed**: the drain is bounded on that arm too |
| 2 | the grandchild test **masked** exactly that — its trailing foreground `sleep 30` kept `bash` alive so only the timeout arm was exercised | **fixed**: two tests now, one per shape |
| 3 | the success arm called `kill_process_tree` **after `try_wait()` had reaped the child**, so it signalled a PID the OS may have recycled — the gate could SIGKILL an unrelated process group | **fixed**: that arm no longer signals at all |

The orchestrator reproduced each before acting. Round 2's claim was confirmed outside the code entirely:

```
$ bash -c 'bash -c "(sleep 8) &" | cat'      # 8 seconds
```

`bash` exits instantly; `cat` blocks until the backgrounded grandchild closes the pipe. That is the hang, in three words of shell.

## Where the guard signals, and where it deliberately does not

| arm | child reaped? | signals? |
|---|---|---|
| timeout (`try_wait` → `None` at deadline) | no | **yes** — group kill, the PID is still ours |
| `Err` from `try_wait` | no | **yes** — same |
| success, drain expired | **yes** | **no** — the PID may be recycled; bounding ourselves is the protection, and the depth guard is what stops recursion |

Only two `kill_process_tree` call sites remain and both are provably pre-reap.

## Verification (every command re-run by the orchestrator, not taken from the worker)

| what | result |
|---|---|
| `cargo test --lib -- metrics_ratchet` | **70 passed, 0 failed** |
| RED, depth refusal deleted | `recursion_at_max_depth_is_refused_without_spawning` **FAILS**, other 67 alive |
| RED, child depth not incremented | `child_process_sees_depth_incremented` **FAILS**, other 67 alive |
| RED, pre-fix `measure.rs`, shell-stays-alive shape | **FAILED after 30.0s**; post-fix **ok in 0.3s** |
| RED, pre-fix, shell-exits-first shape | **hung** (killed at 100s, exit 124); post-fix **ok in 2.02s** |
| RED, round-2 code, left-alone shape | **FAILED** — the marker never appeared, the grandchild had been signalled; post-fix **ok in 3.02s**, marker present |
| `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` | clean |
| ratchets | `panic!(` 785, SATD 324 — unmoved |
| CB-2113 / CB-2115 | ✓ / ✓ against live GitHub |

The refusal test asserts **nothing was spawned** — a command that would create a marker leaves no marker — so it tests the guard rather than the message. No test ever lights the bomb.

## Honest limit

The `Err(e)` arm of the `try_wait` loop has **no dedicated test**: a genuine `waitpid` failure cannot be provoked without mocking the syscall. Its cleanup is code-identical to the timeout arm, which is tested. Stated here rather than papered over.

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 601

routes:
  ph1  class=impl            route=agy-goal w=1.00 basis=absent effort=1[U]  note=paiml-impl-worker, scope src/services/metrics_ratchet
  ph2  class=review          route=agy-quorum w=1.00 basis=absent effort=1[U] note=four rounds, three lanes each, every finding reproduced by the orchestrator before it was acted on
  ph3  class=orchestration   route=self  w=100.00  basis=absent              note=RED controls, scope split, receipt

verification:
  cmd=cargo-test--lib--metrics_ratchet  claimed_exit=0  rerun_exit=0(70-passed)  log_path=/tmp/red3.log
  cmd=RED-depth-refusal-deleted  claimed_exit=-  rerun_exit=1(1-of-68-dies)  log_path=/tmp/red.log
  cmd=RED-child-depth-not-incremented  claimed_exit=-  rerun_exit=1(1-of-68-dies)  log_path=/tmp/red.log
  cmd=RED-shell-stays-alive(pre-fix)  claimed_exit=-  rerun_exit=101(FAILED-30.0s)  log_path=/tmp/red.log
  cmd=RED-shell-exits-first(pre-fix)  claimed_exit=124  rerun_exit=101(FAILED)  log_path=/tmp/red3.log
  cmd=shell-semantics-outside-the-code  claimed_exit=-  rerun_exit=0(8s-confirms-the-hang)  log_path=/tmp/red3.log
  cmd=cargo-fmt+clippy  claimed_exit=0  rerun_exit=0  log_path=/tmp/red3.log
  cmd=comply-check--checks-CB-2113,CB-2115  claimed_exit=-  rerun_exit=0  log_path=/tmp/cc2.log
