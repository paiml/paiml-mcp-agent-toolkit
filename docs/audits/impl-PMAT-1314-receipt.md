# IMPL-PMAT-1314 — two tests that "hung under nextest" were sleeping for 300 seconds, and the ticket blamed the wrong thing

## Identity

| field | value |
|---|---|
| ticket | PMAT-1314 (`kind:code`), filed from the Pareto lane work (PMAT-1313) |
| branch | `PMAT-1314-worker`, from master `4e07ede5c` |
| `gate_cmd` | `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings` |
| model gate | `model=opus class=opus decision=admit basis=file` |

## The ticket's criteria, restated (lanes are briefed with a title — #1333)

1. the root cause is named — what the test depends on that another test supplies — rather than worked around with a longer timeout;
2. both tests pass under `cargo nextest run --lib` in isolation;
3. the `default-filter` in `.config/nextest.toml` is removed in the same PR and `pr-lane-control.sh` arm 3 is updated or retired with it.

## The diagnosis was wrong, and the worker refused to propagate it

The ticket — written by me — said `eof_does_not_signal_while_a_request_is_in_flight` and `waits_for_every_outstanding_request` pass under `cargo test`, hang under nextest, and are therefore **order-dependent**: passing only in a process another test has warmed.

The worker measured it and found otherwise, and I confirmed on master before accepting:

```
$ cargo test --lib -- …eof_drain_tests::waits_for_every_outstanding_request --exact
test result: ok. 1 passed …    wall=384s
```

**It passes in complete isolation under plain `cargo test` — after 384 seconds.** `EofSignalingTransport` runs an unconditional, uncancellable `tokio::time::sleep(Self::DRAIN_BACKSTOP)` — 300 s — on its terminal-receive-error path (`simple_unified_server.rs:~240-256`). Production never waits it out: an outer `select!` drops the future. The tests provide no such thing. So `cargo test` sat through five minutes and reported `ok`; nextest's 120 s slow-timeout killed it and reported a hang. **Neither runner was wrong about what it saw. The framing was.** The issue carries the correction.

Criterion 1 as written asked for "what the test depends on that another test supplies". Nothing did. The honest answer to criterion 1 is the mechanism above, with its line numbers.

## What lands

- The two tests override the drain backstop for their own run, so the terminal-error path completes without the 300 s wait. Production's constant is unchanged.
- `.config/nextest.toml` loses its `default-filter`; the local fast lane runs everything.
- `scripts/pr-lane-control.sh`: arm 3 now asserts the config carries **no** `default-filter`, arm 4 refuses a fixture that reintroduces one, and the two double-exclusion arms are retired with this ticket named. 8 arms.

## Verification (re-run by the orchestrator)

| what | result |
|---|---|
| RED: the acceptance command on the unmodified tree | `timeout` exit **124** (killed at the wrapper) |
| RED, cross-checked on master under plain `cargo test` | passes after **384 s** — the sleep, not a hang |
| GREEN: `cargo nextest run --lib -E 'test(eof_drain_tests)'` | **4 passed in 0.040 s** |
| `cargo test --lib -- eof_drain_tests` | passes |
| `scripts/pr-lane-control.sh` | **8/8 arms** |
| `cargo fmt --all -- --check`, clippy `-D warnings` | clean |
| ratchets | 785 / 324 — unmoved |
| CB-2113 / CB-2115 | ✓ / ✓, 105 ↔ 105 |

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 601

routes:
  ph1  class=impl            route=agy-goal w=1.00 basis=absent effort=1[U]  note=paiml-impl-worker, three files, in parallel with PMAT-1331 on a disjoint scope; it refuted the ticket's diagnosis and was right
  ph2  class=orchestration   route=self  w=100.00  basis=absent              note=confirmed the 384s on master before accepting; corrected the issue

verification:
  cmd=cargo-test--exact(master,one-hanger)  claimed_exit=-  rerun_exit=0(384s)  log_path=/tmp/qr1314.log
  cmd=cargo-nextest-eof_drain_tests(fixed)  claimed_exit=0  rerun_exit=0(4-passed-0.040s)  log_path=/tmp/qr1314.log
  cmd=pr-lane-control(8-arms)  claimed_exit=0  rerun_exit=0  log_path=/tmp/qr1314.log
  cmd=comply-check--checks-CB-2113,CB-2115  claimed_exit=-  rerun_exit=0  log_path=/tmp/qr1314.log
