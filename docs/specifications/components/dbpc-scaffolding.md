# DbPC Scaffolding — CB-1900..1949

**Status**: Draft (proposed 2026-04-18)
**Owner**: scaffold subsystem (`src/scaffold/`)
**Depends on**: provable-contracts.md (CB-1200..1214), pmat-work-verification-ladder.md (L0..L5),
commit-level-contract-enforcement.md (CB-1600..1649)

## 1. Motivation

`pmat scaffold` today emits Makefiles, README stubs, and Rust project skeletons,
but the output does **not** wire Design-by-Provable-Contract (DbPC) enforcement
from day 1. A freshly-scaffolded project fails `pmat comply check` until the
author hand-assembles:

* ProvableContract YAMLs under `contracts/`
* Work tickets under `.pmat-work/<ID>/contract.json`
* `verification-report.json` / `falsification.log` placeholders
* `build.rs` invoking `pmat contract codegen`
* `renacer.toml` golden-trace scenarios
* Kani / Lean harness stubs
* `.pmat-gates.toml` CB-16xx thresholds
* A CLAUDE.md that encodes the pmat-query / no-grep / pre-commit policies

Every new project repeats this setup manually. The DbPC posture is lost the
moment we forget one of these files; the dogfood-report found `pmat-dashboard`
at 0% enforcement penetration for exactly this reason (CB-1340 finding,
2026-04-18).

Goal: **scaffolding emits a tree that is COMPLIANT out of the box** — every
CB-16xx gate that can be satisfied at the chosen verification level is already
satisfied, and the remaining gates Skip cleanly with the intended reason.

## 2. Terminology

**Design by Provable Contract (DbPC)**: Evolution of Meyer's Design by Contract.
Contracts are:

1. Declared in YAML (`equations` with `preconditions` / `postconditions`)
2. Compiled to Rust via `build.rs` → `debug_assert!` + trait bindings (CB-1203)
3. Bound to tickets at `.pmat-work/<ID>/contract.json` with SHA snapshots
4. Verified at escalating levels **L0 paper** → **L1 asserts** → **L2 equations**
   → **L3 falsification tests** → **L4 Kani harness** → **L5 Lean proof**
5. Enforced by `pmat comply check` CB-1200..1214 + CB-1600..1649 gates

Unlike classic DbC, DbPC additionally requires:

* **Verification-level monotonicity** (CB-1618) — level never regresses without
  a documented CB-1617 downgrade note
* **Bind-time SHA snapshots** (CB-1615 Kani harnesses, CB-1621 expected values)
  to catch silent post-bind drift
* **Liskov-Wing weakest-binding-dominates** for multi-bound tickets (CB-1617)

DbPC is the target posture; scaffolding is the one-shot bootstrap that gets a
project there in a single command.

## 3. Scope

Extend `src/scaffold/` to emit DbPC-ready artifacts. Delta vs today:

| Artifact | Today | After this spec |
|---|---|---|
| `Makefile` | Generated (wasm template) | Plus `dbpc-check`, `dbpc-bind`, `dbpc-promote` targets |
| `README.md` | Basic template | DbPC section: CB-16xx pointers, pmat-query policy, Verification-Ladder table, `make dbpc-check` quick-start |
| `Cargo.toml` | Generated | Plus `[build-dependencies]` for contract codegen; `coverage(off)` on codegen output |
| `src/lib.rs` | Hello | `#[cfg(contract_traits)] pub mod contract_traits; include!(concat!(env!("OUT_DIR"), "/contract_traits.rs"));` |
| `build.rs` | Not generated | Invokes `pmat contract codegen --out $OUT_DIR/contract_traits.rs` |
| `contracts/example.yaml` | Not generated | One equation with `preconditions`, `postconditions`, top-level `falsification_tests:` + (optionally) `kani_harnesses:` / `lean_theorem:` |
| `.pmat-work/EXAMPLE-001/contract.json` | Not generated | Ticket at chosen `--dbpc-level`, `implements:` bound to `contracts/example.yaml` |
| `.pmat-work/EXAMPLE-001/verification-report.json` | Not generated | Seeded so CB-1610 Passes on scaffold |
| `.pmat-work/EXAMPLE-001/falsification.log` | Not generated | Seeded so CB-1611 Passes at L3+ |
| `.pmat-work/EXAMPLE-001/kani-harness-shas.json` | Not generated | Seeded when `--with-kani` |
| `.pmat-work/EXAMPLE-001/lean-proof.json` | Not generated | Seeded when `--with-lean` |
| `.pmat-work/ledger/roster-mutations.json` | Not generated | Empty array so CB-1624 Passes |
| `renacer.toml` | Not generated | One scenario stub |
| `.pmat-gates.toml` | Partial | All CB-16xx thresholds, CB-1620 `2026-05-17` cutoff noted |
| `CLAUDE.md` (project-local) | Not generated | pmat-query policy excerpt, no-grep rule, `--faults` enrichment, CB-16xx pointer, batuta-stack link |

**Non-goals** (deferred):

* Monorepo/workspace scaffolding (pmat-workspace scaffold is separate)
* Non-Rust language scaffolds (Python/JS DbPC is CB-2000 series, future)
* Auto-filling `kani_harnesses:` bodies — we emit the stub, user writes the proof

## 4. CLI surface

New flag family on existing `pmat scaffold rust-project` (and by extension
`scaffold agent` / `scaffold wasm`):

```bash
pmat scaffold rust-project \
    --name my-crate \
    --dbpc-level L3 \                    # L0..L5; default L1
    --with-contracts example,invariant \ # comma-list of seed YAMLs
    --with-kani \                        # implies L4 minimum
    --with-lean \                        # implies L5
    --claude-md                          # emit project-local CLAUDE.md (default: true)
```

Flag interactions:

* `--with-kani` upgrades `--dbpc-level` to at least L4 (warns if user asked for lower)
* `--with-lean` upgrades to L5
* `--dbpc-level L0` emits paper-only YAML + skip-compliant stubs; still passes `comply check`
* Omitting `--dbpc-level` defaults to L1 (assert-bound; minimal obligation)

## 5. Post-scaffold invariant (acceptance gate)

```bash
pmat scaffold rust-project --name foo --dbpc-level L3 --with-contracts example
cd foo
cargo build           # succeeds — build.rs runs pmat contract codegen
cargo test            # passes — seeded falsification_tests compile and run
pmat comply check     # exits 0 with COMPLIANT status
```

Within `pmat comply check` output, at L3:

* CB-1610..CB-1616 **Pass** (ladder evidence present, SHAs match)
* CB-1620 **Pass** (derived obligations match)
* CB-1621 **Pass** (no expected-snapshot drift on fresh bind)
* CB-1624 **Pass** (empty roster-mutations ledger)
* CB-1617..CB-1619 **Skip** (no downgrade / no Lean yet at L3)
* CB-1640..CB-1649 (codegen) **Pass** when `build.rs` ran cleanly

At L5 with `--with-lean`, all 50 CB-16xx gates are either Pass or Skip with
documented reason. **Zero Fail on freshly-scaffolded tree** is the hard
acceptance gate — CI on this spec runs `pmat scaffold … && pmat comply check`
as the definition of done.

## 6. Ticket decomposition

| CB | Title | Effort | Depends on |
|---|---|---|---|
| CB-1900 | Define DbPC terminology + this spec | S | — |
| CB-1910 | Extend Makefile scaffold with `dbpc-check/bind/promote` targets | S | CB-1900 |
| CB-1920 | Rewrite `README.md.tmpl` with DbPC section; emit project-local CLAUDE.md | M | CB-1900 |
| CB-1930 | `--dbpc-level` + `--with-contracts` CLI flags on `scaffold rust-project` | M | CB-1910, CB-1920 |
| CB-1940 | Seed `.pmat-work/EXAMPLE-001/` tree (contract.json + evidence stubs) | M | CB-1930 |
| CB-1941 | Seed `contracts/example.yaml` matching chosen level | S | CB-1940 |
| CB-1942 | Seed `build.rs` invoking `pmat contract codegen` | S | CB-1940 |
| CB-1943 | Seed `.pmat-gates.toml` with all CB-16xx thresholds | S | CB-1940 |
| CB-1944 | Seed `renacer.toml` scenario stub | S | CB-1940 |
| CB-1945 | Kani harness scaffold stub (`--with-kani`) | S | CB-1940 |
| CB-1946 | Lean theorem scaffold stub (`--with-lean`) | S | CB-1940 |
| CB-1949 | Acceptance harness: `pmat scaffold … && pmat comply check` passes in CI | M | all above |

Effort totals: **4×S + 3×M ≈ 2 weeks for one engineer** to ship CB-1900..1949
end-to-end. Parallelizable: CB-1910 / CB-1920 / CB-1942 / CB-1943 / CB-1944 /
CB-1945 / CB-1946 are independent once CB-1930 lands the flag plumbing.

## 7. Template-system hooks

Existing infrastructure (`src/scaffold/template.rs`) already uses a registration
pattern with `{{var}}` substitution; no new template engine is needed. Additions:

* New variables: `{{dbpc_level}}`, `{{with_kani}}`, `{{with_lean}}`,
  `{{contract_names}}`
* New templates in `src/scaffold/templates/dbpc/`:
  * `contract_example.yaml.tmpl`
  * `contract_json.tmpl` (for `.pmat-work/<ID>/contract.json`)
  * `verification_report.json.tmpl`
  * `kani_harness.rs.tmpl` (gated on `--with-kani`)
  * `lean_theorem.lean.tmpl` (gated on `--with-lean`)
  * `build.rs.tmpl`
  * `renacer.toml.tmpl`
  * `pmat_gates.toml.tmpl`
  * `claude_md.tmpl`
  * `readme_dbpc_section.md.tmpl` (injected into existing README.md.tmpl)
* New hook: `generate_dbpc_artifacts(level, contracts, with_kani, with_lean)`
  called after the existing `generate_pre_commit_hook_with_gates_fast()`.

## 8. CLAUDE.md emission

The project-local CLAUDE.md MUST include (by reference or inline):

1. **pmat query policy** — the decision table from repo CLAUDE.md lines 45-70
2. **No grep for code search** rule — explicit
3. **Pre-commit gate thresholds** — link to generated `.pmat-metrics.toml`
4. **CB-16xx pointer** — "run `pmat comply check` before every commit; see
   `docs/specifications/components/commit-level-contract-enforcement.md`"
5. **Batuta stack priority** — if the generated `Cargo.toml` has math/ML deps,
   remind to prefer aprender/trueno/renacer
6. **Documentation accuracy** — pmat validate-readme before shipping README edits

Generated via `readme_dbpc_section.md.tmpl` + `claude_md.tmpl` with variable
substitution so the CB-16xx link target matches the scaffolded project's layout.

## 9. Risks & mitigations

* **Risk**: Scaffolded `build.rs` fails in offline environments where `pmat` isn't
  on PATH.
  **Mitigation**: `build.rs` checks for `PMAT=/path/to/pmat` env var and falls
  back to `which pmat || exit 0` with a printed warning — compile succeeds but
  L3+ contracts aren't wired until pmat is installed.

* **Risk**: Seeded `.pmat-work/EXAMPLE-001/` pollutes real tickets and confuses
  `pmat work list`.
  **Mitigation**: Seed ticket ID is `EXAMPLE-001` (sentinel), contract.json has
  `"example": true` flag, `pmat work list` filters it out by default.

* **Risk**: Users copy-paste the example YAML into production and ship it.
  **Mitigation**: Example equation name is `example_placeholder_do_not_ship`;
  CB-1900 gate (new) fails if any active ticket binds to an equation matching
  that pattern.

* **Risk**: Drift between this spec's CB list and code reality.
  **Mitigation**: CB-1949 acceptance harness is the spec's executable contract;
  if it fails, the spec is wrong, not the code (bijection via CB-1700).

## 10. Non-DbPC fallback

`pmat scaffold rust-project --dbpc-level none` continues to emit today's
non-DbPC scaffolding unchanged. This preserves the fast-path for users who
just want a quick Rust stub and are not ready to adopt DbPC. The default
stays `L1` because L1 is nearly free (debug_assert! bindings only, no
verification infra required).

## 11. References

* `src/scaffold/mod.rs:40-275` — `ScaffoldEngine::scaffold()` orchestrator
* `src/scaffold/template.rs:8-230` — `Template` + `TemplateRegistry`
* `src/scaffold/hooks_generation.rs` — existing pre-commit hook generator
* `src/scaffold/templates/README.md.tmpl` — current README template
* `src/scaffold/templates/wasm_Makefile.tmpl` — current Makefile template
* `docs/specifications/components/provable-contracts.md` — CB-1200..1214
* `docs/specifications/components/pmat-work-verification-ladder.md` — L0..L5
* `docs/specifications/components/commit-level-contract-enforcement.md` — CB-16xx
* `examples/cb16xx_ladder_walkthrough.rs` — worked L0..L5 example (PR #325)
