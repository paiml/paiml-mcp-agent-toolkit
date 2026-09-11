# IMPL-PMAT-1307 — sixteen open questions decided by quorum, five of my nine answers overturned, and the round I contaminated myself

## Identity

| field | value |
|---|---|
| ticket | PMAT-1307 (`kind:code`), filed issue-first as #1307 with #1308, #1309 and #1310 |
| spec | `docs/specifications/goal-mode.md` — the revision answering its own failed five-role review (PMAT-1303, #1303) |
| branch | `PMAT-1307-goal-mode-revision`, from master `5af9a0f0d`; merged as **`204eb8959`** (PR #1311) |
| `gate_cmd` | `cargo test --workspace`, **`gate_cmd_fallback=true`** — no Rust changed on this ticket; the roadmap round trip, `work validate` and the live `work sync --check-only` were run instead |
| model gate | `model=opus class=opus decision=admit basis=file` |
| completed in | PMAT-1313's PR — a ticket cannot complete itself under CB-2113 |

### Delegation, quoted verbatim

"pmat-implement docs/specifications/goal-mode.md autonomously (you merge, triage tickets, pull requests, and label, tag, close/open, use quorum for question answers vs stalling decision)"

An earlier message in the same session: "use quorum to decide ANY open question". Both are the authority for every decision below and for closing #1307.

## What happened

**Sixteen questions were decided by five-seat quorums, in three rounds. None was decided by me.**

Each seat read a read-only plain copy of the tree, chose from options it was given **without being told which option I had drafted**, and answered alone. Majority decides; every minority answer is kept verbatim. Artifact: `docs/audits/quorum-PMAT-1307.json`.

| round | questions | copy fingerprint (before → after) | outcome |
|---|---|---|---|
| 1 | D1–D10, P1–P4 | `f79fbf1ab2f3c801` → `f79fbf1ab2f3c801` | 11 clean majorities; 3 left undecided once a contaminated seat was discarded |
| 2 | D5, D6, D10 | `264a0240e6b296ad` → `264a0240e6b296ad` | all three decided |
| 3 | Q15, Q16 | `3b3a9cf88f0981a3` → `3b3a9cf88f0981a3` | both decided |

No fingerprint moved in any round: no lane escaped, which is the fifth quorum in this programme where that had to be checked.

**The quorum overturned five of my nine answers** — D1 (the cut refuses instead of renaming the milestone), D3 (the release leg reports before it fails), D5 (no first-seen state file), D6 (slack 10, not 20), D8 (no CODEOWNERS; the diff is the control) — and confirmed D2, D4, D7, D9, D10.

### Round 1 was contaminated by me

The read-only copy carried `docs/roadmaps/roadmap.yaml` at my own revision commit, and PMAT-1307's acceptance criteria there state the drafted answers as fact. **Seat 5 cited `roadmap.yaml:6217` as grounding for nine of its fourteen votes and voted the drafted option every time.**

- Detected by reading the seats' `grounding` fields, not by the delegate.
- That seat is discarded from every tally in the artifact.
- The roadmap items and `docs/audits/impl-PMAT-1303-receipt.md` (whose table is headed "recommendation") were stripped from the copy, and the three questions the discard left undecided were re-run on the clean copy as round 2.
- **Had that seat been counted, D6 would have landed on 20 instead of 10, and D1 and D5 would have kept my answers.**

The cause was mine: I built the copy from my own revision commit and swapped back only `goal-mode.md`. Evidence that says what you wanted it to say is not evidence.

## Verification — every claim re-run by the orchestrator

| what | command | result | log |
|---|---|---|---|
| roadmap round trip | `bash scripts/work-sync-control.sh <exe>` | 4/4 arms, byte-for-byte canonical | `log_path=scratchpad/rev/control3.log` |
| roadmap schema | `pmat work validate` | exit 0 — it caught a plain scalar with a colon before the commit, not after | `log_path=scratchpad/rev/validate2.log` |
| live bijection | `GH_TOKEN=… pmat work sync --check-only` | exit 0 — **98 items ↔ 98 issues, 0 tolerated** | `log_path=scratchpad/rev/check4.log` |
| CI on the merged head | 45 checks | all green, 0 failures | PR #1311 |

## The review that forced round 3, and the one still open

The five-role review of the revision (`ph-review`) returned **4 FAIL, 1 PASS**. Seven findings were reproduced against the text by the orchestrator before any was acted on:

1. the text called 60 "the median" while its own §14 ledger measures 59 — **fixed**
2. §11.2 claimed ten decisions were marked inline where four were — **fixed**
3. two references to a `§2.2` that does not exist — **fixed**
4. a `§8.10` reference off by one, a regression from my own fix to a duplicated list number — **fixed** (the quality lane caught my repair)
5. P1 against CB-2114's release leg — **Q15**, decided 5 of 5
6. `pmat.toml`'s thresholds are editable by any PR and §8 did not name it — **disclosed as §8 limit 12**
7. P4's master-push carve-out left open in CB-2110/2112/2114 — **Q16**, decided 4 of 5

The second review (`ph-review2`) returned **2 PASS, 3 FAIL**: quality now passes, security still passes. Three findings were reproduced and two more are real:

- **Q17** — CB-2113's PR leg admits a commit for an unscheduled `planned` ticket (which Q15 makes normal), but CB-2116(a) requires every commit in a tag's range to name a ticket **of that release**, so the cut can never succeed. Found independently by the crux and adversarial lanes, one of them measuring it against `scripts/traceability-control.sh`.
- **Q18** — §10.3 lists five release triggers, "first to fire cuts", and the cut's first step is "verify the drain". Triggers 2–5 fire precisely when the milestone still has open issues, so they are dead as written.
- **refuted** — the architecture lane held that the fork carve-out's justification contradicts Q16's master deferral. It does not: master still judges the file half a fork can edit, which is exactly what the carve-out protects.

One finding was fixed as text rather than decided: §5.2 said a fresh orphan is `not_measured` "rather than a finding", while §3.3 maps `not_measured` to **FAIL** — the opposite of the intent. It now says **tolerated**, the verdict the code already prints.

## Status: PARTIAL (escalate)

**Q17 and Q18 are undecided.** Their quorum was dispatched and interrupted by the operator, whose instruction was to stop re-reviewing and land. They are real, reproduced, and unowned — the honest place for them is a ticket, not a silent fix, and **neither is introduced by this revision**: both predate it.

PMAT-1303 keeps the review artifact. `docs/audits/spec-goal-mode-review.json` is **not** recorded: three lanes still FAIL, and `pmat spec review --record` would refuse it as LANE-NOT-PASS. That refusal is the gate working.

## Filed

| ticket | what |
|---|---|
| PMAT-1308 (#1308) | CB-2113's two legs, and D3's release leg that warns before it fails |
| PMAT-1309 (#1309) | CB-2115: grace over the ORPHAN and MISSING legs, and the rule off the master push. **Its original premise was refused by the quorum** (D5=B), and its title, criteria and notes are rewritten to what was actually decided |
| PMAT-1310 (#1310) | the fork carve-out |
| PMAT-1312 (#1312) | Q15 and Q16 — CB-2114 binds `inprogress` only; the live-state assertions defer on a master push |

`GH-1306` took the item for #1306, the operator's own issue, which had none: that orphan, with #1307–#1310, is what had reddened master `5af9a0f0d` at 06:08Z with no commit to blame.

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 601

routes:
  ph2  class=plan            route=agy-plan w=1.00 basis=absent effort=1[U]   note=decision quorum, width 5, fourteen questions, ONE SEAT CONTAMINATED by my own roadmap text and discarded
  ph3  class=plan            route=agy-plan w=1.00 basis=absent effort=1[U]   note=runoff, width 5, D5/D6/D10, decontaminated copy
  ph4  class=review          route=agy-quorum w=1.00 basis=absent effort=1[U] note=five-role review of the revision, width 5 — 4 FAIL 1 PASS
  ph5  class=plan            route=agy-plan w=1.00 basis=absent effort=1[U]   note=Q15/Q16 decision quorum, width 5
  ph6  class=review          route=agy-quorum w=1.00 basis=absent effort=1[U] note=second five-role review, width 5 — 3 FAIL 2 PASS
  ph7  class=orchestration   route=self  w=100.00  basis=absent               note=every finding reproduced against the text before it was acted on; merge, tickets, receipt

verification:
  cmd=work-sync-control(4-arms)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/rev/control3.log
  cmd=pmat-work-validate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/rev/validate2.log
  cmd=pmat-work-sync--check-only(live,98-items)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/rev/check4.log
  cmd=round1-quorum(width-5,14-questions)  claimed_exit=agreed  rerun_exit=1(seat-5-contaminated,discarded)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1307/39a15c63-fab3-4148-8cb7-0cb012369b35/ph-decide/reduce.json
  cmd=round2-runoff(width-5,D5-D6-D10)  claimed_exit=agreed  rerun_exit=0  log_path=/run/user/1000/paiml-implement/agy/PMAT-1307/39a15c63-fab3-4148-8cb7-0cb012369b35/ph-runoff/lane-1.json
  cmd=round3-quorum(width-5,Q15-Q16)  claimed_exit=agreed  rerun_exit=0(Q16-split-4-1)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1307/39a15c63-fab3-4148-8cb7-0cb012369b35/ph-q2/lane-1.json
  cmd=review-1(width-5,five-roles)  claimed_exit=-  rerun_exit=1(4-FAIL-1-PASS)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1307/39a15c63-fab3-4148-8cb7-0cb012369b35/ph-review/lane-1.json
  cmd=review-2(width-5,five-roles)  claimed_exit=-  rerun_exit=1(3-FAIL-2-PASS)  log_path=/run/user/1000/paiml-implement/agy/PMAT-1307/39a15c63-fab3-4148-8cb7-0cb012369b35/ph-review2/lane-1.json
  cmd=ro-copy-fingerprint(3-rounds,before-and-after)  claimed_exit=-  rerun_exit=0(unmoved,no-escape)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/rev/ro-fingerprint.txt
  cmd=spec-dead-path-grep(CLAUDE.md-prescribed)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/rev/control3.log
