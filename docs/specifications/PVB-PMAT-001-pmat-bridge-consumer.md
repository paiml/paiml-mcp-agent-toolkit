# PVB-PMAT-001 v1.1 — pmat consumes the proof↔code bridge

Spec for the paiml-implement harness. **Public**: it lives in the public pmat repo and follows PVB-001 §3 rule 10 (no private hosts, budgets, token figures, revenue or tiers in anything it writes).

- Location: `~/src/paiml-mcp-agent-toolkit/docs/specifications/PVB-PMAT-001-pmat-bridge-consumer.md`. Default branch `master`.
- Launch from `~/src/paiml-mcp-agent-toolkit`; all work in `$WT="$SCRATCH/wt-pmat"`:
  `Implement docs/specifications/PVB-PMAT-001-pmat-bridge-consumer.md autonomously.`
- Idempotent, one PR or one STOP per run. Same landing protocol, probe/accept split, ledger-line schema, receipt shape and STOP vocabulary as PVB-001 v1.3.
- Location rule: 5 of 5 rows land in pmat → pmat.
- Budgets: operator-private sidecar (PVB-001 §0.5). Missing → `STOP(budget: sidecar missing)`.

**v1.1 (2026-10-27)** applies review C-1, C-2, C-3 and C-5:
- C-1/C-2: the probe library is aprender's public `scripts/pvb/lib.sh` (PVB-001 Appendix B), copied byte-for-byte and verified against the same blake3 pin.
- C-3: every probe ends `&& merged PM-<n>`; `$PVB_MAIN=origin/master`.
- C-5: §1 facts frozen from a live read of pmat; PM-0 cites PVL EV-16; PM-3 handles a repo without `docs/roadmaps/entries/`; PM-4 starts from pmat's existing computable models.

**Variables:**

| Name | Path |
|---|---|
| `$LEDGER` | `docs/audits/PVB-PMAT-001/ledger.jsonl`, written inside each row's PR; line schema as PVB-001 (no `merged_sha`) |
| `$PVB_MAIN` | `origin/master` |
| `$PVB_DONE_WHEN_DIRS` | `scripts/pvb-pmat/done_when` |
| Probe library | `scripts/pvb/lib.sh` + `scripts/pvb/lib-check.sh` + `lib.sh.blake3`, copied byte-for-byte from aprender at the commit BR-0 merged. A pin mismatch with aprender's current pin → `STOP(lib-pin: re-copy from aprender)`. Both are public, so copying crosses no boundary. |
| Probes | `scripts/pvb-pmat/done_when/<row>.sh`, each beginning `[ -f scripts/pvb/lib.sh ] \|\| exit 1` |
| Fixtures | `scripts/pvb-pmat/fixtures/<row>/` |
| aprender rows | checked with `merged_remote paiml/aprender main docs/audits/PVB-001/ledger.jsonl BR-<n>` |

---

## §0 Thesis

PVB-001 makes `pv` the single authority on bridged L5 and on freshness. pmat is the fleet's workflow and compliance tool, so it is where that authority becomes *enforced habit*: compliance checks, pre-merge verification, and ticket completion.

Three rules shape every row:
1. **Direction.** pmat depends on `pv`; `pv` never calls pmat (PVB rule 9). This keeps the gate from depending on the tool it gates.
2. **One definition.** pmat never parses `bridge-summary.json` and never recomputes a hash. It reads only PVB-001 §6's stable surface: `pv proof-status --format json`, `pv bridge fresh --format json` (`schema: pvb-fresh/1`), and `pv --version`.
3. **Decline is not pass.** `pv` exit 2, an unknown schema id, or a `pv` version off the fleet pin → pmat reports `Unknown{<reason>}`, which never counts as a pass (ONT R-2).

**Publishing gate first.** pmat adopts only an `aprender-contracts-cli` that has passed aprender's clean-room CI gate on its tag and is published on crates.io at the fleet `pv` pin (PVL EV-13). pmat never builds `pv` from a branch.

---

## §1 Ground truth (quote-frozen)

| Fact | Value | Mark | Re-verify |
|---|---|---|---|
| pmat runs Lean in a required check | yes (`provable ladder` job, `quality-gate.yml:177-246`) | `[C]` PV-LEAN-AUDIT-001 2026-09-10 | `pmat query "provable ladder"` |
| pmat proof gate | `scripts/pv-obligation-gate.py` (`Makefile:227`, `quality-gate.yml:246`), retired by **PVL-001 EV-16** | `[V 2026-10-27 reviewer]` | operator's PVL ledger |
| pmat contracts | 35 files in `contracts/` | `[C]` PVL-001 | `ls contracts \| wc -l` |
| pmat `binding.yaml` | **present**: `contracts/binding.yaml`, ~11 KB | `[V 2026-10-27 reviewer]` | `tracked contracts/binding.yaml` |
| pmat computable `exact` Lean models | `contracts/lean/Theorems/Macs/Ladder.lean` (`VLevel.parseStrict`, for `macs-ladder-kernel-v1.yaml`); `contracts/lean/Theorems/Tdg/Grade.lean` (`GradeL.fromScore`) | `[V 2026-10-27 reviewer]` | `absent 'noncomputable' <file>` |
| Where pmat declares its `pv` pin | `.github/workflows/quality-gate.yml:244`, value `0.63.0` | `[V 2026-10-27 reviewer]` | `rg -n 'aprender-contracts-cli\|provable-contracts-cli' .github/workflows/quality-gate.yml` |
| Where `pmat comply` rules are registered | `[U]` | `[U]` | PM-1 (`pmat query "comply rule registry"`) |
| ONT-11 comply-rule pattern (exit-2 semantics, one fleet-wide monotone rule) | specified | `[C]` ONT-001 v4.3 | ONT ledger |
| Roadmap layout | pmat uses `docs/roadmaps/roadmap.yaml`; **`docs/roadmaps/entries/` does not exist**. Per `contracts/roadmap-fragments-v1.yaml:12`, fragments are opt-in when that directory exists. | `[V 2026-10-27 reviewer]` | `[ -d docs/roadmaps/entries ]` |
| pmat discharge summary in PVL EV-8a format (needed for B0) | `[U]` | `[U]` | PM-4 precondition |
| Fragment-writer defect | `pmat work add` wrote `roadmap.yaml` directly where guard-tree required fragments (#1370) | `[C]` | `pmat query "roadmap fragment writer"` |

---

## §2 Rows

**PM-0** · "PVB-PMAT-0: preflight — bridge published, pin agreed" · `depends_on: []`
- Write into `$WT` first: the library files copied from aprender (Variables table), then source them for the checks below.
- Read-only check:
  - `merged_remote paiml/aprender main docs/audits/PVB-001/ledger.jsonl BR-7` and the same for **BR-7b**.
  - `cargo search aprender-contracts-cli --limit 1` returns a version that has `pv bridge fresh`.
  - `merged_pvl EV-16` (retires `pv-obligation-gate.py` in pmat). Exit 2 → `STOP(env: LEDGER_PVL unset)`.
  - Any failure → `STOP(depends: <which>)`. Nothing is committed.
- If all pass, the PR:
  - commits the library files;
  - bumps the `pv` pin at `quality-gate.yml:244` from `0.63.0` to the published version carrying `pv bridge fresh`, matching the fleet pin;
  - adds `scripts/pvb-pmat/pv-pin.sh`: reads the pin from that workflow line, prints it beside `pv --version`, exits 1 on mismatch.
- Mutation: set the workflow pin back to `0.63.0` → `pv-pin.sh` RED.
- probe: `[ -f scripts/pvb/lib.sh ] && tracked scripts/pvb-pmat/pv-pin.sh && merged PM-0`
- accept: `bash scripts/pvb/lib-check.sh && bash scripts/pvb-pmat/pv-pin.sh`

**PM-1** · "PVB-PMAT-1: `pmat comply` rule `contracts.l5-bridged`" · `depends_on: [PM-0]`
- Change: one rule, registered the way ONT-11 registers rules (location found via `pmat query`, recorded in §1). It runs only if `contracts/` exists, and checks:
  1. `pv proof-status --format json` → `.l5_source == "bridge"`. If not, `pv` is too old: `Unknown{pv-too-old}`.
  2. `pv bridge fresh --format json` exits 0 with `schema == "pvb-fresh/1"`. Exit 1 → FAIL, listing stale contracts. Exit 2 → `Unknown{decline:<reason>}`.
  3. `l5_bridged` at head ≥ at base (never-down, no baseline file).
  4. `pv --version` equals the pin, else `Unknown{pin}`.
  - A repo with zero bridged contracts gets `Unknown{decline: 0 bridged}`, reported, not failed, until that repo adopts PVB. A pass is never reported for zero.
- RED: four fixture repos — fresh, stale, regressed count, pin mismatch. Expected verdicts: PASS, FAIL, FAIL, Unknown.
- Mutation: treat `Unknown` as pass → the pin-mismatch fixture passes → RED.
- probe: `tracked scripts/pvb-pmat/fixtures/pm-1/expected.json && present 'l5-bridged' "$WT/src" && merged PM-1`
- accept: `pmat comply --rule contracts.l5-bridged --format json scripts/pvb-pmat/fixtures/pm-1/stale > "$SCRATCH/c.json"; json_object "$SCRATCH/c.json" '.verdict=="FAIL" and (.stale|length)==1'` (repeat per fixture)

**PM-2** · "PVB-PMAT-2: `pmat verify` freshness tier" · `depends_on: [PM-1]`
- Change: `pmat verify` adds one cheap step. If `contracts/` exists, it runs `pv bridge fresh --changed $(git diff --name-only <base>...HEAD) --format json`.
  - Stale → verify RED, naming each contract and the fix command (`pv bridge diff <id> && pv bridge kani <id>`).
  - Unknown → reported, not RED (the full check lives in CI's `provable-ladder`).
  - It never runs `lake`, `cargo` or `kani`.
- Budget: adds ≤ 5 s to `pmat verify` p95, measured on the pmat repo itself over 10 runs.
- RED: a fixture branch that edits a bound Rust function without updating the summary → `pmat verify` RED.
- Mutation: drop the `--changed` set (check nothing) → fixture passes → RED.
- probe: `present 'bridge fresh' "$WT/src" && merged PM-2`
- accept: fixture run RED, untouched-tree run GREEN, and `hyperfine -r 10 'pmat verify'` delta ≤ 5 s, all in the PR body.

**PM-3** · "PVB-PMAT-3: ticket ↔ contract" · `depends_on: [PM-1]`
- Change:
  - `pmat work add --contract <id>` records the contract id on the ticket's roadmap entry, through pmat's existing roadmap writer:
    - if `docs/roadmaps/entries/` exists (fragments opted in, `roadmap-fragments-v1.yaml:12`): in the ticket's fragment, never in `roadmap.yaml` (#1370 guard);
    - otherwise (pmat today): in the ticket's entry in `docs/roadmaps/roadmap.yaml`.
    - PM-3 does **not** create `docs/roadmaps/entries/`: opting a repo into fragments is a separate decision, not a side effect.
  - RED covers both layouts: one fixture repo with `entries/`, one without.
  - `pmat work complete` on a ticket whose contract is kernel-kind calls `pv proof-status --format json` and refuses unless that contract is at bridged L5. It prints the contract's current level and the missing legs (B0–B3).
  - Non-kernel contracts and tickets without a contract are unaffected.
- This is where "a provable contract per feature" becomes a mechanism, not a habit.
- RED: three fixtures — a kernel contract at bridged L5 completes; one at L4 is refused with the missing legs named; a ticket without a contract completes.
- Mutation: skip the kind check → the non-kernel fixture is refused → RED.
- probe: `present '--contract' "$WT/src" && tracked scripts/pvb-pmat/fixtures/pm-3/expected.json && merged PM-3`
- accept: `rc_is 1 -- pmat work complete <L4-fixture-ticket> 2>"$SCRATCH/e" && rg -q 'missing: B2, B3' "$SCRATCH/e"`

**PM-4** · "PVB-PMAT-4: dogfood — bridge one pmat contract" · `depends_on: [PM-1]`
- Purpose: the first consumer outside aprender. This proves `pv bridge` is generic (PVB §4 metric "external consumers ≥ 1").
- Precondition, B0 evidence: pmat's `provable ladder` must produce a discharge summary in the PVL EV-8a format (keyed on `tree_sha`), or `pv` has nothing to read for B0. Whether it does today is `[U]`. If not → `STOP(depends: pmat discharge-summary — PVL EV-8a pattern not applied to pmat)`, and file that as a PVL ticket. Do not re-implement it here.
- Change: run `pv bridge census contracts/ --out "$SCRATCH/c.json"`. Candidates, in order:
  1. `macs-ladder-kernel-v1.yaml` ↔ `Theorems/Macs/Ladder.lean` (`VLevel.parseStrict`, computable, `exact`).
  2. The contract whose model is `Theorems/Tdg/Grade.lean` (`GradeL.fromScore`, computable). Its `exact` status depends on whether the score input is integral; if the input is a float, it is `ieee`, not `exact` (PVB §0.1).
  3. Any other `bridgeable=yes` from the census.
  - `contracts/binding.yaml` already exists `[V]`: add the bound function to it; do not create a second binding file.
  - Add `proof.bridge` and `rust:` predicates; run `pv bridge diff` and `pv bridge kani`; commit `contracts/bridge-summary.json` (pmat's summary path, passed to `pv` by `--summary`; PVB §6 requires paths from arguments).
  - Add the `pv bridge fresh` step to pmat's existing `provable ladder` job.
  - If candidates 1–3 all fail the census → `STOP(scope: 0 bridgeable in pmat)` and file one issue listing the `no:<reason>` buckets.
- Mutation: bit-flip the bound pmat function → `provable ladder` RED; throwaway PR closed unmerged, run linked in the PR body.
- probe: `tracked contracts/bridge-summary.json && json_object contracts/bridge-summary.json '(.entries|length)>=1' && merged PM-4`
- accept: `pv proof-status --format json > "$SCRATCH/p.json" && json_object "$SCRATCH/p.json" '.l5_source=="bridge" and .l5_bridged>=1'`

---

## §3 Scoreboard

| Metric | Target | Probe |
|---|---|---|
| Comply verdicts that pass on zero, decline or pin mismatch | 0 | PM-1 fixtures |
| `pmat verify` p95 added by freshness tier | ≤ 5 s | PM-2 hyperfine |
| Kernel-contract tickets completed below bridged L5 | 0 | PM-3 refusal log |
| pmat contracts at bridged L5 | ≥ 1 within one pmat release of PM-0 | PM-4 accept |
| pmat code that reads `bridge-summary.json` or computes a bridge hash | 0 lines | `pmat query "bridge-summary"` over `src/` |

---

## §4 Five whys: why pmat and not only CI

1. Why isn't CI enough? CI catches a stale bridge after the PR is queued, which costs a CI slot.
2. Why does that matter? The fleet's flow targets punish late failures; a cheap local check moves the defect left.
3. Why can't each repo write its own check? Each copy would encode its own definition of fresh and of L5, and they would drift.
4. Why does drift matter? Two definitions of L5 is the PVL EV-3 defect again.
5. Countermeasure: `pv` owns the definition; pmat owns enforcement points (comply, verify, ticket completion) and calls `pv`.
