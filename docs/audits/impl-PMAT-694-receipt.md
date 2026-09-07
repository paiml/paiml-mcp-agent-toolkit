# impl receipt — PMAT-694 (#1202, absorbing #1127)

Orchestrator turns: n/a (worker)

## Defect

`ci / coverage` runs `cargo llvm-cov` over `cargo test --lib`. The quality proxy
spawns a REAL `cargo clippy` from inside a lib test, and that child inherited
its parent's environment: `RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS` (so it built
instrumented), `LLVM_PROFILE_FILE` and `CARGO_LLVM_COV*` (so it wrote profile
data over the parent's pattern), and `CARGO_TARGET_DIR` (so a dozen sibling
tests' children queued on one package lock). The 600s test-mode deadline in
`src/services/quality_proxy_analysis.rs` then expired and the lint stage
reported, correctly, that no verdict was produced — which fails the test.

Run 34020631941 on PR #1181 killed
`services::quality_proxy::tests::test_proxy_advisory_mode` and a sibling that
way. Two reruns on 3.39.0 went green. A rerun to green is a flake, not a pass.

`#1127` adds the second half: the children had a deadline and a process-group
kill, so they were bounded in time and guaranteed to die, but nothing bounded
how much memory they took with them.

## Fix

1. `tests/fixtures/quality_proxy/` — a dependency-free fixture crate (15 lines
   of Rust, a committed `Cargo.lock`, and a `[workspace]` table so cargo cannot
   walk up out of the temp directory and find pmat's workspace root). Its
   `Cargo.toml` and `Cargo.lock` are embedded with `include_str!` and written
   into the per-request temp directory, so the manifest the tests assert on is
   byte-identical to the one an installed pmat (which has no checkout) uses.
2. `child_command` in `src/services/quality_proxy_analysis.rs` — the single
   constructor for every child this service spawns. Both existing spawn sites
   (`run_lint_checks`'s `cargo clippy`, `format_rust_code`'s `rustfmt`) are
   routed through it, so a spawn cannot be added later that misses the cap, the
   private target dir or the scrub. The deadline and the process-group kill stay
   where they were, in `run_with_timeout`, which every caller passes the command
   to.
3. The address-space cap: `prlimit --as=8589934592` where `prlimit` is on PATH,
   otherwise `sh -c 'ulimit -v 8388608 2>/dev/null; exec "$0" "$@"'`. Eight GiB
   is far above one rustc invocation over one dependency-free file and far below
   what a runaway child costs a CI box.

### The six variables the child no longer inherits

| variable | why |
|---|---|
| `LLVM_PROFILE_FILE` | child profraw data written over the parent's pattern |
| `CARGO_LLVM_COV` | marks the build as a coverage build |
| `CARGO_LLVM_COV_TARGET_DIR` | redirects the child into the instrumented tree |
| `RUSTFLAGS` | carries `-C instrument-coverage` into the child's compile |
| `CARGO_ENCODED_RUSTFLAGS` | same, and takes precedence over `RUSTFLAGS` |
| `CARGO_INCREMENTAL` | coverage forces it off; the child's fingerprint follows |

`CARGO_TARGET_DIR` is not scrubbed but *replaced*, with `<workdir>/target` — a
shared target dir is a shared package lock, and waiting on it is
indistinguishable from a slow compile until the deadline expires.

## RED (commit 37fe0f8b1, guard tests against the pre-fix spawn)

```text
running 4 tests
test services::quality_proxy::child_process_isolation_tests::fixture_crate_is_small ... FAILED
test services::quality_proxy::child_process_isolation_tests::child_has_a_memory_cap ... FAILED
test services::quality_proxy::child_process_isolation_tests::child_env_is_scrubbed ... FAILED
test services::quality_proxy::child_process_isolation_tests::proxy_modes_finish_well_inside_the_deadline ... ok

---- fixture_crate_is_small stdout ----
the quality proxy fixture crate must exist at <repo>/tests/fixtures/quality_proxy/Cargo.toml
---- child_has_a_memory_cap stdout ----
the child compiler must run under an address-space cap; command line was: cargo clippy
---- child_env_is_scrubbed stdout ----
the child must clear LLVM_PROFILE_FILE; the command carries []

test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 21409 filtered out
```

The fourth guard, the wall-clock bound, passed RED: an idle local box never
reproduced the CI failure, which is the point of the other three. The bound is
what makes the failure mode impossible, not evidence that it was present here.

## GREEN (commit 33d5fcfc4)

```text
running 5 tests
test services::quality_proxy::child_process_isolation_tests::child_has_a_memory_cap ... ok
test services::quality_proxy::child_process_isolation_tests::child_env_is_scrubbed ... ok
test services::quality_proxy::child_process_isolation_tests::fixture_crate_is_small ... ok
test services::quality_proxy::child_process_isolation_tests::the_wrapped_child_still_reports_clippy_findings ... ok
test services::quality_proxy::child_process_isolation_tests::proxy_modes_finish_well_inside_the_deadline ... ok
PMAT-694 wall: advisory 0.10s, strict 0.10s

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 21409 filtered out
```

acceptance (`env -u RUST_MIN_STACK cargo test --lib -- quality_proxy`):

```text
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 21321 filtered out; finished in 55.79s
```

## Wall of the two tests that were killed

| | `test_proxy_advisory_mode` | strict-mode sibling | where |
|---|---|---|---|
| before | killed at the 600s deadline | killed at the 600s deadline | ci / coverage, run 34020631941 |
| before | 0.21s for the whole guard module | (same run) | this box, RED commit 37fe0f8b1 |
| after | 0.10s | 0.10s | this box, GREEN commit 33d5fcfc4 |

Stated plainly: this box never reproduced the CI failure, so the local
before/after walls are indistinguishable and are NOT the evidence for the fix.
The evidence is the three deterministic guards — the environment the child is
given, the cap it runs under and the crate it lints — plus the control
`the_wrapped_child_still_reports_clippy_findings`, without which a wrapper that
silently failed to exec `cargo` would make every timing assertion pass faster
than ever while the lint stage measured nothing. Independently reproduced
outside the test binary: `prlimit --as=8589934592 -- cargo clippy` over the
fixture, in a private target dir, is 0.09s elapsed / 88 MB max RSS and reports
its warning.

## Gate

`pmat verify --format json` (repo-built binary, clean tree, commit b04840239):
`ok: false`, `stages_measured: 4`, `not_measured: ["complexity"]`.

| stage | ok |
|---|---|
| format | true |
| complexity | null — "no Rust files changed vs HEAD, so nothing was measured" |
| satd | true |
| clippy | true |
| tests | false — 21256 passed, 4 failed |

The four failures, none of which this change can pass on its own:

* `services::metrics_ratchet::drive_tests::the_committed_ratchet_holds_at_head`
  — `orphan_files: 408 count exceeds the baseline 407`. This one IS ours: the
  fixture's `src/lib.rs` is a tracked `.rs` file reachable from no target root,
  exactly like the 30-odd other `tests/fixtures/*.rs` rows already in
  `docs/status/orphan-files-ledger.md`. The baseline lives in
  `.pmat-ratchet.toml`, which is outside this ticket's scope_paths, and raising
  a ratchet baseline requires a `justification` on the entry — an orchestrator
  or operator decision, not a worker's. Left red and reported.
* `services::rust_project_score::documentation_scorer::tests::test_changelog_missing`
* `…::test_changelog_minimal`
* `…::test_recommendations_empty_project`
  — three hermetic `TempDir` tests over
  `src/services/rust_project_score/documentation_scorer*.rs`, a file this branch
  does not touch (branch diff: `quality_proxy_analysis.rs`,
  `quality_proxy_tests.rs`, the three fixture files, two ledgers, this receipt
  and the contract). They fail run alone as well as in the suite.

## Contract

`contracts/work/PMAT-694.yaml`, `metadata.kind: pattern`, five obligations —
the four guard tests plus the wall-time measurement, each with its exact
`cargo test` evidence_method. `pv validate` exit 0 ("Contract is valid."),
`pv lint` exit 0 ("Result: PASS").

## Ledgers

Regenerated on a clean tree with the repo-built binary and committed separately:
`docs/status/unrun-tests-ledger.md` (954d4f9bd) and
`docs/status/orphan-files-ledger.md` (b04840239, 4444 → 4445 tracked `.rs`
files, 407 → 408 orphaned).

verdict: PARTIAL — the defect is fixed and every guard is green (acceptance exit 0), but `pmat verify` is red at `ok: false` on four tests: three pre-existing documentation_scorer failures this branch does not touch, and the `orphan_files` ratchet baseline in `.pmat-ratchet.toml`, which is outside scope_paths and needs a justified raise from 407 to 408.

IMPL-PMAT-694-RECEIPT-END
