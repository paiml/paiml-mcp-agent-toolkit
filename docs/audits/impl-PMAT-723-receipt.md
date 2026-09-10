# IMPL-PMAT-723 — goal-mode step 4b: CB-2115 becomes a gate — `--checks CB-2113,CB-2115` with GH_TOKEN in the traceability job, measured coherent on master first, observed RED on the §7 falsifier in CI, then reverted

## Identity

| field | value |
|---|---|
| ticket | PMAT-723 (`kind:code`), issue #1286 (minted by PMAT-1253's `yaml-to-github`); `kind-gate.sh`: `kind=code ticket=PMAT-723 files=3` |
| spec | `docs/specifications/goal-mode.md` §5.1, §5.4 step 3 ("only then does CB-2115 become a gate, and it lands green"), §7 row CB-2115 (falsifier: close one linked issue, leave the item open), §13 items 1 and 2 |
| branch | `PMAT-723-cb2115-gate`, rebuilt on master `7bd20ee95` (worktree `/mnt/nvme-raid0/agent-wt/pmat-723`) |
| HEAD in | `7bd20ee95` (the merge of #1294) |
| `discover.json` sha256 | `1e7f5b8fc09706d676cc1f7059ddffa7f6153c72e85d7ddda4e150c412e3b13a` |
| `gate_cmd` | `cargo test --workspace`, **`gate_cmd_fallback=true`**. The change is one workflow step, one re-rendered ledger and roadmap rows, so the gates are CI's own run of the step (RED, then the revert) and the ledger |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder`; the traceability job is in `gate`'s `needs` |
| model gate | `model=opus class=opus decision=admit basis=file` |

### Delegation, quoted verbatim

"pmat-implement docs/specifications/goal-mode.md autonomously (you merge, triage tickets, pull requests, and label, tag, close/open)". R-5 refuses any ticket after PMAT-719 in this session; that instruction is the reaffirmation.

## The acceptance criteria

1. **`pmat work sync --check-only` exits 0 on master, measured before the step is added.** On master `7bd20ee95` at 2026-09-10T20:10:47Z, live against master's exact roadmap (`git show origin/master:docs/roadmaps/roadmap.yaml` in a temporary directory): **exit 0, "coherent: R ↔ G is a bijection and no matched pair disagrees past the grace window"**. The first measurement, after #1291, was not zero: 15 findings, 2 transferred issues and 13 drift. The follow-up #1294 cleared them. The step was not added until this reading.
2. **The traceability job runs `pmat comply check --checks CB-2113,CB-2115` with GH_TOKEN after the controls, and `pmat comply ledger` writes CB-2115 as ENFORCED.** The step replaces the CB-2113-only step: one invocation, both rules, `env: GH_TOKEN: ${{ github.token }}` in block style. The ledger, re-rendered and **read back before it was committed**:
   - `CB-2113 | Commit Traceability | error | ENFORCED | .github/workflows/ci.yml:traceability step …`
   - `CB-2115 | Roadmap Coherence | error | ENFORCED | .github/workflows/ci.yml:traceability step …`
3. **The step's first CI run is observed RED against the §7 falsifier.** The first valid push ended with a commit pointing PMAT-723 at #1240, which is closed, while PMAT-723 stayed open. That is the falsifier's state without closing a live issue. The traceability job failed at the new step, in **run 34526901843, job 103037900725** (https://github.com/paiml/paiml-mcp-agent-toolkit/actions/runs/34526901843/job/103037900725). CB-2113 passed on the same run, so the red belongs to CB-2115:

   ```
   ✗ CB-2115: Roadmap Coherence: 2 finding(s) — ORPHAN-ROADMAP 1, ORPHAN-GITHUB 1: ORPHAN-ROADMAP PMAT-723: #1240 is closed; ORPHAN-GITHUB #1286: goal-mode step 4b: CB-2115 becomes a gate — add pmat comply check --checks CB-2113,CB-2115 (with GH_TOKEN) to the traceability job once pmat work sync --check-only exits 0 on master; blocked on PMAT-721 (the thirteen #612 collisions) and on running the sync
   ```

   The revert (`f121d9f6f`) restores #1286; the PR's merge-gating run on the revert is the green run.

## The change

- `.github/workflows/ci.yml`: the CB-2113 step becomes "the closed loop holds (CB-2113) and the roadmap and GitHub agree (CB-2115)", with GH_TOKEN. The comment that withheld it now says when and why the step landed, and why the env is block style.
- `docs/status/comply-enforcement-ledger.md`: re-rendered, with CB-2113 and CB-2115 both ENFORCED through the traceability job.
- `docs/roadmaps/roadmap.yaml`: **PMAT-1253 completed, with #1253 closed in the same step.** A ticket cannot complete itself in its own PR (CB-2113), and with CB-2115 live the item and its issue must turn terminal together. **PMAT-1296 filed** from #1296 (the `--checks` defect below), issue-first, so its id ends in its issue number.

## Jidoka log

| # | defect | owner | whys |
|---|---|---|---|
| 1 | the first push's workflow did not parse: `env: { GH_TOKEN: ${{ github.token }} }` | me | I copied the withheld step verbatim from the comment PMAT-722 wrote. A plain scalar in a YAML flow mapping cannot hold `{` or `}`, so the file failed with `ParserError while parsing a flow mapping`. The CI workflow ended in failure at startup and created no jobs. My prep script's YAML check had no `\|\| exit`. Fixed: block style, parse-asserted before the commit, branch rebuilt from master and force-pushed with a lease |
| 2 | the ledger commit on that push claimed "CB-2115 is ENFORCED" | me | the prep script timed out before printing the rows, and I wrote the subject without reading them. The rows said NEUTERED for both CB-2113, which had been ENFORCED, and CB-2115, because the parser lost the step. The rebuilt ledger was read back and asserted before its commit |
| 3 | the CB-2113 step takes 9.5 minutes in CI to report one rule | `handle_check` (`src/cli/handlers/comply_handlers/check_handlers/check.rs:53-62`) | `select_checks` relabels after all 21 groups have run. Filed as **PMAT-1296** (#1296) |
| 4 | master's first post-#1291 measurement read 15 findings | the triage's own writes, and a transfer | the pairings' title and release drift surfaced after the 60-minute grace, and two issues were transferred in; fixed by #1294 before the step was added |

## Estimates

| field | value |
|---|---|
| `K̂` | 48, `basis=docs/audits/impl-estimates.jsonl:L19-L25` |
| actual | **34** turns at this receipt (`k_measured` 471 minus 437 at the first transcript line naming the branch) |

## Gaps

| gap | artifact that closes it |
|---|---|
| CB-2115 reads live GitHub with tolerance zero: a new issue with no roadmap item turns every PR red until an item lands | `pmat work sync --direction github-to-yaml` in the PR that meets it; the rule names the issue |
| CB-2112 and CB-2114 stay withheld | PMAT-725 (#1287): the re-mint of pre-#1240 ids is a human decision by its own criterion |
| CB-2110 stays withheld | PMAT-729 (#1290): the epics and the historical/superseded classification |
| `--checks` does not prune execution | PMAT-1296 |
| no pre-PR quorum on the diff | the RED and green CI runs of the step itself are the discrimination |

## Machine-readable

orch_model: opus [V]   orch_class: opus   orch_decision: admit   orch_basis: -
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 437

routes:
  ph1  class=orchestration  route=self  w=100.00  basis=absent

verification:
  cmd=work-sync-check-only-master@7bd20ee95(live,before-the-step)  claimed_exit=-  rerun_exit=0(coherent)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/sync-master-723b.log  sha256=8dc9c311bec13038
  cmd=pushed-ci-yml-yaml-parse(flow-mapping-env)  claimed_exit=-  rerun_exit=1(ParserError)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/ci-pushed.yml  sha256=c7275d05378d1f25
  cmd=comply-ledger-write(CB-2113,CB-2115-ENFORCED)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/rb-723-ledger.log  sha256=6486b49f42703a87
  cmd=ci-traceability-RED(run-34526901843,job-103037900725)  claimed_exit=-  rerun_exit=1(CB-2115:2-findings)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/red-723-job.log  sha256=f8d1096c01a359de
  cmd=work-sync-control(round-trip)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/f723-control.log  sha256=887dc872c7dec008
  cmd=work-sync-check-only-branch(live,after-revert)  claimed_exit=-  rerun_exit=0(coherent)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/f723-check.log  sha256=a173f2460f8fb7f9
  cmd=transcript-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/transcript-gate-723.log  sha256=1ace288d7e1f246b
  cmd=kind-gate(base-origin/master)  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/kind-gate-723.log  sha256=7fa5320c47571008
  cmd=model-gate  claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/model-gate-723.log  sha256=44878c7c890a08c9

`claimed_exit=-` marks a row with no second party. `log_path` is session scratch; the `sha256` prefix pins the bytes.

## Status blocks

Finding — `k_measured` (471) is transcript-wide; `k` (34) is this ticket's share from the first transcript line naming its branch (437).

[status] ticket=PMAT-723 phase=1/1 global=34/48(K=106) k_measured=471 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L19-L25
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=PMAT-1296 blocker=- next=merge on the green run

## Verdict

**DONE.** CB-2115 is a gate. It landed on a master that measured coherent, ran RED in CI against its own falsifier, and was read back as ENFORCED in the ledger alongside CB-2113. The first push's workflow was broken by my flow-mapping env. That is recorded above, and the branch was rebuilt before the step's first valid run.
