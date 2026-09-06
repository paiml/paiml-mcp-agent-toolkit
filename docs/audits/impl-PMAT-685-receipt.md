# impl receipt — PMAT-685 (work cot derive: hollow obligations for v5.0 steps, #1200)

| field | value |
|---|---|
| ticket | PMAT-685 · kind=code · priority critical (operator: "add to queue as urgent and install local ASAP") |
| branch | `PMAT-685-cot-derive-hollow` · PR #1201 · base `master` 6b15921be (cut from the 3.39.0 mint branch, so it carries PMAT-675..685) |
| HEAD at receipt | the commit that adds this file (child of the mutation revert) |
| discover.json sha256 | same discovery as the 3.39.0 run (`gate_cmd_fallback=true`, recorded, not run; `pmat verify` was the gate) |
| local install | `cargo install --path . --locked --force` from 54fc201e0: `~/.cargo/bin/pmat` derives the issue's step with `statement:` = the claim (measured); the aprender C0-3 session can re-derive GH-663..672 now |

## Defect

`parse_step` treated a `version: "5.0"` step carrying `falsifiable_claim` (and `evidence_method`/`discharged_by`) as structured but read only `implication`, so `render_derivation` wrote `statement: ""` / `hypothesis: ""` verbatim; `pv validate` refused the file (SCHEMA-005), `CB-1658` accepted it (`"" == ""` satisfies the verbatim rule), `pmat work cot check` said the chain held. Reproduced on the installed 3.38.0 with the issue's exact JSON.

## Plan and dispatch

| phase | what | route | trigger |
|---|---|---|---|
| 1 | tests RED (03019cfb9) → fix GREEN (19802626f) | direct (no worker: urgent, three touch points known) | — |
| 1.delegate | quorum ×3 (adversarial / clean-room / semantic), `--mode plan`, `writes=false` | `paiml-agy-delegate` `a1a37e82dce04f288` (hit 30 turns; resumed once to return the receipt); conversations `96f76bfb-…`, `4358af5e-…`, `3c2e4d16-…`; lanes at `/run/user/1000/paiml-implement/agy/ph685/` | Q1 |
| 1b | quorum findings → 3953518fc (digest covers borrowed claims; only-claim step is structured), b840492bc (`cot check` hollow warning), c728332f3 (unwrap → expect, ratchet) | direct | — |
| 2 | mutation RED in CI, reverted | direct | — |
| 3 | pv contract `contracts/work/PMAT-685.yaml` (validate 0 errors, lint PASS) | direct | — |

Slots: peak 1; one hook denial recorded (a `gh pr view` in my own Bash while the delegate held its slot — "push/PR is orchestrator-only; a worker is running"); no retry, re-issued after the delegate returned.

## Verification (claimed vs orchestrator)

| check | result |
|---|---|
| `cargo test --lib -- work_cot tests_macs_derivation cot_derive` | 23 passed (7 new) |
| issue's exact step, built binary | `statement: "apr compare-hf --offline hangs …"`; `pv validate` valid |
| hollow step, built binary | `Error: CB-1658: 1 hollow step(s) (CoT-1) … nothing was written`; no artifact, no digest |
| `pmat verify --skip tests` | ok on every commit |
| `pmat comply ratchet` / `the_committed_ratchet_holds_at_head` | exit 0 after c728332f3 (RED before it: `unwrap_calls_src_total` 20341 > 20336 — five `unwrap()` in the new test) |
| quorum | lane 1 do-not-implement-as-written: `canonical_cot_sha` never hashed the top-level claims the fallback reads → fixed (borrowed texts appended to the digest input only when borrowed; pre-existing digests unchanged, test `digest_covers_borrowed_top_level_claims_and_nothing_else`); lane 3: a step with only `falsifiable_claim` parsed as legacy prose → fixed; lanes 1+2: `cot check` should warn on hollow steps → added. Lane 2's "ratchet tripped, environmental" was NOT environmental — it was real and is fixed above. |
| mutation (3.38.0 parse restored, worktree) | local: 3 failed (the fallback tests), 20 passed incl. `cb1658_red_on_hollow_obligation` and `cot_derive_refuses_a_hollow_step_and_writes_nothing` — discrimination confirmed |
| mutation in CI | run 34023329875: every leg 4 failed (`full` 23108 passed / 4 failed): the three fallback tests + the ratchet regression; reverted |

## Findings outside the ticket

- `discharged_by: ["FC-1"]` (an array) parses to `None` — `check_chain` still reports the chain intact because `evidence_method` anchors the step; unchanged here, recorded.
- `claim_text` accepts `claim`/`text`/`hypothesis` — looser than SCHEMA-005's single spelling; deliberate (the derive normalises), recorded.

## Gaps

pv lane: **Run**. Kani/probar/lean: NotRun (follow-up `pv generate`).

## Verdict

## Scope widened by the operator ("URGENT FIX … Same PR")

PR #1201 now also carries PMAT-676 (one scanner, one validator — receipt `impl-PMAT-676-receipt.md`), PMAT-679 (append-only writer — `impl-PMAT-679-receipt.md`) and PMAT-680 (single-authority mint — `impl-PMAT-680-receipt.md`), plus the fixture test `every_aprender_gh_663_to_672_contract_derives_zero_empty_statements` over aprender's ten real GH-663..672 contracts (RED under the 3.38.0 parse in the worktree: `GH-663: no hollow step` failed; GREEN on the fix).

verdict: DONE pending the green train's required checks on PR #1201.

IMPL-PMAT-685-RECEIPT-END
