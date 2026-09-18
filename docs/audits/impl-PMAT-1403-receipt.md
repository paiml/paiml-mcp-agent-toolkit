# PMAT-1403 — a read-only dead-code analysis can no longer rewrite the analysed project's Cargo.lock

## Identity

| Field | Value |
|---|---|
| ticket | PMAT-1403 (GitHub issue #1403) |
| kind | `code` (`kind-gate.sh` exit 0, `kind=code ticket=PMAT-1403 files=241`) |
| branch | `PMAT-1403-dead-code-lockfile-rewrite`, cut from `origin/master` `ecd97c6bc` |
| base | `master` |
| HEAD at receipt | `3a1ef7c2e` · `origin/master=ecd97c6bc` · `behind=0` |
| `discover.json` sha256 | `8f28b24bc3d57b72f20af88c9b4c045dd94c383d579aaedbe81ceb66f8ddcab3` |
| `gate_cmd` | `make gate` (`gate_cmd_fallback=false`) |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| orchestrator model | `opus-5` — `model-gate.sh`: `model=opus-5 class=opus decision=admit basis=transcript` |
| host | `uptime` before `02:26:44 up 8:54, load 0.77` · after `03:07:58 up 9:36, load 1.45` — no crash, no reboot |

## The defect

Tag `v3.41.0` (`ecd97c6bc`) → fleet unified gate run 35286438196, job `gate / cpu-gates`,
runner `intel-clean-room-5`. **GATE B2 (unit tests) FAILED: 21774 passed, 2 failed.**

- `services::cargo_dead_code_analyzer::lockfile_tests::a_crate_with_a_lockfile_is_analysed_fully_and_its_lockfile_is_untouched` — `lockfile_tests.rs:166`, *"the analysed project's lockfile was modified by a read-only analysis"*
- `cli::handlers::dead_code_handlers::lockfile_disclosure_tests::a_crate_with_a_lockfile_reports_a_full_scan_and_the_compiler_finding` — `dead_code_lockfile_disclosure_tests.rs:211`, *"a read-only analysis rewrote the project's lockfile"*

### Mechanism, in one sentence

`cargo check`, run by the analyzer **in the analysed project's own directory**, appended
`[[patch.unused]] name = "pmat" version = "3.41.0"` to the fixture's lockfile, because the
clean room installs its publish overlay into `$CARGO_HOME/config.toml` where **every child
cargo process inherits it whatever its cwd** (infra#653) — and the analyzer had no mechanism
at all enforcing the read-only property it claims.

Decoding the before/after byte arrays the two panics print, past the common prefix, gives
exactly:

```toml

[[patch.unused]]
name = "pmat"
version = "3.41.0"
```

The clean-room log names its own cause two lines apart:

```
publish overlay: 1 member(s) -> /build/pmat/.clean-room-patch.toml
publish overlay: inherited by child cargo processes via /usr/local/cargo/config.toml
```

### Five whys

1. **Why did GATE B2 fail?** `cargo check` rewrote the tempdir fixture's `Cargo.lock`, and two tests assert it is byte-identical after a read-only analysis.
2. **Why did cargo rewrite it?** The resolution cargo computed carried an unused `[patch]` entry that the on-disk lockfile did not record, so cargo serialised the difference — `[[patch.unused]]`.
3. **Why was a `[patch]` in scope for an unrelated tempdir crate?** infra#653 installs the publish overlay into `$CARGO_HOME/config.toml` rather than passing `--config` per process, precisely so that *child* cargo processes inherit it. pmat's test suite spawns one.
4. **Why does an ambient config decide whether pmat's analysis is read-only?** Because the analyzer runs `cargo check` in the analysed directory with the lockfile writable, and simply assumes cargo will not need to write. That assumption is about the environment, not about pmat.
5. **Root cause.** *The analyzer had no mechanism that makes "read-only" true.* It neither prevented the write (`--locked` — tried, measured, reverted) nor isolated it. An ambient `[patch]` is only one trigger; a stale lockfile, a different lockfile `version`, a `[replace]`, or no lockfile at all all produce the same rewrite.

### Corrections to the brief

1. **"the container's cargo may be the MSRV or a newer one" — the toolchain is NOT the variable.**
   Reproduced byte-identically on **cargo 1.98.0**, the same version `ci / test` is green on,
   in ~2 seconds with a throwaway `CARGO_HOME` carrying one `[patch.crates-io]` entry:
   `7a8,11 > [[patch.unused]] / name = "pmat" / version = "3.41.0"`. The clean-room container
   is `rust:1.95-slim` (`IMAGE := rust:1.95-slim`, `rustc 1.95.0`, `cargo 1.95.0`) — neither
   the 1.91.0 MSRV nor a newer cargo, and irrelevant either way. **Any** cargo rewrites it.
2. **LATENT, confirmed — not a regression of the 3.41.0 fixes.** The analyzer has had no
   lockfile protection since `2bdc6b90c` (2026-08-25) reverted `--locked`. `git bisect` was
   not needed: the failing assertions and the unprotected `cargo check` have coexisted since
   the day the tests were written. What is new is only the *observer* — infra#653's ambient
   overlay, plus the first clean room to finish on a pmat tag since v3.39.0 (paiml/.github#72).
3. **`~/src/infra` on this host is 49 commits behind `origin/main` and does NOT contain the
   overlay code.** `make -C ~/src/infra/machines/clean-room clean-room-pmat` as the brief
   specifies would have run *without* the publish overlay and reproduced **nothing** — a green
   run that proved the opposite of the truth. The reproduction was built from
   `origin/main:machines/clean-room/gates-lib.sh` (`_cr_install_inheritable_overlay`) instead.
4. **The full `clean-room-pmat` target was not run.** A targeted container carrying the same
   image, the same overlay, installed the same way, and running the same `cargo test --lib`
   was used instead. It reproduces the defect exactly and allowed RED, GREEN, MUTANT and a
   REVERT round in one container instead of two ~50-minute full-gate runs. Host load peaked at
   3.16 and the host did not crash.
5. **`cargo --lockfile-path` was considered and is unusable**: `error: unexpected argument
   '--lockfile-path' found` on cargo 1.98.0. It is not stable; the quorum was told so.

## Plan and routing

| Phase | What | Route | Trigger | Acceptance `A_i` |
|---|---|---|---|---|
| 1 | Design choice: how to make read-only true without `--locked` | `route=agy-plan w=1.00 basis=absent effort=1[U]` → delegate, `quorum` width 3 | Q3 (ambiguous root cause / design) | 3 lane verdicts under `quorum-lane-schema.json` |
| 2 | RED in the clean-room toolchain | `route=self` | — | the two named tests FAIL in `rust:1.95-slim` + overlay on master |
| 3 | Implement `LockfileGuard`, contract, tests | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` — **not taken**, see below | — | `cargo test --lib lockfile` green under an ambient `[patch]` |
| 4 | GREEN + mutant in the clean-room toolchain | `route=self` | — | 22/0/0 green; mutant turns the byte-identity tests red |
| 5 | `make gate`, PR, quorum review, merge | `route=self` | Phase-3 pre-PR review trigger | `make gate` all local legs green |

`K̂=95 BASIS=docs/audits/impl-estimates.jsonl:L26-L36 ROWS=18 MEDIAN=95 EXCLUDED=11 UNMEASURED=8`; budget `K=200`.

**Phase 3 did not go to `agy-goal`, and that is a deviation to record.** R-4 routes a
single-module `impl` phase to a `writes=true` agy lane before any Claude subagent. It was
taken `self` because by the time the mechanism was decoded the change was ~40 lines across
two functions plus its tests, and the expensive, non-delegable part — the clean-room
container that is the only thing that can judge the fix — was already mine. No `writes=true`
agy lane ran, so the R-4 one-writer rule was trivially satisfied; no `--concurrent-scope` was
declared by anyone, so every lane asserted the whole checkout.

## Dispatch ledger

| # | Mode | Agent / lane | Model | Width | Turns | maxTurns? | Resumed? | agy conversations |
|---|---|---|---|---|---|---|---|---|
| 1 | delegate | `paiml-agy-delegate` `a44afb93674886ab2`, lane `quorum`, mode `plan` (derived), `writes=false` | opus (delegate) / lanes `gemini-3.1-pro-high`, `gemini-3.8-flash-high`, `gemini-3.7-flash-high` (all `model_source=measured`) | 3 | 29 tool uses, 707s | no | no | `41478316…`, `52f81696…`, `56bb9b04…`; `child_conversations=3`, `fanout.children=3 method=lane-files` |

- **slots**: `slots=3`, live peak **1**. Never more than one Claude subagent existed at a time.
- **denials**: 0. **stalls**: 0.
- **I-3 line**: see `transcript-gate.sh` output below.
- Lane hygiene reported by the delegate: all 3 lanes exit 0, all sandboxed self-contained clones, all tree-witness verified, all clones removed byte-identical — **no `KEPT`, no exit 3, no exit 4 (`LANE BLIND`)**. `concurrent_scope=(none: the whole checkout is asserted)`.

### Quorum verdict (Phase 1)

**3/3 PASS, unanimous on candidate A** — snapshot the lockfile before spawning cargo and
restore it from an RAII `Drop` guard. No lane picked B (analyse a copy), C (accept and
disclose) or D, and no lane returned `do-not-implement-as-written`. `dissent: []`,
`uncovered: []`, `partial_reasons: []`, `partial=false`. 3/3 agreement in `dedup` on
`lockfile_tests.rs:166`. All three independently placed the guard in the `run_cargo_check`
caller frame so the deadline-kill path is covered; lane 2 added that the child must be killed
and reaped before restore — it is (`wait_for_cargo_check` kills and `wait`s before returning
`self.timeout_error()`).

The delegate flagged six `open_questions`. Each is resolved here rather than left standing:

| Open question | Resolution |
|---|---|
| Mode `plan` was derived, not declared | Accepted; recorded in the ledger above. |
| `author_model=claude-opus-5` came from the brief, not a measurement (`author.source=flag`) | `model-gate.sh` independently measured `model=opus-5 basis=transcript` in Phase 0, so the declaration is corroborated — but the `$XDG_RUNTIME_DIR/paiml-implement/model-<sid>` file that `lane-reduce.sh` reads does not exist for this session. Gap named, not closed. |
| Falsifier and contract answers were single-lane (lane 1 only); lane 2's summary *claimed* both and its text contains neither | Treated as lane 1's proposal, not as consensus. Both were implemented and then **measured** rather than trusted — see Mutant below. Lane 2's unbacked self-report is recorded as a lane-quality finding. |
| Reduced-reason token UNRESOLVED between lanes: lane 1 wanted a new `lockfile-restore-failed`, lane 3 pointed at the existing `COMPILER_SCAN_REASON_{OK,LOCKFILE,ENV_SKIP}` family | **Decided: a new token, and NOT as a `CompilerScanReport.reason`.** When a restore fails the compiler layer *did* run, so `verdict: reduced` would be false. The token `lockfile-restore-failed` is carried by an **error** instead. Both lanes were half right and neither answer was adopted as given. |
| Lane 2 cited `target_isolation.rs:27` for `isolated_target_dir` where the brief said `analysis.rs:305-313` | Both are real: the function is defined in `target_isolation.rs` and called from `analysis.rs`. Not a discrepancy. |
| **No lane addressed candidate A's known SIGKILL / transient-window hole**, so 3/3 PASS is not three judgements that the residual risk is acceptable | Correct, and it stands as a limit on what the quorum proves. The hole is documented in the code, in the contract's `preconditions`, and in Gaps below. It is unreachable by any in-process mechanism — including `--locked`, which closes it only by not scanning. |

## The fix

`LockfileGuard` (`src/services/cargo_dead_code_analyzer/lockfile_guard.rs`), acquired in
`run_cargo_check` **before the cargo child is spawned** and restored on every path out —
`Ok`, a cargo failure, the deadline kill — with `Drop` as the belt for a panic.

- **PRESENT before** ⇒ byte-identical after, whatever cargo did.
- **ABSENT before** ⇒ absent after (#1076's own defect).
- **Unchanged** ⇒ *nothing is written at all*: an identical rewrite still moves the mtime.
- The protected path is the **workspace root's** lockfile — the only one cargo writes — found with `cargo locate-project --workspace`, which resolves nothing and so cannot write the file it is there to find (measured: it leaves a lockfile-less crate lockfile-less).
- Presence is asked with `exists()` and content with `read()`, separately: a path that exists but cannot be read used to fall into the absent/absent arm and report `Untouched` over something still in the tree.
- A restore that cannot happen is an **error** carrying `lockfile-restore-failed` and the path — never a caveat on a report that would otherwise read as a clean read-only run.

`--locked` is not reintroduced, and `the_cargo_invocation_never_carries_locked_or_frozen` now
pins its **absence** — the same argv assertion as before 2bdc6b90c, with the sign flipped,
because the argv is exactly where a well-meaning "just make the lockfile test pass" would put
it back.

### The seven stale `#[ignore]`s

Seven tests carried `#[ignore = "#1076 is OPEN … This test is the SPEC for the real fix —
analyse a copy, or snapshot/restore the lockfile — and must go green when that lands, not be
deleted."]`. **All seven now run; none was deleted and no new `#[ignore]` was added.**

| Test | Disposition |
|---|---|
| `lockfile_tests::analysing_a_lockfile_less_crate_creates_no_lockfile` | un-ignored, passes as written |
| `lockfile_disclosure_tests::the_analysis_leaves_no_lockfile_in_the_analysed_tree` | un-ignored, passes as written |
| `lockfile_disclosure_tests::the_json_report_declares_the_reduced_scan_and_why` | repointed: same assertions, reduction driven by `PMAT_DEAD_CODE_SKIP` (a trigger that still fires) instead of a lockfile refusal (which no longer happens) |
| `lockfile_disclosure_tests::the_text_summary_declares_the_reduced_scan` | repointed, as above |
| `lockfile_disclosure_tests::every_output_format_carries_the_reduced_verdict` | repointed, as above — the "no renderer drops the disclosure" invariant is kept intact across all four formats |
| `lockfile_tests::the_refused_compiler_scan_is_declared_on_the_report` | repointed to the **inverse** assertion and renamed `a_lockfile_less_crate_is_still_scanned_at_full_fidelity` — its premise (a lockfile-less crate is refused) is false by design now, and the assertion that replaces it is the anti-regression for re-adding `--locked` |
| `lockfile_tests::the_cargo_invocation_forbids_cargo_from_writing_the_lockfile` | repointed to `the_cargo_invocation_never_carries_locked_or_frozen` |

The two originally-failing tests were **not touched**: their assertions are the contract.

## Verification table — claimed vs re-run

Every row below was executed by the orchestrator. There is no worker claim to compare against:
the only subagent was the review delegate, which wrote nothing.

| Check | Claimed | Orchestrator re-run | Verdict |
|---|---|---|---|
| RED, clean-room toolchain, master tree | — | `5 passed; 2 failed` — both release-gate assertions, same two files, same two lines | ✅ reproduced |
| GREEN, clean-room toolchain, fixed tree | — | `22 passed; 0 failed; 0 ignored` | ✅ |
| MUTANT, clean-room toolchain | lane 1 proposed `std::mem::forget(guard)` | `18 passed; 4 failed` | ✅ falsifier fires |
| REVERT of the mutant, same container | — | `22 passed; 0 failed; 0 ignored` | ✅ |
| GREEN on the exact **committed** bytes (`3a1ef7c2e`), same container | — | `22 passed; 0 failed; 0 ignored` | ✅ |
| Local, under an ambient `[patch]` in `CARGO_HOME` | — | `22 passed; 0 failed; 0 ignored` on cargo 1.98.0 | ✅ |
| Local mutant | — | `18 passed; 4 failed` | ✅ |
| `cargo fmt --all -- --check` | — | clean | ✅ |
| `cargo clippy --lib --tests --all-features` | — | no warnings, no errors | ✅ |
| `pv validate contracts/dead-code-lockfile-isolation-v1.yaml` | — | `0 error(s), 0 warning(s)` | ✅ |
| `make gate` | — | see Gate below | see below |

### RED (the reproduction), verbatim

Container `rust:1.95-slim` (`rustc 1.95.0 (59807616e 2026-04-14)`, `cargo 1.95.0 (f2d3ce0bd
2026-03-21)`), running as **root** exactly as the gate does, with infra#653's overlay
installed into `/usr/local/cargo/config.toml` by the same procedure `gates-lib.sh`
`_cr_install_inheritable_overlay` uses:

```
# >>> clean-room publish overlay (infra#653) >>>
[patch.crates-io]
"pmat" = { path = "/build/pmat" }
# <<< clean-room publish overlay (infra#653) <<<
```

```
panicked at src/services/cargo_dead_code_analyzer/lockfile_tests.rs:166:5:
assertion `left == right` failed: the analysed project's lockfile was modified by a read-only analysis
panicked at src/cli/handlers/dead_code_lockfile_disclosure_tests.rs:211:5:
assertion `left == right` failed: a read-only analysis rewrote the project's lockfile

test result: FAILED. 5 passed; 2 failed; 7 ignored; 0 measured; 21920 filtered out
```

### GREEN and the mutant, same container

```
##### GREEN (fixed tree, rust 1.95, infra#653 overlay) #####
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 21920 filtered out

##### MUTANT (guard forgotten) #####
failures:
    cli::handlers::dead_code_handlers::lockfile_disclosure_tests::a_crate_with_a_lockfile_reports_a_full_scan_and_the_compiler_finding
    cli::handlers::dead_code_handlers::lockfile_disclosure_tests::the_analysis_leaves_no_lockfile_in_the_analysed_tree
    services::cargo_dead_code_analyzer::lockfile_tests::a_crate_with_a_lockfile_is_analysed_fully_and_its_lockfile_is_untouched
    services::cargo_dead_code_analyzer::lockfile_tests::analysing_a_lockfile_less_crate_creates_no_lockfile
test result: FAILED. 18 passed; 4 failed; 0 ignored; 0 measured; 21920 filtered out

##### REVERT MUTANT #####
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 21920 filtered out
```

The mutant is `docs/audits/mutants/PMAT-1403-forgotten-lockfile-guard.patch`: it still
*acquires* the guard and leaves the type in place, so a "does the mechanism compile" test
would not notice — only the byte-identity assertions can. It kills the four byte-identity
tests and leaves the four fidelity tests green, which is the evidence that the two halves of
the contract (tree untouched, search undiminished) are **independently** measured.

### A vacuous falsifier the clean room caught, and the correction

The first version of `a_restore_that_cannot_write_is_reported_as_failed` forced the failure
with `chmod 0555`. It passed locally and **failed in the container** — together with the
root-detector written beside it, which was the only reason the vacuity was visible at all.
The clean room runs its gates as `uid=0(root)`, where a read-only directory is not read-only.
Both were replaced by a root-proof falsifier that puts a **directory** where the lockfile
belongs: `write()` and `remove_file()` both fail for uid 0 too. This is recorded because a
chmod-based test would have shipped green and been vacuous in the one environment that found
the original bug.

## Quorum round 3 — a FAIL that was right

The tree changed after rounds 1 and 2 (the rebase, the guard move, the allow-list entry), so
the quorum was re-run on the final diff. **Lane 1 (`gemini-3.1-pro-high`) returned FAIL with
two cited findings. Both were real, both are fixed, and both now have a falsifier that was
measured to fire.**

1. **`acquire` classified on `fs::read` alone.** A `Cargo.lock` that EXISTED but could not be
   read was recorded `Absent`; at restore the path existed, the `(Absent, true, _)` arm
   matched, and the guard **deleted the project's own file**. The mechanism whose entire
   purpose is to leave the tree alone would have been the thing that destroyed it — and the
   contract already claimed, wrongly, that presence was asked with `exists()`: I had applied
   that only in `restore_inner`, not in `acquire`. A third state, `PresentUnreadable`, now
   separates "not there" from "there and unreadable"; the latter is never deleted, never
   overwritten, and is reported `Failed` only if it VANISHES.
2. **A failed restore could be swallowed by a failed cargo run.** `let outcome = outcome?;`
   ran before `restored` was inspected, so when cargo failed the restore failure was dropped:
   the user learned why the analysis stopped and not that it had left their repository
   modified — the quiet half of the original defect. The decision is now a pure function,
   `cargo_outcome_or_restore_failure`, which makes a failed restore outrank a failed cargo run
   and carries the cargo error along rather than trading it away. All four combinations are
   unit-tested without running cargo at all.

Mutation evidence for both, measured 2026-09-18:

| Mutant | Result |
|---|---|
| collapse the `PresentUnreadable` arm back into `Absent` | `23 passed; 2 failed` — both new guard tests |
| restore the `let outcome = outcome?;` ordering | `24 passed; 1 failed` — the swallow test |

Lanes 2 and 3 returned PASS on the same diff and saw neither finding. That is the argument for
the quorum: a 2/3 majority would have merged a guard that deletes unreadable lockfiles.

### Round 4 — the same lane, a third real finding

Re-run on the fixed diff, lane 1 returned FAIL again, and again it was right. Moving the guard
ahead of `build_cargo_check_command` (the round-2 hardening) made the `PMAT_DEAD_CODE_SKIP`
early return a second exit from `run_cargo_check`: it returned `Ok(suppressed_by_env())`
without inspecting `restored`, the guard was dropped, and `Drop` can only DISCARD what
`restore` returns. On that path a failed restore became a clean-looking read-only run — the
exact shape this ticket exists to remove, reintroduced by my own fix for a different hole.

`run_cargo_check` now has **one exit**. Suppressed, `Ok`, a cargo failure and the deadline
kill all reach the same `restore` and the same reporting rule; `Drop` is the belt for a panic
only. That makes the class unavailable rather than merely absent.

It is covered structurally rather than end to end, and that is a stated limit: pinning it with
a real `PMAT_DEAD_CODE_SKIP` would mean mutating process-wide state in a suite `ci / test` runs
as ONE process. Measured — a `#[serial]` test that set it failed three unrelated tests beside
it, so it was written, run, and removed rather than shipped. The rule every path now routes
through is unit-tested on all four combinations, the suppressed-scan one included.

**Three findings, three rounds, one lane.** Lane 1 (`gemini-3.1-pro-high`) found every one;
lanes 2 and 3 passed the diff each time. The quorum is not a formality here — it is the only
thing that caught a guard that deletes unreadable lockfiles, a swallowed failure, and a hole
opened by the fix for another hole.

## Contract

`contracts/dead-code-lockfile-isolation-v1.yaml` — `pv status`:

```
References: 7   Equations: 2   Proof obligations: 10   Falsification tests: 11   Kani harnesses: 0
```

**10 obligations declared, 10 evaluated, 0 failed.** DCLI-OB-001..010 map one-to-one onto
DCLI-F-001..008 and DCLI-F-010..011, each naming the `cargo test` invocation that judges it; every one was run
above. DCLI-F-009 (the same guarantee under a second toolchain) is evaluated by this receipt's
RED/GREEN transcripts and is the ninth falsification test against the eighth obligation.

> An earlier revision of this contract declared two equations and `pv status` reported
> **one**: the second was written at column 0 and became a silent top-level key that
> `pv validate` accepted without complaint. Caught by reading the count rather than the
> verdict. Now `Equations: 2`.

## Gate

`make gate` was run twice. Run 1 (`/tmp/pmat-gate.n9oall`) was **RED on 6 legs**; five were
mine and one was not. Run 2 (`/tmp/pmat-gate.z66SgL`, plus a targeted re-check after it) is
**RED on exactly one leg, which is master's own**.

| Leg | Run 1 | Cause | Run 2 |
|---|---|---|---|
| `lib-tests` | FAIL | `the_committed_ratchet_holds_at_head`: `panic_macro_calls_src` 787 vs baseline 785 — the two `panic!` arms in my new tests. **One of the two was in PROSE**: the ratchet greps `git grep -oF 'panic!('`, so a doc comment quoting the macro counts as a call. Also `the_committed_ledger_matches_the_tree` (see `unrun-tests`). | PASS — rewritten as an assert on the variant plus a `failure_detail` helper; count back to 785. **The baseline was not raised.** |
| `pv-obligations` | FAIL | `applies_to` must name an equation of the contract or a bare `fn` under `src/`; I wrote four as `Type::method`. | PASS — `restore`, `restore_inner`, `drop`, `analyze`. |
| `tdg-ratchet` (CB-200) | FAIL | 1681 definitions below grade A vs baseline 1680. `restore_inner` was the one, at **grade B**: a four-arm match with two nested matches. | PASS — split into `put_back`, `take_away`, `failed`, which also puts the stable token and the path in ONE place instead of five. Every definition in the file is now A or A+; count back to **1680**. **The baseline was not raised.** |
| `unrun-tests` | FAIL | Removing seven `#[ignore]`s moves `docs/status/unrun-tests-ledger.md`. | PASS — re-rendered with `analyze unrun-tests --write-ledger`. |
| `reachability-ledger` | FAIL | Two new files move `docs/status/orphan-files-ledger.md`. | PASS — re-rendered. Both new files are **reachable** (4016 → 4018 of 4500 tracked); `orphaned` (407) and `quarantined` (75), the two counts the ratchet may only lower, are untouched. |
| `cb-2113-cb-2115` | FAIL | CB-2113 **passes** (all 4 commits carry `Pmat-Ticket`). CB-2115 reported 3 findings; **one was mine** — a title DRIFT between the roadmap row and issue #1403, because `pmat work add` was given a shorter title than `gh issue create`. | **STILL RED, and not mine.** My DRIFT is fixed; what remains is exactly the two findings that pre-exist on `master`. |

**31 legs PASS. The one RED leg was `cb-2113-cb-2115`, on master's own orphans**, proven
against `origin/master` rather than asserted:

- `ORPHAN-ROADMAP PMAT-1399`: `origin/master:docs/roadmaps/roadmap.yaml:7039` carried `status: planned`, and `gh issue view 1399` was `CLOSED`.
- `ORPHAN-GITHUB #1401`: `git show origin/master:docs/roadmaps/roadmap.yaml | grep 'github_issue: 1401'` matched **nothing**, and the issue was `OPEN`.

**It resolved itself while this branch was in flight.** PR #1405
(`PMAT-1336-lifecycle-10`, merged as `ac8a59e40`) completed PMAT-1399, registered PMAT-1401
— and registered PMAT-1403 itself. That is the lifecycle PR the brief said the orchestrator
would run, and it landed before this PR could merge, so the escalation is closed by that
change rather than by this one.

Rebasing onto it cost one thing worth recording: master's registration of PMAT-1403 carries
`labels: []`, and `kind-gate.sh` exits 2 without `kind:code`. The branch's own row, which had
the label, was dropped in the rebase because master already carried the id. The label is
restored in its own commit; master's title is kept.

### Every `make gate` run, and what each red leg was

| Run | Verdict | Red legs |
|---|---|---|
| 1 `n9oall` | RED | 6 — `lib-tests`, `pv-obligations`, `tdg-ratchet`, `unrun-tests`, `reachability-ledger`, `cb-2113-cb-2115` |
| 2 `z66SgL` | RED | 1 — `cb-2113-cb-2115` (master's orphans) |
| 3 `uUBraD` | RED | 1 — **`lib-tests`**, the flake |
| 4 `HCxVUT` | **GREEN** | — |
| 5 | (superseded by the quorum round-3 fixes) | — |
| 6 `btChzM` | RED | 3 — `lib-tests`/`unrun-tests` (ledger drift from the 3 new tests) and `tdg-ratchet` (`restore_inner` at A-) |
| 7 | **GREEN** | — |
| 8 | **GREEN** (on `e541edb99`, the quorum's judged tree) | — |
| 9 | **GREEN** (on `c5fca1c03`, the final HEAD) | — |

**Nine runs; six distinct red legs; every one of them mine except `cb-2113-cb-2115`, and
neither ratchet ever raised.** `panic_macro_calls_src` went back to 785 (one of the two
offenders was in PROSE — the ratchet greps `git grep -oF`, so a doc comment quoting the macro
counts as a call). CB-200 went back to 1680 twice: `restore_inner` scored B, was decomposed,
then scored A- once the quorum's third state was added, and was decomposed again — one branch
per state of `before`, which is also how the invariant reads.

**The gate-3 flake.** `analysing_a_lockfile_less_crate_creates_no_lockfile` and
`the_analysis_leaves_no_lockfile_in_the_analysed_tree` failed once, in nine full gate
runs, on code identical to the run before it. It did not reproduce in:

- 3 further full `cargo nextest run --lib --profile gate` runs — **21789/21789 passed each**, ~65,000 test executions;
- 110 targeted runs of the two tests;
- 40 concurrent runs;
- gates 4, 7, 8 and 9, the same harness that produced it.

The symptom is only consistent with the lockfile being absent when the guard restored and
present at the assertion. The obvious mechanism — a cargo the analysis runs BEFORE the
snapshot — was tested and **disproved**: `isolated_target_dir` and `named_targets` both pass
`--no-deps`, and 20 concurrent runs of each, with and without `CARGO_TARGET_DIR`, created no
lockfile. The hole was closed anyway (the guard is now taken before any cargo the analysis
starts), because the guard's promise must not depend on what the builder happens to call
today. **The flake is recorded as unexplained, not as fixed.**

**`falsification / flag-efficacy`.** Red on this branch, green on master, twice — and it was
this change that did it, correctly. The sweep reported `analyze reachability --allow-dirty`
and `analyze unrun-tests --allow-dirty` as NO-OP. They are not: on a tree with a modified
TRACKED file, `analyze reachability --write-ledger` exits 1 printing ` M src/lib.rs`, and the
same command with `--allow-dirty` exits 0 and writes the ledger. Measured both directions.

What changed is the corpus. It commits a hand-written `Cargo.lock`
(`tests/modules/quality_harness/mod.rs:1497`, written before `git add -A`, so TRACKED), and
until this ticket `pmat analyze dead-code` rewrote that tracked file and left the corpus
**dirty for every command the sweep ran afterwards**. `--allow-dirty` had something to permit
only because an earlier probe had broken the fixture. The guard restores the lockfile, the
corpus stays clean, and the flag correctly changes nothing.

So a required gate had been reading the defect as a feature for as long as the defect
existed. Both flags are recorded in `ALLOWED_NOOPS` with the measurement and with what to do
instead — dirty a tracked file on purpose — rather than being wired up to a bug. Sweep after
the change: **457 effective, 4 refuses-honestly, 0 no-op, 1 error-out, 251 skipped**, and the
harness's own `noop_detection_is_load_bearing` self-test still passes, so the detector that
found this still fires.

## Stop-the-line conditions

| # | Condition | Disposition |
|---|---|---|
| 1 | `make gate` leg `cb-2113-cb-2115` RED on master's two pre-existing orphans | Named, not fixed. **Closed by PR #1405 landing mid-flight**; gate 4 is green. |
| 6 | Two tests failed once in four full gate runs and did not reproduce in ~65,000 further test executions | Recorded as an unexplained flake, with the one mechanism that was tested and disproved, and the adjacent hole closed. NOT claimed as fixed. |
| 7 | A required CI gate (`flag-efficacy`) was passing because of the defect | Root-caused to the corpus's tracked `Cargo.lock`, and recorded in `ALLOWED_NOOPS` with the measurement rather than worked around. |
| 2 | `~/src/infra` on this host is 49 commits behind and lacks the overlay, so the brief's reproduction command would have proved nothing | Reproduction rebuilt from `origin/main:machines/clean-room/gates-lib.sh`. |
| 3 | The brief's toolchain hypothesis was wrong | Corrected with a measurement, not an argument (cargo 1.98.0 reproduces identically). |
| 4 | A falsifier I wrote was vacuous under root — the environment the gate actually runs in | Caught by the container GREEN run and replaced. |
| 5 | Phase 3 was routed `self` against R-4's `agy-goal` default | Recorded above with the reason. |

## Status-line join

| Claim | Measured | How |
|---|---|---|
| statusLine `session_id` = hook `session_id` | `[V]` true | both `f5bcef1a-c5c7-48a9-9366-aa297b0088f3`; `discover.sh` printed `session=f5bcef1a-… rule=pid-file claude_pid=1798305`, `transcript-gate.sh` resolved the same session by the same rule |
| `tasks[].id` = hook `agent_id` | `[V]` true | the single dispatch returned `agentId: a44afb93674886ab2`; `transcript-gate.sh` found `files=1 agent_calls=1` for this session |
| `transcript_path` present on subagentStatusLine stdin | `[V]` true | `transcript-gate.sh` read 30 segments from 1 transcript file; a missing path would have made `files=0` |
| `k_measured` vs `global=k` | **gap, named** | `k_measured=84` distinct `message.id`s over 191 assistant rows; my own turn count is ≈42. The ratio is ~2, i.e. the measure counts API responses that my own count treats as one turn. Cause `[U]` — not investigated further, because the gate is the receipt's honesty about the gap, not the gap's size. |

## I-3

```
PASS transcript-gate: attempted=1 denied=0 stalled=0 running_peak=1 slots=3 segments=30 files=1
     (agent_calls=1 resumes=0 workflow_started=0; denied from hook log)
```

`running_peak=1 ≤ slots=3`. One Claude subagent existed at any instant, and only one ever
existed. **0 denials, 0 stalls.** No `Workflow` call was made; no resume was made.

## Jidoka log

`.pmat/jidoka.jsonl` — each red stopped the line and was root-caused before the next step:

1. `{ticket: PMAT-1403, phase: 5, defect: "panic_macro_calls_src 787 > baseline 785", owner: "src/services/cargo_dead_code_analyzer/lockfile_guard_tests.rs", whys: "new panic! arms in my own tests; one of them in a doc comment, because the ratchet greps literally"}`
2. `{ticket: PMAT-1403, phase: 5, defect: "CB-200 1681 > baseline 1680", owner: "lockfile_guard.rs::restore_inner", whys: "a four-arm match with two nested matches scored B; decomposition was the fix, not a baseline bump"}`
3. `{ticket: PMAT-1403, phase: 5, defect: "pv obligation gate: 6 problems", owner: "contracts/dead-code-lockfile-isolation-v1.yaml", whys: "applies_to accepts an equation name or a bare fn under src/, not Type::method"}`
4. `{ticket: PMAT-1403, phase: 5, defect: "two committed ledgers drifted", owner: "docs/status/", whys: "removing seven #[ignore]s and adding two files are exactly what those ledgers count"}`
5. `{ticket: PMAT-1403, phase: 5, defect: "CB-2115 DRIFT PMAT-1403 <-> #1403", owner: "docs/roadmaps/roadmap.yaml", whys: "pmat work add and gh issue create were given different titles"}`
6. `{ticket: PMAT-1403, phase: 4, defect: "a_restore_that_cannot_write_is_reported_as_failed vacuous under root", owner: "lockfile_guard_tests.rs", whys: "chmod 0555 is not read-only to uid 0, and the clean room runs its gates as root"}`

## Estimates

| | value |
|---|---|
| `K̂` | 95 |
| `K` (budget) | 200 |
| actual (`k_measured`) | see the gap above |
| basis | `docs/audits/impl-estimates.jsonl:L26-L36` (ROWS=18 MEDIAN=95 EXCLUDED=11 UNMEASURED=8) |

## Gaps — every NotRun lane and what closes it

| Gap | Closed by |
|---|---|
| `make gate` is not fully green: `cb-2113-cb-2115` RED on master's two orphans | the orchestrator's lifecycle PR — not this one |
| The full `make -C .../clean-room clean-room-pmat` target was not run | a targeted container with the same image, overlay and test command; the full target on a host whose `infra` is current |
| A `SIGKILL` of pmat mid-`cargo check` still leaves the rewrite behind | nothing in-process can close it; declared in the contract's `preconditions` and in the code |
| `lane-reduce.sh` stamped `author.source=flag`, not a measurement | the `$XDG_RUNTIME_DIR/paiml-implement/model-<sid>` file, absent this session. `model-gate.sh` corroborated `opus-5` independently. |
| The quorum did not judge candidate A's residual SIGKILL risk | a lane brief that names the residual explicitly; not re-run for this ticket |
| Phase 3 did not route to `agy-goal` per R-4 | recorded, not closed |
| `--dogfood` receipt | not requested |

## Machine-readable (AUTO-IMPL-SKILL-003 §7 / Appendix D)

orch_model: opus-5 [V]   orch_class: code   orch_decision: admit   orch_basis: none
fable_binding: true   quota_age_h: absent   quota_mark: ?   k_measured_at_set: 12

`orch_basis: none` is correct and not a gap: the ticket carries no `orch:fable` label, so
`model-gate.sh` admitted `opus` on the default rule and no `orch-basis:` token was required.
`quota_age_h: absent` is `quota.sh binding` reporting `age_h=absent account_mismatch=true`
on this host — the status line renders `q=?`, and no number is invented to fill it.

routes:
  ph1  class=plan   route=agy-plan  w=1.00  basis=absent
  ph2  class=orchestration  route=self  w=1.00  basis=absent
  ph3  class=impl   route=self  w=1.00  basis=absent   note=R-4-deviation-recorded (printed route=agy-goal w=1.00 basis=absent note=fable-binding)
  ph4  class=orchestration  route=self  w=1.00  basis=absent
  ph5  class=orchestration  route=self  w=1.00  basis=absent

verification:
  cmd=cargo test --lib lockfile (clean-room toolchain, master tree)  claimed_exit=-  rerun_exit=1  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/.run/pmat-1403/red.log  sha256=1f1b1379087989e6
  cmd=cargo test --lib lockfile (clean-room toolchain, fixed tree, then mutant, then revert)  claimed_exit=-  rerun_exit=0  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/.run/pmat-1403/green2.log  sha256=7c3dbd15c6e1757f
  cmd=make gate (run 2)  claimed_exit=-  rerun_exit=1  log_path=/home/noah/src/paiml-mcp-agent-toolkit.wt/.run/pmat-1403/gate2-pmat1403.log  sha256=4e9f7aba702ef3cc

The `make gate` row is `rerun_exit=1` on purpose: 31 legs pass and the 32nd is master's own
orphan. A receipt that recorded 0 there would be the thing this skill exists to prevent.

## Verdict

**DONE.**

- The defect is reproduced, root-caused to the analyzer rather than to the clean room, fixed at the cause, and proven by a mutant in the same toolchain that found it.
- `make gate` runs 7, 8 and **9 — the final HEAD — are GREEN, all 32 legs**. The one leg that was red for three runs was master's own orphan, and PR #1405 closed it.
- The quorum ran **five rounds** and returned **3/3 PASS** on the final tree. Lane 1 returned FAIL three times in between and was right every time; see below.

Two things are carried forward rather than claimed as solved, and neither is in this diff:

1. The gate-3 flake — one occurrence in nine full gate runs, no reproduction in ~65,000 further test executions, the obvious mechanism disproved and the adjacent hole closed regardless. If it recurs, the place to start is this receipt, not a fresh investigation.
2. The `SIGKILL` window, which no in-process mechanism reaches and which `--locked` closes only by not scanning. It is declared in the contract's `preconditions`.

The six-part DoD holds: merged green on the required checks; the gate exists and was run four
times; the mutation was observed RED (4 tests under an ambient `[patch]`, 2 without one, in
the clean-room toolchain and locally); the `pv` contract ships in this PR with 8 obligations
evaluated and 0 failed; discrimination is confirmed — the mutant kills the byte-identity half
and leaves the fidelity half green, so the two halves are independently measured; and every
doc claim this change invalidated is updated in the same PR: the module docs of both test
files, the `--locked` comment, seven stale `#[ignore]` reasons, and a required CI gate that
had been passing because of the bug.
