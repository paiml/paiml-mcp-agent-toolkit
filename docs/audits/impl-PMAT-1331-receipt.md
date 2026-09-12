# IMPL-PMAT-1331 — the quality gate returns its failure instead of exiting the process

## Identity

| field | value |
|---|---|
| ticket | PMAT-1331 (`kind:code`), found un-quarantining the dispatcher tests (PMAT-1321) |
| branch | `PMAT-1331-worker`, from master `4e07ede5c` |
| `gate_cmd` | `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings` |
| model gate | `model=opus class=opus decision=admit basis=file` |

## The ticket's criteria, restated (lanes are briefed with `pmat work status`, a title — #1333)

1. `handle_quality_gate` returns an error instead of calling `process::exit`, and `main` maps it to the same exit code — proven by a CLI test asserting the process exit status is unchanged;
2. `test_quality_gate_complexity_check` runs without `#[ignore]` and asserts the error;
3. a count of the remaining `process::exit` call sites is recorded so the next PR can lower it.

## What was wrong, and a correction against myself first

`handle_quality_gate_exit_status` (`quality_gate_part2b.rs`) called `std::process::exit(1)` when a gate failed. A test reaching it **killed the whole test binary** — every test after it never ran, nothing reported a failure — so `test_quality_gate_complexity_check` was `#[ignore]`d.

An earlier attempt on this ticket (#1335, reverted) concluded the conversion was *blocked*: converting to `Err` made the exit-status guard report **101**, and I wrote on the issue that *"there is no CLI error→exit-code mapping — `src/cli/cli_exit*.rs` is not a file."* **That was false.** The mapping is `src/cli_exit.rs`; `src/bin/pmat.rs:182` routes every error through `cli_exit::code_for` → `GeneralError = 1`. I grepped the wrong directory and reported the absence as a fact. A worker had cited the module correctly in its comment and I overruled it. The issue carries the retraction.

The 101 was real but was the **guard's** behaviour: `quality_gate_exit_status_guard_tests.rs` re-execs the test binary, and its child did `outcome.expect("the quality gate must not error out")` — a returned `Err` panicked there. The guard had been written for the `exit(1)` mechanism.

## What lands

- `handle_quality_gate_exit_status` returns `anyhow::Result<()>`: the same `❌ Quality gate FAILED` line, then `Err`. **Undeclared**, so `code_for` maps it to 1 — today's observable code, unchanged. Declaring `ExitCode::QualityGateFailure = 3` is what the enum was built for but would change the status every calling script sees; that is its own decision, not this ticket's.
- Both callers propagate with `?`; both already returned `Result`.
- **The guard's child now mirrors `main`**: `Ok` → `GATE_RETURNED`, `Err(e)` → `code_for(&e)`. Its three assertions are untouched — they assert exit 1, and exit 1 is what they still see.
- `test_quality_gate_complexity_check` runs un-ignored and asserts the returned error contains `Quality gate FAILED` — replacing `is_ok() || is_err()`, a tautology.

## Verification (re-run by the orchestrator)

| what | result |
|---|---|
| `cargo test --lib -- quality_gate_exit_status_guard test_quality_gate_complexity_check quality_gate_part2b` | **8 passed, 0 failed** |
| the three guard cases | still assert **exit 1**, still pass — the shipped exit status is unchanged |
| `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` | clean |
| ratchets | `panic!(` 785, SATD 324 — unmoved |
| CB-2113 / CB-2115 | ✓ / ✓, 105 ↔ 105 |

**Count (criterion 3):** `std::process::exit(` under `src/cli/handlers/` + `src/cli/analysis_utilities/`, non-comment: master 40, branch 40 — the handler's site is gone (−1) and the guard's child gained the `code_for` exit that mirrors `main` (+1), which is the harness's own exit and is where one belongs. 37 handler sites remain for the next PRs.

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 601

routes:
  ph1  class=impl            route=agy-goal w=1.00 basis=absent effort=1[U]  note=paiml-impl-worker, five files, in parallel with PMAT-1314 on a disjoint scope
  ph2  class=orchestration   route=self  w=100.00  basis=absent              note=re-ran the suite; corrected my own false 'no mapping' claim on the issue first

verification:
  cmd=cargo-test--lib(guard+complexity_check+part2b)  claimed_exit=0  rerun_exit=0(8-passed)  log_path=/tmp/qr1331.log
  cmd=guard-asserts-exit-1-unchanged  claimed_exit=-  rerun_exit=0  log_path=/tmp/qr1331.log
  cmd=cargo-fmt+clippy  claimed_exit=0  rerun_exit=0  log_path=/tmp/qr1331.log
  cmd=comply-check--checks-CB-2113,CB-2115  claimed_exit=-  rerun_exit=0  log_path=/tmp/qr1331.log
