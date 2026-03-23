# pv-compatibility: pmat ↔ provable-contracts Integration Spec

**Status:** Gap analysis complete, implementation roadmap defined.

---

## 1. Problem Statement

`pmat comply` and `pmat score` claim to enforce provable-contracts
quality gates, but the integration is shallow. The pv-spec defines
precise scoring dimensions (D1-D5, CD1-CD5), a verification ladder
(L1-L5), and a provability invariant — pmat checks almost none of
these correctly.

### Current Integration Depth

| pv-spec Feature | pmat Status | Gap |
|---|---|---|
| Verification Ladder (L1-L5) | Not checked | Critical |
| Provability Invariant | Delegated to `pv lint` boolean | No per-obligation enforcement |
| Contract Score (D1-D5) | Custom formula, wrong dimensions | Replace with pv score |
| Codebase Score (CD1-CD5) | File-existence heuristic | Replace with pv score |
| SARIF output | Not consumed | No GitHub Code Scanning |
| proof-status.json | Checks existence only | Parse L4/L5 levels |
| Drift Detection (CD5) | Not implemented | Stale contracts undetected |
| pv query cross-project | Not integrated | Separate tool silos |

---

## 2. Architecture: What Should Change

### 2.1 CB-1201: Parse pv score dimensions, not just pv lint boolean

**Current:** Calls `pv lint --format json`, reads `gates[].passed`, computes
custom score from existence + validity + cleanliness + completeness +
pipeline_depth.

**Target:** Call `pv score <contracts_dir> --format json`, parse the 5
contract dimensions directly:

```
PV Lint sub-score = D1*0.20 + D2*0.25 + D3*0.25 + D4*0.10 + D5*0.20
```

Where:
- D1 = Specification Depth (equations, domains, invariants, tolerances)
- D2 = Falsification Coverage (obligations with tests / total)
- D3 = Kani Proof Coverage (obligations with harnesses, strategy-weighted)
- D4 = Lean Proof Coverage (obligations with proved theorems)
- D5 = Binding Coverage (implemented / total bindings)

This aligns pmat scoring with the pv-spec scoring model exactly.

### 2.2 CB-1205: Provability Invariant Check (NEW)

**New check:** For every kernel contract (non-registry), enforce:

```
|proof_obligations| > 0  →  |kani_harnesses| > 0
|proof_obligations| > 0  →  |falsification_tests| >= |proof_obligations|
```

Implementation: Read each YAML in contracts/, check if it has
`proof_obligations:` and verify corresponding `kani_harnesses:` and
`falsification_tests:` sections exist with sufficient entries. Skip
contracts with `registry: true` in metadata.

### 2.3 CB-1206: Verification Level Distribution (NEW)

**New check:** Parse `proof-status.json` from provable-contracts sibling
repo. Report the L1-L5 distribution:

```
Level   Count   Percentage
L5      14      2.3%       (Lean proved)
L4      229     37.1%      (Kani harnesses defined)
L3      618     100%       (falsification tests)
L2      660     106%       (edge case tests)
L1      442     71.5%      (type-level bindings)
```

FAIL if any kernel obligation is below L2 (no falsification test).
WARN if <50% of obligations reach L4 (Kani).

### 2.4 Codebase Score (CD1-CD5) in pmat score

Replace `compute_pipeline_depth()` (crude file-existence check) with
actual codebase scoring dimensions:

| Dimension | Weight | How to compute |
|---|---|---|
| CD1: Contract Coverage | 30% | `#[contract]` annotations / total pub fns in critical modules |
| CD2: Binding Completeness | 20% | Parse binding.yaml: implemented / total |
| CD3: Mean Contract Score | 20% | Call `pv score --format json`, read mean |
| CD4: Proof Depth Distribution | 15% | Parse proof-status.json, compute weighted L1-L5 |
| CD5: Drift Detection | 15% | Compare contract timestamps vs git log of bound files |

### 2.5 SARIF Pipeline

Add `--sarif` flag to `pmat comply`:

```bash
pmat comply check --format sarif > results.sarif
```

Implementation: When contracts/ exists, call `pv lint --format sarif`
and merge the SARIF output with pmat's own findings. Upload combined
SARIF to GitHub Code Scanning via `gh` CLI.

### 2.6 pmat work ↔ provable-contracts scoring alignment

`pmat work` contract scoring (spec_depth, falsification_coverage,
invariant_health, subcontracting, traceability) is for TICKET management.
`pv score` (D1-D5) is for KERNEL contracts. These are different scoring
models for different purposes. Document clearly:

| Scoring System | Purpose | Dimensions |
|---|---|---|
| `pv score` (D1-D5) | Kernel contract quality | spec_depth, falsification, kani, lean, binding |
| `pmat work` contract | Ticket/task quality | spec_depth, falsification, invariant_health, subcontracting, traceability |
| `pmat score` PV Lint | Composite project quality | Should delegate to `pv score` for contract-aware projects |

---

## 3. Implementation Priority

| Priority | Change | Impact | Effort |
|---|---|---|---|
| P0 | CB-1201: Use `pv score` dimensions | Aligns scoring with spec | **DONE** (PMAT-523) |
| P0 | Parse proof-status.json levels | Real L4/L5 coverage | **DONE** (PMAT-523) |
| P1 | CB-1205: Provability Invariant | Catches missing kani/tests | **DONE** (PMAT-523) |
| P1 | CD1-CD5 codebase scoring | Blended pv-score + pipeline depth | **DONE** (PMAT-523) |
| P2 | CB-1206: Verification Level Distribution | 425 obligations tracked | **DONE** (PMAT-524) |
| P2 | CB-1207: Contract Drift Detection | 90-day staleness check via git log | **DONE** (PMAT-525) |
| P2 | CB-1206: Verification Level Distribution | L2/L4/L5 from proof-status.json | **DONE** (PMAT-524) |
| P3 | SARIF pipeline | `pmat comply --format sarif` delegates to `pv lint` | **DONE** (PMAT-525) |
| P3 | pv query integration | `pmat query --contracts` delegates to `pv query` | **DONE** (PMAT-526) |

---

## 4. References

### Provable-Contracts Spec (Internal)

- pv-spec.md Section 2: Verification Ladder (L1-L5)
- pv-spec.md Section 7: Scoring System (D1-D5, CD1-CD5)
- sub/scoring.md: Full scoring formulas and weights
- sub/lint.md: SARIF output schema and CI integration
- sub/lean-kani-composition.md: L4+L5 compositional verification
- sub/escape-proof-enforcement.md: Six-stage pipeline

### Formal Verification in CI (arxiv)

- 2511.14805 — Continuous Assurance with Formal Verification (2025).
  Framework for integrating bounded model checking into CI/CD pipelines.
  Directly applicable to Kani-in-CI gating.

- 2601.18944 — Neural Theorem Proving for Verification Conditions (ICLR 2026).
  LLM-assisted discharge of VCs from Design by Contract specs.
  Relevant to automating Lean proof completion from YAML obligations.

- 2602.18307 — VeriSoftBench: Repository-Scale Lean Verification.
  Benchmark for Lean 4 verification at scale. Validates that our
  64-theorem corpus is meaningful compared to peer repositories.

- 2510.01072 — Rust Standard Library Verification (2025).
  Kani + Miri verification of std. Establishes patterns for
  verifying Rust code at scale that our pipeline follows.

- 2511.12638 — Equivalence Checking of ML GPU Kernels (2025).
  Formal equivalence between scalar/SIMD/PTX implementations.
  Directly applicable to our SIMD=scalar Kani harnesses.

- 2511.12294 — ProofWright: Agentic Formal Verification of CUDA (2025).
  AI agent completing formal proofs for GPU kernels. Relevant to
  automating Kani harness generation from YAML contracts.

### Quality Gates and Static Analysis (arxiv)

- 2403.05986 — Integrating Static Code Analysis Toolchains (Feist et al. 2024).
  SARIF as convergence format for multi-tool pipelines. Directly
  applicable to pmat + pv lint SARIF integration.

- 2508.18816 — Dealing with SonarQube Cloud (Nachman et al. 2025).
  Lessons on quality gate fatigue and false positive management.
  Informs our suppression and per-rule severity design.

- 2509.11787 — CodeCureAgent: Automatic Classification and Repair of
  Static Analysis Warnings (Yang et al. 2025). LLM-assisted auto-fix
  for static analysis findings. Could enhance pv lint auto-fix.

- 2506.10330 — Augmenting LLMs with Static Code Analysis (Shestov et al. 2025).
  Combining LLM code generation with SARIF-formatted static analysis
  feedback. Applicable to pmat + pv lint feedback loops.

### Design by Contract (foundational)

- 2510.12047 — Do Large Language Models Respect Contracts? (Li et al. 2025).
  Evaluates whether LLM-generated code satisfies formal contracts.
  Motivates why we need compiler-enforced contracts, not just prompts.

- 2505.19271 — VerifyThisBench: 580 Verification Tasks (2025).
  Benchmark for Design by Contract verification tools.
  Our 138 YAML contracts + 618 obligations are comparable in scale.

- 2602.22302 — Agent Behavioral Contracts (Bruni et al. 2026).
  Extends Eiffel DbC to multi-agent systems. Relevant to
  pmat work contract scoring for autonomous agent tasks.

- 2403.14064 — Lean4Lean: Verified Typechecker for Lean 4 (2024).
  Self-verifying infrastructure for Lean 4 theorem prover.
  Validates Lean 4 as a sound foundation for our L5 proofs.

- 2603.02668 — SorryDB: Can AI Complete Lean Theorems? (2026).
  LLM-assisted sorry elimination. Directly applicable to completing
  our remaining sorry-free theorem obligations.

### Rust Nightly Contracts

- RFC 128044 — core::contracts (Rust nightly).
  Experimental `#[requires]`/`#[ensures]` in Rust std.
  Our `provable-contracts-macros` implements the same interface
  as a stable proc macro, ready to migrate when stabilized.
