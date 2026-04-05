# Commit-Level Contract Enforcement & Asset Contracts

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 25

## Root-Cause Analysis: Why Contracts Don't Gate Commits

Five Whys (2026-04-05):

1. **Why can developers commit code that violates provable-contracts?**
   Pre-commit hooks check formatting and complexity, not contract obligations.
2. **Why don't pre-commit hooks check contracts?**
   `pmat work` has `contract.json` (DbC v5.0) and `provable-contracts` has
   separate YAML — two parallel systems that never merge at commit time.
3. **Why two parallel systems?**
   `pmat work` was built for task-level falsification; `provable-contracts`
   for kernel-level formal verification. Neither enforces the other at git boundary.
4. **Why is the git boundary unguarded?**
   Contract verification was assumed to be a CI concern, not a commit concern.
5. **Why isn't CI sufficient?**
   CI runs minutes after commit. Developer has context-switched. AWS Cedar
   (ICSE 2025) documents "proof brittleness" — proofs break from unrelated
   commits. The fix: **shift-left** to commit time.

**The fix:** Unify both contract systems under a single commit-level enforcement
pipeline. Extend to non-code assets with layout contracts. All checks O(1)
from cached metrics — no cold verification in the commit path.

---

## Design Principles

### O(1) Firm Requirement

**Every pre-commit check MUST complete in < 30ms from cached data.** No check
may invoke `cargo build`, `cargo test`, `pv lint` (cold), or network calls.
All verification data pre-computed during development and cached in `.pmat/`.

| Check Category | Data Source | Latency |
|----------------|------------|---------|
| Contract obligation status | `.pmat/contract-cache.json` | < 5ms |
| L-level ratchet | `.pmat/verification-levels.json` | < 3ms |
| Asset layout validation | `.pmat/asset-layout-cache.json` | < 10ms |
| Binding diff lookup | `.pmat/binding-index.json` | < 5ms |
| Staleness gate | File mtime comparison | < 1ms |

**Staleness policy:** >7 days = warning, >30 days = error. Emergency bypass:
`git commit --no-verify` (logged in `.pmat-metrics/bypass-log.jsonl`).

### Provable-Contracts as Single Source of Truth

All enforcement flows through provable-contracts YAML. `pmat work` contract.json
becomes a **derived view** — generated from and validated against authoritative YAML.

### rmedia Grid Protocol Paradigm

Non-code assets follow the rmedia model: **content never exists without a
placement contract**. Every README section, Dockerfile block, SVG group, and
book chapter occupies a named slot with typed constraints.

---

## Phase 1: Work Item to YAML Contract Generation

When `pmat work start` creates a work item, it generates both `contract.json`
(DbC v5.0) and `contracts/work/<ID>.yaml` (provable-contracts schema), bridging
task-level falsification with the verification ladder.

**Mapping:** `require[]` → preconditions, `ensure[]` → postconditions,
`invariant[]` → invariants, `falsification_method` → `falsification_tests[].method`,
`verification_level` → `verification_summary.target_level`.

**Cache population:** `pmat work start` and `pmat work checkpoint` write
`.pmat/contract-cache.json` with active work item status, obligation counts,
and verification timestamps for O(1) pre-commit lookup.

**CLI:** `pv validate contracts/work/<ID>.yaml` validates schema,
`pv score` scores quality, `pv audit` checks traceability chain.

---

## Phase 2: Verification Level Monotonicity Ratchet

Each binding in `binding.yaml` carries a verification level (L0-L5). The
ratchet ensures levels **never decrease**, preventing deletion of Kani
harnesses or Lean proofs.

**Mechanism:** Pre-commit reads `.pmat/verification-levels.json` (O(1)),
looks up affected bindings for staged files, blocks if any L-level decreased,
warns if stale (>7 days).

| Transition | Allowed? | Rationale |
|-----------|----------|-----------|
| L1→L2, L2→L3, L3→L4 | Yes | Progress |
| L3→L2, L4→L3 | **BLOCKED** | Regression |
| Any→L0 | **BLOCKED** | Total regression |

**Escape hatch:** `pmat comply ratchet-override --binding <fn> --from L4 --to L2
--reason "..." --work-item <ID>` writes signed entry to
`.pmat-metrics/ratchet-overrides.jsonl`, expires in 14 days. CB-1330 violation
if unrecovered by expiry.

**Basis:** Agent Behavioral Contracts (arXiv:2602.22302) Drift Bounds Theorem;
Gradual Verification (Bader et al., TOPLAS).

---

## Phase 3: Asset Layout Contracts

Non-code assets use the **container model**: every asset has named **slots**
with ID, order, type, required flag, and constraints.

### Seven Asset Types

| CB Check | Asset | Key Enforcement |
|----------|-------|-----------------|
| CB-1320 | README.md | 10 slots (7 required), ordering, accuracy, cross-refs |
| CB-1321 | Dockerfile | No `:latest`, no `curl\|bash`, multi-stage, pinned deps |
| CB-1322 | SVG | ViewBox validation, 6-color palette, element budget, WCAG |
| CB-1323 | forjar.yaml | DAG acyclicity, template resolution, no plaintext secrets |
| CB-1324 | mdBook | SUMMARY link integrity, code block compilation, cross-refs |
| CB-1325 | CHANGELOG | Keep-a-Changelog format, semver ordering |
| CB-1326 | Badges | Required set present, URLs live, header placement |

Each asset type has a contract YAML defining slots, constraints, and
falsification tests. Contracts follow the provable-contracts schema with
`surface: asset-layout` and enforcement levels.

**README (CB-1320):** Slots: header, badges, description, installation, usage,
benchmarks, architecture, api, contributing, license, footer. Accuracy checks
via regex matching against `pmat --version`, `cargo test` output, and coverage
data. Cross-ref validation ensures claimed files exist.

**Dockerfile (CB-1321):** Instruction blocks map to slots: base-image,
dependencies, build-stage, runtime-stage. Security checks: no `curl|bash`,
no root USER, pinned apt versions.

**O(1) cache:** All asset validation runs during `pmat work checkpoint` or
`pmat asset validate`. Results cached in `.pmat/asset-layout-cache.json` for
pre-commit lookup.

---

## Phase 4: Differential Obligation Verification

Full contract verification is expensive. At commit time, only obligations whose
bound functions were modified need re-checking.

**Mechanism:** `git diff --cached --name-only` → lookup in
`.pmat/binding-index.json` (file→binding reverse index) → check cached verdicts
for affected obligations → PASS/FAIL from cache.

**Binding index:** Maps source files to their contract bindings and
obligations. Maps asset files to layout contracts. Rebuilt by
`pmat comply refresh-bindings` or `pmat comply refresh-contracts`.

**Basis:** Mugnier et al. (OOPSLA 2025) documents proof brittleness from
whole-contract reverification; AWS Cedar (ICSE 2025) confirms targeted
verification eliminates this.

---

## Phase 5: Assume-Guarantee Chains for Concurrent Work

When multiple work items touch overlapping code, one commit can break another's
assumptions. Work contracts declare dependencies via `assumes` (references
another contract's obligation) and `guarantees` (obligations this item ensures).

**Pre-commit validation:** Load active contracts → build dependency DAG →
for each modified file, find affected guarantees → check if other work items
assume those guarantees → block if guarantee broken (from cache).

**Conflict resolution:** Error message names the affected work item and
offers options: re-verify (`pmat work checkpoint`), override
(`pmat comply ratchet-override`), or bypass (`--no-verify`).

**Basis:** Pacti (ACM TCPS 2025) algebraic A/G operations; Dewes & Dimitrova
(AAAI 2025) quantitative A/G for multi-agent coordination; Dardik & Kang
(2025) compositional inductive invariant inference.

---

## Phase 6: `pmat query` Provable-Contract Enrichment

Six new flags make contract status a first-class search dimension:

| Flag | Description |
|------|-------------|
| `--contracts` | Enrich results with contract binding status |
| `--contract-gaps` | Show functions without contracts |
| `--min-level L3` | Filter: only functions at or above this level |
| `--max-level L1` | Filter: only functions at or below this level |
| `--contract-score` | Sort by contract quality score |
| `--asset-contracts` | Include non-code assets in results |

**O(1) architecture:** `ContractIndex` lazy-loaded from `.pmat/binding-index.json`
on first `--contracts` query (~500KB, <50ms load). Subsequent lookups O(1)
per result via HashMap.

**Relevance scoring:** `score = base_relevance * 0.7 + contract_signal * 0.3`.
`--contract-gaps` surfaces most undercontracted functions first.

**Composition:** All flags compose with existing enrichment (--churn, --faults,
--duplicates, --entropy, -G, --coverage, --coverage-gaps).

---

## Pre-Commit Hook Integration

All phases share a single pre-commit entry point reading cached data only.

| Phase | Max Latency | Data Source |
|-------|------------|------------|
| Format/complexity/SATD | 15ms | `.pmat-metrics/` |
| Work contract validity | 5ms | `.pmat/contract-cache.json` |
| L-level ratchet | 3ms | `.pmat/verification-levels.json` |
| Asset layout | 10ms | `.pmat/asset-layout-cache.json` |
| Differential obligations | 5ms | `.pmat/binding-index.json` |
| Assume-guarantee chains | 7ms | `.pmat/contract-cache.json` |
| **Total** | **< 45ms** | **All from cache** |

**Cache refresh:** `pmat work checkpoint <ID>` (during development),
`pmat comply refresh-contracts` (after binding changes),
`pmat asset validate` (after asset edits).

---

## Phase 7: Hook Subsystem Consolidation

### Root Cause (Five Whys)

170 hook-related commits in pmat with ~38 bug-fixes — fix-break-fix cycle.
**Why:** 6 independent codepaths write to `.git/hooks/` with no coordination
(`hook_manager.rs`, `git_hooks.rs`, `scaffold/hooks.rs`, `hooks_command.rs`,
`tdg_hooks.rs`, `hooks_stack_handler.rs`). **Why:** No hook ownership model.
**Consequence:** 72 `--no-verify` bypasses across 8 PAIML repos.

### 14 Problem Classes from Git History Audit

| # | Problem | Severity |
|---|---------|----------|
| H-1 | 6 conflicting writers, 0 coordination | Critical |
| H-2 | Non-atomic file writes (`fs::write`, not rename) | Medium |
| H-3 | Timestamps in generated hooks (non-deterministic) | Medium |
| H-4 | TOCTOU race conditions (check-then-write) | Medium |
| H-5 | Shell injection via template substitution | Medium-High |
| H-6 | HashMap non-deterministic serialization | Low-Medium |
| H-7 | Hardcoded `~/src` path assumption | Medium |
| H-8 | Self-contradictory SATD enforcement rules | Low-Medium |
| H-9 | `$?` capture fragility across shells | Low |
| H-10 | Missing `.git` directory verification | Low-Medium |
| H-11 | Inconsistent backup behavior (skip vs overwrite) | Low |
| H-12 | Read-modify-write race on cache files | Low-Medium |
| H-13 | Non-UTF-8 path collision (maps to empty string) | Low |
| H-14 | Hardcoded stack repo list | Low |

### Design Rules (CB-1333..1337)

| Rule | CB Check | Requirement | Falsification |
|------|----------|-------------|---------------|
| HR-1 Single Writer | CB-1333 | All writes through `HookRegistry` (`BTreeMap<String, HookSection>`) | `grep -rn 'fs::write.*hooks' src/ \| grep -v hook_registry.rs` → empty |
| HR-2 Atomic Writes | CB-1334 | Write-then-rename, never `fs::write()` to hook path | Kill during write → file is old or new, never partial |
| HR-3 Deterministic | CB-1335 | No timestamps, no HashMap iteration, byte-identical output | Two consecutive installs → `diff` → zero diff |
| HR-4 No Injection | CB-1336 | Template substitution escapes shell metacharacters | Set path to `$(whoami)` → literal in output |
| HR-5 Performance | CB-1337 | Pre-commit p95 < 45ms | `.pmat-metrics/hook-timing.json` tracked per run |

---

## Phase 8: Falsify Leak Remediation

### Root Cause (Five Whys)

Contracts exist that don't catch bugs because YAML→codegen→binding→call-site
is a 4-step manual pipeline where each stage leaks. `pv` commands are
fire-and-forget with no closed-loop regeneration.

### 7 Leak Classes from Provable-Contracts Git History

| Leak | Class | Evidence | Design Rule | CB Check |
|------|-------|----------|-------------|----------|
| L-1 | Ghost Bindings | 28,206 stripped in PMAT-106 (97% ghosts) | `pv infer` verified against AST before write | CB-1338 |
| L-2 | Placeholder Preconditions | 507 `!is_empty()` in PMAT-129/131 | Zero placeholder ratio for domain-specific equations | CB-1339 |
| L-3 | Zero Enforcement | Fleet avg 0.01 penetration in PMAT-133 | Repos with binding.yaml require ≥10% call-site penetration | CB-1340 |
| L-4 | Codegen Fidelity | Stale codegen, hardcoded var names | `pv codegen --check` dry-run diff in CI | CB-1211 |
| L-5 | Spec Number Inflation | 22 falsified claims in pv-spec.md | Spec numbers generated from `pv status --json` | CB-1341 |
| L-6 | Parser/Domain Bugs | `domain_to_params()` garbage names | `pv codegen --check` compiles own output | CB-1342 |
| L-7 | Assertion Placement | Preconditions before early-return guards | Preconditions placed AFTER argument validation | CB-1343 |

### Quantitative Progress

| Metric | Peak Inflated | Honest (After Strip) | Current |
|--------|--------------|---------------------|---------|
| Bindings | 28,206 | 540 (97% ghosts) | ~17,000 (verified) |
| Unique assertions | 1 (`!is_empty()`) | — | 500 domain-specific |
| Repos with enforcement | "26/26 Grade A" | 7/26 | ~18/26 |
| Enforcement rate | implied 100% | ~1% | ~60% (kaizen Grade A) |

### Dogfood Results (2026-04-05)

| Repo | CB-1354 Readiness | CB-1350 Status | Warnings | Notes |
|------|-------------------|---------------|----------|-------|
| pmat | 2/4 | Skip (empty index) | 8 | Missing contracts/YAML, binding.yaml |
| aprender | **4/4** | Pass | 4 | Full contract infrastructure |
| trueno | 1/4 | Skip (empty index) | 3 | Only pv CLI |
| realizar | 1/4 | Skip (empty index) | 5 | Only pv CLI |

**Resolved:** CB-1336 (0 injections), CB-1334 (tdg_hooks atomic),
CB-1402 (81/81 L1+), CB-1331 (0 invalid contracts).

**Remaining warnings:** CB-1333 (7 writers), CB-1334 (6 non-atomic),
CB-1404 (low receipt rate), CB-1354 (infra gaps in pmat/trueno/realizar).

### Falsification: Spec Claims vs Reality (2026-04-05)

| # | Spec Claim | Status | Evidence |
|---|-----------|--------|----------|
| F-1 | `pmat work start` generates `contracts/work/<ID>.yaml` | **NOT IMPLEMENTED** | `contracts/work/` does not exist; only `contract.json` generated |
| F-2 | `.pmat/contract-cache.json` written by `pmat work start` | **FIXED** (R-5) | Generated by `refresh-bindings`, 81 contracts cached |
| F-3 | `.pmat/verification-levels.json` for L-level ratchet | **FIXED** (R-5) | Generated by `refresh-bindings` |
| F-4 | `.pmat/asset-layout-cache.json` for O(1) asset checks | **FIXED** (R-5) | Generated by `refresh-bindings` |
| F-5 | `.pmat/binding-index.json` for differential obligations | **IMPLEMENTED** | Generated by `pmat comply refresh-bindings` |
| F-6 | `src/services/hook_registry.rs` single writer | **NOT IMPLEMENTED** | File does not exist; 7 independent writers remain |
| F-7 | `src/services/contract_index.rs` for query enrichment | **NOT IMPLEMENTED** | File does not exist |
| F-8 | `src/services/asset_validator/` directory | **NOT IMPLEMENTED** | Directory does not exist; validation is inline in check functions |
| F-9 | `pmat query --contract-gaps`, `--min-level`, etc. (5 flags) | **NOT IMPLEMENTED** | Only `--contracts` exists (delegates to pv) |
| F-10 | `pmat comply ratchet-override` escape hatch | **NOT IMPLEMENTED** | Subcommand does not exist |
| F-11 | `pmat asset validate` command | **NOT IMPLEMENTED** | Command does not exist |
| F-12 | Pre-commit < 45ms from cache | **PARTIALLY TRUE** | CB checks are O(1) but TDG baseline update adds seconds |
| F-13 | CB-1342 (codegen compiles) check | **FIXED** (R-1) | Wired into comply dispatch, 4 tests, passing on pmat |
| F-14 | 28 CB checks | **VERIFIED** | 25 active in `pmat comply check`, +3 skip conditions |

**Honest summary:** Of 14 falsification findings, **4 fixed** (F-2, F-3, F-4, F-13),
**2 partially fixed** (F-5 existed, F-12 partially true). **8 remain open.**
29 CB checks, 98 tests. O(1) cache layer now generated by `refresh-bindings`.

**What works:** Detection (29 checks), O(1) caches (4 files), `refresh-bindings`,
atomic hook writes (3 prod files), shell escape (CB-1336).

**What remains:** HookRegistry singleton (F-6), ContractIndex service (F-7),
asset_validator service (F-8), 5 query flags (F-9), ratchet-override CLI (F-10),
asset validate CLI (F-11), contracts/work/ YAML generation (F-1).

---

## TDG Integration

Extend TDG to grade non-code assets contributing to project-level aggregate:

| Asset | TDG Dimensions | Weight |
|-------|----------------|--------|
| README.md | Completeness 40%, accuracy 30%, freshness 20%, links 10% | 0.15 |
| Dockerfile | Security 40%, layers 30%, pinning 20%, metadata 10% | 0.05 |
| SVG | Structure 50%, accessibility 30%, palette 20% | 0.02 |
| CHANGELOG | Format 50%, version consistency 30%, completeness 20% | 0.03 |
| forjar.yaml | DAG validity 40%, secrets 30%, templates 30% | 0.05 |

---

## CB Check Summary

| CB Check | Phase | Severity | Enforcement |
|----------|-------|----------|-------------|
| CB-1320 | 3 | Error | README layout slots, ordering, accuracy |
| CB-1321 | 3 | Error | Dockerfile security, layers, pinning |
| CB-1322 | 3 | Error | SVG viewBox, palette, accessibility |
| CB-1323 | 3 | Error | forjar DAG, templates, secrets |
| CB-1324 | 3 | Error | mdBook SUMMARY, code blocks, cross-refs |
| CB-1325 | 3 | Warning | CHANGELOG format, version ordering |
| CB-1326 | 3 | Warning | Badge URLs, required set, placement |
| CB-1330 | 2 | Error | L-level regression (ratchet) |
| CB-1331 | 1 | Error | Work contract YAML validity |
| CB-1332 | — | Warning | Cache staleness (7d warn, 30d error) |
| CB-1333 | 7 | Error | Hook single writer (HookRegistry) |
| CB-1334 | 7 | Error | Hook atomic writes (rename) |
| CB-1335 | 7 | Error | Hook deterministic content |
| CB-1336 | 7 | Error | Hook no shell injection |
| CB-1337 | 7 | Error | Hook performance (p95 < 45ms) |
| CB-1338 | 8 | Error | No ghost bindings |
| CB-1339 | 8 | Error | No placeholder preconditions |
| CB-1340 | 8 | Error | Enforcement penetration ≥10% |
| CB-1341 | 8 | Error | Spec numbers from tooling |
| CB-1342 | 8 | Error | Codegen compiles |
| CB-1343 | 8 | Warning | Assertion placement after guards |
| CB-1350 | 4 | Warning | Differential obligations (staged files → binding lookup) |
| CB-1351 | 4 | Error | Binding index freshness (7d warn, 30d error) |
| CB-1352 | 5 | Warning | Assume-guarantee chain validation |
| CB-1353 | 5 | Error | A/G cycle detection (DAG must be acyclic) |
| CB-1354 | 6 | Warning | Contract query readiness (infrastructure check) |

---

## Implementation Status

**Detection layer (complete):** 29 CB checks, 98 tests, dogfooded on 4 repos.
**Infrastructure layer (in progress):** 4 of 14 artifacts remain missing.

| Phase | Checks | Infrastructure | Status |
|-------|--------|---------------|--------|
| 0 Cache | CB-1332 ✓ | ✓ 3 caches via `refresh-bindings` | **Complete** |
| 1 Work→YAML | CB-1331 ✓ | No `contracts/work/<ID>.yaml` gen | **Check done, YAML gen missing** |
| 2 Ratchet | CB-1330 ✓ | ✓ `verification-levels.json` | **Complete** |
| 3 Assets | CB-1320..1326 ✓ | ✓ `asset-layout-cache.json`; no `asset_validator/` | **Caches done, service missing** |
| 4 Diff Obligations | CB-1350,1351 ✓ | ✓ `binding-index.json` via `refresh-bindings` | **Complete** |
| 5 A/G Chains | CB-1352,1353 ✓ | ✓ Reads `.pmat-work/` directly | **Complete** |
| 6 Query Enrich | CB-1354 ✓ | 5 of 6 query flags missing, no `contract_index.rs` | **Check done, flags missing** |
| 7 Hooks | CB-1333..1337 ✓ | No `hook_registry.rs`; 7 writers, 2 non-atomic | **Checks done, 2 test-only non-atomic** |
| 8 Falsify Leaks | CB-1338..1343 ✓ | ✓ CB-1342 wired and passing | **Complete** |

---

## Remediation Backlog (Prioritized)

Priority: **P0** = blocks real enforcement, **P1** = completes spec claim, **P2** = nice to have.

| # | Falsification | Fix | Priority | Status |
|---|--------------|-----|----------|--------|
| R-1 | CB-1342 not wired into comply dispatch | Implemented + 4 tests | P0 | **DONE** |
| R-2 | 6 non-atomic hook writers (CB-1334) | Atomic writes in 3 prod files; 2 remaining are test helpers | P0 | **DONE** (6→2) |
| R-3 | 7 hook writers (CB-1333) | Route all writes through `HookRegistry` facade | P1 | Open |
| R-4 | No `contracts/work/<ID>.yaml` generation | Extend `pmat work start` to emit provable-contracts YAML | P1 | Open |
| R-5 | No O(1) caches (3 files) | `refresh-bindings` now generates all 3 cache files | P1 | **DONE** |
| R-6 | 5 query flags missing | `--contract-gaps`, `--min-level`, `--max-level`, `--contract-score`, `--asset-contracts` | P2 | Open |
| R-7 | No `ratchet-override` CLI | Add `pmat comply ratchet-override` subcommand | P2 | Open |
| R-8 | No `asset validate` CLI | Add `pmat asset validate` subcommand | P2 | Open |
| R-9 | No `contract_index.rs` service | Lazy-loaded ContractIndex from binding-index.json | P2 | Open |
| R-10 | No `asset_validator/` service | Extract inline validation from check functions | P2 | Open |

**Completed:** R-1, R-2, R-5 (3/10). **Next:** R-3 (HookRegistry), R-4 (YAML gen).

---

## Academic References

- **Mugnier et al. (OOPSLA 2025).** Proof brittleness in Dafny-verified codebases. [ACM DL](https://dl.acm.org/doi/10.1145/3763181)
- **Chakarov et al. (ICSE 2025).** Cedar: formally verified authorization at 1B req/sec. [ACM DL](https://dl.acm.org/doi/10.1109/ICSE55347.2025.00166)
- **AWS (CACM 2024).** Systems Correctness Practices at AWS. [CACM](https://cacm.acm.org/practice/systems-correctness-practices-at-amazon-web-services/)
- **Ma et al. (ICSE 2025).** SpecGen: LLM-generated formal specs. [arXiv](https://arxiv.org/abs/2401.08807)
- **Richter & Wehrheim (arXiv 2024).** NL2Contract: NL to functional contracts. [arXiv](https://arxiv.org/abs/2510.12702)
- **Mugnier et al. (OOPSLA 2025).** Laurel: LLM-repaired Dafny proofs. [arXiv](https://arxiv.org/abs/2405.16792)
- **Bhardwaj (arXiv 2026).** Agent Behavioral Contracts. [arXiv](https://arxiv.org/html/2602.22302v1)
- **Incer et al. (ACM TCPS 2025).** Pacti: assume-guarantee contract algebra. [ACM DL](https://dl.acm.org/doi/10.1145/3704736)
- **Dewes & Dimitrova (AAAI 2025).** Quantitative A/G for multi-agent. [arXiv](https://arxiv.org/abs/2412.13114)
- **AI Transparency Atlas (arXiv 2025).** 8-section documentation scoring. [arXiv](https://arxiv.org/abs/2512.12443)
- **Groce et al. (ASE 2018).** Falsification-driven verification. [Springer](https://link.springer.com/article/10.1007/s10515-018-0240-y)

**Tools:** [mdschema](https://github.com/jackchuka/mdschema), [hadolint](https://github.com/hadolint/hadolint), [rumdl](https://github.com/rvben/rumdl), [standard-readme](https://github.com/RichardLitt/standard-readme)

---

## Key Files

| File | Purpose | Status |
|------|---------|--------|
| `src/cli/handlers/comply_handlers/check_handlers/check_commit_enforcement.rs` | CB-1320..1354 checks + refresh-bindings | **EXISTS** (2800+ lines) |
| `src/cli/handlers/comply_handlers/check_handlers/check.rs` | Check dispatch, wires all CB checks | **EXISTS** |
| `src/cli/handlers/hooks_command_handlers/tdg_hooks.rs` | TDG hook install (atomic, escaped) | **EXISTS** |
| `src/cli/handlers/work_handlers/core_handlers/contract.rs` | Work contract.json generation | **EXISTS** |
| `src/cli/commands/misc_commands_comply.rs` | CLI: refresh-bindings subcommand | **EXISTS** |
| `.pmat/binding-index.json` | O(1) file→binding reverse index | **EXISTS** (via `refresh-bindings`) |
| `src/services/hook_registry.rs` | Single hook writer (Phase 7 design) | **PLANNED** |
| `src/services/contract_index.rs` | ContractIndex for query enrichment | **PLANNED** |
| `src/services/asset_validator/` | Asset layout validation service | **PLANNED** |
| `.pmat/contract-cache.json` | O(1) work contract cache | **EXISTS** (via `refresh-bindings`) |
| `.pmat/verification-levels.json` | O(1) L-level ratchet cache | **EXISTS** (via `refresh-bindings`) |
| `.pmat/asset-layout-cache.json` | O(1) asset validation cache | **EXISTS** (via `refresh-bindings`) |

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 2.3 | 2026-04-05 | R-1 (CB-1342 wired, 4 tests), R-2 (atomic writes 6→2), R-5 (3 O(1) caches). 29 checks, 98 tests. 4/14 falsifications fixed. |
| 2.2 | 2026-04-05 | Prioritized remediation backlog (R-1..R-10) replacing phase table. Honest implementation status. |
| 2.1 | 2026-04-05 | **Falsification audit**: 14 claims tested, 8 unimplemented artifacts identified. |
| 2.0 | 2026-04-05 | Dogfood remediation: CB-1336, CB-1334 (tdg), CB-1402 (81/81 L1+), `refresh-bindings`. 94 tests |
| 1.9 | 2026-04-05 | Phases 4-6: CB-1350..1354. All 8 phases detection-complete (28 checks). |
| 1.0–1.8 | 2026-04-05 | Initial spec through Phase 3+7+8 CB checks. See git log for details. |
