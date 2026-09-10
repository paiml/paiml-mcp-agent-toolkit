# IMPL-PMAT-724 — goal-mode step 5: CB-2112 Ticket Linkage and CB-2114 Release Binding, control first, the gate step withheld until master satisfies both

## Identity

| field | value |
|---|---|
| ticket | PMAT-724 (`kind:code`, `orch:fable`, `kind-gate.sh` exit 0 `kind=code ticket=PMAT-724 files=60` against `master`) |
| spec | `docs/specifications/goal-mode.md` §2 (doctrine 2, 6), §3.3, §4.1 (RR-RELEASE), §4.2, §7 rows CB-2112 and CB-2114, §11 step 5, §12 |
| branch | `PMAT-724-cb2112-cb2114-ticket-release-rules`: from `PMAT-722-cb2115-roadmap-coherence` @ `638f9023c` (#1249) — **stacked on #1246 → #1247, #1248 and #1249**; the two rules need `CheckOverrides`/`--github-snapshot`, `SnapshotSource` and the `release:` field |
| HEAD in | `638f9023c` (#1249's head) |
| HEAD out | `3039b77ba` (the last code change: the header fix after mutant M5 survived; PMAT-726 filed in the same commit); the ledger and receipt commits follow |
| PR | opened from this branch after the receipt commit — `gh pr list --head PMAT-724-cb2112-cb2114-ticket-release-rules` |
| `discover.json` sha256 | `c88f18c8b2aa5b5005fe5ccb372a4f50da3640ecc3877034be00bc230a6ebb96` |
| `gate_cmd` | `cargo test --workspace` — **`gate_cmd_fallback=true`**; `pmat verify` is the gate this repository's CLAUDE.md names and is what was run |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| model gate | `model=fable class=fable decision=admit basis=file` |

### One ticket per session — refused, then reaffirmed

`goal.sh set --ticket PMAT-724` was refused: `one ticket per session: PMAT-719 was set here — start a new claude session` (R-5), the fourth ticket this transcript has carried. The operator's instruction for this pass, quoted verbatim: **`pmat-implement docs/specifications/goal-mode.md autonomously`** — the same reaffirmation PMAT-720 and PMAT-722 proceeded under, so this ticket proceeded with the refusal recorded and the turn accounting split by hand below. `goal.sh worker`, `gate` and `phase` accepted their declarations.

### Status-line join `[U]→[V]`

| claim | measured | how |
|---|---|---|
| statusLine `session_id` = hook `session_id` | **true** | `transcript-gate.sh` resolves `39a15c63-…` by `rule=pid-file`; `attempted=9 denied=0` from the hook log |
| `tasks[].id` = hook `agent_id` | **[U]** | the two delegate `agentId`s (`af91ecbc5124cf453`, `a387bd21837a5ffde`) are not paired in the hook log |
| `transcript_path` on subagentStatusLine stdin | **[U]** | not measured |
| `k_measured` vs `global=k` | transcript-wide `k_measured=245`; PMAT-722's receipt closed at 192, so this ticket's share is **53** | `jq -r 'select(.type=="assistant" and ((.isSidechain // false)\|not)) \| (.message.id // .uuid)' <transcript> \| sort -u \| wc -l` |

## What step 5 is

**CB-2112: Ticket Linkage** (`check_ticket_linkage.rs`, invariant A): every open item (status not completed or cancelled, §4.2) names a `github_issue` that exists, is open, and whose number is the item's numeric tail — the id `pmat work add --github-issue N` mints is `<PREFIX>-N` (PMAT-714, #1240), so the tail is the one link two branches cannot mint twice. Four finding classes: NO-ISSUE, ISSUE-ABSENT, ISSUE-CLOSED, TAIL-MISMATCH. **CB-2114: Release Binding** (`check_release_binding.rs`, invariants B/F1): every open item carries a `release:` that is the bare semver string (§4.1), names a milestone with that exact title, that milestone is open, and its issue is on it. Six classes: NO-RELEASE, PREFIXED, NO-MILESTONE, MILESTONE-CLOSED, NO-ISSUE, NOT-ON-MILESTONE. Both judge through pure predicates in `services/work_sync/linkage.rs` from the same `GithubSnapshot` `pmat work sync` and CB-2115 read; the three rules now share one preamble (`check_roadmap_inputs.rs`) with the same early verdicts — Skip for a structural absence, Fail `not_measured:` for a missing input, a `.pmat.yaml` snapshot path refused as the bypass §12 forbids.

**Withheld on purpose:** the direct `--checks CB-2112,CB-2114` step is not in the traceability job. Doctrine 6 — a rule red on the day it lands gets disabled — and this repository violates both today: `docs/roadmaps/roadmap.yaml` on master has **63 open items, 0 carrying `release:`, 0 whose numeric tail is an open issue** (50 name no issue, 13 name #612); this branch, carrying steps 1–5's tickets, has 72 / 0 / 0 (59, 13). The job runs the rules' **control** (`scripts/ticket-release-control.sh`, 17 arms) instead; the exact step that flips them is a comment beside the withheld CB-2115 step and is filed as **PMAT-725**, after PMAT-723's fixers and a human decision on re-minting the open ids (a rename touches every trailer and receipt that names the old id; no sync does it). `pmat comply ledger` therefore writes both **NEUTERED**, which is the truth.

## Plan and routing

| phase | what | class | `route.sh` (verbatim) | taken? | trigger |
|---|---|---|---|---|---|
| 1 | branch from #1249, PMAT-724 filed and canonicalised, kind/model gates, R-5 refusal; RED: 26 tests against Skip stubs (`fe1f24106`, 25 fail) | orchestration; mechanical | `route=self w=100.00 basis=absent`; `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | taken; **not taken** for the RED — the RED test is the orchestrator's by doctrine (Phase 2 step 1) | — |
| 2 | the predicates, the shared preamble, the two rules | impl, single module | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | **taken** — lane FAILED on isolation (below); its commit `4bb0edfa2` verified and kept, then reviewed and cleaned (`75425f62f`) | R-4 |
| 3 | control script (15 arms then), the traceability job's third control step, README 161, PMAT-725, ledgers | impl | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | **not taken** — a worker may not edit `.github/workflows`, and the shared tree must stay untouched while a writes lane runs (its isolation assertion diffs `git status`); done by me | — |
| 4 | pre-PR quorum on `638f9023c...92eafba2d`, adjudication, the six fixes, control to 17 arms | review; impl | `route=agy-quorum w=1.00 basis=absent effort=1[U]`; `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | taken, width 3; **not taken** for the fixes — four modules under a FAIL gate, done directly with each fix re-run | Phase 4 pre-PR review |
| 5 | mutants, verify, ledgers, receipt, PR | orchestration | `route=self w=100.00 basis=absent` | taken | — |

`quota.json` is absent on this host, so every route line carries `basis=absent`; the Phase 1 `teamwork` grill (Q2) was not run (Gaps).

## Dispatch ledger

| phase | mode | description | agent id | turns | maxTurns hit | resumed | lane / width / agy `conversations` |
|---|---|---|---|---|---|---|---|
| 2 | delegate (`paiml-agy-delegate`, opus) → agy `goal`, `writes=true` | `PMAT-724/ph2.delegate goal width 1 on CB-2112/CB-2114 implementation` | `af91ecbc5124cf453` | 35 tool uses, 3577 s | **yes** (while polling) | no — the lane had finished; reduced by me | goal / 1 / `dacfa57a-f48f-4da2-a328-b715cc6488e5` — attempt 1 exit **137** (killed; worktree `lane-fe1f24106-87894`), attempt 2 exit **3, LANE ISOLATION VIOLATED**: `the shared checkout's HEAD moved (fe1f24106 -> 4bb0edfa2)` and `the shared checkout's working tree changed`; its worktree `lane-fe1f24106-2950646` never touched; `structured_output` null (`lane-reduce`: NO-VERDICT, `partial=true`); `fanout.sh` refuses `--lane goal` → `children=unknown method=unsupported label=single-lane` |
| 4 | delegate (`paiml-agy-delegate`, opus) → agy `grillme` quorum, `writes=false`, sandboxed | `PMAT-724/ph4.delegate quorum width 3 on CB-2112/CB-2114 review` | `a387bd21837a5ffde` | 26 tool uses, 992 s | no | no | quorum / 3 / `06267447-8e92-431f-8fc9-bd854f207e7c`, `e1d193bb-2396-451b-b725-a0270f2c5926`, `a748e17d-d114-482e-b722-19bd3da595ab` — 3/3 FAIL; `children=3 method=lane-files label=consensus`; `lane-reduce` `agreed=false partial=false`; `receipt-lint --kind delegate`: `dedup=14 uncovered=0 dissent=3 partial_reasons=0` |

**The escaped commit was verified, not trusted.** `4bb0edfa2` (author `paiml-implement lane <noreply@paiml.invalid>`, trailer `Pmat-Ticket: PMAT-724`, seven `src/` files, no test file, no forbidden path) landed on the branch in the shared checkout; its acceptance re-run by me: **43 passed, 0 failed**, clippy clean, fmt clean, both controls green on cargo's executable. Review found four things and `75425f62f` fixed them: CB-2115's doc comment and the bypass-token explanation lost in the refactor; every type spelled through `crate::cli::handlers::…` where the short names are in scope; the eight-finding cap and the verdict shape repeated three times (now `roadmap_verdict` / `first_eight`); and three `unwrap_or_else(\|\| panic!)` in my own RED test file had moved `panic_macro_calls_src` from 785 to 788.

**Why the lane escaped, third time running:** the lane's cwd was the shared checkout, not the worktree `agy-lane.sh` made for it, on PMAT-720, PMAT-722 and now PMAT-724 — with a prompt that this time named no commit and forbade `cd`, `checkout`, `switch` and `worktree` explicitly. The worktree sits inside the checkout's own directory tree; probable cause `[U]`, named in Gaps. The `writes=false` quorum lanes also wrote into the shared checkout (`scratch`, an ELF; `scratch.rs`; `scratch_test/`), so `--sandbox` does not jail a lane at `repo_root` either.

## Slots, denials, I-3

| field | value |
|---|---|
| `slots` | 3 (`config.json`, `config-lint.sh`: `slots=3 gh_calls_per_min=30 bank=3`) |
| Claude subagents dispatched for this ticket | **2** (both delegates; no worker) |
| `transcript-gate.sh` (session-wide, PMAT-719/720/722 included) | `PASS transcript-gate: attempted=9 denied=0 running_peak=2 slots=3 segments=252 files=9 (agent_calls=9 resumes=0 workflow_started=0; denied from hook log) (session 39a15c63-fab3-4148-8cb7-0cb012369b35, rule=pid-file (/run/user/1000/paiml-implement/pid-21623))` |
| denials | `denied=0` |
| Workflow tool | not used |

## Quorum (Phase 4, `docs/audits/quorum-PMAT-724.json`)

3/3 **FAIL** on `92eafba2d`; 14 dedup groups, 9 distinct claims after my own dedup (the delegate's citation audit is in the artifact's `open_questions`) — 6 CONFIRMED and fixed with a test and a control arm each, 2 REFUTED by a named test, 1 harness, every one re-measured first:

| # | finding | lanes | adjudication | closed by |
|---|---|---|---|---|
| 1 | `\|\| release.starts_with('V')` is a mutant no test or arm catches | 1, 3 | CONFIRMED — and my first fix was **vacuous**: the test and arm 11 asserted the word PREFIXED, which the Fail header carried for every class with its count, so M5 still survived (Mutation table); the header now names only the classes that fired | `a_v_prefixed_release_fails_binding_in_either_case` asserting the rendered finding, `the_fail_header_names_only_the_classes_that_fired`; arm 11 both cases; mutant M5 (second run) |
| 2 | an open item whose milestone exists but is CLOSED passes | 1, 2, 3 | CONFIRMED — §4.1 RR-RELEASE: a tag is cut only when its milestone has zero open issues, and a milestone closes only when its tag exists, so an open item on a closed milestone contradicts the model; the §7 row says "exists" and this is a derivation (Gaps) | `ReleaseFinding::MilestoneClosed`, `a_release_whose_milestone_is_closed_fails_binding`, engine test, arm 9; mutant M4 |
| 3 | the shared preamble skipped when `github_repo` was null, no remote resolved and no item named an issue — reachable by nulling one field | 1, 2, 3 | CONFIRMED (tightened): Skip only when the roadmap DECLARES `github_enabled: false`; with the default `true` and no resolvable repository all three rules are Fail `not_measured` (§3.3: a missing input) | two new tests, CB-2115's skip test re-fixtured, arm 17 both ways |
| 4 | CB-2112 invented an ISSUE-EXCLUDED clause (the `no-roadmap` label) the spec does not name | 1, 3 | CONFIRMED — the label is §5.1's (it removes the issue from G; CB-2115 reports the item as ORPHAN-ROADMAP), one place per judgement (doctrine 5) | variant and clause removed; `an_open_issue_labelled_no_roadmap_still_links`; arm 5 GREEN |
| 5 | the withheld-step comment's "70 open items, 57 with no issue" is not reproducible | 1, 2, 3 | CONFIRMED — I had measured the branch before PMAT-724/725 were filed and labelled it master: master 63/50/13/0/0, branch 72/59/13/0/0 | ci.yml comment and the fifth criterion name both trees |
| 6 | control arm 1 asserts only CB-2112 while the header claims the other rule is proven quiet | 2 | CONFIRMED (assertion + wording): CB-2114 fires NO-ISSUE on the same input, legitimately (membership cannot be checked without an issue) | arm 1 asserts both; the header says each arm states what the other rule did |
| 7 | `tail == Some(n) \|\| tail.is_none()` survives: no test pairs an id without a tail with a real issue | 1 | REFUTED | `an_id_without_a_numeric_tail_is_named_as_such` pairs `EPIC` with open #1 and asserts TAIL-MISMATCH "no numeric tail" |
| 8 | dropping the `IssueAbsent` branch survives every test and arm | 2 | REFUTED (the delegate's own audit found the test) | `an_open_item_naming_an_absent_issue_fails_linkage`; arm 3 now runs the absent case too |
| 9 | the sandboxed lanes wrote `scratch` (ELF), `scratch.rs`, `scratch_test/` into the shared checkout | delegate | CONFIRMED (harness) | inspected and removed; a `test_export` crate at `/tmp/Cargo.toml` (12:21 local, provenance unknown, predates the lanes) had also failed 10 no-manifest lib tests in the first `pmat verify` (`left: Some("/tmp")`); removed after inspection, verify re-run |

## Mutation table

Two runs. Run 1 (at `099a22313`) killed M1–M4 and **M5 survived** (29 tests green, no arm failed); run 2 (at `3039b77ba`, after the header fix) kills all five. Each mutant was applied one at a time to the real source with backups under `.pmat/pmat724-mut/`, the named tests and the 17-arm control run on the executable cargo reports, and the source restored byte-identically before the next (`mut724.sh`; run-1 summary `mut724-summary-run1.txt`).

| mutant | planted as | killed by (run 2) | log |
|---|---|---|---|
| M1: the verdict inverted | `if !findings_empty` in `roadmap_verdict` | **19 of 30 tests** FAILED (every Pass/Fail test of both rules, the scope tests, the header test, the group-list test); control **ARM 1 FAILED** | `mut724-M1-verdict-inverted-{tests,control}.log` 97bbd43efe8389a7 / 735de08b880373a0 |
| M2: `not_measured` reads Skip | `status: CheckStatus::Skip` in the `not_measured` closure | 4 tests FAILED (`a_committed_then_deleted…`, `a_roadmap_declaring_github_but_resolving_no_repository…`, `a_snapshot_path_committed_in_pmat_yaml…`, `a_snapshot_that_cannot_be_read…`); control **ARM 12 FAILED** | `…-M2-not-measured-reads-skip-…` a45043db481b4243 / f53c354fdf167faa |
| M3: the tail clause dropped | `if number > 0` for `if tail == Some(number)` | 3 tests FAILED (`an_id_without_a_numeric_tail_is_named_as_such`, `an_open_item_naming_an_open_issue_that_is_not_its_tail…`, engine `linkage_findings_are_ordered_by_item_id`); control **ARM 4 FAILED** | `…-M3-tail-clause-dropped-…` 33016eee737fdba2 / 55a187aebf8b81e6 |
| M4: milestone state ignored | `&& false` on the `IssueState::Closed` test | 2 tests FAILED (`a_release_whose_milestone_is_closed_fails_binding`, engine `a_closed_milestone_is_a_finding_of_its_own_class`); control **ARM 9 FAILED** | `…-M4-milestone-state-ignored-…` 87a70e05499e9f96 / a12a790697aaa278 |
| M5: the capital `V` dropped | `starts_with('v')` for `starts_with(['v', 'V'])` | run 1: **SURVIVED** (the test and arm 11 read the word PREFIXED off a header that named every class); run 2: `a_v_prefixed_release_fails_binding_in_either_case` FAILED; control **ARM 11 FAILED** | `…-M5-capital-V-dropped-…` 0ffacb24829a87fb / 7b96b01d10595b67 (run 1: d9d0017cf7e64b8d / 747910b265111eda) |
| the rules absent | the Skip stubs at `fe1f24106` | 25 of 26 tests FAILED (the roster entries are in that commit, so the registration test passes) | commit `fe1f24106` |
| the §7 falsifiers and every class | control arms 1–17 | exit code AND both rule rows asserted on every arm; arms 2, 5, 7, 16 and the first half of 17 prove the same command can pass | `control724.log` 2b7e98cb596189af |

Inside the nohup runner cargo reported `./target/debug/pmat` as its executable while the interactive shell's cargo reports `/mnt/nvme-raid0/targets/…/debug/pmat` — two build dirs on this host; each control run used the path cargo printed for that build and fingerprinted it, so nothing here rests on a stale copy.


## Verification table (claimed vs my rerun)

| command | claimed | rerun | at |
|---|---|---|---|
| `cargo test --lib -- tests_ticket_release linkage_tests tests_roadmap_coherence` | lane: no verdict returned | **43 passed, 0 failed** at `4bb0edfa2` and `75425f62f`; **46 passed, 0 failed** at `099a22313`; **47 passed, 0 failed** at `3039b77ba` (26 → 30 new tests, 17 CB-2115) | `3039b77ba` |
| `bash scripts/ticket-release-control.sh <cargo's executable>` | — | **17/17 arms** | `ed3549c1e`, `3039b77ba` |
| `bash scripts/roadmap-coherence-control.sh <cargo's executable>` (CB-2115 after the preamble refactor and the header change) | — | **11/11 arms** | `ed3549c1e`, `3039b77ba` |
| `bash scripts/work-sync-control.sh` / `bash scripts/traceability-control.sh` | — | 4/4 / 5/5 arms | `ed3549c1e` |
| `cargo clippy --lib --tests -- -D warnings` / `cargo fmt --all -- --check` | — | clean / clean | `099a22313` |
| `pmat comply ratchet` | — | at baseline: `panic_macro_calls_src` 785 (788 at `4bb0edfa2` — my RED test file's three `panic!`), `unwrap_calls_src_outside_cfg_test` 9177, `orphan_files` 407 | `ed3549c1e` |
| `pmat comply ledger --write` | — | **161 rules, ENFORCED 1 (CB-2113), NEUTERED 160; CB-2112 and CB-2114 NEUTERED** via quality-gate.yml's `continue-on-error` carrier | `3cdb8a41c`, `ed3549c1e` |
| `analyze unrun-tests --write-ledger` / `analyze reachability --write-ledger` / `--check-ledger` | — | re-rendered / re-rendered / current (both refuse a dirty tree — commit first) | `92eafba2d`, `ed3549c1e` |
| the roadmap measured for both rules (`python3 + yaml`, `is_open` = status ∉ {completed, cancelled}) | — | master: 63 open, 50 no issue, 13 #612, 0 `release:`, 0 tail = issue; branch: 72, 59, 13, 0, 0 | `master` @ `20fda3ca6`, `099a22313` |
| `pmat verify --format json` | — | run 1 at `92eafba2d`, started before the quorum and overlapping its lanes: **`ok:false`**, 10 "no manifest anywhere" tests reading `Some("/tmp")` — the `test_export` crate at `/tmp/Cargo.toml`; run 2 at `ed3549c1e`: **`ok:false`**, 1 test — `quality_proxy_property_tests::test_advisory_mode_…`, the `set_current_dir` race (PMAT-726), green 3 of 3 in isolation; run 3 at `3039b77ba`: **`ok:false`**, 1 test — `unrun_tests::the_committed_ledger_matches_the_tree`, the ledger not yet re-rendered for the header-shape test (self-inflicted; 21449 passed, the race did not recur); run 4 at `2530ce8a8` (ledger re-rendered, docs-only change): **`ok:true`, 5/5 stages (format, complexity, satd, clippy, tests; 366 s)** | `2530ce8a8` |
| `transcript-gate.sh` / `kind-gate.sh` / `model-gate.sh` | — | (I-3 above) / `kind=code files=60` / `decision=admit` | — |

**The executable.** Every control and every mutant reading above is on the executable cargo reports (`/mnt/nvme-raid0/targets/paiml-mcp-agent-toolkit/debug/pmat`, taken from `cargo build --bin pmat --message-format json` before each run, fingerprinted with `strings <exe> \| grep -c MILESTONE-CLOSED` = 3); `./target/debug/pmat` is a stale copy on this host and was never used.

### Discrimination

The control asserts, on every arm, the process exit code AND the rule row in the JSON report, with both rules selected on every run so each arm states what the other rule did on the same input; a missing row is exit 2. Arms 1/2, 6/7 and 8/9/10 are matched sets (one fact flipped), arm 11 flips only the case of the prefix, arm 17 flips only `github_enabled`, and arms 12–15 assert that an unreadable input, a deleted input and a bypass path in the tree each fail rather than pass. The RED commit and the mutation table show which test dies under which change.

## Jidoka log

| defect | owner | disposition |
|---|---|---|
| `goal.sh set` refused a fourth ticket per session (R-5) | skill doctrine | reported; the operator's instruction quoted verbatim (Identity); accounting split by hand |
| the RED commit was refused twice by the hooks: rustfmt on the engine test file, then clippy on a test target that could not compile (the RED's unresolved names) | this row | formatted; the RED re-cut as PMAT-722 did it — Skip stubs so the target compiles and 25 of 26 tests fail at runtime (`fe1f24106`); the commit message says 27 tests, the files hold 26 |
| the agy goal lane escaped its worktree for the third ticket running: attempt 1 killed (exit 137), attempt 2 committed in the shared checkout (`exit 3`) with no verdict; the delegate then hit maxTurns polling | paiml-implement harness | commit verified independently, kept, reviewed (four findings fixed in `75425f62f`); lane verdict FAILED; the probable cause named in Gaps |
| my RED test file carried three `unwrap_or_else(\|\| panic!)` — the ratchet read 788 against 785 | this row | `assert!` + `expect` (`75425f62f`) |
| `analyze unrun-tests --write-ledger` and `analyze reachability --write-ledger` refuse a dirty tree, and the first writes the second's dirt | pre-existing (by design) | commit, write, commit, write |
| quorum 3/3 FAIL, 9 claims | this row | 6 fixed with tests and control arms (table above), 2 refuted, 1 harness; `099a22313` |
| the sandboxed (`writes=false`) quorum lanes wrote an ELF, a `.rs` and a fixture directory into the shared checkout; a `test_export` crate sat at `/tmp/Cargo.toml` (12:21 local, before the lanes — provenance unknown) | harness / environment | both inspected and removed; the `/tmp` crate had failed 10 "no manifest anywhere" lib tests in the first `pmat verify` (`left: Some("/tmp") right: None`) — the TMPDIR trap on record; verify re-run on the fixed tree |
| the first `pmat verify` was started before the quorum and overlapped the lanes' writes | this row | discarded; the second run is the one the table cites |
| I quoted a branch measurement (70 open / 57 no issue) as master's in ci.yml and a criterion | this row | re-measured on both trees; both corrected (`099a22313`); the `176487401` commit message still carries the old figure |
| mutant M5 (capital `V` dropped) SURVIVED the first mutation run — 29 tests green, no arm failed — because every Fail header enumerated every class with its count (`NO-RELEASE 0, PREFIXED 0, NO-MILESTONE 1, …`), so `contains("PREFIXED")` held on any failure; the quorum's finding had been "fixed" with a vacuous assertion | this row | `class_counts` names only the classes that fired (CB-2112, CB-2114 and CB-2115 alike); the V test and arm 11 assert the rendered finding and the absence of NO-MILESTONE; a header-shape test added; the whole table re-run on the final code (`3039b77ba`) |
| `pmat verify` run 2 red on one test my change does not touch: `quality_proxy_property_tests::test_advisory_mode_reports_the_verdict_and_returns_the_content` — `Failed to determine current directory`, `cargo is not on PATH (No such file or directory)`; 21448 passed, 1 failed | pre-existing race: 16 files under `src/` call `set_current_dir` inside the single lib-test process (52 sites; `hooks_command_handlers_tests` 12, `defect_report_service_tests` 10, `deep_context_config_tests` 10, `scaffold/tests` 7) | root cause named, **PMAT-726** filed with the grep as its first criterion; the test is green 3 of 3 in isolation (`flaky724.log`); verify run 3 on the final tree is the row the table cites |

## Estimates

| field | value |
|---|---|
| `K̂` | 48, `basis=docs/audits/impl-estimates.jsonl:L19-L21` (`estimate.sh pmat 4`: `ROWS=3 MEDIAN=48`, `cycles=absent[U]`) |
| `K` | 96 (`2 × K̂`) — andon line 77 |
| actual | **53** turns for this ticket (transcript `k_measured` 245 minus PMAT-722's 192) |
| `pr_runs` / `mg_runs` | not yet observable: the PR is opened after this receipt |

## Gaps

| gap | artifact that closes it |
|---|---|
| the direct CB-2112/CB-2114 step is **withheld** (doctrine 6, §5.4): master violates both today (63 open items, 0 with `release:`, 0 whose tail is an open issue) | **PMAT-725** — the step is written as a comment beside the withheld CB-2115 step; preconditions: PMAT-723's fixers, and a human decision on re-minting the open ids under their issues (the tail clause makes every item minted before #1240 non-compliant, and no sync renames an id) |
| CB-2114 reports MILESTONE-CLOSED, derived from §4.1 RR-RELEASE; the §7 row says only "its milestone exists" | the spec owner confirms or strikes the derivation (goal-mode.md §7 row CB-2114) |
| Phase 1 `teamwork` grill of §4 / §11 step 5 (Q2) **NotRun** — the pre-PR quorum reviewed the diff instead | a `/teamwork-preview` receipt on the section |
| `pv` contract **NotRun** — `contracts_dir=contracts` is set; the spec names none for step 5 | `contracts/ticket-release-v1.yaml` bound by a test |
| `pmat comply check --checks X` computes all 161 rules before selecting (~15 min on this repository) | a pre-selection in `compute_compliance_report` (P1 follow-up) |
| harness: a `writes=true` agy lane runs in the shared checkout, not in its worktree — third occurrence (PMAT-720, 722, 724), now with an explicit prohibition in the prompt; `--sandbox` lanes write into `repo_root` and `/tmp` | an issue on the paiml-implement skill (other repository; not filed by this row) |
| `merged green on required_check` — pending CI on the PR; the traceability job's first run of the 17-arm control is on this PR | the PR's checks |
| the 25 untracked scratch files in the checkout are not this row's and were left untouched | their owner |

## Machine-readable

orch_model: fable [V]   orch_class: fable   orch_decision: admit   orch_basis: M>=3
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 199

routes:
  ph1  class=orchestration  route=self        w=100.00  basis=absent
  ph1  class=mechanical     route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=self (RED test is the orchestrator's)
  ph2  class=impl           route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=agy-goal (lane FAILED isolation; work verified, reviewed and kept)
  ph3  class=impl           route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=self (workflows are not a worker's; tree frozen during the writes lane)
  ph4  class=review         route=agy-quorum  w=1.00    basis=absent   effort=1[U]
  ph4  class=impl           route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=self (four modules under a FAIL gate)
  ph5  class=orchestration  route=self        w=100.00  basis=absent

verification:
  cmd=cargo-test-lib-three-modules(47)              claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/tests724.log  sha256=0adf1ffea5eb26ac
  cmd=ticket-release-control-17-arms                claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/control724.log  sha256=2b7e98cb596189af
  cmd=roadmap-coherence-control-11-arms             claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/cb2115-control724.log  sha256=09e9789b47e6bc80
  cmd=work-sync-control-4-arms                      claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/wsc724.log  sha256=78c910e08c8edd9f
  cmd=traceability-control-5-arms                   claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/tc724.log  sha256=224ad25c16ae1ad5
  cmd=comply-ratchet@ed3549c1e                       claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/ratchet724.log  sha256=ebddd84fd2c66165
  cmd=lane-reduce-ph2                               claimed_exit=-  rerun_exit=1(NO-VERDICT,partial)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/lane-reduce-ph2-724.json  sha256=fff389f637bcdcdd
  cmd=delegate-receipt-ph4(lane-reduce-embedded)    claimed_exit=-  rerun_exit=1(agreed=false)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/ph4-receipt.json  sha256=bb42e210fe8982f5
  cmd=transcript-gate                               claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/transcript-gate-724.log  sha256=3a28f9d6771503e3
  cmd=mutant-M1-verdict-inverted(tests,control)     claimed_exit=-  rerun_exit=101,1(ARM-1-FAILED)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut724-M1-verdict-inverted-control.log  sha256=735de08b880373a0
  cmd=mutant-M2-not-measured-reads-skip(tests,control)  claimed_exit=-  rerun_exit=101,1(ARM-12-FAILED)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut724-M2-not-measured-reads-skip-control.log  sha256=f53c354fdf167faa
  cmd=mutant-M3-tail-clause-dropped(tests,control)  claimed_exit=-  rerun_exit=101,1(ARM-4-FAILED)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut724-M3-tail-clause-dropped-control.log  sha256=55a187aebf8b81e6
  cmd=mutant-M4-milestone-state-ignored(tests,control)  claimed_exit=-  rerun_exit=101,1(ARM-9-FAILED)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut724-M4-milestone-state-ignored-control.log  sha256=a12a790697aaa278
  cmd=mutant-M5-capital-V-dropped(tests,control)    claimed_exit=-  rerun_exit=101,1(ARM-11-FAILED; run 1: 0,0 SURVIVED)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut724-M5-capital-V-dropped-control.log  sha256=7b96b01d10595b67
  cmd=flaky-quality-proxy-test-3-isolated-reruns    claimed_exit=-  rerun_exit=0,0,0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/flaky724.log  sha256=fd4cd3ed56b43694
  cmd=pmat-verify@92eafba2d(overlapped-the-lanes)    claimed_exit=-  rerun_exit=1(10 no-manifest tests: /tmp/Cargo.toml)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify.json  sha256=7a8a63845da8adf8
  cmd=pmat-verify@ed3549c1e(set_current_dir-race)    claimed_exit=-  rerun_exit=1(1 test: quality_proxy property, PMAT-726)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify2.json  sha256=ecc603559aff90c0
  cmd=pmat-verify@3039b77ba(stale-unrun-ledger)      claimed_exit=-  rerun_exit=1(1 test: the_committed_ledger_matches_the_tree)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify3.json  sha256=f21e74e6c40799ae
  cmd=pmat-verify@2530ce8a8                      claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify4.json  sha256=bdad35fefec28e59

`claimed_exit=-` marks a row with no second party. `log_path` is session scratch; the `sha256` prefix pins the bytes.

## Status blocks

[status] ticket=PMAT-724 phase=1/5 global=204/5(K=96) k_measured=245 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L21
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=PMAT-724 blocker=- next=RED committed fe1f24106 (25 of 26 fail); dispatch the goal lane (goal.sh set refused: one ticket per session, operator reaffirmed)
[status] ticket=PMAT-724 phase=2/5 global=209/5(K=96) k_measured=245 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L21
         mode=quorum:goal trigger=R-4-single-module-width-1 route=agy-goal w=1.00 basis=absent q=? gate=FAIL slots=1/3 denied=0
         red=lane-isolation-exit-3 filed=- blocker=- next=verify the escaped commit myself, review it, fix the ratchet and the lost doctrine
[status] ticket=PMAT-724 phase=3/5 global=216/5(K=96) k_measured=245 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L21
         mode=direct trigger=- route=agy-goal w=1.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=PMAT-725 blocker=- next=control 15 arms in the traceability job, README 161, ledgers on a clean tree, quorum
[status] ticket=PMAT-724 phase=4/5 global=228/5(K=96) k_measured=245 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L21
         mode=quorum:quorum trigger=Phase-4-pre-PR-review route=agy-quorum w=1.00 basis=absent q=? gate=FAIL slots=1/3 denied=0
         red=quorum-3/3-FAIL filed=- blocker=- next=adjudicate 9 claims; 6 fixed with tests and arms, 2 refuted; control to 17 arms; lane debris removed
[status] ticket=PMAT-724 phase=5/5 global=245/5(K=96) k_measured=245 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L21
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=mutant-M5-survived-then-killed filed=PMAT-725,PMAT-726 blocker=- next=mutants M1-M5 re-run on the final code, verify ok:true at 2530ce8a8 (run 4), receipt, PR stacked on #1249

Finding: `global=k` and `k_measured` are the transcript-wide count, which includes PMAT-719's 98, PMAT-720's 48 and PMAT-722's 46 turns (Identity); the first four blocks are reconstructed after the fact with the count of the turn they describe. `q=?` because `quota.json` is absent.

## Verdict

**DONE** for the scope doctrine 6 allows today — every acceptance criterion re-run green by the orchestrator: CB-2112 fails on the §7 falsifier (null one `github_issue`), on a closed or absent issue and on a number that is not the tail, and passes a bijection numbered by tail (arms 1–5, mutants M1/M3); CB-2114 fails on the §7 falsifier (remove one `release:`), on a `v`/`V` prefix, on a missing or closed milestone and on an issue elsewhere, and passes a bound item (arms 6–11, mutants M4/M5); both Skip on a structural absence and fail `not_measured` on a missing input, a deleted input or a committed snapshot path (arms 12–17, mutant M2), and say the scope is §4.2's; the predicates are pure functions over the shared snapshot with the mutant each test dies under named on it; the control runs in the traceability job before the CB-2113 step; the ledger says NEUTERED and the reason is here and beside the withheld CB-2115 step. The gate step itself is **PMAT-725**, blocked on PMAT-723's fixers and on a human — not this row's to force.

IMPL-PMAT-724-RECEIPT-END
