# Provable Contracts Integration

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 22

## Root-Cause Analysis: Why Generated Assertions Don't Catch Bugs

Five Whys (2026-03-29):

1. **Why are generated assertions useless?** Every generated precondition
   is `!_contract_input.is_empty()`, regardless of what the YAML says.
2. **Why does codegen emit the same assertion for all equations?**
   `codegen.rs` hardcoded `!_contract_input.is_empty()` instead
   of looping over `equation.preconditions` — the same pattern that
   already works correctly for postconditions in `emit_postcondition_macro()`.
3. **Why wasn't this caught?** CB-1210 checks YAML precondition diversity
   (which is fine — the YAML has real expressions), not generated output
   diversity. Nobody validated that codegen output matches YAML input.
4. **Why is there no codegen-vs-YAML validation?** The enforcement tooling
   (CB-1200..1210) measures contract *metadata* — existence, schema
   validity, binding coverage, verification levels — but never diffs
   generated Rust against source YAML.

**The bug (FIXED — provable-contracts commit 013397a):** `codegen.rs`
previously hardcoded `!_contract_input.is_empty()` for all preconditions.
Now fixed: `emit_precondition_macro()` loops over `equation.preconditions`
in both the domain-specific (multi-arg) and single-input paths, emitting
each YAML expression as a `debug_assert!`.

**What works:** YAML contracts contain domain-specific Rust expressions
(softmax: `x.iter().all(|v| v.is_finite())`; matmul: `a.len() == m * k`;
rmsnorm: `eps > 0.0`). Both precondition and postcondition codegen now
iterate YAML and emit real assertions.

## Enforcement Chain: CB-1200 through CB-1214

pmat enforces provable-contracts compliance via 13 implemented checks
(2 more specified but not yet implemented) spanning the verification
ladder (L0-L5):

| Check | Level | What it enforces | Catches |
|-------|-------|-----------------|---------|
| CB-1200 | L0.5 | Contract existence + pv lint + binding coverage | Missing contracts |
| CB-1201 | L0.5 | pv lint pass/fail with error detail | Schema violations |
| CB-1202 | L1 | Critical keyword coverage (forward, backward, kernel, etc.) | Uncontracted critical fns |
| CB-1203 | L3 | `#[contract]` annotation coverage on bound functions | Missing compile-time assertions |
| CB-1204 | L1 | build.rs pipeline (superseded by traits at L2+) | Missing build enforcement |
| CB-1205 | L4 | Provability invariant (obligations → kani harnesses) | Missing Kani coverage |
| CB-1206 | L4/L5 | Verification level distribution per-project | Kani/Lean coverage gaps |
| CB-1207 | — | Contract drift (stale YAML vs source) | Outdated contracts |
| **CB-1208** | **L1-L3** | **Binding existence + enforcement level (L0-L3)** | **Ghost bindings, paper-only repos** |
| **CB-1209** | **L2** | **Contract trait enforcement (13 kernel traits)** | **Missing trait impls** |
| **CB-1210** | **L3** | **YAML precondition diversity** | **Mass-generated placeholder preconditions in YAML** |
| **CB-1211** | **L3** | **Codegen fidelity — placeholder ratio check** | **Codegen emitting placeholder assertions** |
| **CB-1212** | **L3** | **Postcondition codegen — wrapper macro pattern** | **NOT IMPLEMENTED** |
| **CB-1213** | **L3** | **Binding-level typed assertions** | **NOT IMPLEMENTED** |
| **CB-1214** | **L3** | **Enforcement quality — call-site penetration × quality** | **Contracts exist but never invoked** |

### Enforcement Levels (detected by CB-1208)

| Level | Mechanism | Repos | What it proves |
|-------|-----------|-------|---------------|
| L3 | build.rs + traits | aprender, entrenar, forjar, realizar, ruchy, simular | Function exists + right signature + build enforced |
| L2 | traits only | bashrs | Function exists + right signature |
| L1 | build.rs only | trueno | YAML says "implemented" |
| L0 | paper-only | 19 repos | **Nothing — ghost bindings** |

L0 repos with binding.yaml but no enforcement **FAIL** CB-1208.

## CB-1210..1213: Contract Enforcement Quality (NEW)

### CB-1210: Precondition Quality

**What it checks:** Scans YAML contract preconditions for diversity and
flags mass-generated placeholder patterns. FAIL when:

1. **YAML precondition diversity < 30%** — >70% of preconditions across all
   equations are identical (suggests mass-generation without domain logic)
2. **>5% of equations have only placeholder preconditions** — equations whose
   preconditions are all known placeholders like `!input.is_empty()`

**Note:** CB-1210 checks YAML *authoring* quality. CB-1211 checks *codegen*
fidelity (whether generated `debug_assert!` output matches the YAML).

### CB-1211: Codegen Fidelity (NEW)

**What it checks:** Generated `debug_assert!` assertions in
`generated_contracts.rs` should not be dominated by placeholder patterns.

**Detection:** Count total `debug_assert!` lines in the generated file
(excluding comments). Count how many contain the known placeholder
`_contract_input.is_empty()`. FAIL if:

1. Placeholder assertions are >50% of total generated assertions
2. Falls back to running `pv codegen` if no generated file exists

**Status: FIXED** (provable-contracts commit `013397a`). Codegen now
loops over `equation.preconditions` via `emit_precondition_macro()`,
mirroring the postcondition pattern in `emit_postcondition_macro()`.

CB-1211 remains as a **regression detector**: if a future change
reintroduces hardcoded placeholders, the placeholder-ratio check catches it.

**Note:** Generated assertion count may be less than YAML precondition count
because codegen's `has_unbound_vars()` skips assertions whose variables
can't be mapped to the macro's input parameter (e.g., multi-arg equations
where `m`, `k`, `n` aren't bound). This is expected behavior, not a failure.

**Severity:** FAIL (placeholder ratio >50%). SKIP when unable to verify.

### CB-1212: Combined Wrapper Macro (NEW)

**What it checks:** For equations with both preconditions and postconditions,
codegen should emit a combined `contract_*` wrapper in addition to the
separate `contract_pre_*` / `contract_post_*` macros.

**Why:** The separate `contract_post_*` macro works (`emit_postcondition_macro()`
emits real YAML postconditions), but callers must manually capture the return
value and call the post macro. A combined wrapper makes postcondition
enforcement automatic. Codegen already emits the wrapper via
`emit_combined_macro()` — this check validates its presence.

**Combined pattern (generated alongside existing separate macros):**

```rust
macro_rules! contract_softmax {
    ($input:expr, $body:expr) => {{
        contract_pre_softmax!($input);
        let _result = $body;
        contract_post_softmax!(_result);
        _result
    }};
}
```

**Detection:** WARN if a contract has both preconditions and postconditions
but codegen only emits separate macros without a combined wrapper.

**Severity:** WARN. The separate macros already work; the wrapper is ergonomic.

### CB-1213: Binding-Level Typed Assertions (NEW)

**What it checks:** Each entry in `binding.yaml` with `status: implemented`
should have typed preconditions/postconditions that reference the actual
function signature parameters, not the generic equation parameters.

**Problem:** A contract equation uses `x`, `m`, `k`, `n` — but the bound
function might use `logits`, `rows`, `cols`. The binding must translate.

**Schema extension for binding.yaml:**

```yaml
bindings:
  - contract: softmax-kernel-v1
    equation: softmax
    function: softmax
    module_path: trueno::blis::softmax::softmax
    signature: "fn(&[f32]) -> Vec<f32>"
    status: implemented
    # NEW: typed assertions per binding
    preconditions:
      - 'logits.iter().all(|v| v.is_finite())'
      - 'logits.len() <= 131072'  # max context window
    postconditions:
      - '(result.iter().sum::<f32>() - 1.0).abs() < 1e-5'
```

**Detection:** For each binding with `status: implemented`:
1. WARN if no `preconditions` field (inherits generic equation preconditions)
2. FAIL if binding `preconditions` contains known placeholders
3. INFO if binding adds domain constraints beyond the equation
   (e.g., `logits.len() <= 131072` — deployment-specific bound)

**Severity:** WARN. Binding-level assertions are an upgrade path, not blocking.

## Configurable Thresholds

Configure strictness in `.pmat.yaml`:

```yaml
comply:
  thresholds:
    pv_lint_is_error: true        # CB-1201: WARN → FAIL on lint failure
    min_binding_existence: 95     # CB-1208: % threshold for binding verification
    require_all_traits: true      # CB-1209: require 13/13 traits
    min_kani_coverage: 20         # CB-1206: minimum Kani proof %
```

**Hardcoded thresholds** (not yet configurable):

| Check | Threshold | Value |
|-------|-----------|-------|
| CB-1210 | Precondition diversity minimum | 30% |
| CB-1210 | Placeholder-only equation maximum | 5% |
| CB-1211 | Placeholder assertion ratio maximum | 50% |
| CB-1214 | Enforcement quality minimum | 0.3 (with >30 call sites AND mixed E-levels) |

### CB-1214: Enforcement Quality (NEW)

Runs `pv coverage --enforcement <src> --binding <binding.yaml>` and parses
the enforcement score. Classifies contract call sites:

| Level | Score | Meaning |
|-------|-------|---------|
| E0 | 0.1 | Generic `!is_empty()` assertion at call site |
| E1 | 0.5 | Domain-specific precondition check only |
| E2 | 1.0 | Both precondition and postcondition checks |

**Quality** = weighted average of E levels across call sites.
**Enforcement** = penetration (call sites / bindings) × quality.

FAIL if quality < 0.3 AND >30 call sites AND has mixed E-levels (mature repo regressed).
WARN if quality < 0.3 with E0-only (legitimate transition — macros invoked but generic).
WARN if 0 call sites found (contracts exist but are never invoked).
SKIP if `pv` CLI not available.

**Expected `pv coverage --enforcement` output format:**

```
E0 (<description>):  <count>
E1 (<description>):  <count>
E2 (<description>):  <count>
Quality score:  <float>
Enforcement score:  <float>
```

Both CB-1214 and PV-05 parse E-levels via prefix matching (`"E0 ("`)
followed by colon-split to extract the count. Quality/enforcement scores
are parsed by label then first whitespace-separated float.

## Infra-Score PV Bonus (PV-01..PV-05)

`pmat infra-score` awards up to 12 bonus points for provable-contracts:

| Check | Points | What it checks |
|-------|--------|---------------|
| PV-04 | 2 | contracts/ directory exists with schema-valid YAML |
| PV-01 | 3 | `pv lint` passes (falls back to YAML structure check) |
| PV-02 | 3 | `pv score >= 0.5` (FAILs without pv CLI) |
| PV-03 | 2 | At least one contract at proof level L2+ |
| PV-05 | 2 | Enforcement quality — `pv coverage --enforcement` finds call sites |

## Finding Missing Contracts

```bash
# Full audit
pmat comply check

# Which bound functions don't exist in source?
pmat comply check 2>&1 | grep 'CB-1208'

# Are contract traits implemented?
pmat comply check 2>&1 | grep 'CB-1209'

# Are preconditions real or mass-generated?
pmat comply check 2>&1 | grep 'CB-1210'

# Which critical functions lack contracts?
pmat comply check 2>&1 | grep 'CB-1202'
```

## Key Files

| File | Purpose |
|------|---------|
| `check_pv_enforcement.rs` | CB-1201..1209 enforcement checks |
| `check_pv_quality.rs` | CB-1210, CB-1211, CB-1214 quality checks |
| `check_provable_contracts.rs` | CB-1200 detection + pv lint/score |
| `infra_score_handlers.rs` | `pmat infra-score` CLI handler |
| `provable_contracts.rs` (infra_score) | PV-01..PV-05 bonus scorer |
| `comply_config_types.rs` | PV threshold configuration |
| `codegen.rs` (provable-contracts) | Precondition codegen (fixed in 013397a) |

## Remediation Roadmap

### Fix 1: Precondition codegen loop (CB-1211) — DONE

Fixed in provable-contracts commit `013397a`. Codegen now loops over
`equation.preconditions` and emits each YAML expression.

### Fix 2: Combined wrapper macro (CB-1212) — codegen DONE, pmat check TODO

Codegen already emits `contract_<eq>!` wrapper macros via
`emit_combined_macro()` in `codegen.rs`. The pmat check to detect
repos missing the wrapper pattern is not yet implemented.

**Scope:** pmat `check_pv_quality.rs` (new check function).

### Fix 3: Binding-level assertions (CB-1213) — schema + codegen

Extend `binding.yaml` schema with optional per-binding preconditions using
actual function parameter names. Codegen reads these when present, falls
back to equation-level assertions otherwise.

**Scope:** `schema.rs` (Binding struct), `codegen.rs` (binding-aware generation).

### Fix priority

| Fix | Check | Impact | Effort | Status |
|-----|-------|--------|--------|--------|
| 1 | CB-1211 | 516 real assertions | ~10 lines | **DONE** (013397a) |
| 2 | CB-1212 | Ergonomic postconditions | Codegen done, pmat check ~20 lines | **PARTIAL** |
| 3 | CB-1213 | Per-binding type safety | Schema + codegen | **TODO** |

## References

### Provable-Contracts Spec
- pv-spec.md §2: Verification Ladder (L0-L5)
- pv-spec.md §23: Contract-Trait Enforcement
- pv-spec.md §27: The One Way (unified enforcement)

### arXiv
- Dardik & Kang (2025). arXiv:2509.06250. Assume-guarantee contracts.
- Li et al. (2025). arXiv:2510.12047. LLMs and formal contracts.
- Le Blanc & Lam (2024). arXiv:2410.01981. Rust verification landscape.

### Foundational
- Meyer (1988). Design by Contract. Preconditions, postconditions, invariants.
- `core::contracts` RFC: rust-lang/rust#128045
