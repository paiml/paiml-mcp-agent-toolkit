# IMPL-PMAT-1303 — goal-mode.md's own review (§13 item 6): five lanes, five FAILs; the record is refused, the design findings are escalated, and control arm 20 now counts recorded reviews

## Identity

| field | value |
|---|---|
| ticket | PMAT-1303 (`kind:code`), filed issue-first as #1303 once #1302 had merged and master's traceability job had passed; `kind-gate.sh`: `kind=code ticket=PMAT-1303 files=2` |
| spec | `docs/specifications/goal-mode.md` §13 item 6 ("this specification carries its own `epic:` and its own review artifact"), §6 (E.1) |
| branch | `PMAT-1303-goal-mode-self-review`, from master `f5eccdfb3` (the #1302 merge) |
| `gate_cmd` | `cargo test --workspace`, **`gate_cmd_fallback=true`**. No Rust changed; the control that CI runs was run instead |
| model gate | `model=opus class=opus decision=admit basis=file` |
| `goal.sh set` | refused: `goal: one ticket per session: PMAT-719 was set here — start a new claude session` |

### Delegation, quoted verbatim

"pmat-implement docs/specifications/goal-mode.md autonomously (you merge, triage tickets, pull requests, and label, tag, close/open)". R-5 refuses any ticket after PMAT-719 in this session; that instruction is the reaffirmation.

## What happened

- **The review ran.**
  - Five agy lanes reviewed goal-mode.md, one per role: quality, architecture, security, crux and adversarial.
  - They read a read-only plain copy of the repository at `eab301fa7`. Its goal-mode.md is byte-identical to master `f5eccdfb3`'s (sha256 `e3f59660…`).
  - The copy's fingerprint (`e26c9beef72016cc`) and the main checkout's status never moved, so no lane escaped.
- **Every lane returned FAIL, so nothing was recorded.** `pmat spec review --record` would refuse the artifact as LANE-NOT-PASS, so it was not invoked.
- **The findings were graded twice.**
  - The delegate checked each one against the spec and the code.
  - The orchestrator re-read each cited line.
  - Six hold on the text, and §8 ("Honest limits") names none of them. Three are partial, one is a one-word fix, and ten are staleness since the spec's baseline.
- **Control arm 20 now counts recorded reviews** (`a097390cc`).
  - Until now it failed as soon as any review artifact existed, so the first recorded review would have turned the traceability job red.
  - It now copies the tree's artifacts in, refuses a stale one, and expects NO-REVIEW for exactly the active specs still unreviewed.
  - It passes 27 of 27 arms: 44 active, 0 reviewed, 44 NO-REVIEW.

## Escalation (doctrine 6)

These are design decisions about goal-mode.md itself, and some of them touch gates already enforced. They are the operator's to make, and they are posted on #1303:

| # | the defect (goal-mode.md line) | recommendation |
|---|---|---|
| D1 | §10.3 derives the version at the cut (:532), but §4.1 makes a pre-titled semver milestone the key. The text does not say what happens when a breaking ticket lands in milestone 3.41.0. | Choose one: derive at the cut and retitle the milestone, or fix the title up front and refuse a mismatched bump. |
| D2 | CB-2113 (:389) requires every commit in `v<latest>..HEAD` to name a **non-terminal** item. Once tickets are completed, their own commits fail it. The code judges `merge-base..HEAD` and reports not_applicable on master. | Align §7 with the code. The non-terminal leg covers the PR's own commits; on master, a real item of any status. |
| D3 | CB-2113 has no release leg, so a later release's ticket can merge to master. The next tag then fails CB-2116(a) (:392), and the loop wedges. | Add a PR-time leg: the ticket's release is the current open milestone or an earlier one. Settle D1 first. |
| D4 | EPIC-OPEN (§3.1, :150) says the epic's body names the spec path. CB-2110 (:386) and its code check only sub-issues. | Align §3.1 with §4.3's sub-issue membership, or add the body check to CB-2110. |
| D5 | CB-2115's grace window is `now − max(item.updated, issue.updated_at)` (:304). A timestamp bump restarts it without resolving anything. **CB-2115 is enforced.** | Measure from when the disagreement was first seen. That needs a first-seen record in `pmat work sync`. |
| D6 | CB-2116(b) fails at `count > max_untagged_commits`, and §10.3's volume trigger fires on the same number (60, :510). A merge that crosses it turns master red before any cut, and `pmat goal` stops on red. | Give CI a margin over the trigger, or order the cut before the crossing merge. |

Three more are partial, and also yours:

| # | the defect | recommendation |
|---|---|---|
| D7 | §3.3 marks the GitHub-side rules not_applicable on fork PRs because a fork "cannot change this repository's issues". But a fork **can** edit `roadmap.yaml`, and master turns red after the merge. | Fail closed when a fork PR touches `roadmap.yaml` or `docs/specifications`. |
| D8 | `status: historical` or `superseded` exempts a spec from CB-2110 and CB-2111. The exemption is visible, but nothing bounds who may flip it. | Put CODEOWNERS on `docs/specifications`, or make a flip need its own review. |
| D9 | CB-2116(a) says "every commit", while CB-2113 and §8.4 say every **non-merge** commit. A literal CB-2116 fails every merge commit. | This is a one-word fix and needs no decision. It lands with the revision. |

**Staleness.** This is factual, not a decision. §14 and the counts in §1.2, §5.4 and §8.10 still describe the baseline: 157 rules with 0 enforced, 13 collisions, and no `merge_group`. Today there are 163 rules with 2 enforced, 0 collisions, and a `merge_group`. The `ci.yml` line references in §7.1 have also moved. All of this is re-measured in the same revision.

## Lifecycle

- **PMAT-1303 moves to `inprogress` and stays open.** Its acceptance criteria require every lane to PASS on a revised goal-mode.md.
- **The escalation is posted on #1303.**
- **PMAT-1300, the CB-2111 flip, now also waits on that revision,** because goal-mode.md is one of the 44 active specs.

## Jidoka log

| # | defect | owner | whys |
|---|---|---|---|
| 1 | the dispatch waited for #1302's merge | process | a new issue while #1302's CI ran would have turned its traceability job red, because CB-2115 reads GitHub live |
| 2 | lane 1 ran the binary against the read-only copy (`comply ledger -p`, `comply check -p`), which its prompt forbade | the lane | nothing changed, as the fingerprint shows; recorded |
| 3 | the delegate's first receipt shifted each role by one (a zsh 1-indexed array), and it still linted green | the delegate | rebuilt under bash, with each role checked against its prompt; the archived file is the corrected one |

| 4 | ci / test failed once on #1304: a_crate_that_does_not_compile_is_reported_as_not_measured reported a measured zero for a crate that cannot compile | pre-existing | #1304 changes no Rust and master passed the same test on identical code; filed as PMAT-1305 with what was ruled out, not treated as fixed by a re-run |

## Estimates

| field | value |
|---|---|
| `K̂` | 49, `basis=docs/audits/impl-estimates.jsonl:L19-L29` (the mean of those rows' actuals) |
| actual | **5** turns at this receipt: `k_measured` 606 minus 601 at the first transcript line naming the branch |

## Gaps

| gap | artifact that closes it |
|---|---|
| the review half of §13 item 6 | a revision of goal-mode.md after D1 to D8, re-reviewed by the same five roles |
| the `epic:` half of §13 item 6 | PMAT-729 |
| the lanes' coverage: lane 5 addressed 4 of the 8 rules; lane 3 named neither GH_TOKEN nor the withheld steps; lane 2 did not address §11's order | the re-review brief names each of them |

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 601

routes:
  ph1  class=review         route=agy-quorum w=1.00 basis=absent effort=1[U]   note=five-role spec review, width 5, a read-only copy, no escape
  ph2  class=impl           route=self  w=100.00  basis=absent   note=control arm 20
  ph3  class=orchestration  route=self  w=100.00  basis=absent

verification:
  cmd=spec-review-control(27-arms,arm-20-reworked)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/qgoal/control-arm20.log  sha256=6d464610bfd7d445
  cmd=spec-review-quorum(width-5)  claimed_exit=-  rerun_exit=1(not-agreed,5-of-5-FAIL)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/delegate-ph1/receipt.json  sha256=ad9c137af79de3aa
  cmd=lane-1-quality  claimed_exit=-  rerun_exit=0(FAIL-verdict)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1303/39a15c63-fab3-4148-8cb7-0cb012369b35/ph1/lane-1.json  sha256=430cb8a128855558
  cmd=lane-2-architecture  claimed_exit=-  rerun_exit=0(FAIL-verdict)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1303/39a15c63-fab3-4148-8cb7-0cb012369b35/ph1/lane-2.json  sha256=0a6919660d267a0b
  cmd=lane-3-security  claimed_exit=-  rerun_exit=0(FAIL-verdict)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1303/39a15c63-fab3-4148-8cb7-0cb012369b35/ph1/lane-3.json  sha256=cde1f88197343af5
  cmd=lane-4-crux  claimed_exit=-  rerun_exit=0(FAIL-verdict)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1303/39a15c63-fab3-4148-8cb7-0cb012369b35/ph1/lane-4.json  sha256=f6e627c840c81fd7
  cmd=lane-5-adversarial  claimed_exit=-  rerun_exit=0(FAIL-verdict)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1303/39a15c63-fab3-4148-8cb7-0cb012369b35/ph1/lane-5.json  sha256=51e0fb1c3df8f015
  cmd=lane-reduce  claimed_exit=-  rerun_exit=1(not-agreed)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1303/39a15c63-fab3-4148-8cb7-0cb012369b35/ph1/reduce.json  sha256=6491ec213018ad58
  cmd=kind-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/qgoal/kind-gate.log  sha256=9b4c1ed66b38d294
  cmd=model-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/qgoal/model-gate.log  sha256=44878c7c890a08c9
  cmd=goal.sh-set  claimed_exit=-  rerun_exit=refused(R-5)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/qgoal/goal-set.log  sha256=7d23194d9b0b8fac
  cmd=transcript-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/qgoal/transcript-gate.log  sha256=00af1ea715823d73

`claimed_exit=-` marks a row with no second party. `log_path` is session scratch; the `sha256` prefix pins the bytes.

## Status blocks

Finding — `k_measured` (606) is transcript-wide; `k` (5) counts from the first transcript line naming this branch (601).

[status] ticket=PMAT-1303 phase=1/3 global=5/49(K=98) k_measured=606 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L29
         mode=quorum trigger=Q2 route=agy-quorum w=1.00 basis=absent q=? gate=FAIL slots=0/3 denied=0
         red=review filed=PMAT-1305 blocker=operator:D1-D8 next=escalate

## Verdict

**PARTIAL(escalate).** goal-mode.md fails its own review, five lanes of five. Six findings hold on its text, and §8 names none of them. They are design decisions, and two of them touch gates already enforced: CB-2113 and CB-2115. So the artifact is not recorded, and the decisions go to the operator.

What lands:
- control arm 20, which counts recorded reviews, as the first recorded review will need
- the review record
- this escalation
