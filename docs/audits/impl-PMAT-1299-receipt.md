# IMPL-PMAT-1299 — CB-2111: every active spec's review is judged offline from a file, `pmat spec review --record` validates before it stages, and the direct step waits behind a 27-arm control

## Identity

| field | value |
|---|---|
| ticket | PMAT-1299 (`kind:code`), filed issue-first from #1299 by PMAT-1296 inside #1298, so its id ends in its issue number; `kind-gate.sh`: `kind=code ticket=PMAT-1299 files=22` |
| spec | `docs/specifications/goal-mode.md` §6 (E.1: §6.1 the artifact, §6.2 what CB-2111 checks, §6.3 who produces it), §7 row CB-2111 (the falsifier: append one space), §11 step 7, §11.1 |
| branch | `PMAT-1299-cb2111-spec-review`, worktree `/mnt/nvme-raid0/agent-wt/pmat-1296` (own target dir), stacked on #1298 and rebased onto its merge `f8a4f6fbe` with the tree unchanged |
| `discover.json` sha256 | `9d4a8021cd02980eace4f0e21e0c8f9d4d4ea33962b256f63a2b35dbcf1feaac` |
| `gate_cmd` | `cargo test --workspace`, **`gate_cmd_fallback=true`**; `pmat verify` was run |
| model gate | `model=opus class=opus decision=admit basis=file` |

### Delegation, quoted verbatim

"pmat-implement docs/specifications/goal-mode.md autonomously (you merge, triage tickets, pull requests, and label, tag, close/open)". R-5 refuses any ticket after PMAT-719 in this session; that instruction is the reaffirmation.

## What was missing

Invariant E.1 had no rule. A spec could be worked with no review at all (goal-mode.md §3, T1). Nothing recorded whether a review had happened, of which version of the spec, or by which roles. `pmat spec review --record` did not parse, and §11.1 said so.

## The change

- **`src/services/spec_review/`** holds several pieces:
  - The artifact (§6.1). Every field is required except `plan`, whose absence is NO-PLAN.
  - The closed role set, and the required roles: the five, plus `vendor:<name>` for each vendor the front-matter names. A vendor name needs only to be non-blank.
  - The slug: `components/cli-api.md` becomes `components-cli-api`.
  - `judge`, which returns these finding classes: NO-REVIEW, BAD-REVIEW, SPEC-MISMATCH, STALE-REVIEW (against the spec's sha256 now), NO-PLAN (a plan that is absent, or whose sha256 is not 64 hex digits), UNKNOWN-ROLE, EXTRA-LANE, DUPLICATE-LANE, MISSING-ROLE, LANE-NOT-PASS (anything but the exact word PASS), NOT-AGREED and PARTIAL.
- **CB-2111 "Spec Reviews"** has its own registry group, `spec-reviews`, and a config default. It walks the specs once, in path order:
  - A front-matter that does not parse is UNJUDGEABLE, because its vendor roles cannot be read. §4.3 exempts a spec from the epic leg, "not its parse leg".
  - Two active specs that share one artifact path are SLUG-COLLISION, and each finding names the other spec.
  - Historical and superseded specs are exempt, and every verdict names them.
  - A `docs/specifications` that was committed and then deleted is not_measured (§12).
- **`pmat spec review --record <json> [--path <dir>]`** judges the review exactly as CB-2111 will. It never produces a review (§6.3). It refuses before writing anything when it meets any of these:
  - an unreadable file, or BAD-REVIEW
  - a spec path outside `docs/specifications`: an empty, `.` or `..` segment, a control character, or a name not ending `.md`
  - an unreadable spec, UNJUDGEABLE, or any CB-2111 finding
  - an artifact path git ignores, or no git work tree
  - a symlink anywhere on the way to the artifact

  It then writes a new file beside the artifact and renames it into place, byte for byte, so a hard link there is replaced and never written through. Last it runs `git add`, and a failed stage says the file is written but not staged. The command is documented under `### spec`, and the flag-efficacy sweep denies it because it writes and stages.
- **`scripts/spec-review-control.sh`** has 27 arms and runs in the traceability job. The direct step is **withheld** under doctrine 6, because this tree has 44 active specs and no review artifact. Arm 20 asserts that on THIS tree, so the step cannot be flipped unnoticed. The flip is PMAT-1300 (#1300).
- **goal-mode.md** changes in two places:
  - §11.1 no longer lists `pmat spec review --record` as unparsed.
  - §6.1 and §6.2 now say what the two quorums settled: one lane per required role and no other; a plan sha256 of 64 hex digits; `agreed: false` is red.
- **The ledgers are re-rendered:** the enforcement ledger (163 rules, with CB-2111 NEUTERED, which is the truth until the flip), unrun-tests, and orphan-files.

## RED, then GREEN

| step | commit | result |
|---|---|---|
| RED: the service | `e7cd28c80` | 1 passed (the complete-review guard), **12 failed**, each at its assertion; 0 compile errors |
| GREEN: the service | `9500cc7c8` | **13 passed**, 0 failed |
| RED: the rule and `--record` | `af90b6482` | 14 passed, **16 failed**: exactly the 16 new tests, each at an assertion line |
| GREEN: the rule and `--record` | `21e104f23` | **30 passed**, 0 failed |
| RED: the first quorum's findings | `61877e37a` | 32 passed, **7 failed**: exactly the seven new behaviour tests; three guards pass |
| GREEN: the first quorum's findings | `5ff7b15cb` | **40 passed**, 0 failed |
| RED: the re-run's findings | `538c7f098` | 41 passed, **3 failed**: exactly the three new behaviour tests; three guards pass |
| GREEN: the re-run's findings | `793638b82` | **45 passed**, 0 failed |

Each RED was committed only after its script asserted that exactly the new tests failed, each at an assertion line. Each GREEN was committed only after every targeted test passed. The pre-commit hook (format, complexity, clippy) ran on every commit, and no commit used `--no-verify`.

## Two quorums

### First run

Three agy lanes ran in plan mode on a frozen detached worktree at the pre-quorum commit. The adversarial lane returned FAIL, the conformance lane do-not-implement-as-written, and the test-adequacy lane FAIL. The run is partial.

**The test-adequacy lane escaped.** Despite `--sandbox` and `writes=false`, it applied its own mutant to the shared review worktree with `patch -p0`. The other two lanes then read the mutated tree. The conformance lane's first ground, "`agreed` is omitted", was that mutant, not the commit. This is the fifth lane escape across these tickets.

Freezing a separate worktree contained it. The main checkout's status hash never moved, and the mutants' worktree was never the lanes' tree.

Every finding was reproduced with the orchestrator's own binary before it was accepted:

| reproduction, with the frozen binary, before the fix | observed |
|---|---|
| a symlink at `docs/audits/spec-test-review.json` pointing outside the project, then `--record` | exit 0; the outside file then held the review JSON |
| `vendors: [nvidia cuda]` with a `vendor:nvidia cuda` PASS lane | Fail: UNKNOWN-ROLE `vendor:nvidia cuda`; the spec could never pass |
| a duplicate quality lane, a `vendor:made-up` lane, and plan sha256 `not-a-hash` | Pass |
| lanes with no executor or summary, and no `agreed` key | Pass |
| `docs/audits` gitignored, then `--record` | exit 1 (not staged), with the artifact left on disk |

**Accepted and fixed:**

- writing through a symlinked artifact path
- a git-ignored path left written and unstaged
- a vendor named with a space
- extra and duplicate lanes
- a non-hex plan sha256
- missing §6.1 fields
- slug collisions
- the help text omitting `partial`
- untested exact-PASS, `.` segment and failed-stage behaviour

**Refuted, with the evidence:**

- UNJUDGEABLE is what §4.3 asks for ("not its parse leg").
- The withheld step is doctrine 6.
- A symlinked spec still needs its own artifact naming its own path.
- "`agreed` omitted" was the escaped lane's mutant.

### Re-run

Three lanes ran on the fixed commit. This time they read a **read-only plain copy**, exported with `git archive`. Its fingerprint was taken before, during and after the run, and it never changed. The conformance lane **passed**. The adversarial and test-adequacy lanes returned FAIL.

**Accepted and fixed:**

- **A hard link at the artifact path.** It was written through to a file outside the project, and this was reproduced. The fix writes a new file and renames it into place. That also closes the check-then-write window on the final component.
- **`agreed: false` with every lane PASS.** It passed; it is now NOT-AGREED.
- **A carriage return in a spec path.** It was accepted; it is now refused.
- **Six test gaps**, now closed: duplicate vendors, a failing duplicate lane, a hyphenated hash, a duplicated key, git's exclude file, and the other spec named in a collision.

**Refuted:**

- **Duplicate keys.** Duplicate *known* keys are already refused, and a test pins it. Unknown keys are ignored by design, because producers add metadata.
- **A missing `plan.sha256`.** It is NO-PLAN by design.
- **An empty `//` segment.** It was already tested.
- **The TOCTOU race.** It was not reproduced. The rename closes it on the final component, and git refuses a path beyond a symlinked directory before the walk reaches it.

Both receipts, lane by lane, are in `docs/audits/quorum-PMAT-1299.json`.

## Measured

- **The control**, run against this branch's binary with `bash scripts/spec-review-control.sh <cargo's executable>`: **27/27 arms**. Arm 20 on this tree: 44 specs, 44 active, every one NO-REVIEW.
- **Slug collisions on this tree:** 44 specs, 44 distinct artifact paths, none shared.
- **The adversarial reproductions**, run first with the pre-quorum binary and then with the final one:

| case | before | after |
|---|---|---|
| symlinked artifact path | record exit=0; the file OUTSIDE the project now reads: {"spec":"docs/specifications/test.md","spec_sha256":"3076fd8c5935e9a57 | record exit=1; the file OUTSIDE the project now reads: ORIGINAL |
| vendors: [nvidia cuda] with a vendor:nvidia cuda PASS lane | Fail — 1 finding(s) — UNKNOWN-ROLE 1: UNKNOWN-ROLE docs/specifications/v.md: `vendor:nvidia cuda` is not a review role (quality, architecture, securit | Pass — 1 spec(s) under docs/specifications; 1 active each carry a current review (docs/audits/spec-<slug>-review.json: the spec's sha256 now, a plan s |
| a duplicate quality lane + vendor:made-up + plan sha256 'not-a-hash' | Pass — 1 spec(s) under docs/specifications; 1 active each carry a current review (docs/audits/spec-<slug>-review.json: the spec's sha256 now, a plan,  | Fail — 3 finding(s) — NO-PLAN 1, DUPLICATE-LANE 1, EXTRA-LANE 1: NO-PLAN docs/specifications/d.md: the review carries no plan sha256 of 64 hex digits; |
| lanes without executor or summary, no agreed key | Pass — 1 spec(s) under docs/specifications; 1 active each carry a current review (docs/audits/spec-<slug>-review.json: the spec's sha256 now, a plan,  | Fail — 1 finding(s) — BAD-REVIEW 1: BAD-REVIEW docs/specifications/f.md: docs/audits/spec-f-review.json does not parse (missing field `executor` at li |
| gitignored docs/audits | record exit=1, artifact on disk: yes — hint: "git config advice.addIgnoredFile false") | record exit=1, artifact on disk: no — Error: docs/audits/spec-i-review.json cannot be staged (git ignores it, so it could never be committed); nothing |

- **The lib suite** on the frozen pre-quorum tree: 21,522 passed and 2 failed under a scratchpad TMPDIR.
  - Both failures pass with the default TMPDIR.
  - Their assertions depend on the path, which held `100` and `src`.
  - Neither file is in this diff, and both are filed as PMAT-1301 (#1301).

## Mutation table

| mutant | the tests (45) | the control (27 arms) |
|---|---|---|
| M1-stale-check-dropped | 3 failed: `appending_one_space_to_the_spec_stales_its_review`, `record_refuses_a_stale_review_and_writes_nothing` (+1 more) | ARM 2 |
| M2-partial-defaults-again | 2 failed: `a_misspelt_partial_key_is_refused_never_defaulted_to_complete`, `a_review_missing_a_section_6_1_field_is_bad_review` | ARM 5 |
| M3-closed-role-set-opened | 3 failed: `an_unrecognised_role_is_an_error_not_an_extra_lane`, `a_vendor_named_with_a_space_can_be_reviewed` (+1 more) | ARM 12 |
| M4-vendors-ignored | 6 failed: `a_front_matter_vendor_adds_a_required_role`, `a_front_matter_vendor_is_required_through_the_rule` (+4 more) | ARM 9 |
| M5-lane-verdict-ignored | 2 failed: `a_lane_that_is_not_pass_is_refused`, `a_lowercase_pass_is_not_pass` | ARM 10 |
| M6-partial-ignored | 1 failed: `partial_true_is_red` | ARM 11 |
| M7-plan-sha-format-dropped | 2 failed: `a_plan_sha256_that_is_not_64_hex_digits_is_no_plan`, `the_edges_the_rerun_named_are_pinned` | ARM 24 |
| M8-unjudgeable-dropped | 2 failed: `a_spec_whose_front_matter_does_not_parse_is_unjudgeable_not_skipped`, `findings_render_in_path_order_with_their_class_counts` | ARM 14 |
| M9-exempt-judged | 1 failed: `a_historical_spec_needs_no_review_and_is_named` | ARM 13 |
| M10-deleted-input-skips | 1 failed: `a_committed_then_deleted_specifications_directory_is_not_measured` | ARM 19 |
| M11-record-stages-unjudged | 2 failed: `record_reads_the_vendor_roles_from_the_specs_front_matter`, `record_refuses_a_stale_review_and_writes_nothing` | ARM 17 |
| M12-dotdot-accepted | 1 failed: `record_refuses_a_review_that_names_a_file_outside_docs_specifications` | ARM 17 |
| M13-not-byte-for-byte | 2 failed: `record_replaces_a_hard_linked_artifact_path_without_writing_through_it`, `record_writes_a_valid_review_to_its_artifact_path_and_stages_it` | ARM 17 |
| M14-never-staged | 3 failed: `record_in_place_stages_the_artifact`, `record_reports_a_failed_stage_and_says_the_file_is_written` (+1 more) | ARM 17 |
| M15-slug-keeps-md | 4 failed: `an_active_spec_with_no_review_is_refused`, `record_writes_a_valid_review_to_its_artifact_path_and_stages_it` (+2 more) | ARM 1 |
| M16-spec-mismatch-ignored | 1 failed: `a_review_of_another_spec_is_refused` | ARM 6 |
| M17-rule-drops-vendors | 1 failed: `a_front_matter_vendor_is_required_through_the_rule` | ARM 9 |
| M18-record-drops-vendors | 1 failed: `record_reads_the_vendor_roles_from_the_specs_front_matter` | ARM 17 |
| M19-agreed-optional-again | 1 failed: `a_review_missing_a_section_6_1_field_is_bad_review` | ARM 23 |
| M20-executor-optional-again | 1 failed: `a_review_missing_a_section_6_1_field_is_bad_review` | ARM 23 |
| M21-lanes-optional | 1 failed: `a_review_missing_a_section_6_1_field_is_bad_review` | ARM 23 |
| M22-pass-case-insensitive | 1 failed: `a_lowercase_pass_is_not_pass` | ARM 10 |
| M23-extra-lane-allowed | 1 failed: `an_extra_or_a_duplicate_lane_is_refused` | ARM 22 |
| M24-duplicate-lane-allowed | 2 failed: `an_extra_or_a_duplicate_lane_is_refused`, `the_edges_the_rerun_named_are_pinned` | ARM 21 |
| M25-vendor-space-refused-again | 1 failed: `a_vendor_named_with_a_space_can_be_reviewed` | ARM 25 |
| M26-symlink-check-dropped | 1 failed: `record_refuses_to_write_through_a_symlinked_artifact_path` | ARM 17 |
| M27-ignore-check-dropped | 2 failed: `record_refuses_an_artifact_path_git_ignores_and_writes_nothing`, `record_refuses_a_symlinked_audits_directory_and_an_excluded_path` | ARM 17 |
| M28-slug-collision-dropped | 1 failed: `two_active_specs_sharing_an_artifact_path_are_named_as_a_collision` | ARM 26 |
| M29-stage-result-ignored | 1 failed: `record_reports_a_failed_stage_and_says_the_file_is_written` | ARM 17 |
| M30-dot-segment-accepted | 1 failed: `record_refuses_a_review_that_names_a_file_outside_docs_specifications` | ARM 17 |
| M31-agreed-ignored | 1 failed: `a_review_that_records_agreed_false_is_red` | ARM 27 |
| M32-direct-write-through-links | 1 failed: `record_replaces_a_hard_linked_artifact_path_without_writing_through_it` | ARM 17 |
| M33-control-characters-accepted | 1 failed: `record_refuses_a_spec_path_with_a_control_character` | ARM 17 |
| M34-vendor-dedup-dropped | 1 failed: `the_edges_the_rerun_named_are_pinned` | not caught; tests only |
| M35-collision-names-no-one | 1 failed: `two_active_specs_sharing_an_artifact_path_are_named_as_a_collision` | ARM 26 |

Every mutant was planted on the committed code (`e7eba286a`) in one of two worktrees, each with its own target dir. It was compiled, judged by the 45 targeted tests, and judged again by the 27-arm control against a binary built from the mutated tree. Then its file was restored from HEAD. Each mutant's diff, test log and control log are pinned by sha256 below. **All 35 die under the tests.** The control misses 1 of them (M34-vendor-dedup-dropped). Each is a unit-level behaviour no arm drives, and each dies under the named test.

## Verification

`pmat verify --format json` ran on the final code (`e7eba286a`), in worktree B, with the default TMPDIR. Verdict **ok: null**: format pass; complexity not applicable; satd pass; clippy pass; tests pass. A null verdict is not a pass, so the reason is written down here. The complexity stage judges only Rust files changed against HEAD, and a committed tree has none. Complexity was measured where it applies: the pre-commit hook passed it on every commit's staged change. The default TMPDIR is deliberate, because two lib tests fail under a TMPDIR whose path holds `100` or `src` (PMAT-1301).

## Lifecycle

- **PMAT-1296 is marked completed and #1296 is closed** in the same step. The close waited until master `f8a4f6fbe`'s traceability job had passed (run 34550486355). CB-2115 reads live GitHub with zero tolerance, so closing earlier would have turned master red.
- **Two tickets are filed issue-first,** so their items reach master in this merge:
  - **PMAT-1300** (#1300), the CB-2111 flip
  - **PMAT-1301** (#1301), the two TMPDIR-sensitive lib tests
- **PMAT-1299 moves to `inprogress`.** The next ticket's PR completes it, because CB-2113 refuses a trailer naming an item that is terminal at the PR head.

## Jidoka log

| # | defect | owner | whys |
|---|---|---|---|
| 1 | the first GREEN patch's anchor predated rustfmt's re-wrap of the stub | me | the patch refused before writing anything; it was re-anchored to the formatted text |
| 2 | #1298 went red on its orphan-files ledger | me (PMAT-1296's finish) | a new include!d test file changed the ledger, and I never re-rendered it; it was re-rendered on #1298's branch and pushed, and went green |
| 3 | four of my own chain guards stopped on a green path | me | grep -c piped to grep -qx 0, and strings piped to grep -q, both under pipefail; a test-name regex without digits; an expected file list in unsorted order; and a variable unset under set -u. Each failed closed, and the lesson is recorded in memory |
| 4 | a GREEN anchor used 16 spaces where rustfmt wrote 12 | me | the patch refused; the half already applied was checked before resuming |
| 5 | the first quorum's test-adequacy lane applied its own mutant to the review worktree | the skill's lane harness | `--sandbox` and `writes=false` did not stop `patch -p0`. The frozen worktree contained it, and the re-run read a read-only plain copy whose fingerprint never moved |
| 6 | M18 did not compile in the first mutant run | me | an unused variable is a deny lint in this crate; it was re-planted as `&front.vendors[..0]` |
| 7 | two lib tests failed under the scratchpad TMPDIR | pre-existing tests | their assertions depend on the path; verify ran with the default TMPDIR, and the tests are filed as PMAT-1301 |

## Estimates

| field | value |
|---|---|
| `K̂` | 46, `basis=docs/audits/impl-estimates.jsonl:L19-L28`. `estimate.sh` read no rows for this repository (ROWS=0), so K̂ is the mean of those rows' actuals, taken by hand |
| actual | **83** turns at this receipt: `k_measured` 589 minus 506 at the first transcript line naming the branch. It includes the waits on #1298's CI and two quorum runs |

## Gaps

| gap | artifact that closes it |
|---|---|
| the direct CB-2111 step, which needs every active spec reviewed | PMAT-1300 (#1300), after PMAT-729 decides which specs stay active |
| CB-2115 on pushes to master: any issue opened or closed turns master's next CI red until a PR catches the roadmap up (zero tolerance, live read) | an operator decision, raised on #1287 and in PMAT-1296's receipt. This PR's own issue writes waited for master's traceability job |
| what CB-2111 buys is bounded (§6.2): an auditable lie somebody wrote down, not evidence that a review happened | by design |
| two lib tests depend on the TMPDIR path | PMAT-1301 (#1301) |
| the agy lane harness let a `writes=false`, `--sandbox` lane write into its repo_root | the paiml-implement bundle. Review lanes here now read a read-only plain copy |
| the symlink walk over intermediate components is defense in depth: git refuses a path beyond a symlinked directory first, so the "final component only" mutant is equivalent here | recorded |
| the re-run's conformance lane wrote two empty files under /tmp, outside its scratch dir | recorded; nothing in any checkout moved |

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 506

routes:
  ph1  class=impl           route=self  w=100.00  basis=absent   note=agy-goal not taken: four earlier writes lanes escaped their worktrees
  ph2  class=review         route=agy-quorum w=1.00 basis=absent effort=1[U]   note=the first quorum, width 3; lane 3 escaped
  ph3  class=review         route=agy-quorum w=1.00 basis=absent effort=1[U]   note=the re-run, width 3, a read-only copy, no escape
  ph4  class=orchestration  route=self  w=100.00  basis=absent

verification:
  cmd=cargo-test-lib-RED-service  claimed_exit=-  rerun_exit=101(12-failed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/red-1299.log  sha256=9a8e4224f5e75f69
  cmd=cargo-test-lib-GREEN-service(13)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/green-1299.log  sha256=07517d859ce867a1
  cmd=cargo-test-lib-RED-rule-and-record  claimed_exit=-  rerun_exit=101(16-failed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/red-1299b.log  sha256=a55b825b4a79ac34
  cmd=cargo-test-lib-GREEN-rule-and-record(30)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/green-1299d.log  sha256=85fbe0b54aa6b16f
  cmd=cargo-test-lib-RED-first-quorum  claimed_exit=-  rerun_exit=101(7-failed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/red2-1299.log  sha256=3ab78d930770fa0a
  cmd=cargo-test-lib-GREEN-first-quorum(40)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/green2-1299.log  sha256=9869013580bf3de8
  cmd=cargo-test-lib-RED-rerun  claimed_exit=-  rerun_exit=101(3-failed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/red3-1299.log  sha256=a8083c0d7cf19a71
  cmd=cargo-test-lib-GREEN-rerun(45)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/green3-1299.log  sha256=7bae95e451c80305
  cmd=spec-review-control(27-arms)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/control3-1299.log  sha256=a0280c8aacc2924b
  cmd=repro-before(pre-quorum-binary)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/repro-before.log  sha256=1e1093c619c52003
  cmd=repro-after(final-binary)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/repro-after.log  sha256=cf3ff39f665a9ab6
  cmd=pmat-verify  claimed_exit=-  rerun_exit=0(complexity-not-applicable-on-a-clean-tree)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify-final3.json  sha256=818f56afb08b61c4
  cmd=lib-suite-frozen-tree(TMPDIR=scratchpad)  claimed_exit=-  rerun_exit=101(2-failed,path-sensitive)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/libtest-1299.log  sha256=2563d2bd8a110e6d
  cmd=two-tests-default-TMPDIR  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/two-tests-default-tmp.log  sha256=b6822db18b97da39
  cmd=quorum-first-run(lane-reduce)  claimed_exit=-  rerun_exit=1(not-agreed)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1299/39a15c63-fab3-4148-8cb7-0cb012369b35/ph3/receipt.json  sha256=36ee0fd80c6e7c98
  cmd=quorum-rerun(lane-reduce)  claimed_exit=-  rerun_exit=1(not-agreed)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1299/39a15c63-fab3-4148-8cb7-0cb012369b35/ph3b/receipt.json  sha256=34d6f6883878aea4
  cmd=mutant-M1-stale-check-dropped  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M1-stale-check-dropped-tests.log  sha256=656d3bf0b1ed38bf
  cmd=mutant-M2-partial-defaults-again  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M2-partial-defaults-again-tests.log  sha256=e319c6431b1d593a
  cmd=mutant-M3-closed-role-set-opened  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M3-closed-role-set-opened-tests.log  sha256=57c550d4152dfebe
  cmd=mutant-M4-vendors-ignored  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M4-vendors-ignored-tests.log  sha256=2bbc7f979f0754b8
  cmd=mutant-M5-lane-verdict-ignored  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M5-lane-verdict-ignored-tests.log  sha256=b015abcb6fc0cdac
  cmd=mutant-M6-partial-ignored  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M6-partial-ignored-tests.log  sha256=eda27fcaa3d596bd
  cmd=mutant-M7-plan-sha-format-dropped  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M7-plan-sha-format-dropped-tests.log  sha256=3e4b4c7642da31a7
  cmd=mutant-M8-unjudgeable-dropped  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M8-unjudgeable-dropped-tests.log  sha256=7c46891d2bff679a
  cmd=mutant-M9-exempt-judged  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M9-exempt-judged-tests.log  sha256=41d55ffc54d28903
  cmd=mutant-M10-deleted-input-skips  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M10-deleted-input-skips-tests.log  sha256=0783e37b272b2064
  cmd=mutant-M11-record-stages-unjudged  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M11-record-stages-unjudged-tests.log  sha256=10d5f3832507df85
  cmd=mutant-M12-dotdot-accepted  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M12-dotdot-accepted-tests.log  sha256=e3bf4d51da0df85e
  cmd=mutant-M13-not-byte-for-byte  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M13-not-byte-for-byte-tests.log  sha256=3518b07ac72875ca
  cmd=mutant-M14-never-staged  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M14-never-staged-tests.log  sha256=57e5c80cec2cd79e
  cmd=mutant-M15-slug-keeps-md  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M15-slug-keeps-md-tests.log  sha256=68ac7b90fa20302c
  cmd=mutant-M16-spec-mismatch-ignored  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M16-spec-mismatch-ignored-tests.log  sha256=5ffa046db8d8e735
  cmd=mutant-M17-rule-drops-vendors  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M17-rule-drops-vendors-tests.log  sha256=88318c8da523337f
  cmd=mutant-M18-record-drops-vendors  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-a/M18-record-drops-vendors-tests.log  sha256=48212f1a366b99c3
  cmd=mutant-M19-agreed-optional-again  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M19-agreed-optional-again-tests.log  sha256=99ca0f56d69d994d
  cmd=mutant-M20-executor-optional-again  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M20-executor-optional-again-tests.log  sha256=b1f8c2c5423dbeb1
  cmd=mutant-M21-lanes-optional  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M21-lanes-optional-tests.log  sha256=3d97d80d1b32e9ca
  cmd=mutant-M22-pass-case-insensitive  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M22-pass-case-insensitive-tests.log  sha256=e4bea3ff57423471
  cmd=mutant-M23-extra-lane-allowed  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M23-extra-lane-allowed-tests.log  sha256=0c3be179c7759715
  cmd=mutant-M24-duplicate-lane-allowed  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M24-duplicate-lane-allowed-tests.log  sha256=6d16cbd4a9d3ea86
  cmd=mutant-M25-vendor-space-refused-again  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M25-vendor-space-refused-again-tests.log  sha256=5856803f9eae9a0a
  cmd=mutant-M26-symlink-check-dropped  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M26-symlink-check-dropped-tests.log  sha256=12ad9e5f0883544b
  cmd=mutant-M27-ignore-check-dropped  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M27-ignore-check-dropped-tests.log  sha256=bd086f51325104a4
  cmd=mutant-M28-slug-collision-dropped  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M28-slug-collision-dropped-tests.log  sha256=4603a0e20870833a
  cmd=mutant-M29-stage-result-ignored  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M29-stage-result-ignored-tests.log  sha256=75ef93fd9885db3a
  cmd=mutant-M30-dot-segment-accepted  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M30-dot-segment-accepted-tests.log  sha256=1c232a5dc3cd638a
  cmd=mutant-M31-agreed-ignored  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M31-agreed-ignored-tests.log  sha256=90fc468bda466997
  cmd=mutant-M32-direct-write-through-links  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M32-direct-write-through-links-tests.log  sha256=f67fc8e86655fe15
  cmd=mutant-M33-control-characters-accepted  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M33-control-characters-accepted-tests.log  sha256=3405370ef0ed03cc
  cmd=mutant-M34-vendor-dedup-dropped  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M34-vendor-dedup-dropped-tests.log  sha256=bf21b1699afa7e3f
  cmd=mutant-M35-collision-names-no-one  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mutfinal2-b/M35-collision-names-no-one-tests.log  sha256=101950b8c53a30de
  cmd=transcript-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/transcript-gate-1299.log  sha256=3210459d8520ad0f
  cmd=kind-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/kind-gate-1299.log  sha256=097c0b5e72ffcb6e
  cmd=model-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/model-gate-1299.log  sha256=44878c7c890a08c9

`claimed_exit=-` marks a row with no second party. `log_path` is session scratch; the `sha256` prefix pins the bytes.

## Status blocks

Finding — `k_measured` (589) is transcript-wide; `k` (83) counts from the first transcript line naming this branch (506).

[status] ticket=PMAT-1299 phase=4/4 global=83/46(K=92) k_measured=589 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L28
         mode=direct+quorum trigger=Q2 route=agy-quorum w=1.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=PMAT-1300,PMAT-1301 blocker=- next=CI, merge

## Verdict

**DONE.** CB-2111 judges every active spec's review offline, from a file. `pmat spec review --record` validates before it stages, and it never writes through a link.

Two quorums found holes the tests had not:
- a symlinked or hard-linked artifact path written through
- a vendor that could never pass
- extra and duplicate lanes
- a plan hash that was not one
- `agreed: false` passing

Each was reproduced, fixed under a RED test, and pinned by a control arm. All 35 mutants die under named tests. The direct step is withheld under doctrine 6 and filed as PMAT-1300.
