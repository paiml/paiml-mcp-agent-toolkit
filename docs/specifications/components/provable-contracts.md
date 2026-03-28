# Provable Contracts Integration

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 22

## Enforcement Chain: CB-1200 through CB-1210

pmat enforces provable-contracts compliance via 11 checks spanning
the full verification ladder (L0-L5):

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
| **CB-1210** | **L3** | **Precondition quality + postcondition existence** | **Mass-generated boilerplate** |

### Enforcement Levels (detected by CB-1208)

| Level | Mechanism | Repos | What it proves |
|-------|-----------|-------|---------------|
| L3 | build.rs + traits | aprender, entrenar, forjar, realizar, ruchy, simular | Function exists + right signature + build enforced |
| L2 | traits only | bashrs | Function exists + right signature |
| L1 | build.rs only | trueno | YAML says "implemented" |
| L0 | paper-only | 18 repos | **Nothing — ghost bindings** |

L0 repos with binding.yaml but no enforcement **FAIL** CB-1208.

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

## Infra-Score PV Bonus (PV-01..PV-04)

`pmat infra-score` awards up to 10 bonus points for provable-contracts:

| Check | Points | What it checks |
|-------|--------|---------------|
| PV-04 | 2 | contracts/ directory exists with schema-valid YAML |
| PV-01 | 3 | `pv lint` passes (falls back to YAML structure check) |
| PV-02 | 3 | `pv score >= 0.5` (FAILs without pv CLI) |
| PV-03 | 2 | At least one contract at proof level L2+ |

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
| `check_pv_enforcement.rs` | CB-1201..1210 implementation |
| `check_provable_contracts.rs` | CB-1200 detection + pv lint/score |
| `infra_score_handlers.rs` | `pmat infra-score` CLI handler |
| `provable_contracts.rs` (infra_score) | PV-01..PV-04 bonus scorer |
| `comply_config_types.rs` | PV threshold configuration |

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
