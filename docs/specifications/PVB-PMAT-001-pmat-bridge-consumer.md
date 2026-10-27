# PVB-PMAT-001 v1.0 — pmat consumes the proof↔code bridge

Spec for the paiml-implement harness. **Public**: it lives in the public pmat repo and follows PVB-001 §3 rule 10 (no private hosts, budgets, token figures, revenue or tiers in anything it writes).

- Location: `~/src/paiml-mcp-agent-toolkit/docs/specifications/PVB-PMAT-001-pmat-bridge-consumer.md`. Default branch `master`.
- Launch from `~/src/paiml-mcp-agent-toolkit`; all work in `$WT="$SCRATCH/wt-pmat"`:
  `Implement docs/specifications/PVB-PMAT-001-pmat-bridge-consumer.md autonomously.`
- Idempotent, one PR or one STOP per run. Same landing protocol, probe/accept split, receipt shape and STOP vocabulary as PVB-001 v1.2.
- Location rule: 5 of 5 rows land in pmat → pmat.
- Budgets: operator-private sidecar (PVB-001 §0.5). Missing → `STOP(budget: sidecar missing)`.

**Variables:**

| Name | Path |
|---|---|
| `$LEDGER` | `docs/audits/PVB-PMAT-001/ledger.jsonl`, written inside each row's PR |
| Probe library | `scripts/pvb/lib.sh`, vendored with blake3 pin, same rule as PVB BR-0 |
| Probes | `scripts/pvb-pmat/done_when/<row>.sh` |
| Fixtures | `scripts/pvb-pmat/fixtures/<row>/` |
| `$PVB_LEDGER` | aprender's public `docs/audits/PVB-001/ledger.jsonl`, read-only, via `gh api repos/paiml/aprender/contents/...` |

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
| pmat proof gate | `scripts/pv-obligation-gate.py` (`Makefile:227`, `quality-gate.yml:246`), to be retired by a PVL row | `[C]` | PVL ledger |
| pmat contracts | 35 files in `contracts/` | `[C]` PVL-001 | `ls contracts \| wc -l` |
| pmat `binding.yaml` | `[U]` | `[U]` | PM-0 |
| Where `pmat comply` rules are registered | `[U]` | `[U]` | PM-1 (`pmat query "comply rule registry"`) |
| ONT-11 comply-rule pattern (exit-2 semantics, one fleet-wide monotone rule) | specified | `[C]` ONT-001 v4.3 | ONT ledger |
| Where pmat declares its `pv` pin | `[U]` | `[U]` | PM-0 |
| `roadmap.yaml` vs fragments | `pmat work add` wrote `roadmap.yaml` directly while guard-tree requires fragments under `docs/roadmaps/entries/` (#1370) | `[C]` | `pmat query "roadmap fragment writer"` |

---

## §2 Rows

**PM-0** · "PVB-PMAT-0: preflight — bridge published, pin agreed" · `depends_on: []`
- Read-only check:
  - `$PVB_LEDGER` shows PVB **BR-7** and **BR-7b** merged.
  - `cargo search aprender-contracts-cli --limit 1` returns a version that has `pv bridge fresh`.
  - The version pmat pins for `pv` equals it.
  - The PVL row that retires `pv-obligation-gate.py` in pmat is merged.
  - Any failure → `STOP(depends: <which>)`.
- If all pass, the PR adds:
  - the vendored `scripts/pvb/lib.sh` + pin
  - `scripts/pvb-pmat/pv-pin.sh`: prints `pv --version` and the declared pin, exits 1 on mismatch. This records where the pin lives and fills §1 `[U]`.
- Mutation: set the declared pin to `0.0.0` → `pv-pin.sh` RED.
- probe: `tracked scripts/pvb/lib.sh && tracked scripts/pvb-pmat/pv-pin.sh`
- accept: `bash scripts/pvb-pmat/pv-pin.sh`

**PM-1** · "PVB-PMAT-1: `pmat comply` rule `contracts.l5-bridged`" · `depends_on: [PM-0]`
- Change: one rule, registered the way ONT-11 registers rules (location found via `pmat query`, recorded in §1). It runs only if `contracts/` exists, and checks:
  1. `pv proof-status --format json` → `.l5_source == "bridge"`. If not, `pv` is too old: `Unknown{pv-too-old}`.
  2. `pv bridge fresh --format json` exits 0 with `schema == "pvb-fresh/1"`. Exit 1 → FAIL, listing stale contracts. Exit 2 → `Unknown{decline:<reason>}`.
  3. `l5_bridged` at head ≥ at base (never-down, no baseline file).
  4. `pv --version` equals the pin, else `Unknown{pin}`.
  - A repo with zero bridged contracts gets `Unknown{decline: 0 bridged}`, reported, not failed, until that repo adopts PVB. A pass is never reported for zero.
- RED: four fixture repos — fresh, stale, regressed count, pin mismatch. Expected verdicts: PASS, FAIL, FAIL, Unknown.
- Mutation: treat `Unknown` as pass → the pin-mismatch fixture passes → RED.
- probe: `tracked scripts/pvb-pmat/fixtures/pm-1/expected.json && present 'l5-bridged' "$WT/src"`
- accept: `pmat comply --rule contracts.l5-bridged --format json scripts/pvb-pmat/fixtures/pm-1/stale > "$SCRATCH/c.json"; json_object "$SCRATCH/c.json" '.verdict=="FAIL" and (.stale|length)==1'` (repeat per fixture)

**PM-2** · "PVB-PMAT-2: `pmat verify` freshness tier" · `depends_on: [PM-1]`
- Change: `pmat verify` adds one cheap step. If `contracts/` exists, it runs `pv bridge fresh --changed $(git diff --name-only <base>...HEAD) --format json`.
  - Stale → verify RED, naming each contract and the fix command (`pv bridge diff <id> && pv bridge kani <id>`).
  - Unknown → reported, not RED (the full check lives in CI's `provable-ladder`).
  - It never runs `lake`, `cargo` or `kani`.
- Budget: adds ≤ 5 s to `pmat verify` p95, measured on the pmat repo itself over 10 runs.
- RED: a fixture branch that edits a bound Rust function without updating the summary → `pmat verify` RED.
- Mutation: drop the `--changed` set (check nothing) → fixture passes → RED.
- probe: `present 'bridge fresh' "$WT/src"`
- accept: fixture run RED, untouched-tree run GREEN, and `hyperfine -r 10 'pmat verify'` delta ≤ 5 s, all in the PR body.

**PM-3** · "PVB-PMAT-3: ticket ↔ contract" · `depends_on: [PM-1]`
- Change:
  - `pmat work add --contract <id>` records the contract id in the ticket's roadmap **fragment** under `docs/roadmaps/entries/`, never in `roadmap.yaml` (#1370 guard).
  - `pmat work complete` on a ticket whose contract is kernel-kind calls `pv proof-status --format json` and refuses unless that contract is at bridged L5. It prints the contract's current level and the missing legs (B0–B3).
  - Non-kernel contracts and tickets without a contract are unaffected.
- This is where "a provable contract per feature" becomes a mechanism, not a habit.
- RED: three fixtures — a kernel contract at bridged L5 completes; one at L4 is refused with the missing legs named; a ticket without a contract completes.
- Mutation: skip the kind check → the non-kernel fixture is refused → RED.
- probe: `present '--contract' "$WT/src" && tracked scripts/pvb-pmat/fixtures/pm-3/expected.json`
- accept: `rc_is 1 -- pmat work complete <L4-fixture-ticket> 2>"$SCRATCH/e" && rg -q 'missing: B2, B3' "$SCRATCH/e"`

**PM-4** · "PVB-PMAT-4: dogfood — bridge one pmat contract" · `depends_on: [PM-1]`
- Purpose: the first consumer outside aprender. This proves `pv bridge` is generic (PVB §4 metric "external consumers ≥ 1").
- Change: run `pv bridge census contracts/` on pmat's 35 contracts and pick the first `bridgeable=yes`, preferring `exact`.
  - If none is bridgeable, write one computable `exact` Lean model for the cheapest candidate (BR-3b pattern).
  - If no candidate has a Rust binding → `STOP(scope: 0 bridgeable in pmat)` and file one issue listing the `no:<reason>` buckets.
  - Add `binding.yaml` (if absent), `proof.bridge`, `rust:` predicates, then `pv bridge diff` and `pv bridge kani`, and commit pmat's own `bridge-summary.json`.
  - Add the bridge step to pmat's existing `provable ladder` job.
- Mutation: bit-flip the bound pmat function → `provable ladder` RED; throwaway PR closed unmerged, run linked in the PR body.
- probe: `tracked bridge-summary.json && json_object bridge-summary.json '(.entries|length)>=1'` (path as `pv` writes it for pmat's layout)
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
