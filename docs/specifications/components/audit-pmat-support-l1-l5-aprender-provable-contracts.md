# Audit: PMAT Support for L1–L5 Aprender Provable Contracts

> **Status:** DRAFT — audit + remediation spec | **Version:** 1.0.0 | **Date:** 2026-07-04
> **Author:** PAIML Engineering
> **Question:** Do pmat's **tickets**, **roadmaps**, and **quality-gates** *fully enforce and encourage* the
> aprender L1–L5 provable-contract ladder, with a structural **bias toward full L1–L5**?
> **Reference standard:** `../aprender` provable-contracts infrastructure + `pv` (`aprender-contracts-cli` v0.49.0).
> **Method:** 6 subsystem maps + a per-level (L1–L5 + ratchet) gap analysis whose **49 candidate gaps were
> each adversarially verified** against a named file/command; verdicts (CONFIRMED / PARTIAL / REFUTED) with
> quoted evidence are in Appendix B.
> **Normative refs:** [components/provable-contracts.md](specifications/components/provable-contracts.md) (C22),
> [components/pv-compatibility.md](specifications/components/pv-compatibility.md),
> [components/pmat-work-verification-ladder.md](specifications/components/pmat-work-verification-ladder.md) (C28),
> [components/verification-backends.md](specifications/components/verification-backends.md) (C24),
> [components/commit-level-contract-enforcement.md](specifications/components/commit-level-contract-enforcement.md) (C25),
> [components/dbpc-scaffolding.md](specifications/components/dbpc-scaffolding.md),
> [components/modern-agentic-coding-support.md](specifications/components/modern-agentic-coding-support.md) (MACS F2),
> aprender `.../sub/verification-ladder.md`, `.../sub/gradual-enforcement.md`.

---

## 1. Executive Summary

**pmat has already built a nearly-complete L1–L5 enforcement *engine*. The engine is disconnected from
CI, its floors are dormant, several of its checks trust self-attested YAML instead of tool output, and
pmat does not run the ladder on itself.** The remediation is therefore mostly *wiring, floor-setting, and
evidence-hardening of existing machinery* — not new verification tooling.

### What already exists (verified in source)

| Machinery | What it does | Why it doesn't bite today |
|---|---|---|
| `VerificationLevel{L0..L5}` enum + `achieved_level()` (`src/quality/ladder_evidence.rs`) | Computes the **evidenced** level bottom-up per binding, **min-folded**, anti-hollow (L4 needs a real kani record, L5 needs `sorry`-free Lean) | Correct — but only consulted at `work complete` |
| `check_ladder_shortfall()` wired into `handle_work_complete` (`handlers.rs:610`) | Blocks `pmat work complete` when `achieved < claimed` | **One-directional** (anti-over-claim only); never enforces a *floor* |
| **CB-1308** `check_verification_ladder` (`check_contract_surfaces.rs:875`) | Hard **Fail/Critical** for any equation-bearing contract below `min_level`; reads `[verification_ladder] min_level` / `comply.thresholds.min_verification_level`; **defaults to L5** | pmat resolves it to **L0**; gates contract-definition YAMLs, not tickets; comply never runs in CI |
| **CB-1619** `check_ladder_completion_matches` | Enforces `achieved ≥ target_level` (a floor) | **Dormant** — nothing writes `target_level` |
| **CB-1618** `check_work_ladder_monotonicity` | Hard-Fails an *unaudited* per-ticket level regression | **Inert** — `pmat work checkpoint` never writes `verification_level` into checkpoint JSON |
| **CB-1205** provability invariant / **CB-1206** L-distribution (`check_pv_verification_ladder.rs`) | Detect `falsification_tests`; parse `proof-status.json` L1–L5 counts | CB-1205 is **existence-only** (`content.contains`) & **Warn**; CB-1206 gates only via `min_kani_coverage` (= **0**) |
| **CB-1330** L-level ratchet (`check_commit_enforcement_p3.rs:241`) | Compares `current_level` vs `target_level` | **Warn-only** and **self-attested** (hand-authored YAML, not evidence) |
| **CB-1614** work-ladder L4/kani, **CB-1208** binding existence | L4 kani-report present; binding→fn exists | CB-1614 **Skip/Info** when no report; CB-1208 forward-only, no reverse coverage |
| `contract.json`: typed `verification_level` (default **L3**), `target_level`, `implements[]`, per-ticket proof thresholds | Ticket ↔ equation binding + level | thresholds (`require_proof_verification`, `max_sorry_count`, `min_theorem_coverage`) default **OFF** |
| pmat already shells `pv lint` / `pv score` / `pv coverage` (3 comply sites) | pv scoring integration (CB-1201..1214) | Never shells `pv proof-status`/`kani`/`lean`/`probar`/`unlock`/`kaizen`; comply not in CI |

`pmat comply check` **does exit 1 on any red Fail** (`apply_exit_policy → std::process::exit(1)`); it is
*not* advisory-by-exit-code. It simply is never invoked by CI, the pre-commit hook, or `pmat verify`.

### The five structural failures

1. **Disconnected from the drive shaft.** The **only** CI quality gate is `pmat score --gate 60`
   (`.github/workflows/quality-gate.yml:52`). `grep -rn 'comply|cargo kani|lake build|pv lint|pv proof-status'
   .github/workflows/` → **empty**. Every ladder gate (CB-1205/1206/1308/1330/1614/1618/1619) is real and
   exits non-zero, but **nothing calls it on a merge**. `pmat verify` gate set = fmt/complexity/satd/clippy/tests.
   `.pmat-gates.toml` has no `[verification_ladder]`/`[kani]`/proof knob at all.
2. **Floors dormant / permissive.** CB-1308 defaults to L5 but pmat's floor is **L0**; CB-1619 and CB-1618
   are inert because `target_level` and checkpoint `verification_level` are never written; per-ticket proof
   thresholds default OFF. Every gate is *anti-over-claim*; **none demands a minimum**.
3. **Self-attested, not evidence-derived.** CB-1330 compares hand-authored `current_level`/`target_level`
   (a contract can self-declare L3 while `pv proof-status` computes L1 — verified on `PMAT-606.yaml`).
   L4 completion trusts a `kani-report.json` **that nothing in pmat writes** (hand-author `{"success":true}`
   → L4). L5 is a **text scan** — `grep -rn lake src/` is empty, so a `sorry`-riddled or missing proof passes.
4. **No monotone RAISE.** `target_level` is a hand-authored constant; there is **no MAX-EVER baseline**
   (`.pmat/verification-ratchet-baseline.json` does not exist). Nothing drives a module *up*; pmat drives
   **none** of `pv`'s shipped ratchet primitives (`pv lint --min-level`, `pv unlock --reason`, `pv kaizen`,
   `metadata.locked_level`).
5. **Un-dogfooded.** `pv proof-status` on pmat: **124 contracts, all L1** (by kind: registry 15, schema 109;
   **0 kernels**), Totals `1 obligation, 16 tests, 0 kani, 0 lean, 0/27 bound`. `contracts/binding.yaml` has
   11 `status: planned` MACS bindings that enforce nothing (the build.rs `AllImplemented` panic is **dead
   code** in a stub branch pointed at a deprecated path). pmat declares **24 `lean_theorem:` names with zero
   `.lean` files** — every one dangling. The roadmap has **0** substantive `L4`/`kani` references and no
   proof-level axis.

### Verdict by surface

| Surface | Enforces L1–L5? | Biases toward full? | Score |
|---|---|---|---|
| **Quality-gates / CI** (`verify`, `quality-gate`, `.pmat-gates.toml`, `quality-gate.yml`) | Engine exists but **not invoked**; CI = `score --gate 60` only | No | **1 / 5** |
| **Tickets** (`work` ladder, `contract.json`) | Anti-over-claim yes; **floors dormant**, evidence file-trusted | Weak (default claim L3, but no floor, no writer) | **2 / 5** |
| **Roadmaps** (`roadmap.yaml`, `roadmap sync`, kaizen) | No axis; `sync` drops the typed level | No | **0 / 5** |
| **Self-dogfooding** | Declared, not evidenced (all L1, dangling theorems, dead build.rs) | No | **1 / 5** |
| **pv integration** (`comply` CB-1201..1214) | Scoring yes; ratchet primitives (`--min-level`/`unlock`/`kaizen`) undriven | Partial | **2 / 5** |

### The single highest-leverage move

**Run the ladder gates that already exist in CI and pre-commit.** Adding `pmat comply check --failures-only`
(+ `pv lint contracts/ --min-level standard --strict`) as a **required status check** activates
CB-1205/1206/1308/1330/1614/1618/1619 at once, with near-zero new code (§7 R0). Everything else (floors,
evidence-hardening, the ratchet, scaffolding, roadmap axis, dogfooding) is the climb from "honest" to
"biased-toward-full."

---

## 2. Scope & Method

**In scope:** pmat's own enforcement/encouragement surfaces — the `work` ticket ladder, the `roadmap` +
kaizen backlog, and the `quality-gate`/`verify`/`comply`/CI/O(1)-metrics stack — audited against
aprender/`pv`.

**Out of scope:** re-implementing Kani/Lean/probar invocation (owned by C24 + the `pv` binary); the
correctness of `pv` itself.

**Rigor:** every gap in §6 names a **surface**, an **evidence** pointer, and a **verdict**. Of 49 candidate
gaps, **45 CONFIRMED, 5 PARTIAL, 1 REFUTED** (the "no L5 threshold knob" claim — CB-1308 already provides
one). PARTIAL corrections and the refutation are folded into §1's "what already exists" and the tables below.

---

## 3. The Canonical Standard (aprender / `pv`)

### 3.1 Proof levels (source: aprender `sub/verification-ladder.md`)

| Level | Method | Tool | Guarantee |
|---|---|---|---|
| **L5** | Theorem proving | Lean 4 (no `sorry`) | True for **all** inputs. Period. |
| **L4** | Bounded model check | Kani `#[kani::proof]` | True for all inputs ≤ size *N* (exhaustive for fixed-size kernels). |
| **L3** | Property-based test | probar / proptest | True for ~10,000 random inputs. |
| **L2** | Falsification test | `#[test]` | True for specific edge cases. |
| **L1** | Type system | rustc | True by construction (Poka-Yoke). |
| **L0** | Code review | Human eyes | "Looks right to me." |

**Provability claim:** a kernel is *provable* when **L1 + L3 + L4** hold. **Provability invariant (enforced
in aprender):** for every *kernel* contract, `|obligations|>0 ⇒ |kani_harnesses|>0` **and**
`|falsification_tests| ≥ |obligations|`. `registry:true` (and `kind: registry|model-family|pattern|schema`)
contracts are exempt.

### 3.2 How aprender enforces — and its own honest reality (calibration)

Aprender hard-enforces **only L1** in CI, via `build.rs` **`AllImplemented`**: each crate's `build.rs`
parses the in-tree `contracts/<repo>/binding.yaml` and **panics the build** on any `not_implemented`
binding, so `cargo test` (which compiles build.rs) gates it. L2's count-invariant lives only in the manual
`pv lint`; L3 is `#[contract]` `debug_assert!` (debug builds); **L4/L5 are defined in YAML but not run in
CI**. Aprender's own `pv proof-status`: 1408 contracts → **L1 726, L2 272, L3 384, L4 26, L5(Lean) 29** —
i.e. the reference repo is itself L1–L3-dominant with L4/L5 in the low single digits.

**Implication for pmat:** "bias toward full L1–L5" does **not** mean "everything at L5 tomorrow." It means
*L1 hard-enforced, L2/L3 as an enforced floor, and a monotone ratchet that climbs critical kernels toward
L4/L5* — matched to aprender's own trajectory, not exceeding it dishonestly.

### 3.3 The ratchet model (source: aprender `sub/gradual-enforcement.md`)

Design principle: **smoothest on-ramp, hardest off-ramp.** The primitives pmat must *drive* (all shipped in
`pv`):

1. **`metadata.enforcement_level`** `basic → standard → strict → proven`, gated by `pv lint --min-level`.
2. **Aggregate coverage ratchet** `pv lint --coverage --min-coverage 0.70` — % at/above a tier, tracked in
   `.pv/trend/coverage.json`, **monotonically non-decreasing** (any drop fails CI).
3. **`metadata.locked_level` + `.pv/locks.json`** — regression below the lock is a hard error
   (`PV-LCK-001`), removable only via **`pv unlock --reason`** (mandatory, audited).
4. **Stale-suppression detection** (`PV-SUP-001`).

---

## 4. Current-State Map — the pmat enforcement engine

The six-subsystem map (Appendix C anchors) reduces to: **the engine is present and correct; the drive
shafts are cut.** §1's "what already exists" table is the engine inventory. The gaps below are where the
shafts are cut, by surface:

- **quality-gate/CI:** engine never invoked (CI = `score --gate 60`); `.pmat-gates.toml` has no proof knob;
  `min_kani_coverage: 0`; L4 execution checks (CB-1510/1512, spec'd FAIL in C24) **unshipped**; L5 never
  executes `lake`.
- **tickets:** floor gates (CB-1619/1618) dormant for lack of writers; L4 kani-report and L5 lean status are
  **file-trusted**, not tool-produced; DBC "falsification" dimension measures claim-results, not obligations.
- **roadmap:** no `verification_level` axis; `pmat roadmap sync` drops the typed level; no `ladder-uplift`
  epic; no L-distribution report; kaizen backlog level-blind.
- **dogfooding:** all-L1 contracts, dead `AllImplemented` build.rs, 11 planned MACS bindings, 24 dangling
  Lean theorems, 0 property/kani harnesses, proptest cases forced to **2–5** (never the ~10k bar).

---

## 5. Root Cause (Five Whys)

1. **Why isn't a repo driven toward full L1–L5?** The whole ladder engine lives in `pmat comply` +
   `work complete`, and CI invokes **neither** — only `pmat score --gate 60`.
2. **Why only `score`?** The ladder shipped as a `work`/`comply` feature (C28, MACS F2, pv-compatibility)
   and was never wired into `.pmat-gates.toml` / `pmat verify` / `quality-gate.yml` / branch protection.
3. **Why do the floor gates that exist not bite?** They were built as *mechanisms* awaiting a *policy*:
   `target_level`, checkpoint `verification_level`, and `min_verification_level` are all defaulted to
   OFF/L0, so CB-1308/1619/1618 evaluate to "nothing required."
4. **Why is the evidence trusted, not derived?** L4/L5 backends (C24 CB-1510/1512, a real `lake build`) were
   deferred; the interim checks read a self-declared YAML field or a `kani-report.json` with no writer.
5. **Why no monotone raise?** pmat mirrored `pv`'s *scoring* (CB-1201..1214) but not `pv`'s
   *gradual-enforcement* layer (`--min-level`, `--min-coverage` ratchet, `locked_level`, `unlock`).

**Root cause:** pmat built the ladder as an **honesty mechanism** (you can't over-claim) and never connected
it to a **policy + CI + evidence** loop that would *require and raise* levels. The parts are on the bench;
they are not bolted to the crankshaft.

---

## 6. Verified Gap Catalog (49 gaps)

Severity: 🔴 critical · 🟠 high · 🟡 medium · ⚪ low. Verdict shown where not plain CONFIRMED.

### L1 — Type system / bindings (Poka-Yoke)

| ID | Surface | Sev | Gap | Fix |
|---|---|---|---|---|
| L1-1 | quality-gate | 🔴 | `AllImplemented` build-fail is **dead**: live `emit_contract_env_vars()` never panics; the panic sits in the `PMAT_FAST_BUILD` stub branch reading the **deprecated** `../../provable-contracts/contracts/pmat/binding.yaml` | Relocate the panic into `build.rs` main after `emit_contract_env_vars()`, retarget in-tree `contracts/binding.yaml`, panic on any non-`implemented` binding (allow-list `planned`) |
| L1-2 | quality-gate | 🟠 | Binding coverage is **forward-only** (CB-1208): a `#[contract]`/kernel fn with no `binding.yaml` entry fails nothing (code's own comment: "16,977 bindings … only 35 have `#[contract]`") | New `CB-12xx check_binding_reverse_coverage`: every annotated/kernel fn must appear in `binding.yaml`, else Error |
| L1-3 | quality-gate | 🟠 (PARTIAL) | The L1 binding gates (CB-1208/1209/1600..1609) run nowhere automated — CI = `score --gate 60`, no `[hooks]`, pre-commit skips comply. *(Correction: `pmat comply` does exit 1 on Fails; it's just never invoked.)* | Add binding checks to `pmat verify` + a `pmat quality-gate --checks binding-existence,binding-reverse-coverage --fail-on-error` CI step |
| L1-4 | quality-gate | 🟠 | `.pmat-gates.toml` (the *enforced* config) cannot express any binding requirement; `min_binding_existence` lives only in `.pmat.yaml` (advisory comply path) | Add `[bindings]` (`min_existence_pct`, `fail_on_ghost`, `require_reverse_coverage`, `require_all_implemented`), read in the enforced gate |
| L1-5 | tickets | 🟠 | L1 granted **for free**: `achieved_level` returns L1 for any unbound ticket; a ticket adding a kernel fn can close at L1 with that fn absent from `binding.yaml`. *(Default claim is L3, so this bites only when a ticket lowers to L1.)* | `contract.json.require_binding_for_touched_kernels` (default true); completion Fails if a touched kernel-class fn is unbound |
| L1-6 | roadmap | 🟠 | Roadmap item schema has no binding/level axis; `roadmap sync` drops `WorkContract.verification_level` | Add `verification_level`/`bound_pct` fields + preserve through sync; roadmap aggregate of contracts-per-level |
| L1-7 | scaffolding | 🟡 | No dbpc scaffold, no `pmat contract` subcommand — `binding.yaml`+`build.rs`+`#[contract]` are hand-authored; dbpc-scaffolding (CB-1900..1949) is Draft | Ship `pmat scaffold --dbpc` / `pmat contract init` emitting a fresh tree that is L1-bound out of the box |
| L1-8 | dogfooding | 🟡 | 11/36 `binding.yaml` entries are `status: planned` (all MACS families) → no `=bound` env var → not L1-enforced; only ~34 fns carry `#[contract]` | Drive planned MACS bindings → implemented (or remove); annotate kernel-class fns |

### L2 — Falsification `#[test]` (edge cases)

| ID | Surface | Sev | Gap | Fix |
|---|---|---|---|---|
| L2-1 | quality-gate | 🔴 | The canonical invariant `|falsification_tests| ≥ |obligations|` is enforced **nowhere by count**: CB-1205 only does `content.contains("falsification_tests:")` and reports **Warn** | Rewrite CB-1205 to YAML-parse & count both arrays; **Fail/Error** when tests < obligations |
| L2-2 | quality-gate | 🟠 | No `min_falsification_coverage` knob (ComplyThresholds has `min_kani`, `min_binding`, `min_verification_level` — not falsification) | Add `min_falsification_coverage` (default 0.0; canonical 100.0) to `.pmat.yaml`/`.pmat-gates.toml`; wire into CB-1206 like `min_kani` |
| L2-3 | quality-gate | 🟠 | No CI job runs the falsification invariant (sole gate = `score --gate 60`) | Required CI step `pmat comply check --checks provability --failures-only` |
| L2-4 | tickets | 🟠 | The DBC "falsification" dimension (25%) measures **fraction of the 25 generic claims with a result**, not per-obligation falsification tests; `work score --min-score` defaults 0.0 | Add `obligation_falsification_coverage` dimension **or** gate `work complete` on it with a non-zero default |
| L2-5 | tickets | 🟠 | Anti-over-claim ceiling with no floor; `min_verification_level` resolves to L0 for pmat, so obligation-bearing work is never forced to L2. *(A floor gate CB-1308 exists — it's just set to L0 and scopes contract-files.)* | Set `min_verification_level: L2`; extend the floor to obligation-bearing tickets |
| L2-6 | roadmap | 🟡 | No falsification/level axis; `roadmap sync` drops the level | Roadmap `verification_level`/`falsification_coverage` fields + "raise every kernel to L2" epic with a per-release floor |
| L2-7 | dogfooding | 🟡 | pmat's 16 falsification tests are unbound to obligations; no contract exhibits `tests ≥ obligations` — the invariant has **never been exercised on this repo** | Author ≥1 reference kernel with N obligations + ≥N bound falsification tests that the new count-gate passes |

### L3 — Property-based tests (probar/proptest ~10k)

| ID | Surface | Sev | Gap | Fix |
|---|---|---|---|---|
| L3-1 | quality-gate | 🔴 | **No property gate**: `.pmat-gates.toml` has no proptest knob; `quality-gate --checks` has no `property` option (its `provability` check is the unrelated *Lightweight Provability* static analyzer); `verify` runs none | `[gates] require_property_tests`, `min_property_tests_per_obligation`; new `--checks property`; a `property` stage in `verify` |
| L3-2 | quality-gate | 🟠 | pmat's ladder **mislabels** L3: `ladder_evidence.rs:24` defines L3 as "bound YAML declares non-empty `falsification_tests[]`" — that is aprender-**L2** semantics. pmat has **no rung** mapping to aprender-L3 property testing | Add a distinct property-test tier: `achieved_level` L3 requires ≥1 probar/proptest harness per obligation; keep `falsification_tests` at L2 |
| L3-3 | quality-gate | 🟠 | Where property tests exist, cases are forced to **2–5** (Makefile overrides its own 256 default; in-tree max `with_cases(1000)`; one `with_cases(0)`) — never the ~10k bar; no floor gate | `[gates] min_proptest_cases` (ratcheted toward 10k) + a CI job at the floor |
| L3-4 | quality-gate | 🟡 | The one comply property check greps the Makefile for the **string** `PROPTEST_CASES` and never reads the value — passes at `=2` | Parse the numeric value; fail below `min_proptest_cases`; require per-obligation harnesses |
| L3-5 | scaffolding | 🟡 (PARTIAL) | pmat never shells `pv probar`/`pv generate` (only `pv coverage`); no path from a bound equation to a generated property test | `pmat scaffold contract` / bind step shells `pv probar` to emit a harness per obligation |
| L3-6 | tickets | 🟡 (PARTIAL) | No `work start --target-level`, no per-binding `min_property_tests`. *(Correction: CB-1619 `check_ladder_completion_matches` already enforces `achieved ≥ target_level` — it's dormant because nothing sets `target_level`.)* | Wire `work start --target-level L3` to **write** `target_level` (activates CB-1619); add `min_property_tests` |
| L3-7 | roadmap | 🟡 | No property/level axis; `sync` drops the level (the whole pipeline is a 3-field `id/title/status` projection) | Roadmap `verification_level`/`property_test_target` + "raise MACS kernel classes A–E to L3" epic |
| L3-8 | dogfooding | 🟡 | 0 property harnesses; "proptest"/"property" appear only in prose; kernel classes A–E all L1 | Generate probar harnesses via `pv probar`; add `property_tests[]` to bound YAML; L3 step in the dogfood loop |
| L3-9 | quality-gate | ⚪ | `.pmat.yaml` suppresses CB-951 for `.github/workflows/property-tests.yml` — **a workflow that does not exist** (config-vs-reality rot); no property job runs in CI | Create `property-tests.yml` (`PROPTEST_CASES=10000 cargo test -- property_tests`) as a required check, or remove the dead suppression |

### L4 — Bounded model checking (Kani)

| ID | Surface | Sev | Gap | Fix |
|---|---|---|---|---|
| L4-1 | quality-gate | 🔴 | **No CI runs `cargo kani`**; the ~24 real `#[kani::proof]` harnesses are dead weight — a harness can regress to `VERIFICATION:- FAILED` and nothing blocks merge | Blocking `kani-bmc` CI job + `--checks kani`, driven by `.pmat-gates.toml [kani]` (`run_kani`, `require_all_harnesses_pass`) |
| L4-2 | quality-gate | 🔴 | `.pmat-gates.toml` has no `[kani]`; the one knob (`.pmat.yaml min_kani_coverage`) is set to **0** — the Kani gate is explicitly off | Add `[kani]` (`min_harness_coverage`, `min_bound≥4` per CB-1513); raise `min_kani_coverage` above 0 and enforce |
| L4-3 | quality-gate | 🟠 | The invariant `obligations>0 ⇒ kani_harnesses>0` is metadata-only; the **execution** checks C24 **CB-1510/1512** (spec'd FAIL) are **unshipped** (`grep -rln CB-1510 src/` → empty) | Implement CB-1510/1512 to spawn `cargo kani --harness` and assert `SUCCESSFUL count == obligations` |
| L4-4 | tickets | 🟠 | L4 completion is **file-trust**: `achieved_level` grants L4 from `kani-report.json {success:true}`, but **nothing in pmat writes it** (all writers are `#[cfg(test)]`) — hand-author `{"success":true}` → L4 | `pmat work kani <ID>` runs `cargo kani` and writes a **provenance-signed** report; `achieved_level` rejects unsigned reports |
| L4-5 | quality-gate | 🟠 | comply CB-1614 returns **Skip/Info** (not Fail) when an L4+ ticket has no report; comply not in CI | CB-1614 → Fail/Error; add `--checks work-ladder-l4` to CI |
| L4-6 | scaffolding | 🟠 | pmat can't scaffold Kani harnesses (`pmat scaffold` = project/agent/wasm only; `pmat contract` unrecognized) | `pmat contract codegen --kani` shells `pv kani` → one `#[kani::proof]` per obligation |
| L4-7 | roadmap | 🟠 | No L4 axis, no "raise L1→L4" epic; `sync` drops the level; `grep -c L4 roadmap.yaml` = 0 | Roadmap `target_verification_level` + monotone-ratchet epic (rising floor of contracts-at-L4) |
| L4-8 | quality-gate | 🟡 | The only real Kani spawn (rust-project-score) runs `cargo kani --only-codegen` — **solverless, no BMC**; ~5/289 pts, non-blocking, off in Fast mode | Add a `--full-kani` path (no `--only-codegen`) parsing `VERIFICATION:- SUCCESSFUL`; route L4 credit through the blocking gate |

### L5 — Theorem proving (Lean 4, no `sorry`)

| ID | Surface | Sev | Gap | Fix |
|---|---|---|---|---|
| L5-1 | quality-gate | 🔴 | **No gate executes Lean**: `grep -rn lake src/` is empty; every L5 surface is text/field-level, so a missing or `sorry`-riddled proof passes | `CB-1660`: shell `lake build` on `contracts/lean/`, Fail on non-zero exit or residual `sorry`; add a `lean` stage to `pmat verify` (Skip when no lakefile — on-ramp) |
| — | — | — | **REFUTED — "no L5 threshold knob."** CB-1308 `check_verification_ladder` already provides a configurable hard floor (`[verification_ladder] min_level` / `min_verification_level`, **default L5**, Critical Fail). The real gap is that CB-1206's `l5_pct` is only *formatted*, and pmat's floor is set to L0. | — |
| L5-2 | dogfooding | 🟠 | pmat's contracts declare **24 `lean_theorem:` names, 0 `.lean` files, no lakefile** — every reference dangling; `pv proof-status` = 0 lean proved | Create `contracts/lean/` with real proofs for the score/index invariants (`Geometric_Mean_Bounded`, `Score_Range_NonNeg`, `Index_Roundtrip_Identity`), or downgrade unbacked names to `status: declared`; `CB-1661` verifies each name resolves in a `lake build`-clean project |
| L5-3 | scaffolding | 🟠 | No on-ramp from a declared theorem to a proof (no lean template, no `pmat contract codegen`, never shells `pv lean`) | `pmat contract codegen --backend lean` shells `pv lean` → `Theorems/<Contract>.lean` skeleton + lakefile with `sorry` placeholders |
| L5-4 | tickets | 🟠 | Completion can't **require** L5; `work start` has no `--target-level`; DONE trusts `lean_theorem.status`, never runs `lake build` | `CB-1662` kernel L5 floor: for critical (A–E) kernels require `achieved ≥ required_min_level` with L5 evidence from a live `lake build` |
| L5-5 | roadmap | 🟠 | No proof-level axis; only L5 items are completed one-shot tasks; no standing "L3→L5 uplift" epic, no distribution report | Roadmap `target_verification_level` + `item_type: ladder-uplift` (current→target) + per-release monotone floor |
| L5-6 | quality-gate | 🟠 | No CI job exercises L5; `quality-gate.yml` installs no Lean/lake toolchain and not even `pv` | Nightly `lean-verify` job: install Lean + `pv`, `lake build contracts/lean/ && ! grep -rw sorry`; required once kernels are backed |
| L5-7 | quality-gate | 🟡 | Where L5 touches the CI gate its weight is negligible: D4-Lean = 10% of pv-mean × 40% of pv_lint × 1/8 geometric dims — a proven vs empty theorem differ by <1 pt on `--gate 60` | `pmat score --gate 60 --min-lean-pct N` — an independent hard threshold that fails regardless of the composite |

### RATCHET — bias toward full L1–L5 (cross-cutting)

| ID | Surface | Sev | Gap | Fix |
|---|---|---|---|---|
| R-1 | quality-gate | 🔴 (PARTIAL) | Every ratchet check (CB-1308/1330/1618) lives in `pmat comply`, which **no CI workflow invokes** and is not a branch-protection check. *(Correction: comply exits 1 on Fails — the issue is purely that nothing calls it.)* | Add `pmat comply check --failures-only` (+ `pv lint --min-level standard --strict`) as a **required** CI status check |
| R-2 | quality-gate | 🔴 | CB-1330 L-ratchet is **Warn-only** and **self-attested** — compares hand-authored `current_level` vs `target_level` (verified: `PMAT-606.yaml` self-declares L3 while `pv proof-status` computes L1) | Flip CB-1330 → Fail; derive `current_level` from `pv proof-status`; reject `declared > evidenced` |
| R-3 | quality-gate | 🟠 | **No monotone RAISE**: `target_level` is a hand-authored constant; no MAX-EVER baseline exists | `pmat comply baseline --ladder` writes `.pmat/verification-ratchet-baseline.json = max(evidence, prior)`; `CB-1331` Fails on `evidence < baseline` or an un-committed climb |
| R-4 | tickets | 🟠 (PARTIAL) | `work start` has no `--target-level`; nothing forces a ticket to inherit a floor from the module it edits. *(CB-1619 enforces `achieved ≥ target` but is dormant.)* | `work start --target-level` writes `target_verification_level` (inherit from the touched module's baseline); activate CB-1619 as the DONE-gate |
| R-5 | tickets | 🟠 | CB-1618 per-ticket monotonicity **can** hard-Fail but is **inert**: `pmat work checkpoint` never writes `verification_level` into checkpoint JSON (23/23 checkpoints lack it); no `downgrades.json` ledger | `work checkpoint` writes the evidence-derived level; append a downgrade ledger on audited drops → CB-1618 goes live |
| R-6 | roadmap | 🟠 | No proof-level axis; `sync` drops the level; no `ladder-uplift` type; no L-distribution report | Roadmap `verification_level`/`target_verification_level` + `item_type: ladder-uplift(module_glob, from, to)` + `pmat roadmap ladder-report` |
| R-7 | quality-gate | 🟠 | pmat drives **none** of pv's ratchet primitives (`pv unlock --reason`, `pv lint --min-level`, `pv kaizen`); no `metadata.locked_level` on any pmat contract | comply check shelling `pv lint --min-level standard --strict`; add `locked_level`/`enforcement_level`; route overrides through `pv unlock --reason` (shared audit trail) |
| R-8 | quality-gate | 🟡 | `min_level` is a single **global** scalar — no per-crate/module budget (can't say "kernel crate ≥ L4, CLI crate ≥ L2") | `[[verification_ladder.budget]]` array (`path_glob`, `min_level`); CB-1308 evaluates each contract against its most-specific budget |
| R-9 | quality-gate | 🟡 (PARTIAL) | No bridge to `pv kaizen`; kaizen backlog level-blind; no `.pmat-metrics/ladder-floor.json` snapshot. *(CB-1618 does force per-ticket non-decrease — but not fleet/release-wide.)* | `pmat comply kaizen` shells `pv kaizen`, records a non-decreasing fleet floor, auto-emits `ladder-uplift` items |

---

## 7. Recommendations — biased toward full L1–L5 (leverage-ordered)

### R0 — Connect the engine to CI + pre-commit (highest leverage, ~0 new code)

Activate the existing gates by *invoking* them:

```yaml
# .github/workflows/quality-gate.yml — add as a REQUIRED status check
- run: pmat comply check --failures-only            # activates CB-1205/1206/1308/1330/1614/1618/1619 (exits 1 on Fail)
- run: pv lint contracts/ --min-level standard --strict --format github
```

- Add the same ladder checks to the `pmat verify` gate set (`format · complexity · satd · clippy · tests ·
  **ladder**`) so the everyday pre-flight is proof-aware.
- Cache `proof-status.json` keyed by contract-dir hash (mirror the coverage cache) to stay O(1)-friendly.

This alone moves the quality-gate surface from 1/5 toward 3/5.

### R1 — Set and climb the floors (activate dormant machinery)

- **Set** `[verification_ladder] min_level` (start `L2`, aprender uses `L3`) in `.pmat-gates.toml` and read
  it in the *enforced* gate, not just comply. CB-1308 already Fails below it.
- **Per-crate budget** (R-8): `[[verification_ladder.budget]] { path_glob, min_level }` — kernels `L4`,
  features `L3`, registries/chores `L1` — evaluated most-specific-first.
- **Activate CB-1619**: `pmat work start --target-level <L>` (default from the budget for the touched
  module) **writes** `target_level`; completion Fails on `achieved < target`.
- **Activate CB-1618**: `pmat work checkpoint` writes the evidence-derived `verification_level`; add a
  `downgrades.json` ledger.
- Turn the per-ticket thresholds ON where a kernel is touched (`require_proof_verification`,
  `max_sorry_count=0`, `min_theorem_coverage`).

### R2 — Make evidence tool-derived, not self-attested

- **CB-1330 → Fail**, and compute `current_level` from `pv proof-status`, not the YAML field.
- **CB-1205 → count-based** (`tests ≥ obligations`) and **Fail**; add `min_falsification_coverage`.
- **L4**: `pmat work kani <ID>` runs `cargo kani` and writes a **provenance-signed** `kani-report.json`;
  `achieved_level` rejects unsigned reports; implement C24 **CB-1510/1512** (spawn Kani, assert
  `SUCCESSFUL == obligations`). Fix the solverless `--only-codegen` scoring (L4-8).
- **L5**: `CB-1660` runs `lake build` (exit 0 + zero `sorry`); `CB-1661` resolves each `lean_theorem:` name.

### R3 — Install the monotone RAISE ratchet

- `pmat comply baseline --ladder` → `.pmat/verification-ratchet-baseline.json` = per-module MAX-EVER
  evidenced level; **CB-1331** Fails on `evidence < baseline` (and forces a baseline-bump commit on a climb).
- Add an aggregate `ladder_coverage` O(1) metric (% of kernel obligations at/above budget) recorded in
  `.pmat-metrics/` and **enforced monotone non-decreasing** in the pre-commit O(1) gate + CI — pmat's
  analogue of `pv lint --coverage --min-coverage`.
- Adopt `pv`'s primitives (R-7): `metadata.locked_level` on pmat's contracts, `pv lint --min-level` in CI,
  overrides via `pv unlock --reason` (one signed audit trail).

### R4 — Real L1 (fix the dead poka-yoke)

- Relocate & retarget the `build.rs` `AllImplemented` panic at in-tree `contracts/binding.yaml` (L1-1);
  drive the 11 planned MACS bindings to `implemented`.
- Add **reverse binding coverage** (L1-2): every `#[contract]`/kernel fn must appear in `binding.yaml`.

### R5 — Scaffolding on-ramp (make climbing the easy path)

- Ship **dbpc-scaffolding** (CB-1900..1949): `pmat contract codegen --{kani,lean,probar}` shells the
  matching `pv` generator; `pmat scaffold --dbpc` emits a fresh tree that is L1-bound with harness/lakefile
  stubs, so a kernel's next rung is "fill in the generated stub," not "start from scratch."
- `pmat qa-work`/`falsify` report **distance-to-floor** ("achieved L2, budget L4: add a Kani harness for
  `X`").

### R6 — Schedule the climb (roadmap + kaizen)

- Add `verification_level` + `target_verification_level` to the roadmap item schema; **preserve them
  through `pmat roadmap sync`** (stop the 3-field projection dropping the level).
- `item_type: ladder-uplift(module_glob, from_level, to_level)`; `pmat roadmap` auto-emits one per
  below-budget module (ranked `pagerank × (budget − achieved)`); `pmat roadmap ladder-report` prints the
  L-distribution and deficit.
- `pmat comply kaizen` bridges `pv kaizen`, records a non-decreasing fleet floor (`.pmat-metrics/ladder-floor.json`).

### R7 — Dogfood pmat's own kernels

- Bind pmat's genuine kernels (ladder parse/`Ord`, search-mode dispatch, score composites, `achieved_level`)
  in `binding.yaml`; climb them **L2 (falsification) → L3 (probar) → L4 (Kani)**.
- Prove **≥1 lighthouse invariant at L5** — `Theorems.Macs.Ladder_Parse_Total` is already named in
  `macs-ladder-v1.yaml` — in a real `contracts/lean/` lakefile.
- Acceptance: `pv proof-status` on pmat shows a genuine climb (not all-L1); a synthetic "kernel ticket that
  binds nothing" is **rejected** by `pmat work complete`.

---

## 8. Proposed config surface

```toml
# .pmat-gates.toml  (NEW sections — read by the ENFORCED pmat quality-gate, not just comply)
[verification_ladder]
enabled  = true
min_level = "L2"            # global floor; CB-1308 already Fails below it
[[verification_ladder.budget]]
path_glob = "src/quality/**"
min_level = "L4"            # kernels
[[verification_ladder.budget]]
path_glob = "contracts/**"
min_level = "L3"

[bindings]
require_all_implemented  = true    # drives the fixed build.rs AllImplemented
require_reverse_coverage = true
fail_on_ghost            = true

[kani]
run_kani                   = true
require_all_harnesses_pass = true
min_harness_coverage       = 0.80

[gates]                    # L2/L3 count floors
min_falsification_coverage = 100     # tests >= obligations
require_property_tests     = true
min_proptest_cases         = 1000    # ratchet toward 10000

[ratchet]
baseline = ".pmat/verification-ratchet-baseline.json"   # MAX-EVER, monotone non-decreasing
```

```yaml
# .pmat.yaml  comply.thresholds  (raise off the permissive defaults)
comply:
  thresholds:
    min_verification_level: "L2"    # was resolving to L0
    min_kani_coverage: 0.80         # was 0
    min_lean_coverage: 0            # on-ramp; raise per release for critical kernels
```

---

## 9. Definition of Done — "fully enforces & encourages L1–L5"

Calibrated to aprender's own trajectory (L1 hard, L2/L3 floor, L4/L5 ratcheted on critical kernels):

1. **Wired:** `pmat comply check --failures-only` + `pv lint --min-level` are **required** CI status checks;
   `pmat verify` includes a `ladder` stage (R0).
2. **Floored:** `[verification_ladder] min_level` ≥ L2 with per-crate budgets; CB-1619/1618 live via written
   `target_level`/checkpoint levels; a kernel ticket that binds nothing **cannot close** (R1).
3. **Evidence-derived:** CB-1330 Fails on evidence regression; CB-1205 is count-based; L4 uses a
   pmat-written signed `kani-report.json` (CB-1510/1512); L5 runs a real `lake build` (CB-1660/1661) (R2).
4. **Ratcheting:** `ladder_coverage` is an O(1) metric enforced monotone non-decreasing; `.pmat/verification-ratchet-baseline.json`
   is MAX-EVER; `locked_level` honored with `pv unlock --reason` (R3).
5. **Real L1:** `build.rs AllImplemented` panics on in-tree `binding.yaml`; reverse coverage enforced (R4).
6. **On-ramped:** `pmat contract codegen --{kani,lean,probar}` emits the stubs that raise a level (R5).
7. **Scheduled:** roadmap carries a level axis preserved through `sync`; `ladder-uplift` epics + `ladder-report`
   exist; kaizen ratchets a fleet floor (R6).
8. **Dogfooded:** `pv proof-status` on pmat shows a real climb — ≥1 kernel at L4, ≥1 invariant at L5, 0
   dangling theorems, 0 planned bindings; pmat's own CI ladder gate is green (R7).

Falsifiable acceptance: `pv proof-status --format json` on the pmat repo shows the target distribution, **and**
a synthetic kernel ticket binding nothing is rejected by `pmat work complete`.

---

## 10. Phased Implementation Plan (ticketed; each declares its own target level)

> **Implementation status (landed):** The **lighthouse dogfood kernel is live** —
> `contracts/macs-ladder-kernel-v1.yaml` binds pmat's own `VerificationLevel` parser
> and climbs the ladder: **L1** (bound in `contracts/binding.yaml`), **L2** (falsification
> `#[test]`s), **L3** (proptest — `pv proof-status` reports **L3**), **L4** (three
> `#[kani::proof]` harnesses in `work_verification_level.rs` — authored; runnable
> in-repo since the 3.24.1 MSRV correction (pmat's MSRV is **1.91** ≤ Kani 0.67's
> rustc 1.93), and the CI job executes them), **L5** (`contracts/lean/Theorems/Macs/Ladder.lean`
> — six theorems incl. `Ladder_Parse_Total`, `lake build` clean, `pv lean-status` 4/4
> proved / 0 sorry, `#print axioms` reports **no axiom dependencies**). pmat's kernel
> count went **0 → 1** and its level distribution from **all-L1** to **124×L1 + 1×L3**.
> Also landed: **L1 real** — `build.rs` `AllImplemented` relocated to the live path,
> retargeted at the in-tree `binding.yaml`, and **falsification-verified to panic** on a
> `not_implemented` binding (ALADR-008); `binding.yaml` made pv-parseable (`planned →
> pending`); **floor** `[verification_ladder] min_level` set (ALADR-002 partial);
> **CI** `provable-ladder` job wires L5-Lean (blocking) + `pmat comply` (advisory during
> the ALADR-012 grace period) into `quality-gate.yml` (ALADR-001 partial). Remaining
> tickets below (evidence-hardening CB flips, ratchet baseline, roadmap axis, scaffolding)
> are the next increment — they touch the comply gate handlers and need careful landing
> against the full suite.


| Ticket | Phase | Deliverable | Target | Gaps |
|---|---|---|---|---|
| **ALADR-001** | 1 · Wire | `pmat comply check --failures-only` + `pv lint --min-level standard --strict` as required CI checks; `ladder` stage in `pmat verify`; proof-status cache | L3 | R0, R-1, L1-3, L2-3, L4-5, L5-6 |
| **ALADR-002** | 2 · Floor | `[verification_ladder] min_level` + `[[budget]]` per-crate; read in enforced gate; set `min_verification_level: L2`, `min_kani_coverage` off-zero | L4 | R1, R-8, L2-5, L4-2 |
| **ALADR-003** | 2 · Floor | Activate CB-1619 (`work start --target-level` writer) + CB-1618 (checkpoint level writer + downgrade ledger); turn per-ticket thresholds on for kernels | L4 | R-4, R-5, L3-6, L1-5 |
| **ALADR-004** | 3 · Evidence | CB-1330 → Fail + evidence-derived `current_level`; CB-1205 count-based + `min_falsification_coverage`; fix DBC falsification dimension | L4 | R-2, L2-1, L2-2, L2-4 |
| **ALADR-005** | 3 · Evidence | `pmat work kani` signed report + C24 CB-1510/1512 (spawn Kani); blocking `kani-bmc` CI job; fix solverless scoring | L4 | L4-1, L4-3, L4-4, L4-8 |
| **ALADR-006** | 3 · Evidence | CB-1660 `lake build` + CB-1661 theorem resolution; `lean` verify stage; `--min-lean-pct` hard flag | L5 | L5-1, L5-4, L5-7 |
| **ALADR-007** | 4 · Ratchet | `pmat comply baseline --ladder` + CB-1331 MAX-EVER baseline; `ladder_coverage` O(1) monotone metric; adopt `locked_level`/`pv unlock` | L4 | R-3, R-7, R-9 |
| **ALADR-008** | 4 · L1 | Relocate+retarget `build.rs AllImplemented`; reverse binding coverage CB-12xx; drive 11 planned MACS bindings → implemented | L4 | L1-1, L1-2, L1-4, L1-8 |
| **ALADR-009** | 5 · Scaffold | Ship dbpc-scaffolding CB-1900..1949: `pmat contract codegen --{kani,lean,probar}`, `pmat scaffold --dbpc`; property-cases floor + `property-tests.yml` | L3 | L1-7, L3-1..5, L4-6, L5-3, L3-9 |
| **ALADR-010** | 5 · Roadmap | Roadmap `verification_level`/`target_verification_level` + `ladder-uplift` type + `ladder-report`; preserve through `sync`; `pmat comply kaizen` | L3 | R-6, L1-6, L2-6, L3-7, L4-7, L5-5 |
| **ALADR-011** | 6 · Dogfood | Bind + climb pmat kernels to L4; real `contracts/lean/` proof (`Ladder_Parse_Total`) at L5; green self ladder gate | L5 | L2-7, L3-8, L5-2 |
| **ALADR-012** | 6 · Enforce | Grace-period flip: ladder CI checks warn → error after 30 days (mirror C28 §Migration) | L3 | all |

Order: 001 → 002 → 003 → {004, 005, 006} → 007 → 008 → 009 → 010 → 011 → 012.

---

## 11. Appendix A — Ground-truth command reference

```bash
# Canonical (aprender / pv)
pv proof-status --format json          # L1–L5 per contract (aprender: L1 726 L2 272 L3 384 L4 26 L5 29)
pv lint --coverage --min-coverage 0.70 # aggregate monotone ratchet
pv lint --min-level standard --strict  # per-contract enforcement level
pv unlock <c> --reason "<text>"        # audited off-ramp lock removal

# pmat current state (run in pmat repo)
cat .github/workflows/quality-gate.yml            # sole gate = `pmat score --gate 60`
grep -rn 'comply|cargo kani|lake build|pv lint|pv proof-status' .github/workflows/   # EMPTY
pv proof-status                                   # 124 contracts, ALL L1; 0 kani/lean; 0/27 bound
cat .pmat-gates.toml                              # no [verification_ladder]/[kani]/[bindings]
grep min_kani_coverage .pmat.yaml                 # 0  (Kani gate off)
grep -c 'status: planned' contracts/binding.yaml  # 11 (MACS bindings unenforced)
find . -name '*.lean' | grep -v target            # EMPTY (24 dangling lean_theorem names)
grep -c kani docs/roadmaps/roadmap.yaml           # 0
```

## 12. Appendix B — Adversarial verification summary

49 candidate gaps → **45 CONFIRMED, 5 PARTIAL, 1 REFUTED**. The PARTIALs and the refutation *strengthen*
the audit by crediting existing machinery:

- **REFUTED** `L5-no-min-lean-knob` — CB-1308 already ships a configurable hard L5 floor (default L5,
  Critical Fail). Fix is to *set* it, not build it.
- **PARTIAL** `R-1`/`L1-3` — `pmat comply` **does** exit 1 on Fails; the gap is purely that CI never calls it.
- **PARTIAL** `R-4`/`L3-6` — CB-1619 already enforces `achieved ≥ target_level`; dormant for lack of a
  `target_level` writer.
- **PARTIAL** `R-9` — CB-1618 already forces per-ticket non-decrease; missing is a fleet/release-wide floor.
- **PARTIAL** `L3-5` — verifier timed out; claim (no `pv probar`/`pv generate` invocation) matches the
  independently-confirmed fact that pmat shells only `pv lint`/`score`/`coverage`.

Full per-gap verdicts with quoted file:line evidence: workflow transcript
`…/subagents/workflows/wf_71995a85-85d/journal.jsonl`.

## 13. Appendix C — Key source & CB anchors

`src/quality/ladder_evidence.rs` (`achieved_level`, `check_ladder_shortfall`, `kani_report_success`);
`src/cli/handlers/work_contract_core.rs:159` (`default_verification_level → L3`);
`src/models/comply_config_types.rs:227` (`default_min_verification_level → "L0"`);
`src/cli/handlers/comply_handlers/check_handlers/`: `check_contract_surfaces.rs:875` (CB-1308),
`check_pv_verification_ladder.rs` (CB-1205/1206), `check_commit_enforcement_p3.rs:241` (CB-1330),
`check_work_ladder_l4_kani.rs` (CB-1614), `check_work_ladder_monotonicity.rs` (CB-1618),
`check_ladder_completion_matches` (CB-1619), `check_pv_quality_gate.rs:145` (CB-1208);
`src/services/rust_project_score/formal_verification_scoring.rs:36` (`kani --only-codegen`);
`build.rs:1553` (`emit_contract_env_vars`, no panic), `build.rs:1463-1546` (dead `AllImplemented`);
config: `.pmat-gates.toml`, `.pmat.yaml`, `contracts/binding.yaml`, `contracts/*.yaml`;
CI: `.github/workflows/quality-gate.yml:52`. Spec'd-but-unshipped: C24 CB-1510/1512, dbpc CB-1900..1949.

## 14. References

**Internal:** C22 provable-contracts, pv-compatibility, C28 pmat-work-verification-ladder, C24
verification-backends, C25 commit-level-contract-enforcement, C27 pmat-work-contract-binding, MACS
modern-agentic-coding-support, dbpc-scaffolding, work-management; aprender `sub/verification-ladder.md`,
`sub/gradual-enforcement.md`, `sub/scoring.md`, `pv-spec.md`.

**Foundational:** Popper (1959); Meyer (1997) *OOSC2* (DbC); Liskov & Wing (1994) (min-fold precondition
weakening); Leino (2010) *Dafny*; Siek & Taha (2006) (gradual typing → gradual enforcement).

**arXiv:** 2511.14805 *Continuous Assurance with Formal Verification* (Kani-in-CI); 2510.12047 *Do LLMs
Respect Contracts?*; 2602.22302 *Agent Behavioral Contracts*; 2603.02668 *SorryDB* (L5 `sorry`-elimination);
2511.12638 *Equivalence Checking of ML GPU Kernels* (SIMD=scalar Kani).
