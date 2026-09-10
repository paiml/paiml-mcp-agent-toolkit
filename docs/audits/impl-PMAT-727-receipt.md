# IMPL-PMAT-727 — one predicate, two callers: with a roadmap the commit-msg hook applies CB-2113's rule, and CB-2113 names the remedy for the violation it found

## Identity

| field | value |
|---|---|
| ticket | PMAT-727 (`kind:code`), issue #1289 (minted by PMAT-1253's `yaml-to-github`); `kind-gate.sh`: `kind=code ticket=PMAT-727 files=8` |
| spec | `docs/specifications/goal-mode.md` doctrine 5 (two answers to one question is a defect), §9 ("the `commit-msg` hook only exists if installed… CI is the backstop") |
| branch | `PMAT-727-hook-trailer-predicate`, from `6df22b7f0`, rebased onto `3f3602401` |
| `discover.json` sha256 | `1e7f5b8fc09706d676cc1f7059ddffa7f6153c72e85d7ddda4e150c412e3b13a` |
| `gate_cmd` | `cargo test --workspace`, **`gate_cmd_fallback=true`**; `pmat verify` is the gate this repository names, and was run |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| model gate | `model=opus class=opus decision=admit basis=file` |

### Delegation, quoted verbatim

"pmat-implement docs/specifications/goal-mode.md autonomously (you merge, triage tickets, pull requests, and label, tag, close/open)". R-5 refuses any ticket after PMAT-719 in this session; that instruction is the reaffirmation.

## What was wrong

The hook and CB-2113 read trailers with the same parser: git's own, which reads the LAST paragraph only (`git interpret-trailers --parse` in the hook, `%(trailers:key=Pmat-Ticket)` in the rule). They disagreed on two things:

- **The hook's fallback.** It accepted any message whose body matched `PMAT-[0-9]+|#[0-9]+`. #1250's eleven commits carried `Pmat-Ticket:` in the paragraph above their Co-Authored-By block: the hook accepted them because the subject named the ticket, and CB-2113 refused them in CI.
- **The item behind the id.** CB-2113 refuses a trailer that names an id the roadmap lacks, or a completed or cancelled item. The hook never looked. #1252 went red on a trailer naming the ticket its own commits had marked completed, and then read CB-2113's one remedy for every violation: "add `Pmat-Ticket: <id>` as a git trailer", which was wrong advice for a trailer that was present.

## The choice criterion 1 asks the receipt to name

**The fallback is removed wherever a roadmap exists; CB-2113 is not widened.** Widening the rule to accept a reference outside the trailer block would bring back the ambiguity git's trailer parser removes: a subject or body line naming a ticket is prose, and a trailer is a structural field. Without a roadmap, CB-2113 does not judge (it reports Skip), so there the hook keeps the configured pattern as the repository's own convention. That is the one intended difference, and a test pins it.

**"One predicate, two callers", concretely.** The hook is shell and cannot call the rule's Rust. The predicate is defined once, as CB-2113's, and implemented in two places. What keeps the two equal is a test, not a promise: `the_hook_and_cb2113_give_the_same_verdict_on_the_same_message` runs the real generated hook and the real rule, `measure_against`, on the same five messages, each in a fresh fixture repository, and fails the build on any disagreement.

## RED, then GREEN

| step | commit | result |
|---|---|---|
| RED: the agreement test | `cf5a6c013` | at `6df22b7f0`: 9 passed, **1 failed** — "the trailer sits in the paragraph above Co-Authored-By (#1250): the hook accepts while CB-2113 refuses" |
| RED-2: the remedy tests | `39d18d04c` | 16 passed, **3 failed** — the two remedy tests and the agreement test |
| GREEN | `8b20d741f` | **152 passed, 0 failed**, 7 ignored, across the hook, rule and engine tests |
| ledger | `df867892b` | unrun-tests ledger re-rendered for the four new lib tests |

## Mutation table (each planted on the committed GREEN tree, restored from HEAD after)

| mutant | killed by |
|---|---|
| M1 the pattern fallback restored where a roadmap exists | `the_hook_and_cb2113_give_the_same_verdict_on_the_same_message` |
| M2 the hook's completed/cancelled check dropped | `the_hook_and_cb2113_give_the_same_verdict_on_the_same_message` |
| M3 the hook's unknown-id check dropped | `the_hook_and_cb2113_give_the_same_verdict_on_the_same_message` |
| M4 the whole message read as trailers (no last-paragraph rule) | `the_hook_and_cb2113_give_the_same_verdict_on_the_same_message` |
| M5 a completed ticket told to add a trailer | `a_terminal_ticket_is_told_to_stay_open_not_to_add_a_trailer` |

All five compiled (`test result: FAILED. 18 passed; 1 failed` in each log).

## Verification (all re-run by the orchestrator)

- the targeted tests at GREEN: 152 passed, 0 failed
- `scripts/traceability-control.sh` with the rebuilt binary: 5/5 arms
- `pmat verify --format json` on the committed tree: **ok: true, 5/5 stages** (format, complexity, satd, clippy, tests — the tests stage 352 s) at `df867892b`; the rebase changed no Rust file, so it stands for the rebased tree

## Jidoka log

| # | defect | owner | whys |
|---|---|---|---|
| 1 | the RED commit was refused by the pre-commit hook: `clippy::type_complexity` on the case table | me | the tuple type needed a name; a `type AgreementCase` alias fixed it. The hook runs the lint `ci / lint` runs |
| 2 | the GREEN commit was refused: `clippy::items_after_test_module` | me | the hook template constant had been appended after the file's test module; it now sits above it, inserted before the attribute line so no doc comment moves |
| 3 | the mutant summary labelled every killed mutant "DOES NOT COMPILE" | me (the mutant script) | cargo prints `error: test failed, to rerun pass --lib` when tests fail, and the detector grepped `^error:`; every mutant was re-read by its `test result` line, and each compiled |

## Estimates

| field | value |
|---|---|
| `K̂` | 47, `basis=docs/audits/impl-estimates.jsonl:L19-L26` |
| actual | **36** turns at this receipt |

## Lifecycle

This PR also marks PMAT-723 completed and closes #1286 in the same step, because CB-2115 is now a gate and an item and its issue turn terminal together. It records #1295's green run in PMAT-723's receipt. PMAT-727 itself stays open until the next ticket's PR.

## Gaps

| gap | artifact that closes it |
|---|---|
| the installed hook in an existing checkout keeps the old text until `pmat hooks install` runs again from a pmat carrying this change | the next release, then `pmat hooks install --strict --force` |
| the predicate is implemented twice (shell and Rust) | the agreement test fails the build when they diverge; a hook that calls pmat would make it one implementation, at a process spawn per commit |
| no pre-PR quorum on the diff | the five mutants and the agreement test are the discrimination |

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 447

routes:
  ph1  class=impl           route=self  w=100.00  basis=absent   note=agy-goal not taken: the main checkout was carrying PMAT-723's worktree operations, and four earlier writes lanes escaped their worktrees
  ph2  class=orchestration  route=self  w=100.00  basis=absent

verification:
  cmd=cargo-test-lib-RED(agreement)@6df22b7f0  claimed_exit=-  rerun_exit=101(1-failed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/red-727.log  sha256=fdd45a996f9fd2a0
  cmd=cargo-test-lib-RED-2(remedy)@cf5a6c013  claimed_exit=-  rerun_exit=101(3-failed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/red2-727.log  sha256=8170f91896f5f4ce
  cmd=cargo-test-lib-GREEN(152)@8b20d741f  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/green-727.log  sha256=fd3cec964061dbc8
  cmd=traceability-control(5-arms,rebuilt-binary)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/trace-control-727.log  sha256=3c77619a401a112b
  cmd=pmat-verify@df867892b  claimed_exit=-  rerun_exit=0(ok:true,5/5)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify727.json  sha256=b85e4f6ba776f38a
  cmd=mutant-M1-pattern-fallback-restored  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut727/M1-pattern-fallback-restored.log  sha256=929638e6df3c887f
  cmd=mutant-M2-terminal-check-dropped  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut727/M2-terminal-check-dropped.log  sha256=a169cfc6fe283d84
  cmd=mutant-M3-unknown-id-check-dropped  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut727/M3-unknown-id-check-dropped.log  sha256=b4055ec5e2b05045
  cmd=mutant-M4-whole-message-read-as-trailers  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut727/M4-whole-message-read-as-trailers.log  sha256=58e1ffc31afe1c9e
  cmd=mutant-M5-terminal-told-to-add-a-trailer  claimed_exit=-  rerun_exit=101(killed)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut727/M5-terminal-told-to-add-a-trailer.log  sha256=9dfe393e87b06604
  cmd=transcript-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/transcript-gate-727.log  sha256=1ace288d7e1f246b
  cmd=kind-gate(base-origin/master)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/kind-gate-727.log  sha256=a0e5ab6bd7cb15df
  cmd=model-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/model-gate-727.log  sha256=44878c7c890a08c9

`claimed_exit=-` marks a row with no second party. `log_path` is session scratch; the `sha256` prefix pins the bytes.

## Status blocks

Finding — `k_measured` (483) is transcript-wide; `k` (36) counts from the first transcript line naming this branch (447), and includes PMAT-723's turns, which ran interleaved while CI was queued.

[status] ticket=PMAT-727 phase=2/2 global=36/47(K=94) k_measured=483 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L26
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=- blocker=- next=CI, merge

## Verdict

**DONE.** With a roadmap, the hook and CB-2113 give one verdict on one message. Five cases are asserted against the real hook and the real rule. Five mutants each die under a named test, and `pmat verify` is green. The remedy CB-2113 prints now matches the violation it found.
