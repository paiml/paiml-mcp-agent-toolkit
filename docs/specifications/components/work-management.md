# Work Management

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 14

## Theoretical Foundations

pmat work applies the same formal methods as provable-contracts,
but to **development tasks** instead of kernel functions:

| Foundation | provable-contracts (kernels) | pmat work (tasks) |
|---|---|---|
| **Popper (1959)** Falsificationism | Kani harnesses designed to *refute*, not confirm | Work claims must be *falsifiable* with evidence commands |
| **Meyer (1988)** Design by Contract | `preconditions`/`postconditions`/`invariants` in YAML | `require`/`ensure`/`invariant` clauses in contract.json |
| **Ohno (1988)** Jidoka | `pv lint` stops the build on contract violation | `pmat work complete` blocks if claims fail |
| **Shingo (1986)** Poka-Yoke | `ValidatedTensor` makes bad states unrepresentable | Quality gates make bad commits impossible |
| **Toyota Way** Five Whys | Root cause analysis of *why* a kernel violates a contract | `pmat five-whys` analyzes complexity/SATD/churn evidence |
| **Batuta Oracle** Equation-Driven Dev | Equation → failing test → impl → verify | `batuta oracle --rag` → design → failing test → impl → verify |
| **Dardik & Kang (2025)** Assume-Guarantee | Stage output_invariant = next stage input_requires | Work ensure clauses = next task require clauses |

### The Provable Work Thesis

If provable-contracts proves that `softmax(x)` sums to 1.0 for ALL inputs,
then pmat work should prove that `PMAT-123` achieves its stated goal with
ALL quality gates passing. The verification ladder applies to both:

```
Kernel verification:           Task verification:
  L5: Lean theorem               L5: Formal spec + acceptance tests
  L4: Kani bounded proof         L4: Property-based test (proptest/probar)
  L3: #[contract] assertions     L3: Falsifiable claims with evidence
  L2: Trait enforcement           L2: Compile-time checks (cargo test)
  L1: build.rs AllImplemented    L1: Quality gates (pmat comply)
  L0: Code review                L0: Human review
```

### arXiv References

- **Dardik & Kang (2025).** "Compositional Inductive Invariant Inference
  via Assume-Guarantee Reasoning." arXiv:2509.06250. Work contract
  require/ensure clauses form assume-guarantee pairs across task chains.

- **Li et al. (2025).** "Do Large Language Models Respect Contracts?"
  arXiv:2510.12047. Motivates compiler-enforced contracts over LLM-
  generated code — the same reasoning applies to agent-generated work items.

- **Bruni et al. (2026).** "Agent Behavioral Contracts." arXiv:2602.22302.
  Extends Eiffel DbC to multi-agent systems. Directly applicable to
  pmat work contracts for autonomous agent tasks (pmat agent).

- **Yang et al. (2025).** "CodeCureAgent: Automatic Classification and
  Repair of Static Analysis Warnings." arXiv:2509.11787. LLM-assisted
  auto-fix informs the five-whys → work-contract → auto-fix pipeline.

---

## pmat work System

### Contract-Based Quality Enforcement (DbC v5.0)

Each work item has a falsifiable contract with three clause types
(mapping directly to Meyer's Design by Contract):

```json
{
  "ticket_id": "PMAT-123",
  "profile": "Rust",
  "verification_level": "L3",
  "references": {
    "arxiv": ["2509.06250"],
    "spec_section": "pv-spec.md §23",
    "five_whys_id": "FW-2026-03-27-001"
  },
  "require": [
    {"description": "Project builds successfully", "evidence": "cargo build"},
    {"description": "At least one test exists", "evidence": "cargo test --list"}
  ],
  "invariant": [
    {"description": "Project compiles at every checkpoint", "evidence": "cargo check"},
    {"description": "No file exceeds 500 lines", "evidence": "pmat comply cb-040"},
    {"description": "cargo clippy passes", "evidence": "cargo clippy"},
    {"description": "No new SATD markers", "evidence": "pmat analyze satd"}
  ],
  "ensure": [
    {"description": "All tests pass", "evidence": "cargo test"},
    {"description": "Coverage >= 95%", "evidence": "cargo llvm-cov"},
    {"description": "No F-grade functions", "evidence": "pmat comply cb-200"},
    {"description": "pmat comply COMPLIANT", "evidence": "pmat comply check"}
  ]
}
```

### Verification Level Tracking

Each work contract declares its target verification level (L0-L5):

| Level | Task verification | Example |
|---|---|---|
| L5 | Formal spec + proved acceptance criteria | Lean theorem for algorithm correctness |
| L4 | Property-based tests (proptest/probar) | `proptest!` covering all edge cases |
| L3 | Falsifiable claims with evidence commands | `ensure: coverage >= 95%` |
| L2 | Compile-time enforcement | trait impls, `#[contract]` annotations |
| L1 | Quality gates pass | `pmat comply check` → COMPLIANT |
| L0 | Human review only | Code review, no automated verification |

Default is L3. Use `pmat work start PMAT-123 --level L4` for stricter.

### Chain-of-Thought Audit Trail

Every work contract should record the reasoning chain:

```json
{
  "chain_of_thought": [
    {"step": 1, "question": "What does the fix need to achieve?", "answer": "CB-1208 must verify binding.yaml against src/"},
    {"step": 2, "question": "What do similar systems do?", "answer": "SPARK/Ada: spec/body separation. Eiffel: deferred features. Kani: proof_for_contract."},
    {"step": 3, "question": "What's the Rust-native solution?", "answer": "Parse binding.yaml, walk src/ for fn declarations, cross-reference."},
    {"step": 4, "question": "How do we falsify?", "answer": "Test against repos with known phantom bindings (trueno: 29 missing)."}
  ]
}
```

This mirrors provable-contracts Section 23's chain-of-thought pattern.

### Five Whys Integration

When a work item originates from a defect, link the five-whys analysis:

```bash
# 1. Diagnose
pmat five-whys "CB-1206 reports global totals for all repos" --depth 5 -o fw-report.json

# 2. Create work item linked to diagnosis
pmat work start PMAT-528 --five-whys fw-report.json

# 3. The work contract inherits evidence from five-whys
#    Root cause → require clause
#    Fix target → ensure clause
```

The five-whys evidence (complexity scores, SATD counts, churn metrics,
dead code percentages) flows into the work contract as require/ensure
thresholds.

### Batuta Oracle Integration

When creating a work item, query the sovereign stack for prior art:

```bash
# Search for how similar issues were solved before
batuta oracle --rag "binding existence verification"

# Creates work item with oracle context
pmat work start PMAT-529 --oracle-context
```

The oracle search results are stored in the work contract's
`references.oracle_results` field for traceability.

### Commands

```bash
pmat work start "PMAT-123" --title "Fix memory leak" --level L3
pmat work start "PMAT-124" --five-whys fw-report.json
pmat work status "PMAT-123"
pmat work complete "PMAT-123"  # Validates ALL contract claims
pmat work list --status in_progress
```

### Quality Gate Integration

On `pmat work complete`:
1. Check require clauses are still met
2. Run all ensure clauses as assertions
3. Validate invariant clauses haven't regressed
4. If any claim fails, block completion
5. Generate evidence report with verification level achieved
6. Record chain-of-thought if present

## Ticket Tracking

### Issue Reference Parsing

Supports patterns: `#123`, `PMAT-123`, `GH-123`
- Must `trim_matches` for parens: `(PMAT-472)` -> `PMAT-472`

### Fuzzy ID Matching

Short IDs resolve to full tickets:
- `123` matches `PMAT-123`
- Case-insensitive matching

### arXiv Reference Linking

Work items can reference arXiv papers:
```yaml
references:
  arxiv: ["2509.06250", "2510.12047"]
```

This traces the development task back to the research that motivates it,
mirroring how provable-contracts traces kernels back to papers.

## Roadmap & Todo

### Quality Gate Spec

```bash
pmat roadmap list              # Show all roadmap items
pmat roadmap add "Feature X"   # Add item with auto-priority
pmat roadmap complete "X"      # Mark complete, validate quality
```

### Provable-Contracts Alignment

Roadmap items for provable-contracts client repos should include:
- **Verification level target**: Which L-level the feature achieves
- **Contract stems affected**: Which YAML contracts are relevant
- **Binding coverage delta**: Expected change in CB-1208 coverage

## Storage

```
.pmat-work/
├── PMAT-123/
│   ├── contract.json    # Falsifiable claims (DbC v5.0)
│   ├── chain-of-thought.json  # Reasoning audit trail
│   ├── five-whys.json   # Root cause analysis (if linked)
│   ├── evidence/        # Test results, coverage reports
│   └── notes.md         # Work notes
└── index.json           # Ticket index
```

## Configurable PV Enforcement Thresholds

Work contracts inherit thresholds from `.pmat.yaml`:

```yaml
comply:
  thresholds:
    # Provable-contracts enforcement (L0.5 through L5)
    pv_lint_is_error: true           # CB-1201: FAIL not WARN on lint failure
    min_binding_existence: 95        # CB-1208: 95% binding verification threshold
    require_all_traits: true         # CB-1209: 13/13 traits required for PASS
    min_kani_coverage: 20            # CB-1206: minimum Kani harness percentage
```

## Key Files

| File | Purpose |
|------|---------|
| `src/cli/handlers/work_handlers.rs` | Work command handler |
| `src/cli/handlers/work_contract.rs` | DbC v5.0 contract engine |
| `src/cli/handlers/work_falsification.rs` | Popperian falsification executor |
| `src/cli/handlers/work_ledger.rs` | Append-only receipt tracking |
| `src/cli/handlers/five_whys_handlers.rs` | Toyota Way root cause analysis |
| `src/cli/handlers/comply_handlers/check_handlers/check_pv_enforcement.rs` | CB-1201..1209 |

## References

### Internal
- pv-spec.md Section 2: The Verification Ladder (L0-L5)
- pv-spec.md Section 23: Contract-Trait Enforcement (chain-of-thought)
- pv-compatibility.md: CB-1200..1209 integration spec

### Foundational
- Popper, K. (1959). *The Logic of Scientific Discovery*. Falsificationism.
- Meyer, B. (1988). *Object-Oriented Software Construction*. Design by Contract.
- Ohno, T. (1988). *Toyota Production System*. Jidoka, Five Whys, Kaizen.
- Shingo, S. (1986). *Zero Quality Control*. Poka-Yoke (mistake-proofing).

### arXiv
- Dardik & Kang (2025). arXiv:2509.06250. Assume-Guarantee compositional reasoning.
- Li et al. (2025). arXiv:2510.12047. LLMs and formal contracts.
- Bruni et al. (2026). arXiv:2602.22302. Agent behavioral contracts.
- Yang et al. (2025). arXiv:2509.11787. CodeCureAgent auto-fix.
- SorryDB (2026). arXiv:2603.02668. LLM-assisted Lean sorry elimination.

### Consolidated from
- enhance-pmat-work, enhance-pmat-work-spec, improve-pmat-work,
  master-plan-pmat-work-system, roadmap-todo-quality-gate-spec
